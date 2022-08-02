use either::Either;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, FunctionType, IntType, PointerType, StructType};
use inkwell::values::{
    BasicValue, BasicValueEnum, CallableValue, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};
use once_cell::sync::Lazy;
use std::alloc::System;
use std::collections::{HashMap, HashSet};
use std::fmt::Pointer;
use std::sync::Arc;
use std::vec::Vec;
use Either::Right;

// data Expr
//   = Var Var
//   | Lit Literal
//   | App Expr Expr
//   | Lam Var Expr -- Both term and type lambda
//   | Let Bind Expr
//   | Case Expr Var Type [(AltCon, [Var], Expr)]
//   | Type Type -- Used for type application

// data Var = Id Name Type -- Term variable
//   | TyVar Name Kind -- Type variable

// data Type = TyVarTy Var
//   | LitTy TyLit
//   | AppTy Type Type
//   | TyConApp TyCon [Type]
//   | FunTy Type Type
//   | ForAllTy Var Type

struct ExprInfo {
    expr: Arc<Expr>,
    free_vars: HashSet<String>,
}

#[derive(Clone)]
enum Expr {
    Var(Arc<Var>),
    Lit(Arc<Literal>),
    App(Arc<ExprInfo>, Arc<ExprInfo>),
    Lam(Arc<Var>, Arc<ExprInfo>),
    Let(Arc<Var>, Arc<ExprInfo>, Arc<ExprInfo>),
    // Caseはあとで
    If(Arc<ExprInfo>, Arc<ExprInfo>, Arc<ExprInfo>),
    Type(Arc<Type>),
}

impl Expr {
    fn into_expr_info(self: &Arc<Self>) -> Arc<ExprInfo> {
        self.into_expr_info_with(Default::default())
    }
    fn into_expr_info_with(self: &Arc<Self>, free_vars: HashSet<String>) -> Arc<ExprInfo> {
        Arc::new(ExprInfo {
            expr: self.clone(),
            free_vars,
        })
    }
}

type LiteralGenerator =
    dyn Send + Sync + for<'c, 'm, 'b> Fn(&mut GenerationContext<'c, 'm, 'b>) -> ExprCode<'c>;

struct Literal {
    generator: Arc<LiteralGenerator>,
    free_vars: Vec<String>, // e.g. "+" literal has two free variables.
    ty: Arc<Type>,
}

#[derive(Eq, PartialEq)]
enum Var {
    TermVar { name: String, ty: Arc<Type> },
    TyVar { name: String, kind: Arc<Kind> },
}

impl Var {
    fn name(self: &Self) -> &String {
        match self {
            Var::TermVar { name, ty: _ } => name,
            Var::TyVar { name, kind: _ } => name,
        }
    }
    fn ty(self: &Self) -> &Arc<Type> {
        match self {
            Var::TermVar { name: _, ty } => ty,
            Var::TyVar { name: _, kind: _ } => unimplemented!(),
        }
    }
}

#[derive(Eq, PartialEq)]
enum Kind {
    Star,
    Arrow(Arc<Kind>, Arc<Kind>),
}

#[derive(Eq, PartialEq)]
struct TyLit {
    value: String,
    kind: Arc<Kind>,
}

#[derive(Eq, PartialEq)]
enum Type {
    TyVar(Arc<Var>),
    LitTy(Arc<TyLit>),
    AppTy(Arc<Type>, Arc<Type>),
    TyConApp(Arc<TyCon>, Vec<Type>),
    FunTy(Arc<Type>, Arc<Type>),
    ForAllTy(Arc<Var>, Arc<Type>),
}

// impl Type {
//     fn fn_result(self: &Self) -> Arc<Type> {
//         match self {
//             Type::TyVar(_) => unimplemented!(),
//             Type::LitTy(_) => unimplemented!(),
//             Type::AppTy(_, _) => unimplemented!(),
//             Type::TyConApp(_, _) => unimplemented!(),
//             Type::FunTy(_, res) => res.clone(),
//             Type::ForAllTy(_, _) => unimplemented!(),
//         }
//     }
// }

#[derive(Eq, PartialEq)]
enum TyCon {
    Pair,
}

fn star_kind() -> Arc<Kind> {
    Arc::new(Kind::Star)
}

fn lit_ty(value: &str) -> Arc<Type> {
    let value = String::from(value);
    Arc::new(Type::LitTy(Arc::new(TyLit {
        value,
        kind: Arc::new(Kind::Star),
    })))
}

fn int_ty() -> Arc<Type> {
    lit_ty("Int")
}

fn bool_ty() -> Arc<Type> {
    lit_ty("Bool")
}

fn lambda_ty(src: Arc<Type>, dst: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::FunTy(src, dst))
}

fn tyvar_var(var_name: &str) -> Arc<Var> {
    Arc::new(Var::TyVar {
        name: String::from(var_name),
        kind: star_kind(),
    })
}

fn tyvar_ty(var_name: &str) -> Arc<Type> {
    Arc::new(Type::TyVar(tyvar_var(var_name)))
}

fn forall_ty(var_name: &str, ty: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::ForAllTy(tyvar_var("a"), ty))
}

fn int2int_ty() -> Arc<Type> {
    lambda_ty(
        lambda_ty(lambda_ty(int_ty(), int_ty()), lambda_ty(int_ty(), int_ty())),
        lambda_ty(int_ty(), int_ty()),
    )
}

fn termvar_var(var_name: &str, ty: Arc<Type>) -> Arc<Var> {
    Arc::new(Var::TermVar {
        name: String::from(var_name),
        ty: ty,
    })
}

fn intvar_var(var_name: &str) -> Arc<Var> {
    termvar_var(var_name, int_ty())
}

fn int2intvar_var(var_name: &str) -> Arc<Var> {
    termvar_var(var_name, int2int_ty())
}

fn lit(generator: Arc<LiteralGenerator>, ty: Arc<Type>, free_vars: Vec<String>) -> Arc<ExprInfo> {
    Arc::new(Expr::Lit(Arc::new(Literal {
        generator,
        free_vars,
        ty,
    })))
    .into_expr_info()
}

fn int(val: i32) -> Arc<ExprInfo> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_int_obj = ObjectType::int_obj_type().build_allocate_shared_obj(gc);
        let value = gc.context.i64_type().const_int(val as u64, false);
        build_set_field(ptr_to_int_obj, 1, value, gc);
        ExprCode {
            ptr: ptr_to_int_obj,
        }
    });
    lit(generator, int_ty(), vec![])
}

