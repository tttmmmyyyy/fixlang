// Export syntax: `EXPORT[fix_value_name, c_functio_name];`

use std::sync::Arc;
use std::usize;

use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::TypeNode;
use crate::ast::Type;
use crate::misc::error_exit_with_src;
use crate::sourcefile::Span;

use super::{make_io_ty, make_unit_ty};

#[derive(Clone)]
pub struct ExportStatement {
    pub fix_value_name: FullName,
    pub c_function_name: String,
    pub src: Option<Span>,
}

impl ExportStatement {
    // Validate the export statement.
    pub fn validate(&self) {
        // If `c_function_name` is not a valid C function name, exit with error
        // The first character should be a letter or an underscore
        // The rest of the characters should be a letter, a digit or an underscore
        if !self.c_function_name.chars().next().unwrap().is_alphabetic()
            && self.c_function_name.chars().next().unwrap() != '_'
        {
            let msg = format!(
                "{} is not a valid C function name. The first character should be a letter or an underscore.",
                &self.c_function_name
            );
            error_exit_with_src(&msg, &self.src);
        }
        for c in self.c_function_name.chars() {
            if !c.is_alphanumeric() && c != '_' {
                let msg = format!(
                    "{} is not a valid C function name. The rest of the characters should be a letter, a digit or an underscore.",
                    &self.c_function_name
                );
                error_exit_with_src(&msg, &self.src);
            }
        }
    }

    // Check if a type is valid for a value which is exported.
    pub fn validate_type(
        ty: Arc<TypeNode>,
        type_env: &TypeEnv,
    ) -> Result<ExportFunctionType, String> {
        // Resolve type aliases in `ty`.
        let ty = ty.resolve_type_aliases(type_env);

        // The type cannot contain any type variables.
        if ty.free_vars_vec().len() > 0 {
            return Result::Err(
                "the type of the exported value should not contain any type variables.".to_string(),
            );
        }

        // Split the type `A1 -> A2 -> ... -> An -> B` into `([A1, A2, ..., An], C)`.
        let (doms, mut codom) = ty.collect_app_src(usize::MAX);

        // n should be greater than 0.
        if doms.len() == 0 {
            return Result::Err("the exported value should be a function value.".to_string());
        }

        // If the unit type `()` is in `doms`, then `n` should be 1.
        let unit_ty = make_unit_ty();
        if doms.iter().any(|ty| ty.to_string() == unit_ty.to_string()) {
            if doms.len() != 1 {
                return Result::Err(
                    "the unit type should not appear in the type of the exported value if the arguments are greater than 1.".to_string(),
                );
            }
        }

        // Each `Ai` should be fully unboxed and free from union.
        for dom in doms.iter() {
            if dom.is_fully_unboxed(type_env) {
                return Result::Err(
                    "the type of the value should be constructed without using a boxed type."
                        .to_string(),
                );
            }
            if !dom.is_free_from_union(type_env) {
                return Result::Err(
                    "the type of the value should be constructed without using a union type."
                        .to_string(),
                );
            }
        }

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
        if codom.is_fully_unboxed(type_env) {
            return Result::Err(
                "the type of the value should be constructed without using a boxed type."
                    .to_string(),
            );
        }
        if !codom.is_free_from_union(type_env) {
            return Result::Err(
                "the type of the value should be constructed without using a union type."
                    .to_string(),
            );
        }

        // Return the result.
        Result::Ok(ExportFunctionType { doms, codom, is_io })
    }
}

// A type to represent the type of a Fix value which is exported to C.
// This struct value reresents a type `{doms} -> {codom}` if `is_io` is `false`,
// and a type `{doms} -> IO {codom}` if `is_io` is `true`.
pub struct ExportFunctionType {
    pub doms: Vec<Arc<TypeNode>>,
    pub codom: Arc<TypeNode>,
    pub is_io: bool,
}
