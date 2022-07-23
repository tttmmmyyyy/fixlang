use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, IntType, PointerType, StructType};
use inkwell::values::{BasicValue, FunctionValue, PointerValue};
use inkwell::{AddressSpace, OptimizationLevel};
use once_cell::sync::Lazy;
use std::alloc::System;
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

#[derive(Default)]
struct LocalVariables<'ctx> {
    // map to variable name to pointer value.
    data: HashMap<String, Vec<PointerValue<'ctx>>>,
}

fn generate_code<'ctx>(
    expr: Arc<Expr>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    scope: &mut LocalVariables<'ctx>,
) -> PointerValue<'ctx> {
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

fn generate_code_literal<'ctx>(
    lit: Arc<Literal>,
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
) -> PointerValue<'ctx> {
    match &*lit.ty {
        Type::LitTy(ty) => match ty.value.as_str() {
            "Int" => {
                let int_obj_type = int_object_type(context);
                // NOTE: Only once allocation is needed since we don't implement weak_ptr
                let ptr_int_obj = builder.build_malloc(int_obj_type, "int_obj_type").unwrap();
                generate_code_clear_ref_cnt(context, builder, ptr_int_obj);
                let value = lit.value.parse::<i64>().unwrap();
                let value = context.i64_type().const_int(value as u64, false);
                generate_code_set_field(context, builder, ptr_int_obj, 0, value);
                ptr_int_obj
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
        Type::FunTy(_, _) => panic!("Type of given Literal is FunTy (should be TyLit)."), // e.g., fix
        Type::ForAllTy(_, _) => panic!("Type of given Literal is ForAllTy (should be TyLit)."),
    }
}

fn generate_code_clear_ref_cnt<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    obj: PointerValue<'ctx>,
) {
    let ptr_to_refcnt = builder.build_struct_gep(obj, 0, "ptr_to_refcnt").unwrap();
    builder.build_store(ptr_to_refcnt, context.i64_type().const_zero());
}

fn generate_code_set_field<'ctx, V>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    obj: PointerValue<'ctx>,
    index: u32,
    value: V,
) where
    V: BasicValue<'ctx>,
{
    let ptr_to_field = builder
        .build_struct_gep(obj, index + 1, "ptr_to_field")
        .unwrap();
    builder.build_store(ptr_to_field, value);
}

fn object_type<'ctx>(
    context: &'ctx Context,
    field_types: &[BasicTypeEnum<'ctx>],
) -> StructType<'ctx> {
    let mut fields: Vec<BasicTypeEnum<'ctx>> = vec![context.i64_type().into()];
    fields.append(&mut field_types.to_vec());
    context.struct_type(&fields, false)
}

fn int_object_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    object_type(context, &[context.i64_type().into()])
}

type SystemFunctionId = i32;
#[derive(Eq, Hash, PartialEq)]
enum SystemFunctions {
    Printf,
    PrintIntObj,
}

fn generate_code_printf<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    system_functions: &mut HashMap<SystemFunctions, FunctionValue<'ctx>>,
) {
    let i32_type = context.i32_type();
    let i8_type = context.i8_type();
    let i8_ptr_type = i8_type.ptr_type(inkwell::AddressSpace::Generic);

    let fn_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
    let func = module.add_function("printf", fn_type, None);
    system_functions.insert(SystemFunctions::Printf, func);
}

fn generate_code_print_int_obj<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    system_functions: &mut HashMap<SystemFunctions, FunctionValue<'ctx>>,
) {
    let builder = context.create_builder();

    let void_type = context.void_type();
    let int_obj_type = int_object_type(&context);
    let int_obj_ptr_type = int_obj_type.ptr_type(AddressSpace::Generic);
    let fn_type = void_type.fn_type(&[int_obj_ptr_type.into()], false);
    let func = module.add_function("print_int_obj", fn_type, None);

    let entry_bb = context.append_basic_block(func, "entry");
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

    system_functions.insert(SystemFunctions::PrintIntObj, func);
}

fn generate_code_system_functions<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    system_functions: &mut HashMap<SystemFunctions, FunctionValue<'ctx>>,
) {
    generate_code_printf(context, module, system_functions);
    generate_code_print_int_obj(context, module, system_functions);
}

fn execute_main_module<'ctx>(module: &Module<'ctx>) {
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .unwrap();
    unsafe {
        execution_engine
            .get_function::<unsafe extern "C" fn()>("main")
            .unwrap()
            .call();
    }
}

fn main() {
    let program = mk_int_expr(-42);

    let context = Context::create();
    let module = context.create_module("main");

    let mut system_functions = Default::default();
    generate_code_system_functions(&context, &module, &mut system_functions);

    let i32_type = context.i32_type();
    let main_fn_type = i32_type.fn_type(&[], false);
    let main_function = module.add_function("main", main_fn_type, None);

    let builder = context.create_builder();
    let entry_bb = context.append_basic_block(main_function, "entry");
    builder.position_at_end(entry_bb);

    let mut local_variables: LocalVariables = Default::default();
    let program_result = generate_code(program, &context, &module, &builder, &mut local_variables);

    let print_int_obj = *system_functions.get(&SystemFunctions::PrintIntObj).unwrap();
    builder.build_call(
        print_int_obj,
        &[program_result.into()],
        "print_program_result",
    );

    builder.build_return(Some(&i32_type.const_int(0, false)));

    module.print_to_file("ir").unwrap();
    execute_main_module(&module);
}