fn bool(val: bool) -> Arc<ExprInfo> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_obj = ObjectType::bool_obj_type().build_allocate_shared_obj(gc);
        let value = gc.context.i8_type().const_int(val as u64, false);
        build_set_field(ptr_to_obj, 1, value, gc);
        ExprCode { ptr: ptr_to_obj }
    });
    lit(generator, bool_ty(), vec![])
}

fn add_lit(lhs: &str, rhs: &str) -> Arc<ExprInfo> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope
            .get_field(
                &lhs_str,
                1,
                ObjectType::int_obj_type().to_struct_type(gc.context),
                gc,
            )
            .into_int_value();
        let rhs_val = gc
            .scope
            .get_field(
                &rhs_str,
                1,
                ObjectType::int_obj_type().to_struct_type(gc.context),
                gc,
            )
            .into_int_value();
        let value = gc.builder.build_int_add(lhs_val, rhs_val, "add");
        let ptr_to_int_obj = ObjectType::int_obj_type().build_allocate_shared_obj(gc);
        build_set_field(ptr_to_int_obj, 1, value, gc);
        ExprCode {
            ptr: ptr_to_int_obj,
        }
    });
    lit(generator, int_ty(), free_vars)
}

fn eq_lit(lhs: &str, rhs: &str) -> Arc<ExprInfo> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope
            .get_field(
                &lhs_str,
                1,
                ObjectType::bool_obj_type().to_struct_type(gc.context),
                gc,
            )
            .into_int_value();
        let rhs_val = gc
            .scope
            .get_field(
                &rhs_str,
                1,
                ObjectType::bool_obj_type().to_struct_type(gc.context),
                gc,
            )
            .into_int_value();
        let value = gc
            .builder
            .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, "eq");
        let ptr_to_obj = ObjectType::int_obj_type().build_allocate_shared_obj(gc);
        build_set_field(ptr_to_obj, 1, value, gc);
        ExprCode { ptr: ptr_to_obj }
    });
    lit(generator, bool_ty(), free_vars)
}

fn fix_lit(f: &str, x: &str) -> Arc<ExprInfo> {
    let f_str = String::from(f);
    let x_str = String::from(x);
    let free_vars = vec![f_str.clone(), x_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let bb = gc.builder.get_insert_block().unwrap();
        let func = bb.get_parent().unwrap();
        // The pointer "fixed" is already retained by callee and moved into here.
        let fixed = func.get_nth_param(1).unwrap().into_pointer_value();
        // The pointer "x" is already retained by callee and moved into here.
        let (x, _) = gc.scope.get(&x_str);
        let x = x.ptr;
        // The pointer "f" should be retained here.
        let (f, _) = gc.scope.get(&f_str);
        let f = f.ptr;
        build_retain(f, gc);
        let f_fixed = build_app(f, fixed, gc).ptr;
        let f_fixed_x = build_app(f_fixed, x, gc).ptr;
        ExprCode { ptr: f_fixed_x }
    });
    lit(generator, bool_ty(), free_vars)
}

fn let_in(var: Arc<Var>, bound: Arc<ExprInfo>, expr: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::Let(var, bound, expr)).into_expr_info()
}

fn lam(var: Arc<Var>, val: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::Lam(var, val)).into_expr_info()
}

fn app(lam: Arc<ExprInfo>, arg: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::App(lam, arg)).into_expr_info()
}

fn var(var_name: &str, ty: Arc<Type>) -> Arc<ExprInfo> {
    Arc::new(Expr::Var(termvar_var(var_name, ty))).into_expr_info()
}

fn intvar(var_name: &str) -> Arc<ExprInfo> {
    var(var_name, int_ty())
}

fn int2intvar(var_name: &str) -> Arc<ExprInfo> {
    var(var_name, int2int_ty())
}

fn add() -> Arc<ExprInfo> {
    lam(
        intvar_var("lhs"),
        lam(intvar_var("rhs"), add_lit("lhs", "rhs")),
    )
}

fn eq() -> Arc<ExprInfo> {
    lam(
        intvar_var("lhs"),
        lam(intvar_var("rhs"), eq_lit("lhs", "rhs")),
    )
}

fn fix() -> Arc<ExprInfo> {
    lam(intvar_var("f"), lam(intvar_var("x"), fix_lit("f", "x")))
}

// If(Arc<ExprInfo>, Arc<ExprInfo>, Arc<ExprInfo>),
fn if3(cond: Arc<ExprInfo>, then: Arc<ExprInfo>, else_expr: Arc<ExprInfo>) -> Arc<ExprInfo> {
    Arc::new(Expr::If(cond, then, else_expr)).into_expr_info()
}

// static FIX_INT_INT: Lazy<Arc<ExprInfo>> = Lazy::new(|| {
//     lit(
//         "fixIntInt",
//         lambda_ty(
//             lambda_ty(
//                 lambda_ty(INT_TYPE.clone(), INT_TYPE.clone()),
//                 lambda_ty(INT_TYPE.clone(), INT_TYPE.clone()),
//             ),
//             lambda_ty(INT_TYPE.clone(), INT_TYPE.clone()),
//         ),
//     )
// });

// static FIX_A_TO_B: Lazy<Arc<ExprInfo>> = Lazy::new(|| {
//     lit(
//         "fix",
//         forall_ty(
//             "a",
//             forall_ty(
//                 "b",
//                 lambda_ty(
//                     lambda_ty(
//                         lambda_ty(tyvar_ty("a"), tyvar_ty("b")),
//                         lambda_ty(tyvar_ty("a"), tyvar_ty("b")),
//                     ),
//                     lambda_ty(tyvar_ty("a"), tyvar_ty("b")),
//                 ),
//             ),
//         ),
//     )
// });

