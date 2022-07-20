use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::PointerType;
use inkwell::values::PointerValue;
use inkwell::OptimizationLevel;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::vec::Vec;

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

enum Expr {
    Var(Arc<Var>),
    Lit(Arc<Literal>),
    App(Arc<Expr>, Arc<Expr>),
    Lam(Arc<Var>, Arc<Expr>),
    Let(Arc<Var>, Arc<Expr>, Arc<Expr>),
    // Caseはあとで
    If(Arc<Expr>, Arc<Expr>, Arc<Expr>),
    Type(Arc<Type>),
}

struct Literal {
    value: String,
    ty: Arc<Type>,
}

enum Var {
    TermVar { name: String, ty: Arc<Type> },
    TyVar { name: String, kind: Arc<Kind> },
}

enum Kind {
    Star,
    Arrow(Arc<Kind>, Arc<Kind>),
}

struct TyLit {
    value: String,
    kind: Arc<Kind>,
}

enum Type {
    TyVar(Arc<Var>),
    LitTy(Arc<TyLit>),
    AppTy(Arc<Type>, Arc<Type>),
    TyConApp(Arc<TyCon>, Vec<Type>),
    FunTy(Arc<Type>, Arc<Type>),
    ForAllTy(Arc<Var>, Arc<Type>),
}

enum TyCon {
    Pair,
}

fn mk_lit_expr(value: &str, ty: Arc<Type>) -> Arc<Expr> {
    let value = String::from(value);
    Arc::new(Expr::Lit(Arc::new(Literal { value, ty })))
}

fn mk_int_expr(val: i32) -> Arc<Expr> {
    mk_lit_expr(val.to_string().as_str(), mk_lit_type("Int"))
}

fn mk_lit_type(value: &str) -> Arc<Type> {
    let value = String::from(value);
    Arc::new(Type::LitTy(Arc::new(TyLit {
        value,
        kind: Arc::new(Kind::Star),
    })))
}

fn mk_arrow_type(src: Arc<Type>, dst: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::FunTy(src, dst))
}

static KIND_STAR: Lazy<Arc<Kind>> = Lazy::new(|| Arc::new(Kind::Star));

fn mk_tyvar_var(var_name: &str) -> Arc<Var> {
    Arc::new(Var::TyVar {
        name: String::from(var_name),
        kind: KIND_STAR.clone(),
    })
}

fn mk_tyvar_type(var_name: &str) -> Arc<Type> {
    Arc::new(Type::TyVar(mk_tyvar_var(var_name)))
}

fn mk_forall_type(var_name: &str, ty: Arc<Type>) -> Arc<Type> {
    Arc::new(Type::ForAllTy(mk_tyvar_var("a"), ty))
}

static INT_TYPE: Lazy<Arc<Type>> = Lazy::new(|| mk_lit_type("Int"));

static FIX_INT_INT: Lazy<Arc<Expr>> = Lazy::new(|| {
    mk_lit_expr(
        "fixIntInt",
        mk_arrow_type(
            mk_arrow_type(
                mk_arrow_type(INT_TYPE.clone(), INT_TYPE.clone()),
                mk_arrow_type(INT_TYPE.clone(), INT_TYPE.clone()),
            ),
            mk_arrow_type(INT_TYPE.clone(), INT_TYPE.clone()),
        ),
    )
});

static FIX_A_TO_B: Lazy<Arc<Expr>> = Lazy::new(|| {
    mk_lit_expr(
        "fix",
        mk_forall_type(
            "a",
            mk_forall_type(
                "b",
                mk_arrow_type(
                    mk_arrow_type(
                        mk_arrow_type(mk_tyvar_type("a"), mk_tyvar_type("b")),
                        mk_arrow_type(mk_tyvar_type("a"), mk_tyvar_type("b")),
                    ),
                    mk_arrow_type(mk_tyvar_type("a"), mk_tyvar_type("b")),
                ),
            ),
        ),
    )
});

// MEMO:
// Lazy組み込み型を導入して、fix : (Lazy a -> a) -> Lazy aとするべきではないか。
// fix : ((() -> a) -> (() -> a)) -> (() -> a) と等価で、FIX_A_TO_Bで表せるのでとりあえず良いけど。Lazyにはキャッシュ機能を付けたほうが良い。

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// memo
// data List a = () -> [] | (a, List a) と定義する。Lazy b = () -> b + キャッシュ、なら、data List a = Lazy ([] | (a, List a))
// このときfixと組み合わせて無限リストが正常動作すると思う。fix (\l -> 1:2:l) で、1,2,1,2,... など。
// フィボナッチ数列を計算する有名なコードはどうか？？

