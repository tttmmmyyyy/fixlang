// Export syntax: `FFI_EXPORT[fix_value_name, c_function_name];`

use crate::ast::expr::ExprNode;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::Scheme;
use crate::ast::types::{Type, TypeNode};
use crate::configuration::OutputFileType;
use crate::error::Errors;
use crate::ffi::{assert_crosses_as_c_type, c_boundary_tycon, CSignature};
use crate::fixstd::builtin::{make_iostate_ty, run_io};
use crate::fixstd::runtime::compiler_defined_c_function_reason;
use crate::generator::Generator;
use crate::generator::Object;
use crate::object::create_obj;
use crate::object::ObjectFieldType;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::RcState;
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
    // PROOF: P27 (dev-docs/proof/rc_ir/borrow-cancel)
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