// TODO: use persistent binary search tree as ExprAuxInfo to avoid O(n^2) complexity of calculate_aux_info.
fn calculate_aux_info(ei: Arc<ExprInfo>) -> Arc<ExprInfo> {
    match &*ei.expr {
        Expr::Var(var) => {
            let free_vars = vec![var.name().clone()].into_iter().collect();
            ei.expr.into_expr_info_with(free_vars)
        }
        Expr::Lit(lit) => {
            let free_vars = lit.free_vars.clone().into_iter().collect();
            ei.expr.into_expr_info_with(free_vars)
        }
        Expr::App(func, arg) => {
            let func = calculate_aux_info(func.clone());
            let arg = calculate_aux_info(arg.clone());
            let mut free_vars = func.free_vars.clone();
            free_vars.extend(arg.free_vars.clone());
            app(func, arg).expr.into_expr_info_with(free_vars)
        }
        Expr::Lam(arg, val) => {
            let val = calculate_aux_info(val.clone());
            let mut free_vars = val.free_vars.clone();
            free_vars.remove(arg.name());
            lam(arg.clone(), val).expr.into_expr_info_with(free_vars)
        }
        Expr::Let(var, bound, val) => {
            // NOTE: Our Let is non-recursive let, i.e.,
            // "let x = f x in g x" is equal to "let y = f x in g y",
            // and x ∈ FreeVars("let x = f x in g x") = (FreeVars(g x) - {x}) + FreeVars(f x) != (FreeVars(g x) + FreeVars(f x)) - {x}.
            let bound = calculate_aux_info(bound.clone());
            let val = calculate_aux_info(val.clone());
            let mut free_vars = val.free_vars.clone();
            free_vars.remove(var.name());
            free_vars.extend(bound.free_vars.clone());
            let_in(var.clone(), bound, val)
                .expr
                .into_expr_info_with(free_vars)
        }
        Expr::If(cond, then, else_expr) => {
            let cond = calculate_aux_info(cond.clone());
            let then = calculate_aux_info(then.clone());
            let else_expr = calculate_aux_info(else_expr.clone());
            let mut free_vars = cond.free_vars.clone();
            free_vars.extend(then.free_vars.clone());
            free_vars.extend(else_expr.free_vars.clone());
            if3(cond, then, else_expr)
                .expr
                .into_expr_info_with(free_vars)
        }
        Expr::Type(_) => ei.clone(),
    }
}

// MEMO:
// Lazy組み込み型を導入して、fix : (Lazy a -> a) -> Lazy aとするべきではないか。
// fix : ((() -> a) -> (() -> a)) -> (() -> a) と等価で、FIX_A_TO_Bで表せるのでとりあえず良いけど。Lazyにはキャッシュ機能を付けたほうが良い。

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }

// memo
// data List a = () -> [] | (a, List a) と定義する。Lazy b = () -> b + キャッシュ、なら、data List a = Lazy ([] | (a, List a))
// このときfixと組み合わせて無限リストが正常動作すると思う。fix (\l -> 1:2:l) で、1,2,1,2,... など。
// フィボナッチ数列を計算する有名なコードはどうか？？

#[derive(Clone)]
struct ExprCode<'ctx> {
    ptr: PointerValue<'ctx>,
}

#[derive(Default)]
struct LocalVariables<'ctx> {
    data: HashMap<String, Vec<(ExprCode<'ctx>, Arc<Type>)>>,
}

impl<'c> LocalVariables<'c> {
    fn push(self: &mut Self, var_name: &str, code: &ExprCode<'c>, ty: &Arc<Type>) {
        if !self.data.contains_key(var_name) {
            self.data.insert(String::from(var_name), Default::default());
        }
        self.data
            .get_mut(var_name)
            .unwrap()
            .push((code.clone(), ty.clone()));
    }
    fn pop(self: &mut Self, var_name: &str) {
        self.data.get_mut(var_name).unwrap().pop();
        if self.data.get(var_name).unwrap().is_empty() {
            self.data.remove(var_name);
        }
    }
    fn get(self: &Self, var_name: &str) -> (ExprCode<'c>, Arc<Type>) {
        self.data.get(var_name).unwrap().last().unwrap().clone()
    }
    fn get_field<'m, 'b>(
        self: &Self,
        var_name: &str,
        field_idx: u32,
        ty: StructType<'c>,
        gc: &GenerationContext<'c, 'm, 'b>,
    ) -> BasicValueEnum<'c> {
        let (expr, _) = self.get(var_name);
        let ptr = expr.ptr.const_cast(ty.ptr_type(AddressSpace::Generic));
        build_get_field(ptr, field_idx, gc)
    }
}

struct GenerationContext<'c, 'm, 'b> {
    context: &'c Context,
    module: &'m Module<'c>,
    builder: &'b Builder<'c>,
    scope: LocalVariables<'c>,
    system_functions: HashMap<SystemFunctions, FunctionValue<'c>>,
}

impl<'c, 'm, 'b> GenerationContext<'c, 'm, 'b> {
    fn push_builder<'s, 'd>(
        self: &'s mut Self,
        builder: &'d Builder<'c>,
    ) -> (
        GenerationContext<'c, 'm, 'd>,
        impl 's + FnOnce(GenerationContext<'c, 'm, 'd>),
    ) {
        let new_gc = GenerationContext {
            context: self.context,
            module: self.module,
            builder: builder,
            scope: std::mem::replace(&mut self.scope, Default::default()),
            system_functions: std::mem::replace(&mut self.system_functions, Default::default()),
        };
        let pop = |gc: GenerationContext<'c, 'm, 'd>| {
            self.scope = gc.scope;
            self.system_functions = gc.system_functions;
        };
        (new_gc, pop)
    }
}

fn generate_expr<'c, 'm, 'b>(
    expr: Arc<ExprInfo>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    match &*expr.expr {
        Expr::Var(var) => generate_var(var.clone(), gc),
        Expr::Lit(lit) => generate_literal(lit.clone(), gc),
        Expr::App(lambda, arg) => generate_app(lambda.clone(), arg.clone(), gc),
        Expr::Lam(arg, val) => generate_lam(arg.clone(), val.clone(), gc),
        Expr::Let(var, bound, expr) => generate_let(var.clone(), bound.clone(), expr.clone(), gc),
        Expr::If(cond_expr, then_expr, else_expr) => {
            generate_if(cond_expr.clone(), then_expr.clone(), else_expr.clone(), gc)
        }
        Expr::Type(_) => todo!(),
    }
}

fn generate_var<'c, 'm, 'b>(var: Arc<Var>, gc: &mut GenerationContext<'c, 'm, 'b>) -> ExprCode<'c> {
    match &*var {
        Var::TermVar { name, ty: _ } => {
            let (code, _) = gc.scope.get(name);
            // We need to retain here since the pointer value is cloned.
            build_retain(code.ptr, gc);
            code.clone()
        }
        Var::TyVar { name: _, kind: _ } => unreachable!(),
    }
}

fn generate_app<'c, 'm, 'b>(
    lambda: Arc<ExprInfo>,
    arg: Arc<ExprInfo>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    let builder = gc.builder;
    let lambda = generate_expr(lambda, gc);
    let arg = generate_expr(arg, gc);
    build_app(lambda.ptr, arg.ptr, gc)
    // We do not release arg.ptr and lambda.ptr here since we have moved them into the arguments of lambda_func.
}