static INT_CODE: Lazy<Arc<Expr>> = Lazy::new(|| mk_int_expr(-42));

#[derive(Default)]
struct Scope<'ctx> {
    // map to variable name to pointer value.
    data: HashMap<String, Vec<PointerValue<'ctx>>>,
}

fn generate_code<'a>(
    expr: Arc<Expr>,
    context: &'a Context,
    module: &'a Module,
    builder: &'a Builder,
    scope: &'a mut Scope,
) -> PointerValue<'a> {
    // enum Expr {
    //     Var(Arc<Var>),
    //     Lit(Arc<Literal>),
    //     App(Arc<Expr>, Arc<Expr>),
    //     Lam(Arc<Var>, Arc<Expr>),
    //     Let(Arc<Var>, Arc<Expr>, Arc<Expr>),
    //     // Caseはあとで
    //     If(Arc<Expr>, Arc<Expr>, Arc<Expr>),
    //     Type(Arc<Type>),
    // }
    match &*expr {
        Expr::Var(var) => {
            todo!();
            // TODO: term variable のとき、scopeからポインタを取り出して返す
            // TODO: type variable のコード生成はエラーにする。
        }
        Expr::Lit(lit) => generate_code_literal(lit.clone(), context, module, builder),
        Expr::App(_, _) => todo!(),
        Expr::Lam(_, _) => todo!(),
        Expr::Let(_, _, _) => todo!(),
        Expr::If(_, _, _) => todo!(),
        Expr::Type(_) => todo!(),
    }
}

fn generate_code_literal<'a>(
    lit: Arc<Literal>,
    context: &'a Context,
    module: &'a Module,
    builder: &'a Builder,
) -> PointerValue<'a> {
    match &*lit.ty {
        Type::LitTy(ty) => match ty.value.as_str() {
            "Int" => {
                let ty = context.struct_type(
                    &[context.i64_type().into(), context.i64_type().into()],
                    false,
                );
                let ptr = builder.build_malloc(ty, "ptr").unwrap();
                let ptr_to_refcnt = builder.build_struct_gep(ptr, 0, "ptr_to_refcnt").unwrap();
                builder.build_store(ptr_to_refcnt, context.i64_type().const_zero());
                let ptr_to_int_value = builder
                    .build_struct_gep(ptr, 1, "ptr_to_int_value")
                    .unwrap();
                let value = lit.value.parse::<i64>().unwrap();
                let value = context.i64_type().const_int(value as u64, false);
                builder.build_store(ptr_to_int_value, value);
                ptr
            }
            _ => {
                panic!(
                    "Cannot generate literal value {} of type {}.",
                    lit.value, ty.value,
                )
            }
        },
        Type::TyVar(_) => panic!("Type of given Literal is TyVar (should be TyLit)."),
        Type::AppTy(_, _) => panic!("Type of given Literal is AppTy (should be TyLit)."),
        Type::TyConApp(_, _) => panic!("Type of given Literal is TyConApp (should be TyLit)."),
        Type::FunTy(_, _) => panic!("Type of given Literal is FunTy (should be TyLit)."),
        Type::ForAllTy(_, _) => panic!("Type of given Literal is ForAllTy (should be TyLit)."),
    }
}

fn main() {
    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);

    let printf_fn_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let printf_function = module.add_function("printf", printf_fn_type, None);

    let main_fn_type = i32_type.fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);

    let entry_basic_block = context.append_basic_block(main_function, "entry");
    builder.position_at_end(entry_basic_block);

    let mut scope: Scope = Default::default();
    generate_code(INT_CODE.clone(), &context, &module, &builder, &mut scope);

    let hw_string_ptr = builder.build_global_string_ptr("Hello, world!", "hw");
    builder.build_call(
        printf_function,
        &[hw_string_ptr.as_pointer_value().into()],
        "call",
    );
    builder.build_return(Some(&i32_type.const_int(0, false)));

    module.print_to_file("ir").unwrap();

    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::Aggressive)
        .unwrap();
    unsafe {
        execution_engine
            .get_function::<unsafe extern "C" fn()>("main")
            .unwrap()
            .call();
    }
}
