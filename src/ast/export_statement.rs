// Export syntax: `FFI_EXPORT[fix_value_name, c_function_name];`

use crate::ast::expr::ExprNode;
use crate::ast::name::{FullName, Name};
use crate::ast::program::TypeEnv;
use crate::ast::types::Scheme;
use crate::ast::types::{tycon, TyCon, Type, TypeNode};
use crate::configuration::OutputFileType;
use crate::constants::{I32_NAME, PTR_NAME, STD_NAME};
use crate::error::Errors;
use crate::fixstd::builtin::{make_iostate_ty, run_io};
use crate::fixstd::runtime::compiler_defined_c_function_reason;
use crate::generator::Generator;
use crate::generator::Object;
use crate::object::create_obj;
use crate::object::ObjectFieldType;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::RcState;
use inkwell::attributes::AttributeLoc;
use inkwell::types::{BasicMetadataTypeEnum, BasicType};
use inkwell::values::FunctionValue;
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
    /// * `output` — what is being built, which decides the names the compiler writes itself.
    pub fn validate_names(&self, src: &Option<Span>, output: OutputFileType) -> Result<(), Errors> {
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
        // An export writes the function's body, so a name the compiler writes a body under is out.
        if let Some(reason) = compiler_defined_c_function_reason(&self.function_name, output) {
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
        let function_type = self.function_type.as_ref().unwrap();
        let ExportedFunctionType {
            doms,
            codom,
            io_type,
        } = function_type.clone();

        // Take the name. An `FFI_CALL` of this C function has declared it by now — code generation
        // implements the program's symbols before it reaches here — and a declaration and this
        // definition describe one C function, so the body goes onto that declaration.
        let signature = CSignature::of_ffi_export(function_type, gc.type_env());
        let func = signature.get_or_declare_in_module(&self.function_name, gc);
        assert_eq!(
            func.count_basic_blocks(),
            0,
            "the C function `{}` has one definition",
            self.function_name
        );

        // Each value the function exchanges travels in its Fix representation, and the C type the
        // signature gave it names that same representation.
        for (param_ty, dom) in signature.param_tys.iter().zip(doms.iter()) {
            assert_crosses_as_c_type(param_ty, dom, gc);
        }
        if signature.ret_tycon.get_c_type(gc.context).is_some() {
            assert_crosses_as_c_type(&signature.ret_tycon, &codom, gc);
        }

        // Implement the function.
        let bb = gc.context.append_basic_block(func, "entry");
        gc.builder().position_at_end(bb);

        // Create Fix values from arguments. Each parameter is the value's one scalar.
        let param_vals = func.get_params();
        let mut args = param_vals
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

/// Assert that a value of Fix type `ty` travels as the one scalar the C type `c_ty` names, which is
/// what lets the generated function hand it to C as it stands.
///
/// Counting the parts alone would let an aggregate through, since a value too wide to split is
/// carried as one part holding the whole of it, and C would then be handed a structure whose layout
/// it classifies by its own rules. `c_boundary_tycon` admits nothing with either shape.
fn assert_crosses_as_c_type<'c, 'm>(
    c_ty: &Arc<TyCon>,
    ty: &Arc<TypeNode>,
    gc: &mut Generator<'c, 'm>,
) {
    let embedded_ty = ty.get_embedded_type(gc);
    let parts = gc.type_parts(embedded_ty);
    assert_eq!(
        parts.len(),
        1,
        "`{}` reached an exported signature, where a value has to be one scalar",
        ty.to_string()
    );
    assert_eq!(
        parts[0],
        c_ty.get_c_type(gc.context).unwrap(),
        "`{}` crosses as the C type `{}`, so the two lay it down alike",
        ty.to_string(),
        c_ty.to_string()
    );
}

/// The C type a value of `ty` crosses the FFI boundary as, and `None` for a value the C ABI cannot
/// carry the way Fix lays it down.
///
/// A value with one scalar — an integer, a floating point number, or a pointer — is laid down
/// identically by Fix and by C, and a boxed value is a pointer. An aggregate is laid down
/// differently: the C ABI classifies a structure by its size and by the class of each of its
/// eightbytes (System V AMD64), or by whether it is a homogeneous floating-point aggregate
/// (AAPCS64), and the shapes on which that agrees with Fix's element-wise layout differ from target
/// to target.
fn c_boundary_tycon(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> Option<Arc<TyCon>> {
    let head = ty.toplevel_tycon()?;
    if ty.is_box(type_env) {
        return Some(tycon(FullName::from_strs(&[STD_NAME], PTR_NAME)));
    }
    if !head.is_c_scalar() {
        return None;
    }
    Some(head)
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
    pub param_tys: Vec<Arc<TyCon>>,
    /// The type of the result, the unit type where the C function returns nothing.
    pub ret_tycon: Arc<TyCon>,
    /// Whether the signature ends in `...`.
    pub is_var_args: bool,
}

/// The signature of the entry point the compiler generates: `int32_t (int32_t, void *)`, taking
/// `argc` and `argv` as the C runtime passes them.
///
/// The compiler writes this function's body, so it is the one C function a program describes without
/// naming it. A program that does name it — an `FFI_CALL` of `main`, which re-runs the program —
/// gives this signature, and `Program::validate_c_function_calls` reports one that gives another.
pub fn c_entry_point_signature() -> CSignature {
    let std_tycon = |name: &str| tycon(FullName::from_strs(&[STD_NAME], name));
    CSignature {
        param_tys: vec![std_tycon(I32_NAME), std_tycon(PTR_NAME)],
        ret_tycon: std_tycon(I32_NAME),
        is_var_args: false,
    }
}

impl CSignature {
    /// The signature an `FFI_CALL` writes for the function it calls.
    pub fn of_ffi_call(
        ret_tycon: &Arc<TyCon>,
        param_tys: &Vec<Arc<TyCon>>,
        is_var_args: bool,
    ) -> CSignature {
        CSignature {
            param_tys: param_tys.clone(),
            ret_tycon: ret_tycon.clone(),
            is_var_args,
        }
    }

    /// The signature of the C function generated for a value exported at `exported_ty`, whose every
    /// position `ExportedFunctionType::validate` has admitted as one the C ABI can carry.
    pub fn of_ffi_export(exported_ty: &ExportedFunctionType, type_env: &TypeEnv) -> CSignature {
        let boundary_tycon = |ty: &Arc<TypeNode>| {
            c_boundary_tycon(ty, type_env)
                .unwrap_or_else(|| panic!("`{}` reached an exported signature", ty.to_string()))
        };
        CSignature {
            param_tys: exported_ty.doms.iter().map(boundary_tycon).collect(),
            ret_tycon: if exported_ty.codom.is_unit() {
                exported_ty.codom.toplevel_tycon().unwrap()
            } else {
                boundary_tycon(&exported_ty.codom)
            },
            is_var_args: false,
        }
    }

    /// Whether this signature and `other` declare the same C function: the two describe every
    /// position the same way, down to what a declaration of it carries — `CTypeShape` holds which
    /// differences between two Fix types that is, and which it is not.
    pub fn agrees_with(&self, other: &CSignature) -> bool {
        self.is_var_args == other.is_var_args
            && self.param_tys.len() == other.param_tys.len()
            && self.ret_tycon.c_type_shape() == other.ret_tycon.c_type_shape()
            && (self.param_tys.iter())
                .zip(other.param_tys.iter())
                .all(|(a, b)| a.c_type_shape() == b.c_type_shape())
    }

    /// The function `name` of this signature in the module, declaring it where nothing declares it
    /// yet. Every description of one C name goes through here, which is what puts the calls a
    /// program makes and the definition it exports on one function.
    pub fn get_or_declare_in_module<'c, 'm>(
        &self,
        name: &Name,
        gc: &Generator<'c, 'm>,
    ) -> FunctionValue<'c> {
        if let Some(declared) = gc.module.get_function(name) {
            return declared;
        }
        let param_c_tys = self
            .param_tys
            .iter()
            .map(|param_ty| {
                // A parameter of type `()` is rejected where the signature is written: `void` is a
                // result alone.
                param_ty.get_c_type(gc.context).unwrap().into()
            })
            .collect::<Vec<BasicMetadataTypeEnum>>();
        let fn_ty = match self.ret_tycon.get_c_type(gc.context) {
            None => gc
                .context
                .void_type()
                .fn_type(&param_c_tys, self.is_var_args),
            Some(ret_c_ty) => ret_c_ty.fn_type(&param_c_tys, self.is_var_args),
        };
        let func = gc.module.add_function(name, fn_ty, None);
        assert_eq!(
            func.get_name().to_str().unwrap(),
            name,
            "the C function enters the module under the name it was given"
        );
        gc.add_c_integer_extension_attribute(func, AttributeLoc::Return, &self.ret_tycon);
        for (i, param_ty) in self.param_tys.iter().enumerate() {
            gc.add_c_integer_extension_attribute(func, AttributeLoc::Param(i as u32), param_ty);
        }
        func
    }

    /// The signature as a C declaration of the function `name` reads.
    ///
    /// # Examples
    /// A function of two arguments reads `int32_t c_pick(int32_t, int32_t)`, one of none
    /// `void c_now(void)`, and a variadic one `int32_t c_report(void *, ...)`.
    pub fn declaration_of(&self, name: &Name) -> String {
        let mut params = self
            .param_tys
            .iter()
            .map(|param_ty| param_ty.c_type_name())
            .collect::<Vec<_>>();
        if self.is_var_args {
            params.push("...".to_string());
        }
        // C writes the empty parameter list of a declaration as `(void)`.
        if params.is_empty() {
            params.push("void".to_string());
        }
        format!(
            "{} {}({})",
            self.ret_tycon.c_type_name(),
            name,
            params.join(", ")
        )
    }
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

        // Each argument and the result should be a type C can carry.
        for dom in &doms {
            if c_boundary_tycon(dom, type_env).is_none() {
                return Err(Errors::from_msg_srcs(
                    err_msg_prefix + &unexportable_type_msg(dom, "an argument"),
                    &[src],
                ));
            }
        }
        if !codom.is_unit() && c_boundary_tycon(&codom, type_env).is_none() {
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