fn build_app<'c, 'm, 'b>(
    ptr_to_lambda: PointerValue<'c>,
    ptr_to_arg: PointerValue<'c>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    let ptr_to_func = build_ptr_to_func_of_lambda(ptr_to_lambda, gc);
    let lambda_func = CallableValue::try_from(ptr_to_func).unwrap();
    let ret = gc.builder.build_call(
        lambda_func,
        &[ptr_to_arg.into(), ptr_to_lambda.into()],
        "call_lambda",
    );
    ret.set_tail_call(true); // TODO: Precusely understand the implication of this line!
    let ret = ret.try_as_basic_value().unwrap_left().into_pointer_value();
    ExprCode { ptr: ret }
}

fn generate_literal<'c, 'm, 'b>(
    lit: Arc<Literal>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    (lit.generator)(gc)
    // match &*lit.ty {
    //     Type::LitTy(ty) => match ty.value.as_str() {
    //         "Int" => {
    //             let ptr_to_int_obj = ObjectType::int_obj_type().build_allocate_shared_obj(gc);
    //             let value = lit.value.parse::<i64>().unwrap();
    //             let value = gc.context.i64_type().const_int(value as u64, false);
    //             build_set_field(ptr_to_int_obj, 1, value, gc);
    //             ExprCode {
    //                 ptr: ptr_to_int_obj,
    //             }
    //         }
    //         _ => {
    //             panic!(
    //                 "Cannot generate literal value {} of type {}.",
    //                 lit.value, ty.value,
    //             )
    //         }
    //     },
    //     Type::TyVar(_) => panic!("Type of given Literal is TyVar (should be TyLit)."),
    //     Type::AppTy(_, _) => panic!("Type of given Literal is AppTy (should be TyLit)."),
    //     Type::TyConApp(_, _) => panic!("Type of given Literal is TyConApp (should be TyLit)."),
    //     Type::FunTy(_, _) => panic!("Type of given Literal is FunTy (should be TyLit)."), // e.g., fix
    //     Type::ForAllTy(_, _) => panic!("Type of given Literal is ForAllTy (should be TyLit)."),
    // }
}

fn generate_lam<'c, 'm, 'b>(
    arg: Arc<Var>,
    val: Arc<ExprInfo>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    let context = gc.context;
    let module = gc.module;
    // Fix ordering of captured names
    let mut captured_names = val.free_vars.clone();
    captured_names.remove(arg.name());
    let captured_names: Vec<String> = captured_names.into_iter().collect();
    // Determine the type of closure
    let mut field_types = vec![
        ObjectFieldType::ControlBlock,
        ObjectFieldType::LambdaFunction,
    ];
    for _ in captured_names.iter() {
        field_types.push(ObjectFieldType::SubObject);
    }
    let obj_type = ObjectType { field_types };
    let closure_ty = obj_type.to_struct_type(context);
    // Declare lambda function
    let lam_fn_ty = lambda_function_type(context);
    let lam_fn = module.add_function("lambda", lam_fn_ty, None);
    // Implement lambda function
    {
        // Create new builder
        let builder = gc.context.create_builder();
        let bb = context.append_basic_block(lam_fn, "entry");
        builder.position_at_end(bb);
        // Create new scope
        let mut scope = LocalVariables::default();
        let arg_ptr = lam_fn.get_first_param().unwrap().into_pointer_value();
        scope.push(&arg.name(), &ExprCode { ptr: arg_ptr }, arg.ty());
        let closure_obj_ptr = lam_fn.get_nth_param(1).unwrap().into_pointer_value();
        let closure_ptr = closure_obj_ptr.const_cast(closure_ty.ptr_type(AddressSpace::Generic));
        for (i, cap_name) in captured_names.iter().enumerate() {
            let ptr_to_cap_ptr = builder
                .build_struct_gep(closure_ptr, i as u32 + 2, "ptr_to_captured_field")
                .unwrap();
            let cap_ptr = builder
                .build_load(ptr_to_cap_ptr, "ptr_to_captured_obj")
                .into_pointer_value();
            let (_, cap_ty) = gc.scope.get(cap_name);
            scope.push(cap_name, &ExprCode { ptr: cap_ptr }, &cap_ty);
        }
        // Create new gc
        let mut gc = GenerationContext {
            context,
            module,
            builder: &builder,
            scope,
            system_functions: gc.system_functions.clone(),
        };
        // Generate value
        let val = generate_expr(val, &mut gc);
        // Release closure and arg
        build_release(arg_ptr, &gc);
        build_release(closure_obj_ptr, &gc);
        // Return result
        builder.build_return(Some(&val.ptr));
    }
    // Allocate and set up closure
    let obj = obj_type.build_allocate_shared_obj(gc);
    build_set_field(obj, 1, lam_fn.as_global_value().as_pointer_value(), gc);
    for (i, cap) in captured_names.iter().enumerate() {
        let (code, _) = gc.scope.get(cap);
        build_retain(code.ptr, gc);
        build_set_field(obj, i as u32 + 2, code.ptr, gc);
    }
    // Return closure object
    ExprCode { ptr: obj }
}

fn generate_let<'c, 'm, 'b>(
    var: Arc<Var>,
    bound: Arc<ExprInfo>,
    expr: Arc<ExprInfo>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    let bound_val = generate_expr(bound.clone(), gc);
    // We don't retain here because the result of generate_expr is considered to be "moved into" the variable.
    let var_name = var.name();
    let var_type = var.ty();
    gc.scope.push(&var_name, &bound_val, &var_type);
    let expr_val = generate_expr(expr.clone(), gc);
    gc.scope.pop(&var_name);
    build_release(bound_val.ptr, gc);
    expr_val
}

