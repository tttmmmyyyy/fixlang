use either::Either;
use inkwell::basic_block::BasicBlock;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::{BasicTypeEnum, FunctionType, IntType, PointerType, StructType};
use inkwell::values::{
    BasicValue, BasicValueEnum, CallableValue, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, OptimizationLevel};
use once_cell::sync::Lazy;
use std::alloc::System;
use std::collections::HashMap;
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

fn mk_termvar_var(var_name: &str, ty: Arc<Type>) -> Arc<Var> {
    Arc::new(Var::TermVar {
        name: String::from(var_name),
        ty: ty,
    })
}

fn mk_intvar_var(var_name: &str) -> Arc<Var> {
    mk_termvar_var(var_name, mk_lit_type("Int"))
}

fn mk_let(var: Arc<Var>, bound: Arc<Expr>, expr: Arc<Expr>) -> Arc<Expr> {
    Arc::new(Expr::Let(var, bound, expr))
}

fn mk_var_expr(var_name: &str, ty: Arc<Type>) -> Arc<Expr> {
    Arc::new(Expr::Var(mk_termvar_var(var_name, ty)))
}

fn mk_intvar_expr(var_name: &str) -> Arc<Expr> {
    mk_var_expr(var_name, mk_lit_type("Int"))
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

struct GenerationContext<'c, 'm, 'b> {
    context: &'c Context,
    module: &'m Module<'c>,
    builder: &'b Builder<'c>,
    scope: LocalVariables<'c>,
    system_functions: HashMap<SystemFunctions, FunctionValue<'c>>,
}

fn generate_expr<'c, 'm, 'b>(
    expr: Arc<Expr>,
    context: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    match &*expr {
        Expr::Var(var) => match &**var {
            Var::TermVar { name, ty: _ } => {
                let (code, _) = context.scope.data.get(name).unwrap().last().unwrap();
                // We need to retain here since the pointer value is cloned.
                build_retain(code.ptr, context);
                code.clone()
            }
            Var::TyVar { name: _, kind: _ } => unreachable!(),
        },
        Expr::Lit(lit) => generate_literal(lit.clone(), context),
        Expr::App(lambda, arg) => generate_app(lambda.clone(), arg.clone(), context),
        Expr::Lam(_, _) => todo!(),
        Expr::Let(var, bound, expr) => {
            generate_let(var.clone(), bound.clone(), expr.clone(), context)
        }
        Expr::If(_, _, _) => todo!(),
        Expr::Type(_) => todo!(),
    }
}

fn generate_app<'c, 'm, 'b>(
    lambda: Arc<Expr>,
    arg: Arc<Expr>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    let builder = gc.builder;
    let lambda = generate_expr(lambda, gc);
    let arg = generate_expr(arg, gc);
    let ptr_to_func = build_ptr_to_func_of_lambda(lambda.ptr, gc);
    let lambda_func = CallableValue::try_from(ptr_to_func).unwrap();
    let ret = builder.build_call(
        lambda_func,
        &[arg.ptr.into(), lambda.ptr.into()],
        "call_lambda",
    );
    let ret = ret.try_as_basic_value().unwrap_left().into_pointer_value();
    // TODO: convert ret into appropriate type.
    ExprCode { ptr: ret }
    // We do not release arg.ptr and lambda.ptr here since we have moved them into the arguments of lambda_func.
}

fn generate_literal<'c, 'm, 'b>(
    lit: Arc<Literal>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    match &*lit.ty {
        Type::LitTy(ty) => match ty.value.as_str() {
            "Int" => {
                let ptr_to_int_obj = ObjectType::int_obj_type().build_allocate_shared_obj(gc);
                let value = lit.value.parse::<i64>().unwrap();
                let value = gc.context.i64_type().const_int(value as u64, false);
                build_set_field(ptr_to_int_obj, 0, value, gc);
                ExprCode {
                    ptr: ptr_to_int_obj,
                }
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

fn generate_let<'c, 'm, 'b>(
    var: Arc<Var>,
    bound: Arc<Expr>,
    expr: Arc<Expr>,
    gc: &mut GenerationContext<'c, 'm, 'b>,
) -> ExprCode<'c> {
    // We don't retain here because the result of generate_expr is considered to be "moved into" the variable.
    let bound_val = generate_expr(bound.clone(), gc);
    let var_name;
    let var_type;
    match &*var {
        Var::TermVar { name, ty } => {
            var_name = name.clone();
            var_type = ty.clone();
        }
        Var::TyVar { name: _, kind: _ } => unimplemented!(),
    }
    if !gc.scope.data.contains_key(&var_name) {
        gc.scope.data.insert(var_name.clone(), Default::default());
    }
    gc.scope
        .data
        .get_mut(&var_name)
        .unwrap()
        .push((bound_val.clone(), var_type));
    let expr_val = generate_expr(expr.clone(), gc);
    gc.scope.data.get_mut(&var_name).unwrap().pop();
    build_release(bound_val.ptr, gc);
    expr_val
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
        .build_struct_gep(obj, index + 1, "ptr_to_field")
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
        .build_struct_gep(obj, index + 1, "ptr_to_field")
        .unwrap();
    builder.build_load(ptr_to_field, "field_value")
}

fn build_ptr_to_func_of_lambda<'c, 'm, 'b>(
    obj: PointerValue<'c>,
    gc: &GenerationContext<'c, 'm, 'b>,
) -> PointerValue<'c> {
    build_get_field(obj, 0, gc).into_pointer_value()
}

