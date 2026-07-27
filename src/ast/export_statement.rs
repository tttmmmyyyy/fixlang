// Export syntax: `FFI_EXPORT[fix_value_name, c_function_name];`

use crate::ast::expr::ExprNode;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::Scheme;
use crate::ast::types::{Type, TypeNode};
use crate::error::Errors;
use crate::fixstd::builtin::{make_iostate_ty, run_io};
use crate::generator::Generator;
use crate::generator::Object;
use crate::object::create_obj;
use crate::object::ObjectFieldType;
use crate::parse::sourcefile::Span;
use inkwell::types::BasicType;
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

    // Validate the names in the export statement.
    // - src: The source of the export statement. Used for error messages.
    pub fn validate_names(&self, src: &Option<Span>) -> Result<(), Errors> {
        // If `c_function_name` is not a valid C function name, exit with error
        // The first character should be a letter or an underscore
        // The rest of the characters should be a letter, a digit or an underscore
        if !self.function_name.chars().next().unwrap().is_alphabetic()
            && self.function_name.chars().next().unwrap() != '_'
        {
            let msg = format!(
                "`{}` is not a valid C function name. The first character should be a letter or an underscore.",
                &self.function_name
            );
            return Err(Errors::from_msg_srcs(msg, &vec![src]));
        }
        for c in self.function_name.chars() {
            if !c.is_alphanumeric() && c != '_' {
                let msg = format!(
                    "`{}` is not a valid C function name. The rest of the characters should be a letter, a digit or an underscore.",
                    &self.function_name
                );
                return Err(Errors::from_msg_srcs(msg, &vec![src]));
            }
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

        // Create the LLVM type of the exported C function.
        let dom_llvm_tys = doms
            .iter()
            .map(|dom| dom.get_embedded_type(gc, &vec![]).into())
            .collect::<Vec<_>>();
        let func_ty = if codom.is_unit() {
            gc.context.void_type().fn_type(&dom_llvm_tys, false)
        } else {
            codom
                .get_embedded_type(gc, &vec![])
                .fn_type(&dom_llvm_tys, false)
        };

        // Declare the function.
        let func = gc.module.add_function(&self.function_name, func_ty, None);

        // Implement the function.
        let bb = gc.context.append_basic_block(func, "entry");
        gc.builder().position_at_end(bb);

        // Create Fix values from arguments.
        let mut args = func
            .get_params()
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                let arg_ty = doms[i].clone();
                Object::new(*arg, arg_ty, gc)
            })
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
                fix_value = ObjectFieldType::get_struct_fields(gc, &fix_value, &[1])[0].clone();
            }
        }

        // Return the result.
        if codom.is_unit() {
            gc.builder().build_return(None).unwrap();
        } else {
            let ret_val = fix_value.value(gc);
            gc.builder().build_return(Some(&ret_val)).unwrap();
        }
    }
}

// Whether a value of `ty` reaches C the way the C ABI says a value of the corresponding C type is
// passed.
//
// The wrapper generated for an exported function passes every argument and the result by value in
// the LLVM type Fix uses internally, and LLVM assigns registers to an aggregate element by element.
// The C ABI instead classifies a structure by its size and by the class of each of its eightbytes
// (System V AMD64), or by whether it is a homogeneous floating-point aggregate (AAPCS64). The
// shapes on which the two agree differ from target to target, so an aggregate is exportable on
// none of them. A scalar and a pointer are laid down identically by both.
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

// The error message for a type the C ABI cannot carry appearing in an exported function's
// signature. `position` names where it appears, as "an argument" or "the return value".
fn unexportable_type_msg(ty: &Arc<TypeNode>, position: &str) -> String {
    let head = format!(
        "`{}` cannot be used as {} of an exported function",
        ty.to_string(),
        position
    );
    if ty.is_boolean() {
        return head + ", because the width of `_Bool` in C is implementation-defined. Use `U8` or `CInt`, and convert it on the Fix side.";
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

        // Split the type `A1 -> A2 -> ... -> An -> B` into `([A1, A2, ..., An], C)`.
        let (doms, mut codom) = ty.collect_app_src(usize::MAX);

        // If `B` is `IO C`, then replace `B` with `C` and set `is_io` to `true`.
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