fn generate_if<'c, 'm, 'b>(
    cond_expr: Arc<ExprInfo>,
    then_expr: Arc<ExprInfo>,
    else_expr: Arc<ExprInfo>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    let ptr_to_cond_obj = generate_expr(cond_expr, gc).ptr;
    let ptr_to_cond_obj = ptr_to_cond_obj.const_cast(
        ObjectType::bool_obj_type()
            .to_struct_type(gc.context)
            .ptr_type(AddressSpace::Generic),
    );
    let cond_val = build_get_field(ptr_to_cond_obj, 1, gc).into_int_value();
    let bb = gc.builder.get_insert_block().unwrap();
    let func = bb.get_parent().unwrap();
    let then_bb = gc.context.append_basic_block(func, "then");
    let else_bb = gc.context.append_basic_block(func, "else");
    let cont_bb = gc.context.append_basic_block(func, "cont");
    gc.builder
        .build_conditional_branch(cond_val, then_bb, else_bb);
    gc.builder.position_at_end(then_bb);
    let then_code = generate_expr(then_expr, gc);
    gc.builder.build_unconditional_branch(cont_bb);
    gc.builder.position_at_end(else_bb);
    let else_code = generate_expr(else_expr, gc);
    gc.builder.build_unconditional_branch(cont_bb);
    gc.builder.position_at_end(cont_bb);
    let phi = gc.builder.build_phi(ptr_to_object_type(gc.context), "phi");
    phi.add_incoming(&[(&then_code.ptr, then_bb), (&else_code.ptr, else_bb)]);
    let ret_ptr = phi.as_basic_value().into_pointer_value();
    ExprCode { ptr: ret_ptr }
}

fn generate_clear_object<'c, 'm, 'b>(obj: PointerValue<'c>, gc: &GenerationContext<'c, 'm, 'b>) {
    let builder = gc.builder;
    let ptr_to_refcnt = builder.build_struct_gep(obj, 0, "ptr_to_refcnt").unwrap();
    builder.build_store(ptr_to_refcnt, gc.context.i64_type().const_zero());
}

fn build_set_field<'c, 'm, 'b, V>(
    obj: PointerValue<'c>,
    index: u32,
    value: V,
    gc: &GenerationContext<'c, 'm, 'b>,
) where
    V: BasicValue<'c>,
{
    let builder = gc.builder;
    let ptr_to_field = builder
        .build_struct_gep(obj, index, "ptr_to_field")
        .unwrap();
    builder.build_store(ptr_to_field, value);
}

fn build_get_field<'c, 'm, 'b>(
    obj: PointerValue<'c>,
    index: u32,
    gc: &GenerationContext<'c, 'm, 'b>,
) -> BasicValueEnum<'c> {
    let builder = gc.builder;
    let ptr_to_field = builder
        .build_struct_gep(obj, index, "ptr_to_field")
        .unwrap();
    builder.build_load(ptr_to_field, "field_value")
}

fn build_ptr_to_func_of_lambda<'c, 'm, 'b>(
    obj: PointerValue<'c>,
    gc: &GenerationContext<'c, 'm, 'b>,
) -> PointerValue<'c> {
    let ptr_to_lam_obj_ty = ObjectType::lam_obj_type()
        .to_struct_type(gc.context)
        .ptr_type(AddressSpace::Generic);
    let obj = obj.const_cast(ptr_to_lam_obj_ty);
    build_get_field(obj, 1, gc).into_pointer_value()
}

fn build_retain<'c, 'm, 'b>(ptr_to_obj: PointerValue, gc: &GenerationContext<'c, 'm, 'b>) {
    gc.builder.build_call(
        *gc.system_functions
            .get(&SystemFunctions::RetainObj)
            .unwrap(),
        &[ptr_to_obj.clone().into()],
        "retain",
    );
}

fn build_release<'c, 'm, 'b>(ptr_to_obj: PointerValue, gc: &GenerationContext<'c, 'm, 'b>) {
    gc.builder.build_call(
        *gc.system_functions
            .get(&SystemFunctions::ReleaseObj)
            .unwrap(),
        &[ptr_to_obj.clone().into()],
        "release",
    );
}

fn build_panic<'c, 'm, 'b>(msg: &str, gc: &GenerationContext<'c, 'm, 'b>) {
    const SIGABRT: i32 = 22;
    build_debug_printf(msg, gc);
    build_raise(SIGABRT, gc);
}

fn build_raise<'c, 'm, 'b>(signal: i32, gc: &GenerationContext<'c, 'm, 'b>) {
    //I don't know how to raise signal
}

fn build_debug_printf<'c, 'm, 'b>(msg: &str, gc: &GenerationContext<'c, 'm, 'b>) {
    let builder = gc.builder;
    let system_functions = &gc.system_functions;
    let string_ptr = builder.build_global_string_ptr(msg, "debug_printf");
    let printf_func = *system_functions.get(&SystemFunctions::Printf).unwrap();
    builder.build_call(
        printf_func,
        &[string_ptr.as_pointer_value().into()],
        "build_debug_printf",
    );
}

#[derive(Eq, Hash, PartialEq, Clone)]
enum ObjectFieldType {
    ControlBlock,
    LambdaFunction,
    SubObject,
    Int,
    Bool,
}

