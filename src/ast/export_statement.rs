// Export syntax: `FFI_EXPORT[fix_value_name, c_functio_name];`

use crate::ast::expr::ExprNode;
use crate::ast::name::{FullName, NameSpace};
use crate::ast::program::TypeEnv;
use crate::ast::types::Scheme;
use crate::ast::types::{Type, TypeNode};
use crate::constants::{
    F32_NAME, F64_NAME, I16_NAME, I32_NAME, I64_NAME, I8_NAME, PTR_NAME, STD_NAME, U16_NAME,
    U32_NAME, U64_NAME, U8_NAME,
};
use crate::error::Errors;
use crate::fixstd::builtin::{make_io_ty, make_iostate_ty, make_unit_ty, run_io};
use crate::generator::Generator;
use crate::generator::Object;
use crate::object::create_obj;
use crate::object::ObjectFieldType;
use crate::parse::sourcefile::Span;
use inkwell::types::BasicType;
use std::sync::Arc;
use std::usize;

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

    // Implement the exported c function.
    // This function requires `self.exported_function_type` and `self.instantiated_value_expr` to already be set.
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
        let func_ty = if is_unit_type(&codom) {
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
        if is_unit_type(&codom) {
            gc.builder().build_return(None).unwrap();
        } else {
            let ret_val = fix_value.value(gc);
            gc.builder().build_return(Some(&ret_val)).unwrap();
        }
    }
}

// The unboxed types an exported function can exchange with C. Each of them is one integer, one
// floating point number, or one pointer, which C and Fix both pass in a single register. A boxed
// type is exchangeable as well; `is_exportable_type` admits it separately.
const EXPORTABLE_UNBOXED_TYPE_NAMES: &[&str] = &[
    I8_NAME, I16_NAME, I32_NAME, I64_NAME, U8_NAME, U16_NAME, U32_NAME, U64_NAME, F32_NAME,
    F64_NAME, PTR_NAME,
];

// Whether the C ABI passes a value of `ty` the way `ExportStatement::implement` passes it.
//
// `implement` passes every argument and the result by value in the LLVM type Fix uses internally,
// and LLVM assigns registers to an aggregate element by element. The C ABI instead classifies a
// structure by its size and by the class of each of its eightbytes (System V AMD64), or by whether
// it is a homogeneous floating-point aggregate (AAPCS64). The two agree only where the shape
// happens to line up, and the shapes that line up differ between targets, so an aggregate is
// exportable on neither. A scalar and a pointer are laid down identically by both.
fn is_exportable_type(ty: &Arc<TypeNode>, type_env: &TypeEnv) -> bool {
    let tycon = match ty.toplevel_tycon() {
        Some(tycon) => tycon,
        None => return false,
    };
    // A boxed value is a pointer.
    if ty.is_box(type_env) {
        return true;
    }
    tycon.name.namespace == NameSpace::new_str(&[STD_NAME])
        && EXPORTABLE_UNBOXED_TYPE_NAMES.contains(&tycon.name.name.as_str())
}

// The message shown when `ty` is used as `position` ("an argument" / "the return value") of an
// exported function although `is_exportable_type` rejects it.
fn unexportable_type_msg(ty: &Arc<TypeNode>, position: &str) -> String {
    let head = format!(
        "`{}` cannot be used as {} of an exported function",
        ty.to_string(),
        position
    );
    if ty
        .toplevel_tycon()
        .map_or(false, |tycon| tycon.is_boolean())
    {
        return head + ", because the width of `_Bool` in C is implementation-defined. Use `U8` or `CInt`, and convert it on the Fix side.";
    }
    head + ". An exported function can exchange integers (`I8` to `I64`, `U8` to `U64`), floating point numbers (`F32`, `F64`), `Ptr`, and boxed values; the C types in `Std::FFI` such as `CInt` are aliases of these. To exchange a struct, take a `Ptr` to a region the foreign side owns and copy through it by `FFI_CALL[Ptr memcpy(Ptr, Ptr, U64), ...]`."
}

// Whether `ty` is the unit type `()`.
fn is_unit_type(ty: &Arc<TypeNode>) -> bool {
    ty.to_string() == make_unit_ty().to_string()
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

        // Each argument and the result should have a C ABI.
        for dom in &doms {
            if !is_exportable_type(dom, type_env) {
                return Err(Errors::from_msg_srcs(
                    err_msg_prefix + &unexportable_type_msg(dom, "an argument"),
                    &[src],
                ));
            }
        }
        if !is_unit_type(&codom) && !is_exportable_type(&codom, type_env) {
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