fn build_retain<'c, 'm, 'b>(ptr_to_obj: PointerValue, context: &GenerationContext<'c, 'm, 'b>) {
    context.builder.build_call(
        *context
            .system_functions
            .get(&SystemFunctions::RetainObj)
            .unwrap(),
        &[ptr_to_obj.clone().into()],
        "retain",
    );
}

fn build_release<'c, 'm, 'b>(ptr_to_obj: PointerValue, context: &GenerationContext<'c, 'm, 'b>) {
    context.builder.build_call(
        *context
            .system_functions
            .get(&SystemFunctions::ReleaseObj)
            .unwrap(),
        &[ptr_to_obj.clone().into()],
        "release",
    );
}

#[derive(Eq, Hash, PartialEq, Clone)]
enum ObjectFieldType {
    ControlBlock,
    LambdaFunction,
    SubObject,
    Int,
}

impl ObjectFieldType {
    fn to_basic_type<'ctx>(&self, context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(context).into(),
            ObjectFieldType::LambdaFunction => ptr_to_lambda_function_type(context).into(),
            ObjectFieldType::SubObject => refcnt_type(context).into(),
            ObjectFieldType::Int => context.i64_type().into(),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
struct ObjectType {
    field_types: Vec<ObjectFieldType>,
}

impl ObjectType {
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
        let void_type = gc.context.void_type();
        let ptr_to_obj_type = ptr_to_refcnt_type(gc.context);
        let func_type = dtor_type(gc.context);
        let func = gc.module.add_function("dtor", func_type, None);
        let bb = gc.context.append_basic_block(func, "entry");
        let builder = gc.context.create_builder();
        builder.position_at_end(bb);
        let ptr_to_obj = func
            .get_first_param()
            .unwrap()
            .into_pointer_value()
            .const_cast(struct_type.ptr_type(AddressSpace::Generic));
        for (i, ft) in self.field_types.iter().enumerate() {
            match ft {
                ObjectFieldType::SubObject => {
                    let ptr_to_field = builder
                        .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_field")
                        .unwrap();
                    build_release(ptr_to_field, &gc);
                }
                ObjectFieldType::ControlBlock => {}
                ObjectFieldType::Int => {}
                ObjectFieldType::LambdaFunction => {}
            }
        }
        builder.build_return(None);
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

// TODO: reomve use of this
fn ptr_to_object_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    context.i8_type().ptr_type(AddressSpace::Generic)
}

fn dtor_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    context.void_type().fn_type(
        &[refcnt_type(context).ptr_type(AddressSpace::Generic).into()],
        false,
    )
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

#[derive(Eq, Hash, PartialEq)]
enum SystemFunctions {
    Printf,
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
    let ptr_to_refcnt_type = context.i64_type().ptr_type(AddressSpace::Generic);
    let func_type = void_type.fn_type(&[ptr_to_refcnt_type.into()], false);
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

fn generate_func_release_obj<'c, 'm, 'b>(gc: &GenerationContext<'c, 'm, 'b>) -> FunctionValue<'c> {
    let context = gc.context;
    let module = gc.module;
    let system_functions = &gc.system_functions;
    let void_type = context.void_type();
    let ptr_to_refcnt_type = ptr_to_refcnt_type(context);
    let func_type = void_type.fn_type(&[ptr_to_refcnt_type.into()], false);
    let release_func = module.add_function("release_obj", func_type, None);
    let mut bb = context.append_basic_block(release_func, "entry");