impl ObjectFieldType {
    fn to_basic_type<'ctx>(&self, context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(context).into(),
            ObjectFieldType::LambdaFunction => ptr_to_lambda_function_type(context).into(),
            ObjectFieldType::SubObject => ptr_to_object_type(context).into(),
            ObjectFieldType::Int => context.i64_type().into(),
            ObjectFieldType::Bool => context.i8_type().into(),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct ObjectType {
    field_types: Vec<ObjectFieldType>,
}

impl ObjectType {
    // fn from_type(ty: Arc<Type>) -> Self {
    //     if ty == *INT_TYPE {
    //         return Self::int_obj_type();
    //     }
    //     match &*ty {
    //         Type::TyVar(var) => ObjectType::from_type(var.ty().clone()),
    //         Type::LitTy(_) => unreachable!("Should have treated above."),
    //         Type::AppTy(_, _) => todo!(),
    //         Type::TyConApp(_, _) => todo!(),
    //         Type::FunTy(_, _) => {
    //             let mut field_types: Vec<ObjectFieldType> = Default::default();
    //             field_types.push(ObjectFieldType::ControlBlock);
    //             field_types.push(ObjectFieldType::LambdaFunction);
    //             // Following fields may exist, but their types are unknown.
    //             ObjectType { field_types }
    //         }
    //         Type::ForAllTy(_, _) => todo!(),
    //     }
    //     // let mut field_types: Vec<ObjectFieldType> = Default::default();
    //     // field_types.push(ObjectFieldType::ControlBlock);
    //     // ObjectType { field_types }
    // }
    fn to_struct_type<'ctx>(&self, context: &'ctx Context) -> StructType<'ctx> {
        let mut fields: Vec<BasicTypeEnum<'ctx>> = vec![];
        for field_type in &self.field_types {
            fields.push(field_type.to_basic_type(context));
        }
        context.struct_type(&fields, false)
    }

    fn shared_obj_type(mut field_types: Vec<ObjectFieldType>) -> Self {
        let mut fields = vec![ObjectFieldType::ControlBlock];
        fields.append(&mut field_types);
        Self {
            field_types: fields,
        }
    }

    fn int_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::Int])
    }

    fn bool_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::Bool])
    }

    fn lam_obj_type() -> Self {
        let mut field_types: Vec<ObjectFieldType> = Default::default();
        field_types.push(ObjectFieldType::ControlBlock);
        field_types.push(ObjectFieldType::LambdaFunction);
        // Following fields may exist, but their types are unknown.
        ObjectType { field_types }
    }

    fn generate_func_dtor<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm, 'b>,
    ) -> FunctionValue<'c> {
        if gc
            .system_functions
            .contains_key(&SystemFunctions::Dtor(self.clone()))
        {
            return *gc
                .system_functions
                .get(&SystemFunctions::Dtor(self.clone()))
                .unwrap();
        }
        let struct_type = self.to_struct_type(gc.context);
        let func_type = dtor_type(gc.context);
        let func = gc.module.add_function("dtor", func_type, None);
        let bb = gc.context.append_basic_block(func, "entry");
        let builder = gc.context.create_builder();
        {
            let context = gc.context;
            let module = gc.module;
            // Create new gc
            let gc = GenerationContext {
                context,
                module,
                builder: &builder,
                scope: Default::default(), // This gc use used only for build_release, and it doesn't use scope.
                system_functions: gc.system_functions.clone(),
            };
            builder.position_at_end(bb);
            let ptr_to_obj = func
                .get_first_param()
                .unwrap()
                .into_pointer_value()
                .const_cast(struct_type.ptr_type(AddressSpace::Generic));
            for (i, ft) in self.field_types.iter().enumerate() {
                match ft {
                    ObjectFieldType::SubObject => {
                        let ptr_to_subobj =
                            build_get_field(ptr_to_obj, i as u32, &gc).into_pointer_value();
                        build_release(ptr_to_subobj, &gc);
                    }
                    ObjectFieldType::ControlBlock => {}
                    ObjectFieldType::Int => {}
                    ObjectFieldType::LambdaFunction => {}
                    ObjectFieldType::Bool => {}
                }
            }
            builder.build_return(None);
        }
        gc.system_functions
            .insert(SystemFunctions::Dtor(self.clone()), func);
        func
    }

    fn build_allocate_shared_obj<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm, 'b>,
    ) -> PointerValue<'c> {
        let context = gc.context;
        let builder = gc.builder;
        let struct_type = self.to_struct_type(context);
        // NOTE: Only once allocation is needed since we don't implement weak_ptr
        let ptr_to_obj = builder.build_malloc(struct_type, "ptr_to_obj").unwrap();
        for (i, ft) in self.field_types.iter().enumerate() {
            match ft {
                ObjectFieldType::ControlBlock => {
                    let ptr_to_control_block = builder
                        .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_control_block")
                        .unwrap();
                    let ptr_to_refcnt = builder
                        .build_struct_gep(ptr_to_control_block, 0, "ptr_to_refcnt")
                        .unwrap();
                    // The initial value of refcnt should be one (as std::make_shared of C++ does).
                    builder.build_store(ptr_to_refcnt, refcnt_type(context).const_int(1, false));
                    let ptr_to_dtor_field = builder
                        .build_struct_gep(ptr_to_control_block, 1, "ptr_to_dtor_field")
                        .unwrap();
                    let dtor = self.generate_func_dtor(gc);
                    builder
                        .build_store(ptr_to_dtor_field, dtor.as_global_value().as_pointer_value());
                }
                ObjectFieldType::Int => {}
                ObjectFieldType::SubObject => {}
                ObjectFieldType::LambdaFunction => {}
                ObjectFieldType::Bool => {}
            }
        }
        ptr_to_obj
    }
}

fn refcnt_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

fn ptr_to_refcnt_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    refcnt_type(context).ptr_type(AddressSpace::Generic)
}

fn ptr_to_object_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    context.i8_type().ptr_type(AddressSpace::Generic)
}

fn dtor_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    context
        .void_type()
        .fn_type(&[ptr_to_object_type(context).into()], false)
}

fn ptr_to_dtor_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    dtor_type(context).ptr_type(AddressSpace::Generic)
}

fn control_block_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    context.struct_type(
        &[
            refcnt_type(context).into(),
            ptr_to_dtor_type(context).into(),
        ],
        false,
    )
}

fn ptr_to_control_block_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    control_block_type(context).ptr_type(AddressSpace::Generic)
}

fn lambda_function_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    // A function that takes argument and context (=lambda object itself).
    ptr_to_object_type(context).fn_type(
        &[
            ptr_to_object_type(context).into(),
            ptr_to_object_type(context).into(),
        ],
        false,
    )
}

fn ptr_to_lambda_function_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    lambda_function_type(context).ptr_type(AddressSpace::Generic)
}

#[derive(Eq, Hash, PartialEq, Clone)]
enum SystemFunctions {
    Printf,
    // Raise,
    PrintIntObj,
    RetainObj,
    ReleaseObj,
    EmptyDestructor,
    Dtor(ObjectType),
}

fn generate_func_printf<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm, 'b>) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;

    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);

    let fn_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let func = module.add_function("printf", fn_type, None);

    func
}

// fn generate_func_raise<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm, 'b>) -> FunctionValue<'c> {
//     let context = gc.context;
//     let module = gc.module;

//     let i32_type = context.i32_type();

//     let fn_type = i32_type.fn_type(&[i32_type.into()], false);
//     let func = module.add_function("raise", fn_type, None);

//     func
// }

fn generate_func_print_int_obj<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let system_functions = &gc.system_functions;
    let void_type = context.void_type();
    let int_obj_type = ObjectType::int_obj_type().to_struct_type(context);
    let int_obj_ptr_type = int_obj_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[int_obj_ptr_type.into()], false);
    let func = module.add_function("print_int_obj", fn_type, None);

    let entry_bb = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(entry_bb);
    let int_obj_ptr = func.get_first_param().unwrap().into_pointer_value();
    let int_field_ptr = builder
        .build_struct_gep(int_obj_ptr, 1, "int_field_ptr")
        .unwrap();
    let int_val = builder
        .build_load(int_field_ptr, "int_val")
        .into_int_value();
    let string_ptr = builder.build_global_string_ptr("%lld\n", "int_placefolder");
    let printf_func = *system_functions.get(&SystemFunctions::Printf).unwrap();
    builder.build_call(
        printf_func,
        &[string_ptr.as_pointer_value().into(), int_val.into()],
        "call_print_int",
    );
    builder.build_return(None);

    func
}

