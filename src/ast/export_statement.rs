// Export syntax: `FFI_EXPORT[fix_value_name, c_functio_name];`

use std::sync::Arc;
use std::usize;

use inkwell::types::BasicType;

use crate::ast::expr::ExprNode;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::Scheme;
use crate::ast::types::TypeNode;
use crate::ast::Type;
use crate::builtin::*;
use crate::generator::GenerationContext;
use crate::generator::Object;
use crate::object::create_obj;
use crate::object::ObjectFieldType;
use crate::sourcefile::Span;

use super::error::Errors;

// The export statement.
#[derive(Clone)]
pub struct ExportStatement {
    // The name of the Fix value to be exported.
    // This is the name of the Fix value in the source code, and not the name of the symbol.
    // To get the name of the instantiated Fix value, use `self.instantiated_value_expr`.
    pub value_name: FullName,
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
    pub fn new(
        fix_value_name: FullName,
        c_function_name: String,
        src: Option<Span>,
    ) -> ExportStatement {
        ExportStatement {
            value_name: fix_value_name,
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

    // Implement the exported c function.
    // This function requires `self.exported_function_type` and `self.instantiated_value_expr` to already be set.
    pub fn implement<'c, 'm>(&self, gc: &mut GenerationContext<'c, 'm>) {
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
        let func_ty = if codom.to_string() == make_unit_ty().to_string() {
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

        // Get the Fix value to be exported.
        let fix_expr = self.value_expr.clone().unwrap();
        let mut fix_value = gc.eval_expr(fix_expr, false).unwrap();

        // Pass the arguments to the Fix value.
        match io_type {
            IOType::Pure => {}
            IOType::IO => {}
            IOType::IOState => {
                let ios = create_obj(make_iostate_ty(), &vec![], None, gc, Some("iostate"));
                args.push(ios);
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
        if codom.to_string() == make_unit_ty().to_string() {
            gc.builder().build_return(None).unwrap();
        } else {
            gc.builder().build_return(Some(&fix_value.value)).unwrap();
        }
    }
}

// A type to represent the type of an exported Fix value.
// This struct value reresents a type `{doms} -> {codom}` if `is_io` is `false`,
// and a type `{doms} -> IO {codom}` if `is_io` is `true`.
#[derive(Clone)]
pub struct ExportedFunctionType {
    pub doms: Vec<Arc<TypeNode>>,
    pub codom: Arc<TypeNode>,
    pub io_type: IOType,
}

// Pure, IO a or IOState -> (IOState, a).
#[derive(Clone)]
pub enum IOType {
    Pure,
    IO,
    IOState, // The user cannot export a function of this type, but optimization may convert `IO a` to `IOState -> (IOState, a)`.
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
                if fun.to_string() == make_io_ty().to_string() {
                    codom = arg.clone();
                    io_type = IOType::IO;
                }
            }
            _ => {}
        }

        // Return the result.
        Result::Ok(ExportedFunctionType {
            doms,
            codom,
            io_type,
        })
    }
}