    let builder = context.create_builder();
    builder.position_at_end(bb);
    let ptr_to_obj = release_func.get_first_param().unwrap().into_pointer_value();
    let ptr_to_control_block = ptr_to_obj.const_cast(ptr_to_control_block_type(context));
    let ptr_to_refcnt = builder
        .build_struct_gep(ptr_to_control_block, 0, "ptr_to_refcnt")
        .unwrap();
    let refcnt = builder.build_load(ptr_to_refcnt, "refcnt").into_int_value();

    if DEBUG_MEMORY {
        // check if refcnt is positive
        let zero = context.i64_type().const_zero();
        let is_positive = builder.build_int_compare(
            inkwell::IntPredicate::ULE,
            refcnt,
            zero,
            "is_refcnt_positive",
        );
        let then_bb = context.append_basic_block(release_func, "error_refcnt_already_leq_zero");
        let cont_bb = context.append_basic_block(release_func, "refcnt_positive");
        builder.build_conditional_branch(is_positive, then_bb, cont_bb);

        builder.position_at_end(then_bb);
        let string_ptr = builder.build_global_string_ptr(
            "Release object whose refcnt is already %lld\n",
            "release_error_msg",
        );
        builder.build_call(
            *system_functions.get(&SystemFunctions::Printf).unwrap(),
            &[string_ptr.as_pointer_value().into(), refcnt.into()],
            "print_error_in_release",
        );
        builder.build_unreachable();
        // builder.build_unconditional_branch(cont_bb);

        bb = cont_bb;
        builder.position_at_end(bb);
    }

    let one = context.i64_type().const_int(1, false);
    let refcnt = builder.build_int_sub(refcnt, one, "refcnt");
    let zero = context.i64_type().const_zero();
    let is_refcnt_zero =
        builder.build_int_compare(inkwell::IntPredicate::EQ, refcnt, zero, "is_refcnt_zero");
    let then_bb = context.append_basic_block(release_func, "refcnt_zero_after_release");
    let cont_bb = context.append_basic_block(release_func, "end");
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
    gc.system_functions.insert(
        SystemFunctions::PrintIntObj,
        generate_func_print_int_obj(gc),
    );
    gc.system_functions
        .insert(SystemFunctions::RetainObj, generate_func_retain_obj(gc));
    gc.system_functions
        .insert(SystemFunctions::ReleaseObj, generate_func_release_obj(gc));
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

fn test_int_program(program: Arc<Expr>, answer: i32) {
    let context = Context::create();
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
    let value = build_get_field(program_result.ptr, 0, &gc);
    if let BasicValueEnum::IntValue(value) = value {
        builder.build_return(Some(&value));
    } else {
        unreachable!()
        // builder.build_return(Some(&i32_type.const_int(0, false)));
    }

    module.print_to_file("ir").unwrap();
    assert_eq!(execute_main_module(&module), answer);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn int_literal() {
        let program = mk_int_expr(-42);
        test_int_program(program, -42);
    }
    #[test]
    fn let0() {
        let program = mk_let(mk_intvar_var("x"), mk_int_expr(-42), mk_int_expr(42));
        test_int_program(program, 42);
    }
    #[test]
    fn let1() {
        let program = mk_let(mk_intvar_var("x"), mk_int_expr(-42), mk_intvar_expr("x"));
        test_int_program(program, -42);
    }
    #[test]
    fn let2() {
        let program = mk_let(
            mk_intvar_var("n"),
            mk_int_expr(-42),
            mk_let(mk_intvar_var("p"), mk_int_expr(42), mk_intvar_expr("n")),
        );
        test_int_program(program, -42);
    }
    #[test]
    fn let3() {
        let program = mk_let(
            mk_intvar_var("n"),
            mk_int_expr(-42),
            mk_let(mk_intvar_var("p"), mk_int_expr(42), mk_intvar_expr("p")),
        );
        test_int_program(program, 42);
    }
    #[test]
    fn let4() {
        let program = mk_let(
            mk_intvar_var("x"),
            mk_int_expr(-42),
            mk_let(mk_intvar_var("x"), mk_int_expr(42), mk_intvar_expr("x")),
        );
        test_int_program(program, 42);
    }
    #[test]
    fn let5() {
        let program = mk_let(
            mk_intvar_var("x"),
            mk_let(mk_intvar_var("y"), mk_int_expr(42), mk_intvar_expr("y")),
            mk_intvar_expr("x"),
        );
        test_int_program(program, 42);
    }
}

fn main() {}