fn generate_func_retain_obj<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm, 'b>) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let void_type = context.void_type();
    let func_type = void_type.fn_type(&[ptr_to_object_type(context).into()], false);
    let retain_func = module.add_function("retain_obj", func_type, None);
    let bb = context.append_basic_block(retain_func, "entry");

    let builder = context.create_builder();
    builder.position_at_end(bb);
    let ptr_to_refcnt = retain_func.get_first_param().unwrap().into_pointer_value();
    let refcnt = builder.build_load(ptr_to_refcnt, "refcnt").into_int_value();
    let one = context.i64_type().const_int(1, false);
    let refcnt = builder.build_int_add(refcnt, one, "refcnt");
    builder.build_store(ptr_to_refcnt, refcnt);
    builder.build_return(None);

    retain_func
    // TODO: Add fence instruction for incrementing refcnt
    // TODO: Add code for leak detector
    // TODO: Raise error when trying to retain object of refcnt is zero (which implies use of deallocate memory).
}

fn generate_func_release_obj<'c, 'm, 'b>(
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let void_type = gc.context.void_type();
    let func_type = void_type.fn_type(&[ptr_to_object_type(gc.context).into()], false);
    let release_func = gc.module.add_function("release_obj", func_type, None);
    let mut bb = gc.context.append_basic_block(release_func, "entry");

    let builder = gc.context.create_builder();
    let (mut new_gc, pop_gc) = gc.push_builder(&builder);
    {
        let gc = &mut new_gc;
        builder.position_at_end(bb);
        let ptr_to_obj = release_func.get_first_param().unwrap().into_pointer_value();
        let ptr_to_control_block = ptr_to_obj.const_cast(ptr_to_control_block_type(gc.context));
        let ptr_to_refcnt = builder
            .build_struct_gep(ptr_to_control_block, 0, "ptr_to_refcnt")
            .unwrap();
        let refcnt = builder.build_load(ptr_to_refcnt, "refcnt").into_int_value();

        if DEBUG_MEMORY {
            // check if refcnt is positive
            let zero = gc.context.i64_type().const_zero();
            let is_positive = builder.build_int_compare(
                inkwell::IntPredicate::ULE,
                refcnt,
                zero,
                "is_refcnt_positive",
            );
            let then_bb = gc
                .context
                .append_basic_block(release_func, "error_refcnt_already_leq_zero");
            let cont_bb = gc
                .context
                .append_basic_block(release_func, "refcnt_positive");
            builder.build_conditional_branch(is_positive, then_bb, cont_bb);

            builder.position_at_end(then_bb);
            build_panic("Release object whose refcnt is already zero.", gc);
            builder.build_unconditional_branch(cont_bb);

            bb = cont_bb;
            builder.position_at_end(bb);
        }

        let one = gc.context.i64_type().const_int(1, false);
        let refcnt = builder.build_int_sub(refcnt, one, "refcnt");
        let zero = gc.context.i64_type().const_zero();
        let is_refcnt_zero =
            builder.build_int_compare(inkwell::IntPredicate::EQ, refcnt, zero, "is_refcnt_zero");
        let then_bb = gc
            .context
            .append_basic_block(release_func, "refcnt_zero_after_release");
        let cont_bb = gc.context.append_basic_block(release_func, "end");
        builder.build_conditional_branch(is_refcnt_zero, then_bb, cont_bb);

        builder.position_at_end(then_bb);
        let ptr_to_dtor_ptr = builder
            .build_struct_gep(ptr_to_control_block, 1, "ptr_to_dtor_ptr")
            .unwrap();
        let ptr_to_dtor = builder
            .build_load(ptr_to_dtor_ptr, "ptr_to_dtor")
            .into_pointer_value();

        let dtor_func = CallableValue::try_from(ptr_to_dtor).unwrap();
        builder.build_call(dtor_func, &[ptr_to_obj.into()], "call_dtor");
        builder.build_free(ptr_to_refcnt);
        builder.build_unconditional_branch(cont_bb);

        builder.position_at_end(cont_bb);
        builder.build_return(None);
    }
    pop_gc(new_gc);
    release_func
    // TODO: Add fence instruction for incrementing refcnt
    // TODO: Add code for leak detector
}

fn generate_func_empty_destructor<'c, 'm, 'b>(
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let void_type = context.void_type();
    let ptr_to_obj_type = context.i64_type().ptr_type(AddressSpace::Generic);
    let func_type = void_type.fn_type(&[ptr_to_obj_type.into()], false);
    let func = module.add_function("empty_destructor", func_type, None);
    let bb = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(bb);
    builder.build_return(None);

    func
}

fn generate_func_dtor<'c, 'm, 'b>(
    obj_type: StructType<'c>,
    subobj_indices: &[i32],
    gc: &GenerationContext<'c, 'm, 'b>,
) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let void_type = context.void_type();
    let ptr_to_obj_type = obj_type.ptr_type(AddressSpace::Generic);
    let func_type = void_type.fn_type(&[ptr_to_obj_type.into()], false);
    let func = module.add_function("destructor", func_type, None); // TODO: give appropriate name
    let bb = context.append_basic_block(func, "entry");
    let builder = context.create_builder();
    builder.position_at_end(bb);
    builder.build_return(None);
    func
}

fn generate_system_functions<'c, 'm, 'b>(gc: &mut GenerationContext<'c, 'm, 'b>) {
    gc.system_functions.insert(
        SystemFunctions::EmptyDestructor,
        generate_func_empty_destructor(gc),
    );
    gc.system_functions
        .insert(SystemFunctions::Printf, generate_func_printf(gc));
    // gc.system_functions
    //     .insert(SystemFunctions::Raise, generate_func_raise(gc));
    gc.system_functions.insert(
        SystemFunctions::PrintIntObj,
        generate_func_print_int_obj(gc),
    );
    gc.system_functions
        .insert(SystemFunctions::RetainObj, generate_func_retain_obj(gc));
    let release_func = generate_func_release_obj(gc);
    gc.system_functions
        .insert(SystemFunctions::ReleaseObj, release_func);
}

fn execute_main_module<'ctx>(module: &Module<'ctx>) -> i32 {
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    unsafe {
        let func = execution_engine
            .get_function::<unsafe extern "C" fn() -> i32>("main")
            .unwrap();
        func.call()
    }
}

const DEBUG_MEMORY: bool = true;

