// Export syntax: `FFI_EXPORT[fix_value_name, c_function_name];`

use crate::ast::expr::ExprNode;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::Scheme;
use crate::ast::types::{tycon, TyCon, Type, TypeNode};
use crate::constants::{PTR_NAME, STD_NAME};
use crate::error::Errors;
use crate::fixstd::builtin::{make_iostate_ty, run_io};
use crate::fixstd::runtime::reserved_c_function_name_reason;
use crate::generator::Generator;
use crate::generator::Object;
use crate::object::create_obj;
use crate::object::ObjectFieldType;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::RcState;
use inkwell::attributes::AttributeLoc;
use inkwell::types::{BasicType, BasicTypeEnum};
use std::sync::Arc;

// The export statement.
#[derive(Clone)]
pub struct ExportStatement {
    // The name of the Fix value to be exported.
    // This is the name of the Fix value in the source code, and not the name of the symbol.
    // To get the name of the instantiated Fix value, use `self.instantiated_value_expr`.
    pub value_name: FullName,
    /// Span of the value-name token inside `FFI_EXPORT[<here>, c_name];`.
    /// Used for LSP rename / find-references; `None` for export statements
    /// synthesized internally without a corresponding source token.
    pub value_name_src: Option<Span>,
    // The expression (symbol) to be exported.
    // `None` at first, and set after the fix value is instantiated to a symbol.
    pub value_expr: Option<Arc<ExprNode>>,
    // The name of the exported function.
    pub function_name: String,
    // The type of the exported function.
    // `None` at first, and set by `ExportedFunctionType::validate`.
    pub function_type: Option<ExportedFunctionType>,
    // The source of the export statement.
    pub src: Option<Span>,
}

impl ExportStatement {
    // Create an export statement carrying what the source gives.
    // `ExportedFunctionType::validate` fills in `function_type` later, and instantiation of the
    // exported value fills in `value_expr`.
    pub fn new(
        fix_value_name: FullName,
        c_function_name: String,
        src: Option<Span>,
    ) -> ExportStatement {
        ExportStatement {
            value_name: fix_value_name,
            value_name_src: None,
            function_name: c_function_name,
            src,
            function_type: None,
            value_expr: None,
        }
    }

