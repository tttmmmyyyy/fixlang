use super::*;

// Borrowing optimization.
// Consider an application `f(x)`, where `f` is a function which takes the ownership of `x` but releases it in its body.
// Assume that the expression `x` is a variable, i.e., it is not a temporary value.
// Furthermore, assume that the variable `x` is used after `f(x)`, i.e., `x` is not released after `f(x)`.
// Borrowing optimization replaces `f(x)` to `f1(x1)`, where
// - `f1` is a the same function as `f` but does not release `x`.
// - `x1` is a borrowed (i.e., not retained) value for the variable `x`.

// Perform borrowing optimization.
pub fn borrowing_optimization(program: &mut Program) {
    // Define borrowing versions of each function if possible.
    define_borrowing_functions(program);

    // NOTE: Replacement of call expressions is handled in `borrowing_optimization_evaluating_application` which is called from `Generator::eval_app`.
}

// This function converts a name of a function to the name of a same function but it only borrows its argument (i.e., does not release the argument in its body).
pub fn convert_to_borrowing_function_name(name: &mut FullName, mut borrowed_vars: Vec<FullName>) {
    borrowed_vars.sort();

    let name = name.name_as_mut();
    *name = name.clone()
        + "#borrowing_"
        + &borrowed_vars
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("_");
}

// Creates a borrowing version of a function expression.
// Returns None if the borrowing version cannot be created.
fn create_borrowing_function(
    function: &ExprNode,
    borrowed_args: Vec<FullName>,
) -> Option<Rc<ExprNode>> {
    let body = function.get_lam_body();
    // Currently, we handle only when `expr_body` is InlineLLVM.
    if !body.is_llvm() {
        return None;
    }
    Some(body.set_llvm_borrowed_vars(borrowed_args))
}

// Adds a borrowing version of functions in a program.
pub fn define_borrowing_functions(program: &mut Program) {
    let mut new_functions: HashMap<FullName, InstantiatedSymbol> = Default::default();
    for (sym_name, sym) in &program.instantiated_global_symbols {
        let expr = sym.expr.as_ref().unwrap();
        let expr_body = expr.get_lam_body();
        // Currently, we handle only when `expr_body` is InlineLLVM.
        if !expr_body.is_llvm() {
            continue;
        }
        let llvm = expr_body.get_llvm();
        let released_vars = llvm.generator.released_vars();
        // If `released_vars` is not provided by the generator, we do not add borrowing function for it.
        if released_vars.is_none() {
            continue;
        }
        // For any subsequences of `released_vars`, try to define a borrowing version of the function.
        let released_vars = released_vars.unwrap();
        for borrowed_vars in nonempty_subsequences(&released_vars) {
            let expr = create_borrowing_function(expr, borrowed_vars.clone());
            if expr.is_none() {
                continue;
            }
            let expr = expr.unwrap();
            let mut name = sym_name.clone();
            convert_to_borrowing_function_name(&mut name, borrowed_vars);
            new_functions.insert(
                name.clone(),
                InstantiatedSymbol {
                    template_name: FullName::local(&format!(
                        "{} created by borrowing optimization from {}",
                        name.to_string(),
                        sym_name.to_string()
                    )),
                    ty: sym.ty.clone(),
                    expr: Some(expr),
                    type_resolver: sym.type_resolver.clone(),
                },
            );
        }
    }
    program.instantiated_global_symbols.extend(new_functions);
}