fn test_int_program(program: Arc<ExprInfo>, answer: i32) {
    let program = calculate_aux_info(program);

    let context = Context::create();
    {
        let module = context.create_module("main");
        let builder = context.create_builder();
        let mut gc = GenerationContext {
            context: &context,
            module: &module,
            builder: &builder,
            scope: Default::default(),
            system_functions: Default::default(),
        };
        generate_system_functions(&mut gc);

        let i32_type = context.i32_type();
        let main_fn_type = i32_type.fn_type(&[], false);
        let main_function = module.add_function("main", main_fn_type, None);

        let entry_bb = context.append_basic_block(main_function, "entry");
        builder.position_at_end(entry_bb);

        let program_result = generate_expr(program, &mut gc);
        // let print_int_obj = *system_functions.get(&SystemFunctions::PrintIntObj).unwrap();
        // builder.build_call(
        //     print_int_obj,
        //     &[program_result.ptr.into()],
        //     "print_program_result",
        // );
        let int_obj_ptr = program_result.ptr.const_cast(
            ObjectType::int_obj_type()
                .to_struct_type(&context)
                .ptr_type(AddressSpace::Generic),
        );
        let value = build_get_field(int_obj_ptr, 1, &gc);
        if let BasicValueEnum::IntValue(value) = value {
            builder.build_return(Some(&value));
        } else {
            unreachable!()
            // builder.build_return(Some(&i32_type.const_int(0, false)));
        }

        module.print_to_file("ir").unwrap();
        assert_eq!(execute_main_module(&module), answer);
    }
    std::mem::forget(context); // To avoid crash in destructor of LLVMContext
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    pub fn test0() {
        let program = int(-42);
        test_int_program(program, -42);
    }
    #[test]
    pub fn test1() {
        let program = let_in(intvar_var("x"), int(-42), int(42));
        test_int_program(program, 42);
    }
    #[test]
    pub fn test2() {
        let program = let_in(intvar_var("x"), int(-42), intvar("x"));
        test_int_program(program, -42);
    }
    #[test]
    pub fn test3() {
        let program = let_in(
            intvar_var("n"),
            int(-42),
            let_in(intvar_var("p"), int(42), intvar("n")),
        );
        test_int_program(program, -42);
    }
    #[test]
    pub fn test4() {
        let program = let_in(
            intvar_var("n"),
            int(-42),
            let_in(intvar_var("p"), int(42), intvar("p")),
        );
        test_int_program(program, 42);
    }
    #[test]
    pub fn test5() {
        let program = let_in(
            intvar_var("x"),
            int(-42),
            let_in(intvar_var("x"), int(42), intvar("x")),
        );
        test_int_program(program, 42);
    }
    #[test]
    pub fn test6() {
        let program = let_in(
            intvar_var("x"),
            let_in(intvar_var("y"), int(42), intvar("y")),
            intvar("x"),
        );
        test_int_program(program, 42);
    }
    #[test]
    pub fn test7() {
        let program = app(lam(intvar_var("x"), int(0)), int(1));
        test_int_program(program, 0);
    }
    #[test]
    pub fn test8() {
        let program = app(lam(intvar_var("x"), intvar("x")), int(1));
        test_int_program(program, 1);
    }
    #[test]
    pub fn test9() {
        let program = app(app(add(), int(2)), int(3));
        test_int_program(program, 5);
    }
    #[test]
    pub fn test10() {
        let program = let_in(
            intvar_var("x"),
            int(5),
            app(app(add(), int(2)), intvar("x")),
        );
        test_int_program(program, 7);
    }
    #[test]
    pub fn test11() {
        let program = let_in(
            intvar_var("x"),
            int(5),
            let_in(
                intvar_var("y"),
                int(-3),
                app(app(add(), intvar("y")), intvar("x")),
            ),
        );
        test_int_program(program, 2);
    }
    #[test]
    pub fn test12() {
        let program = let_in(
            intvar_var("x"),
            int(5),
            let_in(
                intvar_var("y"),
                int(-3),
                let_in(
                    intvar_var("z"),
                    int(12),
                    let_in(
                        intvar_var("xy"),
                        app(app(add(), intvar("x")), intvar("y")),
                        app(app(add(), intvar("xy")), intvar("z")),
                    ),
                ),
            ),
        );
        test_int_program(program, 5 - 3 + 12);
    }
    #[test]
    pub fn test13() {
        let program = let_in(
            int2intvar_var("f"),
            app(add(), int(3)),
            app(int2intvar("f"), int(5)),
        );
        test_int_program(program, 3 + 5);
    }
    #[test]
    pub fn test14() {
        let program = let_in(
            intvar_var("x"),
            int(3),
            let_in(
                intvar_var("y"),
                int(5),
                let_in(
                    int2intvar_var("f"),
                    app(add(), intvar("x")),
                    app(int2intvar("f"), intvar("y")),
                ),
            ),
        );
        test_int_program(program, 3 + 5);
    }
    #[test]
    pub fn test15() {
        let program = let_in(
            int2intvar_var("f"),
            lam(intvar_var("x"), app(app(add(), int(3)), intvar("x"))),
            app(int2intvar("f"), int(5)),
        );
        test_int_program(program, 3 + 5);
    }
    #[test]
    pub fn test16() {
        let program = let_in(
            int2intvar_var("f"),
            lam(intvar_var("x"), app(app(add(), intvar("x")), int(3))),
            app(int2intvar("f"), int(5)),
        );
        test_int_program(program, 3 + 5);
    }
    #[test]
    pub fn test17() {
        let program = if3(bool(true), int(3), int(5));
        test_int_program(program, 3);
    }
    #[test]
    pub fn test18() {
        let program = if3(bool(false), int(3), int(5));
        test_int_program(program, 5);
    }
    #[test]
    pub fn test19() {
        let program = if3(app(app(eq(), int(3)), int(3)), int(3), int(5));
        test_int_program(program, 3);
    }
    #[test]
    pub fn test20() {
        let program = if3(app(app(eq(), int(3)), int(5)), int(3), int(5));
        test_int_program(program, 5);
    }
    #[test]
    pub fn test21() {
        let program = let_in(
            int2intvar_var("F"),
            app(
                fix(),
                lam(
                    int2intvar_var("f"),
                    lam(
                        intvar_var("x"),
                        if3(
                            app(app(eq(), intvar("x")), int(0)),
                            int(0),
                            app(
                                app(add(), intvar("x")),
                                app(int2intvar("f"), app(app(add(), intvar("x")), int(-1))),
                            ),
                        ),
                    ),
                ),
            ),
            app(int2intvar("F"), int(10)),
        );
        test_int_program(program, 55);
    }
}

fn main() {}