    /// Check that the C function name this statement gives is one C can spell and one the compiler
    /// leaves to the program.
    ///
    /// # Arguments
    /// * `src` — where to place the error message.
    pub fn validate_names(&self, src: &Option<Span>) -> Result<(), Errors> {
        // A C identifier is written in ASCII: a letter or an underscore, then letters, digits and
        // underscores.
        let first = self
            .function_name
            .chars()
            .next()
            .expect("the grammar gives an export statement a non-empty C function name");
        if !first.is_ascii_alphabetic() && first != '_' {
            let msg = format!(
                "`{}` is not a valid C function name. The first character should be an ASCII letter or an underscore.",
                &self.function_name
            );
            return Err(Errors::from_msg_srcs(msg, &vec![src]));
        }
        for c in self.function_name.chars() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                let msg = format!(
                    "`{}` is not a valid C function name. The rest of the characters should be an ASCII letter, a digit or an underscore.",
                    &self.function_name
                );
                return Err(Errors::from_msg_srcs(msg, &vec![src]));
            }
        }
        if let Some(reason) = reserved_c_function_name_reason(&self.function_name) {
            let msg = format!(
                "`{}` cannot be the name of an exported function: {}.",
                &self.function_name, reason
            );
            return Err(Errors::from_msg_srcs(msg, &vec![src]));
        }
        Ok(())
    }

    // Implement the exported C function.
    // Requires `self.function_type` and `self.value_expr` to already be set.
    pub fn implement<'c, 'm>(&self, gc: &mut Generator<'c, 'm>) {
        let ExportedFunctionType {
            doms,
            codom,
            io_type,
        } = self.function_type.clone().unwrap();

        // Create the LLVM type of the exported C function. Each exchanged value is its own
        // scalar — an integer, a floating point number or a pointer — which is the type a C
        // declaration of the same function names. `has_c_abi` admits nothing with another shape.
        let dom_llvm_tys = doms
            .iter()
            .map(|dom| c_scalar_type(dom, gc).into())
            .collect::<Vec<_>>();
        let func_ty = if codom.is_unit() {
            gc.context.void_type().fn_type(&dom_llvm_tys, false)
        } else {
            c_scalar_type(&codom, gc).fn_type(&dom_llvm_tys, false)
        };

        // Take the name. An `FFI_CALL` of this C function has declared it by now — code generation
        // implements the program's symbols before it reaches here — and a declaration and this
        // definition describe one C function, so the body goes onto that declaration. Anything else
        // found under the name is a name the compiler owns, which `validate_names` rejects, and a
        // declaration of another signature, which `validate_c_function_signatures` rejects.
        let func = match gc.module.get_function(&self.function_name) {
            Some(declared) => {
                assert_eq!(
                    declared.get_type(),
                    func_ty,
                    "every description of the C function `{}` gives one signature",
                    self.function_name
                );
                assert_eq!(
                    declared.count_basic_blocks(),
                    0,
                    "the C function `{}` has one definition",
                    self.function_name
                );
                declared
            }
            None => gc.module.add_function(&self.function_name, func_ty, None),
        };
        assert_eq!(
            func.get_name().to_str().unwrap(),
            self.function_name,
            "the exported function enters the module under the C name it was given"
        );
        if let Some(tycon) = codom.toplevel_tycon() {
            gc.set_c_integer_extension_attribute(func, AttributeLoc::Return, &tycon);
        }
        for (i, dom) in doms.iter().enumerate() {
            if let Some(tycon) = dom.toplevel_tycon() {
                gc.set_c_integer_extension_attribute(func, AttributeLoc::Param(i as u32), &tycon);
            }
        }

        // Implement the function.
        let bb = gc.context.append_basic_block(func, "entry");
        gc.builder().position_at_end(bb);

        // Create Fix values from arguments. Each parameter is the value's one scalar.
        let params = func.get_params();
        let mut args = params
            .iter()
            .enumerate()
            .map(|(i, arg)| Object::from_parts(vec![*arg], doms[i].clone(), gc))
            .collect::<Vec<_>>();

        // Get the Fix value to be exported. `value_expr` is a reference to the instantiated symbol
        // (see `instantiate_exported_value`), which the RC-IR back end has already implemented;
        // materialize that symbol's object here.
        let fix_expr = self.value_expr.clone().unwrap();
        let fix_name = fix_expr.get_var().name.clone();
        let mut fix_value = gc.get_scoped_obj(&fix_name);

        // Pass the arguments to the Fix value.
        match io_type {
            IOType::Pure => {}
            IOType::IO => {}
            IOType::IOState => {
                let iostate = create_obj(make_iostate_ty(), &vec![], None, gc, Some("iostate"));
                args.push(iostate);
            }
        }
        while args.len() > 0 {
            let arity = fix_value.ty.get_lambda_srcs().len();
            let rest = args.split_off(arity);
            fix_value = gc.apply_lambda(fix_value, args, false).unwrap();
            args = rest;
        }
        match io_type {
            IOType::Pure => {}
            IOType::IO => {
                fix_value = run_io(gc, &fix_value);
            }
            IOType::IOState => {
                fix_value =
                    ObjectFieldType::get_struct_fields(gc, &fix_value, &[1], RcState::Unknown)[0]
                        .clone();
            }
        }

        // Return the result as its one scalar.
        if codom.is_unit() {
            gc.builder().build_return(None).unwrap();
        } else {
            let ret_val = fix_value.parts()[0];
            gc.builder().build_return(Some(&ret_val)).unwrap();
        }
    }
}

/// The LLVM type an exported function exchanges a value of `ty` as: the value's one scalar,
/// which is the type a C declaration of the same function names.
fn c_scalar_type<'c, 'm>(ty: &Arc<TypeNode>, gc: &mut Generator<'c, 'm>) -> BasicTypeEnum<'c> {
    let embedded_ty = ty.get_embedded_type(gc);
    let parts = gc.type_parts(embedded_ty);
    // The one part has to be the scalar itself. Counting the parts alone would let an aggregate
    // through, since a value too wide to split is carried as one part holding the whole of it, and
    // C would then be handed a structure whose layout it classifies by its own rules.
    let is_scalar = parts.len() == 1
        && !matches!(
            parts[0],
            BasicTypeEnum::StructType(_)
                | BasicTypeEnum::ArrayType(_)
                | BasicTypeEnum::VectorType(_)
        );
    assert!(
        is_scalar,
        "`{}` reached an exported signature, where a value has to be one scalar",
        ty.to_string()
    );
    parts[0]
}