pub fn borrowing_optimization_evaluating_application(
    gc: &mut GenerationContext,
    fun: Rc<ExprNode>,
    args: &Vec<Rc<ExprNode>>,
) -> Option<(Rc<ExprNode>, Vec<usize>)> {
    // If the function is not a variable, we do not perform borrowing optimization.
    if !fun.is_var() {
        return None;
    }

    // Emulate the state of used_later values at the time when the argument is evaluated.
    for arg in args {
        gc.scope_lock_as_used_later(arg.free_vars());
    }

    // Get a list of arguments which CAN be borrowed here.
    // That is, arguments which are variables and are also used after evaluating the argument.
    let mut borrowable_args_indices: Vec<usize> = Default::default();
    for (i, arg) in args.iter().enumerate() {
        // Emulate the state of used_later values at the time when the argument is evaluated.
        gc.scope_unlock_as_used_later(arg.free_vars());
        // Handle only local variables.
        if !arg.is_var() {
            continue;
        }
        let var_name = arg.get_var().name.clone();
        if !var_name.is_local() || gc.is_var_used_later(&var_name) {
            borrowable_args_indices.push(i);
        }
    }

    // Get a list of parameters which are released in the function body.
    let (params, body) = fun.destructure_lam();
    assert_eq!(args.len(), params.len());
    let released_params = body.released_vars();
    if released_params.is_none() {
        return None; // We do not perform borrowing optimization.
    }

    // Get a list of parameters which are released in the function body.
    let released_params = released_params.unwrap();
    let released_args_indices = (0..params.len())
        .filter(|i| released_params.contains(&params[*i].name))
        .collect::<Vec<_>>();

    // Get a list of arguments which SHOULD be borrowed here.
    // Filter out arguments in `borrowed_args_indices` which are not released by the function.
    let borrowed_args_indices = borrowable_args_indices
        .iter()
        .cloned()
        .filter(|i| released_args_indices.contains(i))
        .collect::<Vec<_>>();

    // Get borrowing version of the function.
    let borrowed_params = borrowed_args_indices
        .iter()
        .map(|i| params[*i].name.clone())
        .collect::<Vec<_>>();
    let mut borrowing_fun_name = fun.get_var().name.clone();
    convert_to_borrowing_function_name(&mut borrowing_fun_name, borrowed_params);

    // Get the borrowing version of the function is defined.
    if !gc.global.contains_key(&borrowing_fun_name) {
        return None;
    }

    // Return the borrowing version of the given function and the list of arguments which should be borrowed.
    let borrowing_fun_var = fun.get_var().set_name(borrowing_fun_name);
    let borrowing_fun = fun.set_var_var(borrowing_fun_var);
    Some((borrowing_fun, borrowed_args_indices))
}

// // Perform borrowing optimization for an expression.
// // - `used_later_lock` - Keys are variables defined locally. When `used_later_lock[v]` is positive, then the variable `v` is also used after `expr`.
// fn borrowing_optimization_expr(
//     expr: &Rc<ExprNode>,
//     used_later_lock: &mut HashMap<FullName, u32>,
// ) -> Rc<ExprNode> {
//     match &*expr.expr {
//         Expr::Var(_) => expr.clone(),
//         Expr::LLVM(_) => expr.clone(),
//         Expr::App(fun, args) => {
//             let args = args
//                 .iter()
//                 .map(|arg| borrowing_optimization_expr(arg))
//                 .collect();
//             let expr = expr
//                 .set_app_func(borrowing_optimization_expr(fun))
//                 .set_app_args(args);
//             borrowing_optimization_application(&expr)
//         }
//         Expr::Lam(_, body) => expr.set_lam_body(borrowing_optimization_expr(body)),
//         Expr::Let(_, bound, value) => expr
//             .set_let_bound(borrowing_optimization_expr(bound))
//             .set_let_value(borrowing_optimization_expr(value)),
//         Expr::If(c, t, e) => expr
//             .set_if_cond(borrowing_optimization_expr(c))
//             .set_if_then(borrowing_optimization_expr(t))
//             .set_if_else(borrowing_optimization_expr(e)),
//         Expr::TyAnno(e, _) => expr.set_tyanno_expr(borrowing_optimization_expr(e)),
//         Expr::ArrayLit(elems) => {
//             let mut expr = expr.clone();
//             for (i, e) in elems.iter().enumerate() {
//                 expr = expr.set_array_lit_elem(borrowing_optimization_expr(e), i)
//             }
//             expr
//         }
//         Expr::MakeStruct(_, fields) => {
//             let fields = fields.clone();
//             let mut expr = expr.clone();
//             for (field_name, field_expr) in fields {
//                 let field_expr = borrowing_optimization_expr(&field_expr);
//                 expr = expr.set_make_struct_field(&field_name, field_expr);
//             }
//             expr
//         }
//         Expr::CallC(_, _, _, _, args) => {
//             let mut expr = expr.clone();
//             for (i, e) in args.iter().enumerate() {
//                 expr = expr.set_call_c_arg(borrowing_optimization_expr(e), i)
//             }
//             expr
//         }
//     }
// }

// // Perform borrowing optimization for an application expression.
// fn borrowing_optimization_application(app_expr: &Rc<ExprNode>) -> Rc<ExprNode> {
//     let fun = app_expr.get_app_func();
//     let args = app_expr.get_app_args();
// }
