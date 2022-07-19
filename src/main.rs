/*

use once_cell::sync::Lazy;
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

// TODO: まず総称型を削除する（テンプレートを展開する）フェーズが入りそう

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

fn main() {
    println!("Hello, world!");
}

// data List a = () -> [] | (a, List a) と定義する。Lazy b = () -> b + キャッシュ、なら、data List a = Lazy ([] | (a, List a))
// このときfixと組み合わせて無限リストが正常動作すると思う。fix (\l -> 1:2:l) で、1,2,1,2,... など。

 */

use inkwell::context::Context;
use inkwell::OptimizationLevel;

fn main() {
    let context = Context::create();
    // moduleを作成
    let module = context.create_module("main");
    // builderを作成
    let builder = context.create_builder();

    // 型関係の変数
    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);

    // printf関数を宣言
    let printf_fn_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let printf_function = module.add_function("printf", printf_fn_type, None);

    // main関数を宣言
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);

    // main関数にBasic Blockを追加
    let entry_basic_block = context.append_basic_block(main_function, "entry");
    // builderのpositionをentry Basic Blockに設定
    builder.position_at_end(entry_basic_block);

    // ここからmain関数に命令をビルドしていく
    // globalに文字列を宣言
    let hw_string_ptr = builder.build_global_string_ptr("Hello, world!", "hw");
    // printfをcall
    builder.build_call(
        printf_function,
        &[hw_string_ptr.as_pointer_value().into()],
        "call",
    );
    // main関数は0を返す
    builder.build_return(Some(&i32_type.const_int(0, false)));

    // JIT実行エンジンを作成し、main関数を実行
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
