// Export syntax: `FFI_EXPORT[fix_value_name, c_functio_name];`

use std::sync::Arc;
use std::usize;

use inkwell::types::BasicType;

use crate::ast::expr::ExprNode;
use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::type_funptr;
use crate::ast::type_tyapp;
use crate::ast::types::Scheme;
use crate::ast::types::TypeNode;
use crate::ast::Type;
use crate::builtin::*;
use crate::generator::GenerationContext;
use crate::generator::Object;
use crate::sourcefile::Span;

use super::error::Errors;
use super::optimization::uncurry_optimization::convert_to_funptr_name;

#[derive(Clone)]
pub struct ExportStatement {
    pub fix_value_name: FullName,
    pub c_function_name: String,
    pub src: Option<Span>,
    pub exported_function_type: Option<ExportedFunctionType>, // `None` at first, and set by `ExportedFunctionType::validate`.
    pub instantiated_value_expr: Option<Arc<ExprNode>>, // `None` at first, and set after the fix value is instantiated.
}

impl ExportStatement {
    pub fn new(
        fix_value_name: FullName,
        c_function_name: String,
        src: Option<Span>,
    ) -> ExportStatement {
        ExportStatement {
            fix_value_name,
            c_function_name,
            src,
            exported_function_type: None,
            instantiated_value_expr: None,
        }
    }

    // Validate the names in the export statement.
    // - src: The source of the export statement. Used for error messages.
    pub fn validate_names(&self, src: &Option<Span>) -> Result<(), Errors> {
        // If `c_function_name` is not a valid C function name, exit with error
        // The first character should be a letter or an underscore
        // The rest of the characters should be a letter, a digit or an underscore
        if !self.c_function_name.chars().next().unwrap().is_alphabetic()
            && self.c_function_name.chars().next().unwrap() != '_'
        {
            let msg = format!(
                "`{}` is not a valid C function name. The first character should be a letter or an underscore.",
                &self.c_function_name
            );
            return Err(Errors::from_msg_srcs(msg, &vec![src]));
        }
        for c in self.c_function_name.chars() {
            if !c.is_alphanumeric() && c != '_' {
                let msg = format!(
                    "`{}` is not a valid C function name. The rest of the characters should be a letter, a digit or an underscore.",
                    &self.c_function_name
                );
                return Err(Errors::from_msg_srcs(msg, &vec![src]));
            }
        }
        Ok(())
    }

    // Implement the exported c function.
    // This function requires `self.exported_function_type` and `self.instantiated_value_expr` to already be set.
    pub fn implement<'c, 'm>(&self, gc: &mut GenerationContext<'c, 'm>) {
        let ExportedFunctionType { doms, codom, is_io } =
            self.exported_function_type.clone().unwrap();

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
        let func = gc.module.add_function(&self.c_function_name, func_ty, None);

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
                let arg_obj = Object::create_from_value(*arg, arg_ty, gc);
                arg_obj
            })
            .collect::<Vec<_>>();

        // If the exported Fix value is a function, then try to use the uncurried version.
        let mut fix_expr = self.instantiated_value_expr.clone().unwrap();
        if args.len() > 0 && gc.config.perform_uncurry_optimization() {
            let mut var = fix_expr.get_var().as_ref().clone();
            convert_to_funptr_name(&mut var.name.name, args.len());
            if gc.global.contains_key(&var.name) {
                let funptr_ty = self.exported_function_type.as_ref().unwrap().to_funptr_ty();
                fix_expr = fix_expr
                    .set_var_var(Arc::new(var))
                    .set_inferred_type(funptr_ty);
            }
        }

        // Get the Fix value to be exported.
        let mut fix_value = gc.eval_expr(fix_expr);

        // Pass the arguments to the Fix value.
        while args.len() > 0 {
            let arity = fix_value.ty.get_lambda_srcs().len();
            let rest = args.split_off(arity);
            fix_value = gc.apply_lambda(fix_value, args);
            args = rest;
        }

        // If the `fix_value` is `IO C`, then run it.
        if is_io {
            fix_value = run_io_value(gc, &fix_value);
        }

        // Return the result.
        if codom.to_string() == make_unit_ty().to_string() {
            gc.builder().build_return(None);
        } else {
            gc.builder().build_return(Some(&fix_value.value(gc)));
        }
    }
}

// A type to represent the type of a Fix value which is exported to C.
// This struct value reresents a type `{doms} -> {codom}` if `is_io` is `false`,
// and a type `{doms} -> IO {codom}` if `is_io` is `true`.
#[derive(Clone)]
pub struct ExportedFunctionType {
    pub doms: Vec<Arc<TypeNode>>,
    pub codom: Arc<TypeNode>,
    pub is_io: bool,
}

impl ExportedFunctionType {
    // Create the uncurried function pointer type of the exported Fix value.
    pub fn to_funptr_ty(&self) -> Arc<TypeNode> {
        assert!(self.doms.len() > 0); // The exported value should be a function.
        let mut ty = self.codom.clone();
        if self.is_io {
            ty = type_tyapp(make_io_ty(), ty);
        }
        type_funptr(self.doms.clone(), ty)
    }

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

        // The unit type `()` should not appear in the type of the exported value if the arguments are greater than 1.

        // // If the unit type `()` is in `doms`, then `n` should be 1.
        // let unit_ty = make_unit_ty();
        // if doms.iter().any(|ty| ty.to_string() == unit_ty.to_string()) {
        //     if doms.len() != 1 {
        //         return Result::Err(
        //             "the unit type should not appear in the type of the exported value if the arguments are greater than 1.".to_string(),
        //         );
        //     }
        // }

        // Each `Ai` should be fully unboxed and free from union.
        // for dom in doms.iter() {
        //     if !dom.is_fully_unboxed(type_env) {
        //         return Result::Err(
        //                 "the type of an exported value should be constructed without using any boxed type."
        //                     .to_string(),
        //             );
        //     }
        //     if !dom.is_free_from_union(type_env) {
        //         return Result::Err(
        //                 "the type of an exported value should be constructed without using any union type."
        //                     .to_string(),
        //             );
        //     }
        // }

        // If `B` is `IO C`, then replace `B` with `C` and set `is_io` to `true`.
        let mut is_io = false;
        match &codom.ty {
            Type::TyApp(fun, arg) => {
                if fun.to_string() == make_io_ty().to_string() {
                    codom = arg.clone();
                    is_io = true;
                }
            }
            _ => {}
        }

        // `B` should be fully unboxed and free from union.
        // if !codom.is_fully_unboxed(type_env) {
        //     return Result::Err(
        //         "the type of an exported value should be constructed without using any boxed type."
        //             .to_string(),
        //     );
        // }
        // if !codom.is_free_from_union(type_env) {
        //     return Result::Err(
        //         "the type of an exported value should be constructed without using any union type."
        //             .to_string(),
        //     );
        // }

        // Return the result.
        Result::Ok(ExportedFunctionType { doms, codom, is_io })
    }
}