/// Whether a value of `ty` reaches C the way the C ABI says a value of the corresponding C type is
/// passed.
///
/// A value with one scalar — an integer, a floating point number, or a pointer — is laid down
/// identically by Fix and by C. An aggregate is laid down differently: the C ABI classifies a
/// structure by its size and by the class of each of its eightbytes (System V AMD64), or by whether
/// it is a homogeneous floating-point aggregate (AAPCS64), and the shapes on which that agrees with
/// Fix's element-wise layout differ from target to target.
fn has_c_abi(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> bool {
    let tycon = match ty.toplevel_tycon() {
        Some(tycon) => tycon,
        None => return false,
    };
    // A boxed value is a pointer.
    if ty.is_box(type_env) {
        return true;
    }
    tycon.is_c_scalar()
}

/// The error message for a type the C ABI cannot carry appearing in an exported function's
/// signature.
///
/// # Arguments
/// * `position` — where the type appears, phrased to follow "cannot be used as": "an argument" or
///   "the return value".
fn unexportable_type_msg(ty: &Arc<TypeNode>, position: &str) -> String {
    let head = format!(
        "`{}` cannot be used as {} of an exported function",
        ty.to_string(),
        position
    );
    if ty.is_boolean() {
        return head + ". Use `U8` or `CInt`, and convert it on the Fix side.";
    }
    head + ". An exported function can exchange scalar values: integers (`I8` to `I64`, `U8` to `U64`), floating point numbers (`F32`, `F64`), and pointers (`Ptr`, and boxed values, which cross as an opaque pointer). The C types in `Std::FFI` such as `CInt` are aliases of these. To exchange a struct, take a `Ptr` to memory the foreign side owns and copy through it with `memcpy`; `Std::FFI::borrow_boxed` and `mutate_boxed` give a pointer to the payload of a boxed value, and `Std::Array::borrow_elements` and `mutate_elements` a pointer to an array's elements."
}

/// The type of a C function, as the Fix type constructors standing for the C types it exchanges.
///
/// A program describes a C function in two ways: an `FFI_CALL` declares one it calls, and an
/// `FFI_EXPORT` defines one it offers. Both descriptions of one name reach the module as a single
/// function, through which every call in the program goes, so they have to give one signature.
pub struct CSignature {
    /// The type of each parameter, in the order the C function takes them.
    pub params: Vec<Arc<TyCon>>,
    /// The type of the result, the unit type where the C function returns nothing.
    pub ret: Arc<TyCon>,
    /// Whether the signature ends in `...`.
    pub is_var_args: bool,
}

impl CSignature {
    /// The signature an `FFI_CALL` writes for the function it calls.
    pub fn of_ffi_call(
        ret: &Arc<TyCon>,
        params: &Vec<Arc<TyCon>>,
        is_var_args: bool,
    ) -> CSignature {
        CSignature {
            params: params.clone(),
            ret: ret.clone(),
            is_var_args,
        }
    }

    /// The signature of the C function generated for a value exported at `ty`.
    pub fn of_exported(ty: &ExportedFunctionType, type_env: &TypeEnv) -> CSignature {
        CSignature {
            params: ty
                .doms
                .iter()
                .map(|dom| c_boundary_tycon(dom, type_env))
                .collect(),
            ret: c_boundary_tycon(&ty.codom, type_env),
            is_var_args: false,
        }
    }

    /// Whether this signature and `other` declare the same C function.
    pub fn agrees_with(&self, other: &CSignature) -> bool {
        self.is_var_args == other.is_var_args
            && self.params.len() == other.params.len()
            && self.ret.c_type_shape() == other.ret.c_type_shape()
            && (self.params.iter())
                .zip(other.params.iter())
                .all(|(a, b)| a.c_type_shape() == b.c_type_shape())
    }

    /// The signature as a C declaration of a nameless function reads, with each C type written as
    /// the Fix type constructor standing for it.
    pub fn to_string(&self) -> String {
        let mut params = self
            .params
            .iter()
            .map(|param| param.to_string())
            .collect::<Vec<_>>();
        if self.is_var_args {
            params.push("...".to_string());
        }
        format!("{} ({})", self.ret.to_string(), params.join(", "))
    }
}

/// The Fix type constructor standing for the C type a value of `ty` crosses the FFI boundary as: a
/// boxed value crosses as a pointer, and every other exportable value as its own type constructor.
fn c_boundary_tycon(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Arc<TyCon> {
    if ty.is_box(type_env) {
        return tycon(FullName::from_strs(&[STD_NAME], PTR_NAME));
    }
    ty.toplevel_tycon()
        .expect("`has_c_abi` admits only a type with a type constructor at its head")
}

// The type of an exported Fix value, split into the parts the generated C function is built from.
// The value has type `{doms} -> {codom}` when `io_type` is `Pure`, `{doms} -> IO {codom}` when it
// is `IO`, and `{doms} -> IOState -> (IOState, {codom})` when it is `IOState`.
#[derive(Clone)]
pub struct ExportedFunctionType {
    // The types of the arguments, in the order the C function takes them.
    pub doms: Vec<Arc<TypeNode>>,
    // The type of the result, with the `IO` wrapper or the `IOState` threading taken off.
    pub codom: Arc<TypeNode>,
    // How the value produces a result of type `codom`.
    pub io_type: IOType,
}

// How an exported Fix value produces its result.
#[derive(Clone)]
pub enum IOType {
    // The value is the result itself.
    Pure,
    // The value is an `IO` action, which the generated C function runs.
    IO,
    // The value takes an `IOState` token and returns it alongside the result. An exported value is
    // written as `IO {codom}`; an optimization may rewrite it into this form.
    IOState,
}

impl ExportedFunctionType {
    // Check if a type is valid for a value which is exported.
    // - src: Used for error messages.
    pub fn validate(
        scm: Arc<Scheme>,
        type_env: &TypeEnv,
        err_msg_prefix: String,
        src: &Option<Span>,
    ) -> Result<ExportedFunctionType, Errors> {
        // The scheme should have no constraints.
        if scm.to_string_normalize() != scm.ty.to_string() {
            return Err(Errors::from_msg_srcs(
                err_msg_prefix + "the type of an exported value should not have any constraints.",
                &[src],
            ));
        }

        let ty = scm.ty.clone();

        // The type cannot contain any type variables.
        if ty.free_vars_vec().len() > 0 {
            return Err(Errors::from_msg_srcs(
                err_msg_prefix
                    + "the type of an exported value should not contain any type variables.",
                &[src],
            ));
        }

        // Resolve type aliases in `ty`.
        let ty = ty.resolve_type_aliases(type_env)?;

        // Split the type `A1 -> A2 -> ... -> An -> B` into `([A1, A2, ..., An], B)`.
        let (doms, mut codom) = ty.collect_app_src(usize::MAX);

        // If `B` is `IO C`, then replace `B` with `C` and set `io_type` to `IO`.
        let mut io_type = IOType::Pure;
        match &codom.ty {
            Type::TyApp(fun, arg) => {
                if fun.is_io() {
                    codom = arg.clone();
                    io_type = IOType::IO;
                }
            }
            _ => {}
        }

        // Each argument and the result should have a C ABI.
        for dom in &doms {
            if !has_c_abi(dom, type_env) {
                return Err(Errors::from_msg_srcs(
                    err_msg_prefix + &unexportable_type_msg(dom, "an argument"),
                    &[src],
                ));
            }
        }
        if !codom.is_unit() && !has_c_abi(&codom, type_env) {
            return Err(Errors::from_msg_srcs(
                err_msg_prefix + &unexportable_type_msg(&codom, "the return value"),
                &[src],
            ));
        }

        // Return the result.
        Result::Ok(ExportedFunctionType {
            doms,
            codom,
            io_type,
        })
    }
}
