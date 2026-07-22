use crate::ast::{
    expr::{
        expr_abs, expr_abs_many, expr_app, expr_if, expr_let, expr_llvm, expr_make_struct,
        expr_var, var_local, AppSourceCodeOrderType, ExprNode,
    },
    inline_llvm::{unique_check_on_boxed_leaf, LLVMGen},
    name::{FullName, Name, NameSpace},
    pattern::PatternNode,
    predicate::Predicate,
    program::TypeEnv,
    qual_pred::QualPred,
    traits::{TraitId, TraitImpl},
    typedecl::{Field, Struct, TypeDeclValue, TypeDefn},
    types::{
        kind_arrow, kind_star, make_tyvar, tycon, type_from_tyvar, type_fun, type_tyapp,
        type_tycon, type_tyvar, type_tyvar_star, Kind, Scheme, TyCon, TyConInfo, TyConVariant,
        TypeNode,
    },
};
use crate::constants::{
    ARRAY_BUF_IDX, ARRAY_CAP_IDX, ARRAY_LEN_IDX, ARRAY_NAME, ARRAY_UNSAFE_EMPTY_NAME,
    ARRAY_UNSAFE_GET_LINEAR_BOUNDS_UNCHECKED_UNRETAINED, ARROW_NAME, BOOL_NAME, BOXED_TRAIT_NAME,
    BOXED_TYPE_DATA_IDX, CAP_NAME, CLOSURE_CAPTURE_IDX, CLOSURE_FUNPTR_IDX, CONST_NAME,
    DESTRUCTOR_NAME, DESTRUCTOR_OBJECT_DTOR_FIELD_IDX, DESTRUCTOR_OBJECT_VALUE_FIELD_IDX,
    DYNAMIC_OBJECT_NAME, F32_NAME, F64_NAME, FFI_NAME, FUNCTOR_NAME, FUNPTR_ARGS_MAX, FUNPTR_NAME,
    I16_NAME, I32_NAME, I64_NAME, I8_NAME, IDENTITY_NAME, IOSTATE_NAME, IO_NAME, LAZY_NAME,
    PTR_NAME, PUNCHED_ARRAY_NAME, STD_NAME, STRING_NAME, STRUCT_GETTER_SYMBOL,
    STRUCT_PLUG_IN_FORCE_UNIQUE_SYMBOL, STRUCT_PLUG_IN_SYMBOL, STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL,
    STRUCT_PUNCH_SYMBOL, STRUCT_SETTER_SYMBOL, TraverserWorkType, TUPLE_NAME, TUPLE_UNBOX, U16_NAME,
    U32_NAME, U64_NAME, U8_NAME, UNION_DATA_IDX,
};
use crate::error::panic_with_msg;
use crate::fixstd::runtime::{RUNTIME_ABORT, RUNTIME_EPRINTLN, RUNTIME_REALLOC};
use crate::generator::{Generator, Object};
use crate::misc::{make_map, Map, Set};
use crate::object::{create_obj, ObjectFieldType};
use crate::optimization::rename::generate_new_names;
use crate::parse::sourcefile::Span;
use crate::rc_ir::ast::{FieldPath, UniqueCheckOperand};
use crate::rc_ir::provenance::{LeafOrigin, Provenance};
use inkwell::module::Linkage;
use inkwell::values::{BasicValue, IntValue, PointerValue};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

// Implement built-in functions, types, etc.

pub fn bulitin_tycons() -> Map<TyCon, TyConInfo> {
    let mut ret = Map::default();
    // Primitive types
    ret.insert(
        TyCon::new(make_iostate_name()),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some(include_str!("../docs/std_iostate.md").to_string()),
        },
    );

    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], PTR_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of pointers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], U8_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 8-bit unsinged integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], I8_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 8-bit signed integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], U16_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 16-bit unsigned integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], I16_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 16-bit signed integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], I32_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 32-bit signed integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], U32_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 32-bit unsigned integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], I64_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 64-bit signed integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], U64_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 64-bit unsigned integers.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], F32_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 32-bit floating point values.".to_string()),
        },
    );
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], F64_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of 64-bit floating point values.".to_string()),
        },
    );

    // Array
    ret.insert(
        make_array_tycon(),
        TyConInfo {
            kind: kind_arrow(kind_star(), kind_star()),
            variant: TyConVariant::Array,
            is_unbox: false,
            tyvars: vec![make_tyvar("a", &kind_star())],
            fields: vec![Field::make(
                "array_elem".to_string(), // Unused
                type_tyvar_star("a"),
                None,
            )],
            source: None,
            document: Some("The type of variable length arrays. This is a boxed type.".to_string()),
        },
    );

    // Arrow
    ret.insert(
        make_arrow_tycon(),
        TyConInfo {
            kind: kind_arrow(kind_star(), kind_arrow(kind_star(), kind_star())),
            variant: TyConVariant::Arrow,
            is_unbox: true,
            tyvars: vec![
                make_tyvar("a", &kind_star()),
                make_tyvar("b", &kind_star()),
            ],
            fields: vec![],
            source: None,
            document: Some("`Arrow a b` represents the type of a function that takes a value of type `a` and returns a value of type `b`. Usually written as `a -> b`.".to_string()),
        },
    );

    // Function Pointers
    for arity in 1..=FUNPTR_ARGS_MAX {
        ret.insert(
            make_funptr_tycon(arity),
            TyConInfo {
                kind: make_kind_fun(arity),
                variant: TyConVariant::Primitive,
                is_unbox: true,
                tyvars: (0..arity)
                    .map(|i| make_tyvar(&format!("a{}", i), &kind_star()))
                    .collect(),
                fields: vec![],
                source: None,
                document: None,
            },
        );
    }

    // Dynamic object
    ret.insert(
        make_dynamic_object_tycon(),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::DynamicObject,
            is_unbox: false,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: None,
        },
    );

    ret
}

pub fn make_arrow_name_abs() -> FullName {
    let mut name = FullName::from_strs(&[STD_NAME], ARROW_NAME);
    name.set_absolute();
    name
}

pub fn make_arrow_tycon() -> TyCon {
    TyCon::new(make_arrow_name_abs())
}

pub fn make_dynamic_object_name() -> FullName {
    FullName::from_strs(&[STD_NAME], DYNAMIC_OBJECT_NAME)
}

pub fn make_dynamic_object_tycon() -> TyCon {
    TyCon::new(make_dynamic_object_name())
}

pub fn make_destructor_name() -> FullName {
    FullName::from_strs(&[STD_NAME, FFI_NAME], DESTRUCTOR_NAME)
}

pub fn make_destructor_ty(val_ty: Arc<TypeNode>) -> Arc<TypeNode> {
    type_tyapp(type_tycon(&tycon(make_destructor_name())), val_ty)
}

pub fn make_functor_name() -> FullName {
    FullName::from_strs(&[STD_NAME], FUNCTOR_NAME)
}

pub fn make_funptr_name(arity: u32) -> Name {
    format!("{}{}", FUNPTR_NAME, arity)
}

pub fn make_funptr_tycon(arity: u32) -> TyCon {
    TyCon::new(FullName::from_strs(&[STD_NAME], &make_funptr_name(arity)))
}

pub fn make_array_tycon() -> TyCon {
    TyCon::new(make_array_name())
}

pub fn make_array_name() -> FullName {
    FullName::from_strs(&[STD_NAME], ARRAY_NAME)
}

// If given tycon is function pointer, returns its arity
pub fn is_funptr_tycon(tc: &TyCon) -> Option<u32> {
    if tc.name.namespace != NameSpace::new(vec![STD_NAME.to_string()]) {
        return None;
    }
    if !tc.name.name.starts_with(FUNPTR_NAME) {
        return None;
    }
    let mut chars = tc.name.name.chars();
    for _ in 0..FUNPTR_NAME.len() {
        chars.next();
    }
    let number = chars.as_str().to_string();
    Some(number.parse::<u32>().unwrap())
}

// Returns whether given tycon is dyanmic object
pub fn is_dynamic_object_tycon(tc: &TyCon) -> bool {
    tc.name == make_dynamic_object_name()
}

// Returns whether given tycon is Std::Destructor
pub fn is_destructor_object_tycon(tc: &TyCon) -> bool {
    tc.name == make_destructor_name()
}

// Returns whether given tycon is array
pub fn is_array_tycon(tc: &TyCon) -> bool {
    *tc == make_array_tycon()
}

pub fn make_punched_array_tycon() -> TyCon {
    TyCon::new(FullName::from_strs(&[STD_NAME], PUNCHED_ARRAY_NAME))
}

// Returns whether given tycon is a punched array (`Std::PunchedArray`).
pub fn is_punched_array_tycon(tc: &TyCon) -> bool {
    *tc == make_punched_array_tycon()
}

// Make `Std::Boxed` trait.
pub fn make_boxed_trait() -> TraitId {
    TraitId::from_fullname(FullName::from_strs(&[STD_NAME], BOXED_TRAIT_NAME))
}

pub fn make_kind_fun(arity: u32) -> Arc<Kind> {
    let mut res = kind_star();
    for _ in 0..arity {
        res = kind_arrow(kind_star(), res);
    }
    res
}

pub fn make_iostate_name() -> FullName {
    FullName::from_strs(&[STD_NAME, IO_NAME], IOSTATE_NAME)
}

// Make the `IOState` type.
pub fn make_iostate_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(make_iostate_name()))
}

// Get Ptr type.
pub fn make_ptr_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], PTR_NAME)))
}

// Get U8 type.
pub fn make_u8_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], U8_NAME)))
}

// Get I8 type.
pub fn make_i8_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], I8_NAME)))
}

// Get U16 type.
pub fn make_u16_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], U16_NAME)))
}

// Get I16 type.
pub fn make_i16_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], I16_NAME)))
}

// Get I32 type.
pub fn make_i32_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], I32_NAME)))
}

// Get U32 type.
pub fn make_u32_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], U32_NAME)))
}

// Get I64 type.
pub fn make_i64_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], I64_NAME)))
}

// Get U32 type.
pub fn make_u64_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], U64_NAME)))
}

// Get F32 type.
pub fn make_f32_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], F32_NAME)))
}

// Get F64 type.
pub fn make_f64_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], F64_NAME)))
}

// Get Bool type.
pub fn make_bool_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], BOOL_NAME)))
}

// Get Array type.
pub fn make_array_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], ARRAY_NAME)))
}

pub fn make_punched_array_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], PUNCHED_ARRAY_NAME)))
}

pub fn integral_types() -> Vec<Arc<TypeNode>> {
    vec![
        make_i8_ty(),
        make_u8_ty(),
        make_i16_ty(),
        make_u16_ty(),
        make_i32_ty(),
        make_u32_ty(),
        make_i64_ty(),
        make_u64_ty(),
    ]
}

pub fn floating_types() -> Vec<Arc<TypeNode>> {
    vec![make_f32_ty(), make_f64_ty()]
}

// Get the TyCon of String type.
pub fn make_string_tycon() -> Arc<TyCon> {
    tycon(FullName::from_strs(&[STD_NAME], STRING_NAME))
}

// Get integral types from its name.
pub fn make_integral_ty(name: &str) -> Option<Arc<TypeNode>> {
    if name == I8_NAME {
        Some(make_i8_ty())
    } else if name == U8_NAME {
        Some(make_u8_ty())
    } else if name == I16_NAME {
        Some(make_i16_ty())
    } else if name == U16_NAME {
        Some(make_u16_ty())
    } else if name == I32_NAME {
        Some(make_i32_ty())
    } else if name == U32_NAME {
        Some(make_u32_ty())
    } else if name == I64_NAME {
        Some(make_i64_ty())
    } else if name == U64_NAME {
        Some(make_u64_ty())
    } else {
        None
    }
}

pub fn integral_ty_range(name: &str) -> (BigInt, BigInt) {
    if name == I8_NAME {
        (BigInt::from(i8::MIN), BigInt::from(i8::MAX))
    } else if name == U8_NAME {
        (BigInt::from(0), BigInt::from(u8::MAX))
    } else if name == I16_NAME {
        (BigInt::from(i16::MIN), BigInt::from(i16::MAX))
    } else if name == U16_NAME {
        (BigInt::from(0), BigInt::from(u16::MAX))
    } else if name == I32_NAME {
        (BigInt::from(i32::MIN), BigInt::from(i32::MAX))
    } else if name == U32_NAME {
        (BigInt::from(0), BigInt::from(u32::MAX))
    } else if name == I64_NAME {
        (BigInt::from(i64::MIN), BigInt::from(i64::MAX))
    } else if name == U64_NAME {
        (BigInt::from(0), BigInt::from(u64::MAX))
    } else {
        panic!("Not an integral type: {}", name);
    }
}

// Get floating types from its name.
pub fn make_floating_ty(name: &str) -> Option<Arc<TypeNode>> {
    if name == F32_NAME {
        Some(make_f32_ty())
    } else if name == F64_NAME {
        Some(make_f64_ty())
    } else {
        None
    }
}

// Get numeric types from its name.
// Returns (type, is_float)
pub fn make_numeric_ty(name: &str) -> (Option<Arc<TypeNode>>, bool) {
    let int_opt = make_integral_ty(name);
    if int_opt.is_some() {
        return (int_opt, false);
    }
    (make_floating_ty(name), true)
}

// Get dynamic object type.
pub fn make_dynamic_object_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(
        &[STD_NAME],
        DYNAMIC_OBJECT_NAME,
    )))
}

// Get tuple type.
pub fn make_tuple_ty(tys: Vec<Arc<TypeNode>>) -> Arc<TypeNode> {
    let mut ty = type_tycon(&tycon(make_tuple_name_abs(tys.len() as u32)));
    for field_ty in tys {
        ty = type_tyapp(ty, field_ty);
    }
    ty
}

// Make tuple name
pub fn make_tuple_name(size: u32) -> FullName {
    FullName::from_strs(&[STD_NAME], &format!("{}{}", TUPLE_NAME, size))
}

// Make absolute tuple name, e.g., `::Std::Tuple3`
pub fn make_tuple_name_abs(size: u32) -> FullName {
    let mut name = make_tuple_name(size);
    name.set_absolute();
    name
}

// Get Unit type.
pub fn make_unit_ty() -> Arc<TypeNode> {
    make_tuple_ty(vec![])
}

// Get Lazy.
pub fn make_lazy_ty() -> Arc<TypeNode> {
    let name = FullName::from_strs(&[STD_NAME], LAZY_NAME);
    type_tycon(&tycon(name))
}

// Make type `IO`
pub fn make_io_ty() -> Arc<TypeNode> {
    type_tycon(&make_io_tycon())
}

// Make type `IOState -> (IOState, a)`
pub fn make_io_runner_ty(res_ty: Arc<TypeNode>) -> Arc<TypeNode> {
    type_fun(
        make_iostate_ty(),
        make_tuple_ty(vec![make_iostate_ty(), res_ty]),
    )
}

// Make tycon `IO`
pub fn make_io_tycon() -> Arc<TyCon> {
    tycon(FullName::from_strs(&[STD_NAME], IO_NAME))
}

// Make type `IO ()`
pub fn make_io_unit_ty() -> Arc<TypeNode> {
    type_tyapp(make_io_ty(), make_unit_ty())
}

// Check if given name has form `TupleN` and returns N.
pub fn get_tuple_n(name: &FullName) -> Option<u32> {
    if name.namespace != NameSpace::new_str(&[STD_NAME]) {
        return None;
    }
    if name.name.len() < TUPLE_NAME.len() {
        return None;
    }
    let prefix = &name.name[..TUPLE_NAME.len()];
    if prefix != TUPLE_NAME {
        return None;
    }
    let number_str = &name.name[TUPLE_NAME.len()..];
    number_str.parse::<u32>().ok()
}

pub fn tuple_defn(size: u32) -> TypeDefn {
    let tyvars = (0..size)
        .map(|i| make_tyvar(&("t".to_string() + &i.to_string()), &kind_star()))
        .collect::<Vec<_>>();
    TypeDefn {
        name: make_tuple_name(size),
        tyvars: tyvars.clone(),
        value: TypeDeclValue::Struct(Struct {
            fields: (0..size)
                .map(|i| {
                    Field::make(
                        i.to_string(),
                        type_from_tyvar(tyvars[i as usize].clone()),
                        None,
                    )
                })
                .collect(),
            is_unbox: TUPLE_UNBOX,
        }),
        source: None,
        name_src: None,
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLit {
    val: i64, // Since `serde_pickle` only supports i64 and not u64, we use i64 here.
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntLit {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        let obj = create_obj(
            ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("LLVM<int_lit_{}>", self.val)),
        );
        let int_ty = ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_int_type();
        let value = int_ty.const_int(self.val as u64, false);
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!("int({})", self.val)
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    fn is_primitve_literal(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn expr_int_lit(val: u64, ty: Arc<TypeNode>, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(Box::new(InlineLLVMIntLit { val: val as i64 }), ty, source).global_to_absolute()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatLit {
    val: f64,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatLit {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        let obj = create_obj(
            ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("float_lit_{}", self.val)),
        );
        let float_ty = ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_float_type();
        let value = float_ty.const_float(self.val);
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!("float({})", self.val)
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    fn is_primitve_literal(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn expr_float_lit(val: f64, ty: Arc<TypeNode>, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(Box::new(InlineLLVMFloatLit { val }), ty, source)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMNullPtrLit {}

#[typetag::serde]
impl LLVMGen for InlineLLVMNullPtrLit {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        let obj = create_obj(ty.clone(), &vec![], None, gc, Some("nullptr"));
        let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
        let value = ptr_ty.const_null();
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        "nullptr".to_string()
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    fn is_primitve_literal(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn expr_nullptr_lit(source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        Box::new(InlineLLVMNullPtrLit {}),
        make_ptr_ty().set_source(source.clone()),
        source,
    )
    .global_to_absolute()
}

pub fn expr_bool_lit(val: bool, source: Option<Span>) -> Arc<ExprNode> {
    // Desugar `true` / `false` to the `Bool` union's constructors, referenced by absolute path
    // so the desugaring resolves without an `import` at the use site.
    let mut ctor =
        FullName::from_strs(&[STD_NAME, BOOL_NAME], if val { "_true" } else { "_false" });
    ctor.set_absolute();
    let unit = expr_make_struct(tycon(make_tuple_name_abs(0)), vec![]);
    expr_app(expr_var(ctor, source.clone()), vec![unit], source)
}

// Create a byte array by copying from given pointer.
pub fn make_byte_array_copy<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    buf: PointerValue<'c>,
    len: IntValue<'c>,
) -> Object<'c> {
    // Create `Array U8` which contains null-terminated string.
    let array_ty = type_tyapp(make_array_ty(), make_u8_ty());
    let array = create_obj(
        array_ty,
        &vec![],
        Some(len),
        gc,
        Some("array@make_byte_array_copy"),
    );
    let array = array.insert_field(gc, ARRAY_LEN_IDX, len);
    let dst = array.gep_boxed(gc, ARRAY_BUF_IDX);
    let len = gc
        .builder()
        .build_int_cast(
            len,
            gc.context.ptr_sized_int_type(&gc.target_data, None),
            "len_ptr@make_byte_array_copy",
        )
        .unwrap();
    gc.builder().build_memcpy(dst, 1, buf, 1, len).ok().unwrap();

    array
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStringBuf {
    string: String,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMStringBuf {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let string_ptr = gc.add_global_string(&self.string).as_pointer_value();
        let len_with_null_terminator = gc
            .context
            .i64_type()
            .const_int(self.string.as_bytes().len() as u64 + 1, false);
        make_byte_array_copy(gc, string_ptr, len_with_null_terminator)
    }

    fn name(&self) -> String {
        format!("string_buf(\"{}\")", self.string)
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    fn is_primitve_literal(&self) -> bool {
        true
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn make_string_lit(string: String, source: Option<Span>) -> Arc<ExprNode> {
    let array_ty = make_array_ty().set_source(source.clone());
    let u8_ty = make_u8_ty().set_source(source.clone());
    let byte_array_ty = type_tyapp(array_ty, u8_ty).set_source(source.clone());
    let expr = expr_make_struct(
        make_string_tycon(),
        vec![(
            "_data".to_string(),
            expr_llvm(
                Box::new(InlineLLVMStringBuf { string }),
                byte_array_ty,
                source.clone(),
            ),
        )],
    )
    .set_source(source)
    .global_to_absolute();

    expr
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFixBody {
    x_str: FullName,
    f_str: FullName,
    cap_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFixBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        self.generate_tail(gc, ty, false).unwrap()
    }

    fn generate_tail<'c, 'm>(
        &self,
        gc: &mut Generator<'c, 'm>,
        _ty: &Arc<TypeNode>,

        tail: bool,
    ) -> Option<Object<'c>> {
        // Get arguments
        let x = gc.get_scoped_obj(&self.x_str);
        let f = gc.get_scoped_obj(&self.f_str);

        // Create "fix(f)" closure.
        let fixf_ty = f.ty.get_lambda_dst();
        let fixf = create_obj(fixf_ty.clone(), &vec![], None, gc, Some("fix(f)"));
        let fixf_funptr = gc
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
            .as_global_value()
            .as_pointer_value();
        let fixf = fixf.insert_field(gc, CLOSURE_FUNPTR_IDX, fixf_funptr);
        let cap_obj = gc.get_scoped_obj(&self.cap_name);
        let cap_obj_ptr = cap_obj.value;
        let fixf = fixf.insert_field(gc, CLOSURE_CAPTURE_IDX, cap_obj_ptr);

        let f_fixf = gc.apply_lambda(f, vec![fixf], false).unwrap();
        gc.apply_lambda(f_fixf, vec![x], tail)
    }

    fn name(&self) -> String {
        format!(
            "fix({}, {}, {})",
            self.f_str.to_string(),
            self.x_str.to_string(),
            self.cap_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.x_str, &mut self.f_str, &mut self.cap_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn fix_body(b: &str, f: &str, x: &str) -> Arc<ExprNode> {
    let f_str = FullName::local(f);
    let x_str = FullName::local(x);
    let cap_name = FullName::local(CAP_NAME);
    expr_llvm(
        Box::new(InlineLLVMFixBody {
            x_str,
            f_str,
            cap_name,
        }),
        type_tyvar_star(b),
        None,
    )
}

// fix = \f: ((a -> b) -> (a -> b)) -> \x: a -> fix_body(b, f, x): b
pub fn fix() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(vec![var_local("x")], fix_body("b", "f", "x"), None),
        None,
    );
    let fixed_ty = type_fun(type_tyvar_star("a"), type_tyvar_star("b"));
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(type_fun(fixed_ty.clone(), fixed_ty.clone()), fixed_ty),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCastIntegralBody {
    from_name: FullName,
    is_source_signed: bool,
    is_target_signed: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMCastIntegralBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, to_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get value
        let from_val = gc.get_scoped_obj_field(&self.from_name, 0).into_int_value();

        // Get target type.
        let to_int = to_ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_int_type();

        // Perform cast.
        let to_val = gc
            .builder()
            .build_int_cast_sign_flag(
                from_val,
                to_int,
                self.is_source_signed,
                "build_int_cast_sign_flag@cast_between_integral_function",
            )
            .unwrap();

        // Return result.
        let obj = create_obj(
            to_ty.clone(),
            &vec![],
            None,
            gc,
            Some("alloca@cast_between_integral_function"),
        );
        obj.insert_field(gc, 0, to_val)
    }

    fn name(&self) -> String {
        format!(
            "cast_int[{}, {}]({})",
            self.is_source_signed,
            self.is_target_signed,
            self.from_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Cast function of integrals
//
// - `to_alias`: A type alias to the target type. If set, it will appear in the documentation.
pub fn cast_between_integral_function(
    from: Arc<TypeNode>,
    to: Arc<TypeNode>,
    to_alias: Option<Arc<TypeNode>>,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const FROM_NAME: &str = "from";
    let from_name = FullName::local(FROM_NAME);

    let is_source_signed = from.toplevel_tycon().unwrap().is_singned_intger();
    let is_target_signed = to.toplevel_tycon().unwrap().is_singned_intger();
    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(from.clone(), to_alias.unwrap_or(to.clone())),
    );
    let expr = expr_abs(
        vec![var_local(FROM_NAME)],
        expr_llvm(
            Box::new(InlineLLVMCastIntegralBody {
                from_name,
                is_target_signed,
                is_source_signed,
            }),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCastFloatBody {
    from_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMCastFloatBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, to_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get value
        let from_val = gc
            .get_scoped_obj_field(&self.from_name, 0)
            .into_float_value();

        // Get target type.
        let to_float = to_ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_float_type();

        // Perform cast.
        let to_val = gc
            .builder()
            .build_float_cast(from_val, to_float, "float_cast@cast_between_float_function")
            .unwrap();

        // Return result.
        let obj = create_obj(
            to_ty.clone(),
            &vec![],
            None,
            gc,
            Some("alloca@cast_between_float_function"),
        );
        obj.insert_field(gc, 0, to_val)
    }

    fn name(&self) -> String {
        format!("cast_float({})", self.from_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Cast function of integrals
//
// - `to_alias`: A type alias to the target type. If set, it will appear in the documentation.
pub fn cast_between_float_function(
    from: Arc<TypeNode>,
    to: Arc<TypeNode>,
    to_alias: Option<Arc<TypeNode>>,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const FROM_NAME: &str = "from";
    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(from.clone(), to_alias.unwrap_or(to.clone())),
    );
    let expr = expr_abs(
        vec![var_local(FROM_NAME)],
        expr_llvm(
            Box::new(InlineLLVMCastFloatBody {
                from_name: FullName::local(FROM_NAME),
            }),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCastIntToFloatBody {
    from_name: FullName,
    is_signed: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMCastIntToFloatBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, to_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get value
        let from_val = gc.get_scoped_obj_field(&self.from_name, 0).into_int_value();

        // Get target type.
        let to_float = to_ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_float_type();

        // Perform cast.
        let to_val = if self.is_signed {
            gc.builder().build_signed_int_to_float(
                from_val,
                to_float,
                "signed_int_to_float@cast_int_to_float_function",
            )
        } else {
            gc.builder().build_unsigned_int_to_float(
                from_val,
                to_float,
                "unsigned_int_to_float@cast_int_to_float_function",
            )
        }
        .unwrap();

        // Return result.
        let obj = create_obj(
            to_ty.clone(),
            &vec![],
            None,
            gc,
            Some("alloca@cast_int_to_float_function"),
        );
        obj.insert_field(gc, 0, to_val)
    }

    fn name(&self) -> String {
        format!(
            "cast_int_to_float[{}]({})",
            self.is_signed,
            self.from_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Cast function from int to float.
pub fn cast_int_to_float_function(
    from: Arc<TypeNode>,
    to: Arc<TypeNode>,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const FROM_NAME: &str = "from";
    let is_signed = from.toplevel_tycon().unwrap().is_singned_intger();

    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(from.clone(), to.clone()),
    );
    let expr = expr_abs(
        vec![var_local(FROM_NAME)],
        expr_llvm(
            Box::new(InlineLLVMCastIntToFloatBody {
                from_name: FullName::local(FROM_NAME),
                is_signed,
            }),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCastFloatToIntBody {
    from_name: FullName,
    is_signed: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMCastFloatToIntBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, to_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get value
        let from_val = gc
            .get_scoped_obj_field(&self.from_name, 0)
            .into_float_value();

        // Get target type.
        let to_int = to_ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_int_type();

        // Perform cast.
        let to_val = if self.is_signed {
            gc.builder()
                .build_float_to_signed_int(
                    from_val,
                    to_int,
                    "float_to_signed_int@cast_float_to_int_function",
                )
                .unwrap()
        } else {
            gc.builder()
                .build_float_to_unsigned_int(
                    from_val,
                    to_int,
                    "float_to_unsigned_int@cast_float_to_int_function",
                )
                .unwrap()
        };

        // Return result.
        let obj = create_obj(
            to_ty.clone(),
            &vec![],
            None,
            gc,
            Some("alloca@cast_float_to_int_function"),
        );
        obj.insert_field(gc, 0, to_val)
    }

    fn name(&self) -> String {
        format!(
            "cast_float_to_int[{}]({})",
            self.is_signed,
            self.from_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Cast function from int to float.
pub fn cast_float_to_int_function(
    from: Arc<TypeNode>,
    to: Arc<TypeNode>,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const FROM_NAME: &str = "from";
    let is_signed = to.toplevel_tycon().unwrap().is_singned_intger();

    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(from.clone(), to.clone()),
    );
    let expr = expr_abs(
        vec![var_local(FROM_NAME)],
        expr_llvm(
            Box::new(InlineLLVMCastFloatToIntBody {
                from_name: FullName::local(FROM_NAME),
                is_signed,
            }),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMShiftBody {
    value_name: FullName,
    n_name: FullName,
    is_left: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMShiftBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // Get value
        let val = gc
            .get_scoped_obj_field(&self.value_name, 0)
            .into_int_value();
        let n = gc.get_scoped_obj_field(&self.n_name, 0).into_int_value();

        let is_signed = ty.toplevel_tycon().unwrap().is_singned_intger();

        // Perform shift operation.
        let to_val = if self.is_left {
            gc.builder()
                .build_left_shift(val, n, "left_shift@shift_function")
                .unwrap()
        } else {
            gc.builder()
                .build_right_shift(val, n, is_signed, "right_shift@shift_function")
                .unwrap()
        };

        // Return result.
        let obj = create_obj(ty.clone(), &vec![], None, gc, Some("alloca@shift_function"));
        obj.insert_field(gc, 0, to_val)
    }

    fn name(&self) -> String {
        format!(
            "shift_{}({}, {})",
            if self.is_left { "left" } else { "right" },
            self.value_name.to_string(),
            self.n_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.value_name, &mut self.n_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Shift functions
pub fn shift_function(ty: Arc<TypeNode>, is_left: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    const VALUE_NAME: &str = "val";
    const N_NAME: &str = "n";

    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(ty.clone(), type_fun(ty.clone(), ty.clone())),
    );
    let expr = expr_abs(
        vec![var_local(N_NAME)],
        expr_abs(
            vec![var_local(VALUE_NAME)],
            expr_llvm(
                Box::new(InlineLLVMShiftBody {
                    value_name: FullName::local(VALUE_NAME),
                    n_name: FullName::local(N_NAME),
                    is_left,
                }),
                ty,
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum BitOperationType {
    Xor,
    Or,
    And,
}

impl BitOperationType {
    pub fn to_string(&self) -> String {
        match self {
            BitOperationType::Xor => "xor",
            BitOperationType::Or => "or",
            BitOperationType::And => "and",
        }
        .to_string()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBitwiseOperationBody {
    lhs_name: FullName,
    rhs_name: FullName,
    op_type: BitOperationType,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMBitwiseOperationBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // Get value
        let lhs = gc.get_scoped_obj_field(&self.lhs_name, 0).into_int_value();
        let rhs = gc.get_scoped_obj_field(&self.rhs_name, 0).into_int_value();

        // Perform cast.
        let val = match self.op_type {
            BitOperationType::Xor => gc
                .builder()
                .build_xor(lhs, rhs, "xor@bitwise_operation_function")
                .unwrap(),
            BitOperationType::Or => gc
                .builder()
                .build_or(lhs, rhs, "or@bitwise_operation_function")
                .unwrap(),
            BitOperationType::And => gc
                .builder()
                .build_and(lhs, rhs, "and@bitwise_operation_function")
                .unwrap(),
        };

        // Return result.
        let obj = create_obj(
            ty.clone(),
            &vec![],
            None,
            gc,
            Some("alloca@bitwise_operation_function"),
        );
        obj.insert_field(gc, 0, val)
    }

    fn name(&self) -> String {
        format!(
            "bit_{}({}, {})",
            self.op_type.to_string(),
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn bitwise_operation_function(
    ty: Arc<TypeNode>,
    op_type: BitOperationType,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const LHS_NAME: &str = "lhs";
    const RHS_NAME: &str = "rhs";

    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(ty.clone(), type_fun(ty.clone(), ty.clone())),
    );
    let expr = expr_abs(
        vec![var_local(LHS_NAME)],
        expr_abs(
            vec![var_local(RHS_NAME)],
            expr_llvm(
                Box::new(InlineLLVMBitwiseOperationBody {
                    lhs_name: FullName::local(LHS_NAME),
                    rhs_name: FullName::local(RHS_NAME),
                    op_type,
                }),
                ty,
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBitNotBody {
    operand_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMBitNotBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // Get value
        let lhs = gc
            .get_scoped_obj_field(&self.operand_name, 0)
            .into_int_value();

        // Perform cast.
        let val = gc
            .builder()
            .build_not(lhs, "not@bitwise_not_function")
            .unwrap();

        // Return result.
        let obj = create_obj(ty.clone(), &vec![], None, gc, Some("alloca@bit_not"));
        obj.insert_field(gc, 0, val)
    }

    fn name(&self) -> String {
        format!("bit_not({})", self.operand_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.operand_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn bit_not_function(ty: Arc<TypeNode>) -> (Arc<ExprNode>, Arc<Scheme>) {
    const OPERAND_NAME: &str = "operand";

    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(ty.clone(), ty.clone()),
    );
    let expr = expr_abs(
        vec![var_local(OPERAND_NAME)],
        expr_llvm(
            Box::new(InlineLLVMBitNotBody {
                operand_name: FullName::local(OPERAND_NAME),
            }),
            ty,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayUnsafeEmpty {
    capacity_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayUnsafeEmpty {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, arr_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get capacity
        let cap = gc
            .get_scoped_obj_field(&self.capacity_name, 0)
            .into_int_value();

        // Allocate
        let array = create_obj(
            arr_ty.clone(),
            &vec![],
            Some(cap),
            gc,
            Some(&format!(
                "{ARRAY_NAME}::{ARRAY_UNSAFE_EMPTY_NAME}({})",
                self.capacity_name.to_string()
            )),
        );

        // Set size to zero.
        let cap = gc.context.i64_type().const_zero();
        array.insert_field(gc, ARRAY_LEN_IDX, cap)
    }

    fn name(&self) -> String {
        format!("array_empty({})", self.capacity_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.capacity_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Make an empty array.
pub fn array_unsafe_empty() -> (Arc<ExprNode>, Arc<Scheme>) {
    const CAPACITY_NAME: &str = "cap";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar);

    let expr = expr_abs(
        vec![var_local(CAPACITY_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayUnsafeEmpty {
                capacity_name: FullName::local(CAPACITY_NAME),
            }),
            array_ty.clone(),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], type_fun(make_i64_ty(), array_ty));
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayUnsafeSetBoundsUniquenessUncheckedUnreleased {
    arr_name: FullName,
    idx_name: FullName,
    value_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayUnsafeSetBoundsUniquenessUncheckedUnreleased {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.arr_name);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();
        let value = gc.get_scoped_obj(&self.value_name);

        // Get array cap and buffer.
        let array_buf = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Perform write and return.
        ObjectFieldType::write_to_array_buf(gc, None, array_buf, idx, value, false);
        array
    }

    fn name(&self) -> String {
        format!(
            "array_set_unreleased({}, {}, {})",
            self.idx_name.to_string(),
            self.value_name.to_string(),
            self.arr_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name, &mut self.value_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // This op writes into the array in place, which the caller may ask for only where nothing else
        // holds it: the name says the uniqueness is unchecked, not absent. So the array it returns is
        // uniquely owned, exactly as it is out of a checked `set` — which is what lets the operation
        // after a fill loop drop its check.
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Set an element to an array, with no uniqueness checking and without releasing the old value.
pub fn array_unsafe_set_bounds_uniqueness_unchecked_unreleased() -> (Arc<ExprNode>, Arc<Scheme>) {
    const IDX_NAME: &str = "idx";
    const ARR_NAME: &str = "array";
    const VALUE_NAME: &str = "val";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(IDX_NAME)],
        expr_abs(
            vec![var_local(VALUE_NAME)],
            expr_abs(
                vec![var_local(ARR_NAME)],
                expr_llvm(
                    Box::new(
                        InlineLLVMArrayUnsafeSetBoundsUniquenessUncheckedUnreleased {
                            arr_name: FullName::local(ARR_NAME),
                            idx_name: FullName::local(IDX_NAME),
                            value_name: FullName::local(VALUE_NAME),
                        },
                    ),
                    array_ty.clone(),
                    None,
                ),
                None,
            ),
            None,
        ),
        None,
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            make_i64_ty(),
            type_fun(elem_tyvar.clone(), type_fun(array_ty.clone(), array_ty)),
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayUnsafeGetBoundsUnchecked {
    arr_name: FullName,
    idx_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayUnsafeGetBoundsUnchecked {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        // // let array = gc.get_var_retained_if_used_later(&self.arr_name);
        // let array = gc.get_scoped_obj_noretain(&self.arr_name);

        // let len = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
        // let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        // let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();
        // let elem = ObjectFieldType::read_from_array_buf(gc, Some(len), buf, ty.clone(), idx);

        // if !gc.is_var_used_later(&self.arr_name) {
        //     gc.release(array);
        // }
        // elem

        // Get argments
        let array = gc.get_scoped_obj_noretain(&self.arr_name);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();

        // Get array buffer
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Get element
        let elem = ObjectFieldType::read_from_array_buf(gc, None, buf, ty.clone(), idx);

        elem
    }

    fn name(&self) -> String {
        format!(
            "array_get({}, {})",
            self.idx_name.to_string(),
            self.arr_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Gets a value from an array, without bounds checking
pub fn array_unsafe_get_bounds_unchecked() -> (Arc<ExprNode>, Arc<Scheme>) {
    const IDX_NAME: &str = "idx";
    const ARR_NAME: &str = "array";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(IDX_NAME)],
        expr_abs(
            vec![var_local(ARR_NAME)],
            expr_llvm(
                Box::new(InlineLLVMArrayUnsafeGetBoundsUnchecked {
                    arr_name: FullName::local(ARR_NAME),
                    idx_name: FullName::local(IDX_NAME),
                }),
                elem_tyvar.clone(),
                None,
            ),
            None,
        ),
        None,
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(make_i64_ty(), type_fun(array_ty, elem_tyvar.clone())),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayUnsafeGetLinearBoundsUncheckedUnretained {
    force_unique: bool,
    arr_name: FullName,
    idx_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayUnsafeGetLinearBoundsUncheckedUnretained {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argments
        let mut array = gc.get_scoped_obj(&self.arr_name);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();

        // If force_unique is set, we need to force the array to be unique.
        if self.force_unique {
            array = make_array_unique(gc, array);
        }

        // Get array buffer
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Get the element.
        let elem_ty = ret_ty.collect_type_argments().get(1).unwrap().clone();
        let elem =
            ObjectFieldType::read_from_array_buf_noretain(gc, None, buf, elem_ty.clone(), idx);

        // Create the return value.
        let res = create_obj(
            ret_ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!(
                "alloca@{}",
                ARRAY_UNSAFE_GET_LINEAR_BOUNDS_UNCHECKED_UNRETAINED
            )),
        );
        let res = ObjectFieldType::move_into_struct_field(gc, res, 0, &array);
        let res = ObjectFieldType::move_into_struct_field(gc, res, 1, &elem);

        res
    }

    fn name(&self) -> String {
        format!(
            "array_get_linear{}({}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.idx_name.to_string(),
            self.arr_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Gets a value from an array, without bounds checking and retaining the returned value.
// Type: I64 -> Array a -> (Array a, a)
pub fn array_unsafe_get_linear_bounds_unchecked_unretained(
    force_unique: bool,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const IDX_NAME: &str = "idx";
    const ARR_NAME: &str = "array";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());
    let res_ty = make_tuple_ty(vec![array_ty.clone(), elem_tyvar.clone()]);

    let expr = expr_abs_many(
        vec![var_local(IDX_NAME), var_local(ARR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayUnsafeGetLinearBoundsUncheckedUnretained {
                force_unique,
                arr_name: FullName::local(ARR_NAME),
                idx_name: FullName::local(IDX_NAME),
            }),
            res_ty.clone(),
            None,
        ),
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(make_i64_ty(), type_fun(array_ty, res_ty.clone())),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayTruncateBoundsUnchecked {
    arr_name: FullName,
    len_name: FullName,
    // When true, clone the array first if it is shared, so the shrink lands in a uniquely owned
    // array. Set false only where the array is statically known to be unique.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayTruncateBoundsUnchecked {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let array = gc.get_scoped_obj(&self.arr_name);
        let new_len = gc.get_scoped_obj_field(&self.len_name, 0).into_int_value();

        // Force the array to be unique before shrinking it in place.
        let array = if self.force_unique {
            make_array_unique(gc, array)
        } else {
            array
        };

        // Release the dropped tail `[new_len, size)`, then lower the length to `new_len`. The caller
        // guarantees `0 <= new_len <= size`, so there is no size check.
        let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
        let size = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        ObjectFieldType::release_or_mark_array_slice(
            gc,
            buf,
            new_len,
            size,
            elem_ty,
            TraverserWorkType::release(),
        );
        array.insert_field(gc, ARRAY_LEN_IDX, new_len)
    }

    fn name(&self) -> String {
        format!(
            "array_truncate{}({}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.len_name.to_string(),
            self.arr_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.len_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Truncates an array to `new_len` elements, releasing the dropped tail, with an internal
// clone-if-shared and no size check.
// The caller must ensure `0 <= new_len <= the array's size`.
// Type: I64 -> Array a -> Array a
pub fn array_truncate_bounds_unchecked() -> (Arc<ExprNode>, Arc<Scheme>) {
    const LEN_NAME: &str = "new_len";
    const ARR_NAME: &str = "array";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(LEN_NAME)],
        expr_abs(
            vec![var_local(ARR_NAME)],
            expr_llvm(
                Box::new(InlineLLVMArrayTruncateBoundsUnchecked {
                    arr_name: FullName::local(ARR_NAME),
                    len_name: FullName::local(LEN_NAME),
                    force_unique: true,
                }),
                array_ty.clone(),
                None,
            ),
            None,
        ),
        None,
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(make_i64_ty(), type_fun(array_ty.clone(), array_ty)),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayAppendValueCapacityUnchecked {
    arr_name: FullName,
    value_name: FullName,
    count_name: FullName,
    // When true, clone the array first if it is shared, so the appended slots land in a uniquely
    // owned array. Set false only where the array is statically known to be unique.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayAppendValueCapacityUnchecked {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let array = gc.get_scoped_obj(&self.arr_name);
        let value = gc.get_scoped_obj(&self.value_name);
        let count = gc.get_scoped_obj_field(&self.count_name, 0).into_int_value();

        // Force the array to be unique before appending in place.
        let array = if self.force_unique {
            make_array_unique(gc, array)
        } else {
            array
        };

        // Write `value` into the `count` uninitialized slots at the end and grow the length. The
        // caller guarantees `count >= 0` and `size + count <= capacity`.
        let size = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        ObjectFieldType::append_value_into_array_buf(gc, buf, size, count, value);
        let new_size = gc.builder().build_int_add(size, count, "new_size").unwrap();
        array.insert_field(gc, ARRAY_LEN_IDX, new_size)
    }

    fn name(&self) -> String {
        format!(
            "array_append_value{}({}, {}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.value_name.to_string(),
            self.count_name.to_string(),
            self.arr_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![
            &mut self.arr_name,
            &mut self.value_name,
            &mut self.count_name,
        ]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Appends `count` copies of `value` to the end of an array, with an internal clone-if-shared and no
// capacity check. The caller must ensure `count >= 0` and `size + count <= capacity`.
// Type: a -> I64 -> Array a -> Array a
pub fn array_append_value_capacity_unchecked() -> (Arc<ExprNode>, Arc<Scheme>) {
    const VALUE_NAME: &str = "value";
    const COUNT_NAME: &str = "count";
    const ARR_NAME: &str = "array";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(VALUE_NAME)],
        expr_abs(
            vec![var_local(COUNT_NAME)],
            expr_abs(
                vec![var_local(ARR_NAME)],
                expr_llvm(
                    Box::new(InlineLLVMArrayAppendValueCapacityUnchecked {
                        arr_name: FullName::local(ARR_NAME),
                        value_name: FullName::local(VALUE_NAME),
                        count_name: FullName::local(COUNT_NAME),
                        force_unique: true,
                    }),
                    array_ty.clone(),
                    None,
                ),
                None,
            ),
            None,
        ),
        None,
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            elem_tyvar.clone(),
            type_fun(make_i64_ty(), type_fun(array_ty.clone(), array_ty)),
        ),
    );
    (expr, scm)
}

// Resize a uniquely owned array's single malloc block to hold `new_cap` elements with `realloc`,
// then update its capacity field. The elements are not touched: `realloc` preserves the block's
// contents, often growing it in place. The caller must ensure the array is unique.
fn realloc_array<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    array: Object<'c>,
    new_cap: IntValue<'c>,
) -> Object<'c> {
    let arr_ptr = array.value.into_pointer_value();
    let object_type = array.ty.get_object_type(&vec![], gc.type_env());
    let sizeof = object_type.size_of(gc, Some(new_cap));
    let realloc_fn = gc
        .module
        .get_function(RUNTIME_REALLOC)
        .expect("realloc is not declared");
    let new_ptr = gc
        .builder()
        .build_call(realloc_fn, &[arr_ptr.into(), sizeof.into()], "realloc_array")
        .unwrap()
        .try_as_basic_value()
        .left()
        .unwrap();
    let new_array = Object::new(new_ptr, array.ty.clone(), gc);
    new_array.insert_field(gc, ARRAY_CAP_IDX, new_cap)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArraySetCapacityBoundsUnchecked {
    arr_name: FullName,
    cap_name: FullName,
    // When true, branch on uniqueness: `realloc` a unique array in place, or allocate a new one and
    // retain-copy a shared array's elements. Set false only where the array is statically known to
    // be unique, leaving just the `realloc`.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArraySetCapacityBoundsUnchecked {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let array = gc.get_scoped_obj(&self.arr_name);
        let new_cap = gc.get_scoped_obj_field(&self.cap_name, 0).into_int_value();

        if !self.force_unique {
            return realloc_array(gc, array, new_cap);
        }

        // Branch on whether the array is unique. `build_branch_by_is_unique` routes a GLOBAL array to
        // the shared side, so `realloc` only ever runs on a genuinely uniquely owned block.
        let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
        let arr_ptr = array.value.into_pointer_value();
        let (unique_bb, shared_bb) = gc.build_branch_by_is_unique(arr_ptr);
        let current_func = unique_bb.get_parent().unwrap();
        let end_bb = gc.context.append_basic_block(current_func, "end_bb@set_capacity");

        // Unique: resize in place with `realloc`.
        gc.builder().position_at_end(unique_bb);
        let realloced_ptr = realloc_array(gc, array.clone(), new_cap)
            .value
            .into_pointer_value();
        let succ_of_unique_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(end_bb).unwrap();

        // Shared: allocate a new block of `new_cap` and retain-copy the live elements, then release
        // the old array.
        gc.builder().position_at_end(shared_bb);
        let len = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
        let cloned = create_obj(
            array.ty.clone(),
            &vec![],
            Some(new_cap),
            gc,
            Some("array_for_set_capacity"),
        );
        let cloned = cloned.insert_field(gc, ARRAY_LEN_IDX, len);
        let cloned_buf = cloned.gep_boxed(gc, ARRAY_BUF_IDX);
        let array_buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        ObjectFieldType::clone_array_buf(gc, len, array_buf, cloned_buf, elem_ty, None);
        gc.release(array.clone());
        let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
        let cloned_ptr = cloned.value.into_pointer_value();
        gc.builder().build_unconditional_branch(end_bb).unwrap();

        // Merge.
        gc.builder().position_at_end(end_bb);
        let phi = gc
            .builder()
            .build_phi(arr_ptr.get_type(), "array_phi@set_capacity")
            .unwrap();
        phi.add_incoming(&[
            (&realloced_ptr, succ_of_unique_bb),
            (&cloned_ptr, succ_of_shared_bb),
        ]);
        Object::new(phi.as_basic_value(), array.ty.clone(), gc)
    }

    fn name(&self) -> String {
        format!(
            "array_set_capacity{}({}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.cap_name.to_string(),
            self.arr_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.cap_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Sets an array's capacity to `new_cap`, `realloc`ing a unique array in place or allocating and
// copying a shared one, with no check that `new_cap` fits the elements. The caller must ensure
// `new_cap >= size`; a smaller capacity causes undefined behavior.
// Type: I64 -> Array a -> Array a
pub fn array_set_capacity_bounds_unchecked() -> (Arc<ExprNode>, Arc<Scheme>) {
    const CAP_NAME: &str = "new_cap";
    const ARR_NAME: &str = "array";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(CAP_NAME)],
        expr_abs(
            vec![var_local(ARR_NAME)],
            expr_llvm(
                Box::new(InlineLLVMArraySetCapacityBoundsUnchecked {
                    arr_name: FullName::local(ARR_NAME),
                    cap_name: FullName::local(CAP_NAME),
                    force_unique: true,
                }),
                array_ty.clone(),
                None,
            ),
            None,
        ),
        None,
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(make_i64_ty(), type_fun(array_ty.clone(), array_ty)),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayAppendCapacityBoundsUnchecked {
    dst_name: FullName,
    src_name: FullName,
    begin_name: FullName,
    end_name: FullName,
    // When true, clone `dst` first if it is shared, so the appended slots land in a uniquely owned
    // array. Set false only where `dst` is statically known to be unique. `src` is read either way.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayAppendCapacityBoundsUnchecked {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let dst = gc.get_scoped_obj(&self.dst_name);
        let src = gc.get_scoped_obj(&self.src_name);
        let begin = gc.get_scoped_obj_field(&self.begin_name, 0).into_int_value();
        let end = gc.get_scoped_obj_field(&self.end_name, 0).into_int_value();
        let elem_ty = dst.ty.field_types(gc.type_env())[0].clone();
        let elem_value_ty = elem_ty.get_embedded_type(gc, &vec![]);
        let n = gc.builder().build_int_sub(end, begin, "append_n").unwrap();

        // Clone `dst` if it is shared, so the append writes into a uniquely owned array.
        let dst = if self.force_unique {
            make_array_unique(gc, dst)
        } else {
            dst
        };
        let dst_len = dst.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
        let dst_buf = dst.gep_boxed(gc, ARRAY_BUF_IDX);
        let dst_write = unsafe {
            gc.builder()
                .build_gep(elem_value_ty, dst_buf, &[dst_len], "append_dst_write")
                .unwrap()
        };

        let src_ptr = src.value.into_pointer_value();
        let src_buf = src.gep_boxed(gc, ARRAY_BUF_IDX);
        let src_len = src.extract_field(gc, ARRAY_LEN_IDX).into_int_value();

        // The elements can be moved out of `src` (with no reference counting) only when `src` is
        // uniquely owned and the whole of it is being appended: a partial move would leave the
        // elements outside the range with no one to release them.
        let zero = gc.context.i64_type().const_zero();
        let is_begin_zero = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, begin, zero, "append_begin_zero")
            .unwrap();
        let is_end_full = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, end, src_len, "append_end_full")
            .unwrap();
        let is_full_range = gc
            .builder()
            .build_and(is_begin_zero, is_end_full, "append_full_range")
            .unwrap();

        let current_func = gc
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let maybe_move_bb = gc.context.append_basic_block(current_func, "append_maybe_move");
        let copy_bb = gc.context.append_basic_block(current_func, "append_copy");
        let end_bb = gc.context.append_basic_block(current_func, "append_end");
        gc.builder()
            .build_conditional_branch(is_full_range, maybe_move_bb, copy_bb)
            .unwrap();

        // Full range: move the elements if `src` is unique, otherwise fall through to the copy.
        gc.builder().position_at_end(maybe_move_bb);
        let (src_unique_bb, src_shared_bb) = gc.build_branch_by_is_unique(src_ptr);

        // Unique `src`: memcpy the elements and zero `src`'s length so releasing it frees the block
        // without touching the moved-out elements. No reference counting.
        gc.builder().position_at_end(src_unique_bb);
        let n_span = unsafe {
            gc.builder()
                .build_gep(
                    elem_value_ty,
                    gc.context.ptr_type(AddressSpace::from(0)).const_null(),
                    &[n],
                    "append_n_span",
                )
                .unwrap()
        };
        let n_bytes = gc
            .builder()
            .build_ptr_to_int(n_span, gc.context.i64_type(), "append_n_bytes")
            .unwrap();
        gc.builder()
            .build_memcpy(dst_write, 1, src_buf, 1, n_bytes)
            .ok()
            .unwrap();
        let src_emptied = src.clone().insert_field(gc, ARRAY_LEN_IDX, zero);
        gc.release(src_emptied);
        gc.builder().build_unconditional_branch(end_bb).unwrap();

        // Shared `src`: the elements stay in `src`, so join the copy path.
        gc.builder().position_at_end(src_shared_bb);
        gc.builder().build_unconditional_branch(copy_bb).unwrap();

        // Copy: retain each element of `src[begin, end)` into `dst`'s tail, then release `src`.
        gc.builder().position_at_end(copy_bb);
        let src_copy_start = unsafe {
            gc.builder()
                .build_gep(elem_value_ty, src_buf, &[begin], "append_src_copy_start")
                .unwrap()
        };
        ObjectFieldType::clone_array_buf(gc, n, src_copy_start, dst_write, elem_ty, None);
        gc.release(src.clone());
        gc.builder().build_unconditional_branch(end_bb).unwrap();

        // Grow `dst`'s length by the number of appended elements.
        gc.builder().position_at_end(end_bb);
        let new_dst_len = gc
            .builder()
            .build_int_add(dst_len, n, "append_new_dst_len")
            .unwrap();
        dst.insert_field(gc, ARRAY_LEN_IDX, new_dst_len)
    }

    fn name(&self) -> String {
        format!(
            "array_append_range{}({}, {}, {}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.src_name.to_string(),
            self.begin_name.to_string(),
            self.end_name.to_string(),
            self.dst_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![
            &mut self.dst_name,
            &mut self.src_name,
            &mut self.begin_name,
            &mut self.end_name,
        ]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Appends `src[begin, end)` to the end of `dst`, moving the elements when `src` is uniquely owned
// and the whole of it is appended, and copying them (with a retain each) otherwise, with no capacity
// check. The caller must ensure `0 <= begin <= end <= src.size` and `dst.size + (end - begin) <=
// dst.capacity`; violating either causes undefined behavior.
// Type: Array a -> I64 -> I64 -> Array a -> Array a
pub fn array_append_capacity_bounds_unchecked() -> (Arc<ExprNode>, Arc<Scheme>) {
    const SRC_NAME: &str = "src";
    const BEGIN_NAME: &str = "begin";
    const END_NAME: &str = "end";
    const DST_NAME: &str = "dst";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs_many(
        vec![
            var_local(SRC_NAME),
            var_local(BEGIN_NAME),
            var_local(END_NAME),
            var_local(DST_NAME),
        ],
        expr_llvm(
            Box::new(InlineLLVMArrayAppendCapacityBoundsUnchecked {
                dst_name: FullName::local(DST_NAME),
                src_name: FullName::local(SRC_NAME),
                begin_name: FullName::local(BEGIN_NAME),
                end_name: FullName::local(END_NAME),
                force_unique: true,
            }),
            array_ty.clone(),
            None,
        ),
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            array_ty.clone(),
            type_fun(
                make_i64_ty(),
                type_fun(make_i64_ty(), type_fun(array_ty.clone(), array_ty)),
            ),
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayUnsafeSetSizeBody {
    arr_name: FullName,
    len_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayUnsafeSetSizeBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.arr_name);
        let length = gc.get_scoped_obj_field(&self.len_name, 0).into_int_value();

        array.insert_field(gc, ARRAY_LEN_IDX, length)
    }

    fn name(&self) -> String {
        format!(
            "array_set_size({}, {})",
            self.len_name.to_string(),
            self.arr_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.len_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // Writing the length in place carries the same promise from the caller as writing an element
        // does, so the array this returns is uniquely owned — see
        // `InlineLLVMArrayUnsafeSetBoundsUniquenessUncheckedUnreleased::result_prov`.
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Set the length of an array, with no uniqueness checking, no validation of size argument.
pub fn unsafe_set_size_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "array";
    const LENGTH_NAME: &str = "length";
    const ELEM_TYPE: &str = "a";

    let arr_name = FullName::local(ARR_NAME);
    let len_name = FullName::local(LENGTH_NAME);

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(LENGTH_NAME)],
        expr_abs(
            vec![var_local(ARR_NAME)],
            expr_llvm(
                Box::new(InlineLLVMArrayUnsafeSetSizeBody { arr_name, len_name }),
                array_ty.clone(),
                None,
            ),
            None,
        ),
        None,
    );

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(make_i64_ty(), type_fun(array_ty.clone(), array_ty)),
    );
    (expr, scm)
}

// Force array object to be unique.
// If it is unique, do nothing.
// If it is shared, clone the object.
fn make_array_unique<'c, 'm>(gc: &mut Generator<'c, 'm>, array: Object<'c>) -> Object<'c> {
    make_array_unique_with_hole(gc, array, None)
}

// Force array object to be unique, as `make_array_unique`. When `hole` is `Some(idx)`, a shared
// array is cloned skipping the element at `idx` (its slot in the clone is left uninitialized).
fn make_array_unique_with_hole<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    array: Object<'c>,
    hole: Option<IntValue<'c>>,
) -> Object<'c> {
    assert!(array.ty.is_array());

    let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
    let arr_ptr = array.value.into_pointer_value();
    let current_bb = gc.builder().get_insert_block().unwrap();
    let current_func = current_bb.get_parent().unwrap();

    // Branch by whether the array is unique or not.
    let (unique_bb, shared_bb) = gc.build_branch_by_is_unique(arr_ptr);
    let end_bb = gc.context.append_basic_block(current_func, "end_bb");

    // Implement shared_bb.
    // In this block, create new array and clone array field.
    gc.builder().position_at_end(shared_bb);

    // Allocate cloned array.
    let array_cap = array.extract_field(gc, ARRAY_CAP_IDX).into_int_value();
    let cloned_array = create_obj(
        array.ty.clone(),
        &vec![],
        Some(array_cap),
        gc,
        Some("cloned_array_for_uniqueness"),
    );
    // Set the length of the cloned array.
    let array_len = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
    let cloned_array = cloned_array.insert_field(gc, ARRAY_LEN_IDX, array_len);
    // Copy elements to the cloned array.
    let cloned_array_buf = cloned_array.gep_boxed(gc, ARRAY_BUF_IDX);
    let array_buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
    ObjectFieldType::clone_array_buf(gc, array_len, array_buf, cloned_array_buf, elem_ty, hole);
    gc.release(array.clone()); // Given array should be released here.

    // Jump to the end_bb.
    let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
    let cloned_array_ptr = cloned_array.value.into_pointer_value();
    gc.builder().build_unconditional_branch(end_bb).unwrap();

    // Implement unique_bb
    gc.builder().position_at_end(unique_bb);
    // Jump to end_bb.
    gc.builder().build_unconditional_branch(end_bb).unwrap();

    // Implement end_bb.
    gc.builder().position_at_end(end_bb);
    // Build phi value of array_ptr.
    let array_phi = gc
        .builder()
        .build_phi(arr_ptr.get_type(), "array_phi")
        .unwrap();
    array_phi.add_incoming(&[
        (&arr_ptr, unique_bb),
        (&cloned_array_ptr, succ_of_shared_bb),
    ]);
    Object::new(array_phi.as_basic_value(), array.ty.clone(), gc)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArraySetBody {
    array_name: FullName,
    idx_name: FullName,
    value_name: FullName,
    // When true, clone the array first if it is shared, so the write lands in a uniquely owned
    // array. Set false only where the array is statically known to be unique.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArraySetBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.array_name);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();
        let value = gc.get_scoped_obj(&self.value_name);

        // Force array to be unique
        let array = if self.force_unique {
            make_array_unique(gc, array)
        } else {
            array
        };

        // Perform write and return. Bounds-check unless `--no-runtime-check` is set.
        let len = if gc.config.runtime_check() {
            Some(array.extract_field(gc, ARRAY_LEN_IDX).into_int_value())
        } else {
            None
        };
        let array_buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        ObjectFieldType::write_to_array_buf(gc, len, array_buf, idx, value, true);
        array
    }

    fn name(&self) -> String {
        format!(
            "array_set{}({}, {}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.idx_name.to_string(),
            self.value_name.to_string(),
            self.array_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![
            &mut self.array_name,
            &mut self.idx_name,
            &mut self.value_name,
        ]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `Array::set` built-in function.
pub fn set_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let elem_ty = type_tyvar_star("a");
    let array_ty = type_tyapp(make_array_ty(), elem_ty.clone());
    let body = expr_llvm(
        Box::new(InlineLLVMArraySetBody {
            array_name: FullName::local("array"),
            idx_name: FullName::local("idx"),
            value_name: FullName::local("value"),
            force_unique: true,
        }),
        array_ty.clone(),
        None,
    );
    let expr = expr_abs(
        vec![var_local("idx")],
        expr_abs(
            vec![var_local("value")],
            expr_abs(vec![var_local("array")], body, None),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            make_i64_ty(),
            type_fun(elem_ty, type_fun(array_ty.clone(), array_ty)),
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArraySwapBody {
    array_name: FullName,
    i_name: FullName,
    j_name: FullName,
    // When true, clone the array first if it is shared, so the swap writes into a uniquely
    // owned array. Set false only where the array is statically known to be unique.
    pub(crate) force_unique: bool,
    // When true, panic if `i` or `j` is out of range.
    bounds_checked: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArraySwapBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get arguments.
        let array = gc.get_scoped_obj(&self.array_name);
        let i = gc.get_scoped_obj_field(&self.i_name, 0).into_int_value();
        let j = gc.get_scoped_obj_field(&self.j_name, 0).into_int_value();

        // Force array to be unique.
        let array = if self.force_unique {
            make_array_unique(gc, array)
        } else {
            array
        };

        let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
        // `swap` bounds-checks unless `--no-runtime-check` is set; `unsafe_swap_bounds_unchecked`
        // never bounds-checks.
        let len = if self.bounds_checked && gc.config.runtime_check() {
            Some(array.extract_field(gc, ARRAY_LEN_IDX).into_int_value())
        } else {
            None
        };
        let array_buf = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Read both elements without retaining, then store them back into each other's slot
        // without releasing: the two elements only change places, so their reference counts
        // are unchanged.
        let elem_i =
            ObjectFieldType::read_from_array_buf_noretain(gc, len, array_buf, elem_ty.clone(), i);
        let elem_j = ObjectFieldType::read_from_array_buf_noretain(gc, len, array_buf, elem_ty, j);
        ObjectFieldType::write_to_array_buf(gc, len, array_buf, i, elem_j, false);
        ObjectFieldType::write_to_array_buf(gc, len, array_buf, j, elem_i, false);
        array
    }

    fn name(&self) -> String {
        format!(
            "array_swap{}{}({}, {}, {})",
            if self.bounds_checked {
                ""
            } else {
                "_unchecked"
            },
            if self.force_unique { "" } else { "[unique]" },
            self.i_name.to_string(),
            self.j_name.to_string(),
            self.array_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.array_name, &mut self.i_name, &mut self.j_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn swap_array_common(bounds_checked: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    let body = expr_llvm(
        Box::new(InlineLLVMArraySwapBody {
            array_name: FullName::local("array"),
            i_name: FullName::local("i"),
            j_name: FullName::local("j"),
            force_unique: true,
            bounds_checked,
        }),
        type_tyapp(make_array_ty(), type_tyvar_star("a")),
        None,
    );
    let expr = expr_abs(
        vec![var_local("i")],
        expr_abs(
            vec![var_local("j")],
            expr_abs(vec![var_local("array")], body, None),
            None,
        ),
        None,
    );
    let array_ty = type_tyapp(make_array_ty(), type_tyvar_star("a"));
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            make_i64_ty(),
            type_fun(make_i64_ty(), type_fun(array_ty.clone(), array_ty)),
        ),
    );
    (expr, scm)
}

// `Array::swap` built-in function.
pub fn swap_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    swap_array_common(true)
}

// `Array::unsafe_swap_bounds_unchecked` built-in function.
pub fn swap_bounds_unchecked_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    swap_array_common(false)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayPunchBody {
    pub(crate) force_unique: bool,
    idx_name: FullName,
    arr_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayPunchBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // ret_ty = (PunchedArray a, a)
        let mut array = gc.get_scoped_obj(&self.arr_name);
        let idx_obj = gc.get_scoped_obj(&self.idx_name);
        let idx = idx_obj.extract_field(gc, 0).into_int_value();

        // The array has no hole yet, so this is an ordinary clone-if-shared.
        if self.force_unique {
            array = make_array_unique(gc, array);
        }

        // Move the element at `idx` out without retaining, leaving its slot as the hole; the
        // length is unchanged.
        let punched_ty = ret_ty.collect_type_argments().get(0).unwrap().clone();
        let elem_ty = ret_ty.collect_type_argments().get(1).unwrap().clone();
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        let elem = ObjectFieldType::read_from_array_buf_noretain(gc, None, buf, elem_ty, idx);

        // Build `(PunchedArray { _arr : array, _idx : idx }, elem)`.
        let punched = create_obj(punched_ty, &vec![], None, gc, Some("alloca@_punch"));
        let punched = ObjectFieldType::move_into_struct_field(gc, punched, 0, &array);
        let punched = ObjectFieldType::move_into_struct_field(gc, punched, 1, &idx_obj);
        let res = create_obj(ret_ty.clone(), &vec![], None, gc, Some("alloca@_punch_ret"));
        let res = ObjectFieldType::move_into_struct_field(gc, res, 0, &punched);
        let res = ObjectFieldType::move_into_struct_field(gc, res, 1, &elem);
        res
    }

    fn name(&self) -> String {
        format!(
            "array_punch{}({}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.idx_name.to_string(),
            self.arr_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // The result is `(PunchedArray a, a)`: the array with a hole, and the element moved out of it.
        // The punched array is uniquely owned either way — force-uniquing clones it when it is shared,
        // and the version without that check runs only where uniqueness is established already, by the
        // optimizer having proven it or by the caller of the unsafe primitive having promised it. That
        // is what lets the `plug` completing the update drop its own check. The element is moved out
        // without a retain, so another holder of it may still be live: its leaves stay `Unknown`, since
        // calling them `Fresh` would let a later in-place update overwrite shared data.
        Provenance::fresh_under(result_ty, type_env, &[PUNCHED_ARRAY_FIELD])
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The index of the punched array in the result of an array punch, `(PunchedArray a, a)`.
const PUNCHED_ARRAY_FIELD: usize = 0;

// Moves the element at `idx` out of an array (without bounds checking), leaving a hole, and
// returns the punched array together with the moved-out element.
// Type: I64 -> Array a -> (PunchedArray a, a)
pub fn array_punch(force_unique: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    const IDX_NAME: &str = "idx";
    const ARR_NAME: &str = "array";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());
    let punched_ty = type_tyapp(make_punched_array_ty(), elem_tyvar.clone());
    let res_ty = make_tuple_ty(vec![punched_ty, elem_tyvar.clone()]);

    let expr = expr_abs_many(
        vec![var_local(IDX_NAME), var_local(ARR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayPunchBody {
                force_unique,
                idx_name: FullName::local(IDX_NAME),
                arr_name: FullName::local(ARR_NAME),
            }),
            res_ty.clone(),
            None,
        ),
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(make_i64_ty(), type_fun(array_ty, res_ty)),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMPunchedArrayPlugBody {
    pub(crate) force_unique: bool,
    elem_name: FullName,
    punched_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMPunchedArrayPlugBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ret_ty: &Arc<TypeNode>) -> Object<'c> {
        let elem = gc.get_scoped_obj(&self.elem_name);
        let punched = gc.get_scoped_obj(&self.punched_name);

        // Deconstruct PunchedArray { _arr : array, _idx : idx }.
        let mut array = ObjectFieldType::move_out_struct_field(gc, &punched, 0);
        let idx_obj = ObjectFieldType::move_out_struct_field(gc, &punched, 1);
        let idx = idx_obj.extract_field(gc, 0).into_int_value();

        // On a shared array, clone skipping the hole so this plug gets a private array.
        if self.force_unique {
            array = make_array_unique_with_hole(gc, array, Some(idx));
        }

        // Write the element back into the hole (no bounds check, and no release of the hole slot).
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        ObjectFieldType::write_to_array_buf(gc, None, buf, idx, elem, false);
        array
    }

    fn name(&self) -> String {
        format!(
            "array_plug{}({}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.elem_name.to_string(),
            self.punched_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.elem_name, &mut self.punched_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(1, vec![PUNCHED_ARRAY_FIELD], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Writes an element back into a punched array's hole, returning the completed array.
// Type: a -> PunchedArray a -> Array a
pub fn punched_array_plug(force_unique: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    const ELEM_NAME: &str = "elem";
    const PUNCHED_NAME: &str = "punched";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());
    let punched_ty = type_tyapp(make_punched_array_ty(), elem_tyvar.clone());

    let expr = expr_abs_many(
        vec![var_local(ELEM_NAME), var_local(PUNCHED_NAME)],
        expr_llvm(
            Box::new(InlineLLVMPunchedArrayPlugBody {
                force_unique,
                elem_name: FullName::local(ELEM_NAME),
                punched_name: FullName::local(PUNCHED_NAME),
            }),
            array_ty.clone(),
            None,
        ),
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(elem_tyvar, type_fun(punched_ty, array_ty)),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayForceUniqueBody {
    arr_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayForceUniqueBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.arr_name);

        // Make array unique
        let array = make_array_unique(gc, array);

        array
    }

    fn name(&self) -> String {
        format!("array_force_unique({})", self.arr_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn force_unique_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayForceUniqueBody {
                arr_name: FullName::local(ARR_NAME),
            }),
            array_ty.clone(),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], type_fun(array_ty.clone(), array_ty));
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayCheckRange {
    idx_name: FullName,
    size_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayCheckRange {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        if gc.config.runtime_check() {
            let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();
            let size = gc.get_scoped_obj_field(&self.size_name, 0).into_int_value();
            ObjectFieldType::panic_if_out_of_range(gc, size, idx);
        }
        gc.get_scoped_obj(&self.idx_name)
    }

    fn name(&self) -> String {
        format!(
            "array_check_range({}, {})",
            self.idx_name.to_string(),
            self.size_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.idx_name, &mut self.size_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// _check_range : I64 -> I64 -> I64
pub fn array_check_range() -> (Arc<ExprNode>, Arc<Scheme>) {
    const IDX_NAME: &str = "idx";
    const SIZE_NAME: &str = "size";

    let expr = expr_abs_many(
        vec![var_local(IDX_NAME), var_local(SIZE_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayCheckRange {
                idx_name: FullName::local(IDX_NAME),
                size_name: FullName::local(SIZE_NAME),
            }),
            make_i64_ty(),
            None,
        ),
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(make_i64_ty(), type_fun(make_i64_ty(), make_i64_ty())),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayCheckSize {
    size_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayCheckSize {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        if gc.config.runtime_check() {
            let size = gc.get_scoped_obj_field(&self.size_name, 0).into_int_value();
            ObjectFieldType::panic_if_size_negative(gc, size);
        }
        gc.get_scoped_obj(&self.size_name)
    }

    fn name(&self) -> String {
        format!("array_check_size({})", self.size_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.size_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// _check_size : I64 -> I64
pub fn array_check_size() -> (Arc<ExprNode>, Arc<Scheme>) {
    const SIZE_NAME: &str = "size";

    let expr = expr_abs_many(
        vec![var_local(SIZE_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayCheckSize {
                size_name: FullName::local(SIZE_NAME),
            }),
            make_i64_ty(),
            None,
        ),
    );
    let scm = Scheme::generalize(&[], vec![], vec![], type_fun(make_i64_ty(), make_i64_ty()));
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayGetPtrBody {
    arr_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayGetPtrBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argment
        let array = gc.get_scoped_obj_noretain(&self.arr_name);

        // Get pointer
        let ptr = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Make returned object
        let obj = create_obj(
            make_ptr_ty(),
            &vec![],
            None,
            gc,
            Some("alloca@get_ptr_array"),
        );
        obj.insert_field(gc, 0, ptr)
    }

    fn name(&self) -> String {
        format!("array_data_ptr({})", self.arr_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `get_ptr` function for Array.
pub fn get_ptr_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayGetPtrBody {
                arr_name: FullName::local(ARR_NAME),
            }),
            make_ptr_ty(),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(array_ty.clone(), make_ptr_ty()),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayGetSizeBody {
    arr_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayGetSizeBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        let array_obj = gc.get_scoped_obj_noretain(&self.arr_name);
        let len = array_obj.extract_field(gc, ARRAY_LEN_IDX).into_int_value();

        let int_obj = create_obj(make_i64_ty(), &vec![], None, gc, Some("length_of_arr"));
        int_obj.insert_field(gc, 0, len)
    }

    fn name(&self) -> String {
        format!("array_size({})", self.arr_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `get_size` built-in function for Array.
pub fn array_get_size() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayGetSizeBody {
                arr_name: FullName::local(ARR_NAME),
            }),
            make_i64_ty(),
            None,
        ),
        None,
    );
    let array_ty = type_tyapp(make_array_ty(), type_tyvar_star("a"));
    let scm = Scheme::generalize(&[], vec![], vec![], type_fun(array_ty, make_i64_ty()));
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayGetCapacityBody {
    arr_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayGetCapacityBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        let array_obj = gc.get_scoped_obj_noretain(&self.arr_name);
        let len = array_obj.extract_field(gc, ARRAY_CAP_IDX).into_int_value();

        let int_obj = create_obj(make_i64_ty(), &vec![], None, gc, Some("cap_of_arr"));
        int_obj.insert_field(gc, 0, len)
    }

    fn name(&self) -> String {
        format!("array_capacity({})", self.arr_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `Array::get_capacity : Array a -> I64` built-in function.
pub fn array_get_capacity() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMArrayGetCapacityBody {
                arr_name: FullName::local(ARR_NAME),
            }),
            make_i64_ty(),
            None,
        ),
        None,
    );
    let array_ty = type_tyapp(make_array_ty(), type_tyvar_star("a"));
    let scm = Scheme::generalize(&[], vec![], vec![], type_fun(array_ty, make_i64_ty()));
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructGetBody {
    pub var_name: FullName,
    field_idx: usize,
}

impl InlineLLVMStructGetBody {
    /// The index of the field this operation reads.
    pub fn field_index(&self) -> usize {
        self.field_idx
    }

    /// Whether reading a field of type `field_ty` only borrows the container. A fully unboxed field
    /// holds no reference, so the value read out of it takes nothing from the container.
    ///
    /// A field that does hold one is read by taking ownership of the container instead: as a borrow
    /// the result would alias the container's leaf, and reference-count insertion releases a
    /// *variable* at its last use without following aliases, so that leaf would be released twice.
    fn borrows_container(field_ty: &Arc<TypeNode>, type_env: &TypeEnv) -> bool {
        field_ty.is_fully_unboxed(type_env)
    }
}

#[typetag::serde]
impl LLVMGen for InlineLLVMStructGetBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // The value of a field getter is the field, so `ty` is the field's type.
        if Self::borrows_container(ty, gc.type_env()) {
            let str = gc.get_scoped_obj_noretain(&self.var_name);
            return ObjectFieldType::move_out_struct_field(gc, &str, self.field_idx as u32);
        }
        let str = gc.get_scoped_obj(&self.var_name);
        ObjectFieldType::get_struct_fields(gc, &str, &[self.field_idx as u32])[0].clone()
    }

    fn name(&self) -> String {
        format!(
            "struct_get_{}({})",
            self.field_idx,
            self.var_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    fn borrows_operand(&self, i: usize, arg_tys: &[Arc<TypeNode>], type_env: &TypeEnv) -> bool {
        // A field getter takes exactly the container, so `arg_tys[0]` is it.
        i == 0
            && Self::borrows_container(&arg_tys[0].field_types(type_env)[self.field_idx], type_env)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // From a boxed container the field is `Unknown` (contents not tracked); from an unboxed
        // container it is a pure projection carrying the container's leaf at that field. A field
        // getter takes exactly the container, so `arg_tys[0]` is it.
        let container_boxed = arg_tys[0].is_box(type_env);
        if container_boxed {
            Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)
        } else {
            let field = self.field_index();
            Provenance::build_shape(result_ty, type_env, &|sigma: &FieldPath| {
                let mut p = vec![field];
                p.extend_from_slice(sigma);
                Provenance::leaf(LeafOrigin::Arg(0, p))
            })
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `get` built-in function for a given struct.
pub fn struct_get_body(var_name: &str, field_idx: usize, field_ty: Arc<TypeNode>) -> Arc<ExprNode> {
    let var_name_clone = FullName::local(var_name);
    expr_llvm(
        Box::new(InlineLLVMStructGetBody {
            var_name: var_name_clone,
            field_idx,
        }),
        field_ty,
        None,
    )
}

// field getter function for a given struct.
pub fn struct_get(definition: &TypeDefn, field_name: &str) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let (field_idx, field) = definition.get_field_by_name(field_name).unwrap();

    let str_ty = definition.applied_type();
    const VAR_NAME: &str = "str_obj";
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        struct_get_body(VAR_NAME, field_idx as usize, field.ty.clone()),
        None,
    );
    let ty = type_fun(str_ty, field.ty.clone());
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// Allocate a struct/tuple and fill it with the operand values, in field-declaration order. The
// struct type is the value type of the enclosing expression. This is the RC IR counterpart of the
// `Expr::MakeStruct` AST node, reading its operands as pre-evaluated atoms.
#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMMakeStructBody {
    pub field_names: Vec<FullName>,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMMakeStructBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        let mut str_obj = create_obj(ty.clone(), &vec![], None, gc, Some("allocate_MakeStruct"));
        let offset = if ty.is_box(gc.type_env()) { 1 } else { 0 };
        for (i, name) in self.field_names.iter().enumerate() {
            let field_obj = gc.get_scoped_obj_noretain(name);
            str_obj = str_obj.insert_field(gc, i as u32 + offset, field_obj.value);
        }
        str_obj
    }

    fn name(&self) -> String {
        format!(
            "struct_make({})",
            self.field_names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        self.field_names.iter_mut().collect()
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // A boxed struct is a fresh allocation (its single boxed leaf is at the root path). An
        // unboxed struct lays out its fields, so field `i`'s boxed leaves carry constructor operand
        // `i` (the path's head is the field index, its tail the position within that field).
        Provenance::build_shape(result_ty, type_env, &|path| match path.split_first() {
            None => Provenance::leaf(LeafOrigin::Fresh),
            Some((i, rest)) => Provenance::leaf(LeafOrigin::Arg(*i, rest.to_vec())),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Allocate an array whose length equals the number of operands and fill it with them. The array
// type is the value type of the enclosing expression. This is the RC IR counterpart of the
// `Expr::ArrayLit` AST node, reading its operands as pre-evaluated atoms.
#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayLitBody {
    pub elem_names: Vec<FullName>,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMArrayLitBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        let len = gc
            .context
            .i64_type()
            .const_int(self.elem_names.len() as u64, false);
        let array = create_obj(ty.clone(), &vec![], Some(len), gc, Some("array_literal"));
        let buffer = array.gep_boxed(gc, ARRAY_BUF_IDX);
        let array = array.insert_field(gc, ARRAY_LEN_IDX, len);
        for (i, name) in self.elem_names.iter().enumerate() {
            let value = gc.get_scoped_obj_noretain(name);
            let idx = gc.context.i64_type().const_int(i as u64, false);
            ObjectFieldType::write_to_array_buf(gc, None, buffer, idx, value, false);
        }
        array
    }

    fn name(&self) -> String {
        format!(
            "array_lit({})",
            self.elem_names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        self.elem_names.iter_mut().collect()
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Call a C function. This is the RC IR counterpart of the `Expr::FFICall` AST node. `arg_names` are
// the operands; when `is_io`, the last one is the input `IOState` token, which establishes the
// ordering dependency but is not passed to C.
#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFFICallBody {
    pub fun_name: Name,
    pub ret_tycon: Arc<TyCon>,
    pub param_tycons: Vec<Arc<TyCon>>,
    pub is_var_args: bool,
    pub is_io: bool,
    pub arg_names: Vec<FullName>,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFFICallBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // The return object's type is the value type of this expression: `(IOState, ret)` when
        // `is_io`, else `ret`.
        let obj = create_obj(ty.clone(), &vec![], None, gc, Some("allocate_CallC"));
        // The C arguments are all operands except the trailing IOState token (when `is_io`).
        let c_arg_count = if self.is_io {
            self.arg_names.len() - 1
        } else {
            self.arg_names.len()
        };
        let arg_objs = self.arg_names[..c_arg_count]
            .iter()
            .map(|name| gc.get_scoped_obj_noretain(name))
            .collect::<Vec<_>>();
        gc.build_ffi_call_core(
            &None,
            obj,
            &self.fun_name,
            &self.ret_tycon,
            &self.param_tycons,
            self.is_var_args,
            arg_objs,
            self.is_io,
        )
    }

    fn name(&self) -> String {
        format!(
            "ffi_call{}[{}]({})",
            if self.is_io { "_ios" } else { "" },
            self.fun_name,
            self.arg_names
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        self.arg_names.iter_mut().collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Project a captured value out of a lifted closure's capture object, retaining it (a retain-getter).
// Lowering emits this at the entry of a lifted closure function to bind each captured variable.
// `cap_tys` are the types of all captured values, needed to reconstruct the capture object's layout.
#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCaptureProjectBody {
    pub cap_name: FullName,
    pub cap_idx: usize,
    pub cap_tys: Vec<Arc<TypeNode>>,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMCaptureProjectBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        gc.build_capture_project(&self.cap_name, self.cap_idx, &self.cap_tys, ty)
    }

    fn name(&self) -> String {
        format!(
            "capture_project_{}({})",
            self.cap_idx,
            self.cap_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.cap_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructPunchBody {
    pub var_name: FullName,
    field_idx: usize,
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMStructPunchBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get the argument object (the struct value).
        let mut str = gc.get_scoped_obj(&self.var_name);

        if self.force_unique {
            // If the struct is shared, we should clone it to make it unique.
            str = make_struct_unique(gc, str);
        }

        // Move out struct field value without releasing the struct itself.
        let field = ObjectFieldType::move_out_struct_field(gc, &str, self.field_idx as u32);

        // Create the return value.
        let pair = create_obj(ret_ty.clone(), &vec![], None, gc, Some("ret_of_punch"));
        let pair = pair.insert_field(gc, 0, field.value);
        let pair = pair.insert_field(gc, 1, str.value);

        pair
    }

    fn name(&self) -> String {
        format!(
            "struct_punch_{}{}({})",
            self.field_idx,
            if self.force_unique { "" } else { "[unique]" },
            self.var_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(0, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // The result is `(field, punched struct)`.
        //
        // A punched boxed struct is uniquely owned either way, for the reason
        // `InlineLLVMArrayPunchBody::result_prov` gives, and that is what lets the `plug_in` completing
        // the update drop its own check. The field is moved out without a retain, so another holder of
        // it may still be live and its leaves stay `Unknown`.
        //
        // Punching an unboxed struct only takes it apart in registers: the field and the remaining
        // fields carry the argument's, and nothing is retained or released. Declaring those
        // passthroughs is what carries a boxed field — an array in a loop state, say — through
        // `mod`/`act` with what is known about it intact. The punched-out field is left holding a
        // value the struct no longer owns; it names nothing, so `Unknown` says the least about it.
        let punched_ty = &result_ty.field_types(type_env)[PUNCHED_STRUCT_FIELD];
        if punched_ty.is_box(type_env) {
            return Provenance::fresh_under(result_ty, type_env, &[PUNCHED_STRUCT_FIELD]);
        }
        Provenance::build_shape(result_ty, type_env, &|path| {
            // A boxed leaf of the result descends through the field or through the punched struct.
            let (head, rest) = path
                .split_first()
                .expect("a boxed leaf of an unboxed pair has a non-empty path");
            if *head != PUNCHED_STRUCT_FIELD {
                let mut p = vec![self.field_idx];
                p.extend_from_slice(rest);
                return Provenance::leaf(LeafOrigin::Arg(0, p));
            }
            // The punched struct is unboxed here, so a boxed leaf of it also starts with a field
            // index.
            let (field, _) = rest
                .split_first()
                .expect("a boxed leaf of an unboxed punched struct has a non-empty path");
            if *field == self.field_idx {
                Provenance::leaf(LeafOrigin::Unknown)
            } else {
                Provenance::leaf(LeafOrigin::Arg(0, rest.to_vec()))
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The index of the punched struct in the result of a struct punch, `(field, punched struct)`.
const PUNCHED_STRUCT_FIELD: usize = 1;

// Field punching function for a given struct.
//
// If the struct is `S` and the field is `x` of type `F`, then the function has the type `S -> (F, Sx)` where `Sx` is the punched `S` at the field `x`.
// i.e., `Sx` has the same memory layout as `S`, but does not contain the field `x`.
//
// We are not sure whether we should clone the struct when a shared struct is given to `punch_x`.
// If we do not clone the struct, it will lead to different types of values sharing the same memory area, which feels dangerous,
// but we have not found an example that causes a memory management issue yet.
//
// There are two use cases for the current `punch_x` function.
// One is the implementation of `act_x`, where the struct given to `punch_x` is guaranteed to be unique.
// The other is the implementation of `mod_x`, where it is acceptable to clone the struct if it is shared.
// So we have two versions of `punch_x`: one that does not clone the struct and one that does.
// The above problem is unresolved, but the current use cases do not require solving this problem.
pub fn struct_punch(
    definition: &TypeDefn,
    field_name: &str,
    force_unique: bool,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let (field_idx, field) = definition.get_field_by_name(field_name).unwrap();

    let str_ty = definition.applied_type();
    let punched_ty = str_ty.to_punched_struct(field_idx as usize);
    let dst_ty = make_tuple_ty(vec![field.ty.clone(), punched_ty]);
    let ty = type_fun(str_ty, dst_ty.clone());
    let scm = Scheme::generalize(&[], vec![], vec![], ty);

    const VAR_NAME: &str = "struct_value";
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMStructPunchBody {
                var_name: FullName::local(VAR_NAME),
                field_idx: field_idx as usize,
                force_unique,
            }),
            dst_ty,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructPlugInBody {
    punched_str_name: FullName,
    pub field_name: FullName,
    field_idx: usize,
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMStructPlugInBody {
    fn generate<'c, 'm>(
        &self,
        gc: &mut Generator<'c, 'm>,
        struct_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get the first argument, a punched struct value, and the second argument, a field value.
        let mut punched_str = gc.get_scoped_obj(&self.punched_str_name);
        let field = gc.get_scoped_obj(&self.field_name);

        // Make the punched struct unique before plugging-in the field value.
        if self.force_unique {
            punched_str = make_struct_unique(gc, punched_str);
        }

        // Convert type of punched_str into the struct type.
        let punched_value = punched_str.value;
        let str = Object::new(punched_value, struct_ty.clone(), gc);

        // Move the field value into the struct value.
        let str = ObjectFieldType::move_into_struct_field(gc, str, self.field_idx as u32, &field);

        str
    }

    fn name(&self) -> String {
        format!(
            "struct_plug_in_{}{}({}, {})",
            self.field_idx,
            if self.force_unique { "" } else { "[unique]" },
            self.field_name.to_string(),
            self.punched_str_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.punched_str_name, &mut self.field_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(PLUG_IN_PUNCHED_ARG, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        replaced_field_prov(
            result_ty,
            type_env,
            self.field_idx,
            PLUG_IN_PUNCHED_ARG,
            PLUG_IN_FIELD_ARG,
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The operand positions of a struct `plug_in`: the punched struct, then the field value.
const PLUG_IN_PUNCHED_ARG: usize = 0;
const PLUG_IN_FIELD_ARG: usize = 1;

/// The provenance of a struct rebuilt with the field at `field_idx` replaced, given the operand
/// positions of the struct and of the new field value.
///
/// A boxed struct comes back uniquely owned: the op clones it when shared, and the version without
/// that check runs only where uniqueness is established already, by the optimizer having proven it or
/// by the caller of the unsafe primitive having promised it. The result's uniqueness therefore does
/// not depend on the input, like an array set, which is what lets a chain of field updates drop every
/// check after the first.
///
/// An unboxed struct is repackaged in registers: the replaced field carries the value operand's leaf
/// and every other field the struct operand's, with nothing retained or released. Declaring those
/// passthroughs is what carries a boxed field — an array in a loop state, say — through `mod`/`act`
/// with what is known about it intact. The struct operand's leaf at the replaced field reaches no
/// result path and so stays consumed: `set` releases the value it replaces, and `plug_in` fills a
/// hole holding a value the struct no longer owns.
fn replaced_field_prov(
    result_ty: &Arc<TypeNode>,
    type_env: &TypeEnv,
    field_idx: usize,
    struct_arg: usize,
    value_arg: usize,
) -> Provenance {
    if result_ty.is_box(type_env) {
        return Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh);
    }
    Provenance::build_shape(result_ty, type_env, &|path| {
        // A boxed leaf of an unboxed struct starts with a field index.
        let (field, rest) = path
            .split_first()
            .expect("a boxed leaf of an unboxed struct has a non-empty path");
        if *field == field_idx {
            Provenance::leaf(LeafOrigin::Arg(value_arg, rest.to_vec()))
        } else {
            Provenance::leaf(LeafOrigin::Arg(struct_arg, path.clone()))
        }
    })
}

// Field plugging-in function for a given struct.
// If the struct is `S` and the field is `F`, then the function has the type `Sx -> F -> S` where `Sx` is the punched struct type.
//
// In principle, `plug_in_x : Sx -> F -> S` should clone the punched struct if it is shared,
// because the memory region will be modified by plugging-in the field value.
pub fn struct_plug_in(
    definition: &TypeDefn,
    field_name: &str,
    force_unique: bool,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let (field_idx, field) = definition.get_field_by_name(field_name).unwrap();

    let str_ty = definition.applied_type();
    let punched_ty = str_ty.to_punched_struct(field_idx as usize);
    let ty = type_fun(punched_ty, type_fun(field.ty.clone(), str_ty.clone()));
    let scm = Scheme::generalize(&[], vec![], vec![], ty);

    const PUNCHED_STR_NAME: &str = "punched_str_obj";
    const FIELD_NAME: &str = "field_obj";
    let expr = expr_abs(
        vec![var_local(PUNCHED_STR_NAME)],
        expr_abs(
            vec![var_local(FIELD_NAME)],
            expr_llvm(
                Box::new(InlineLLVMStructPlugInBody {
                    punched_str_name: FullName::local(PUNCHED_STR_NAME),
                    field_name: FullName::local(FIELD_NAME),
                    field_idx: field_idx as usize,
                    force_unique,
                }),
                str_ty.clone(),
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

// `mod` built-in function for a given struct.
pub fn struct_mod(definition: &TypeDefn, field_name: &str) -> (Arc<ExprNode>, Arc<Scheme>) {
    let (_, field) = definition.get_field_by_name(field_name).unwrap();
    let str_ty = definition.applied_type();
    let ty = type_fun(
        type_fun(field.ty.clone(), field.ty.clone()),
        type_fun(str_ty.clone(), str_ty.clone()),
    );
    let struct_name = &definition.name;

    // The implementation of `mod` function as AST.
    //
    // ```
    // |f, x| (
    //     let (x, p) = x.#punch_fu_{field}; // here, force uniqueness.
    //     #plug_in_{field}(p, f(x)) // uniqueness is guaranteed here.
    // )
    // ```

    // `#punch_fu_{field}`
    let punch_func = expr_var(
        FullName::new(
            &struct_name.to_namespace(),
            &format!("{}{}", STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL, field_name),
        ),
        None,
    );
    // `x.#punch_fu_{field}`
    let punch_expr = expr_app(punch_func, vec![expr_var(FullName::local("x"), None)], None);

    // `#plug_in_{field}`
    let plug_in_func = expr_var(
        FullName::new(
            &struct_name.to_namespace(),
            &format!("{}{}", STRUCT_PLUG_IN_SYMBOL, field_name),
        ),
        None,
    );
    // `f(x)`
    let fx = expr_app(
        expr_var(FullName::local("f"), None),
        vec![expr_var(FullName::local("x"), None)],
        None,
    );
    // `#plug_in_{field}(p, f(x))`
    let plug_in_expr = expr_app(
        expr_app(
            plug_in_func,
            vec![expr_var(FullName::local("p"), None)],
            None,
        ),
        vec![fx],
        None,
    );

    // let (x, p) = x.#punch_fu_{field};
    // #plug_in_{field}(p, f(x))
    let let_expr = expr_let(
        PatternNode::make_struct(
            tycon(make_tuple_name_abs(2)),
            vec![
                ("0".to_string(), PatternNode::make_var(var_local("x"), None)),
                ("1".to_string(), PatternNode::make_var(var_local("p"), None)),
            ],
        ),
        punch_expr,
        plug_in_expr,
        None,
    );

    // The whole expression is `|f, x| let (x, p) = x.#punch_fu_{field}; #plug_in_{field}(f(x), p)`
    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(vec![var_local("x")], let_expr, None),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// Field act function for a given struct.
// If the struct is `S` and the field is `F`, then the function has the type `[f : Functor] (F -> f F) -> S -> f S`.
// The implementation uses `#punch_{field}` and `#plug_in_{field}` for the struct.
pub fn struct_act(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index and the `Field` instance of `field_name` in the given struct.
    let (_field_idx, field) = definition.get_field_by_name(field_name).unwrap();

    // Create type scheme of this function.
    let str_ty = definition.applied_type();
    let field_ty = field.ty.clone();
    // To determine the name of the type variable for Functor, avoid names used in `str_ty` and `field_ty`.
    let mut used_tyvar_namess = Set::default();
    str_ty.collect_tyvar_names(&mut used_tyvar_namess);
    field_ty.collect_tyvar_names(&mut used_tyvar_namess);
    let used_tyvar_names: Set<FullName> = used_tyvar_namess
        .into_iter()
        .map(|name| FullName::local(&name))
        .collect();
    let new_name = generate_new_names(&used_tyvar_names, 1)[0].name.clone();
    let functor_ty = type_tyvar(&new_name, &kind_arrow(kind_star(), kind_star()));
    let src_ty = type_fun(
        field_ty.clone(),
        type_tyapp(functor_ty.clone(), field_ty.clone()),
    );
    let dst_ty = type_fun(
        str_ty.clone(),
        type_tyapp(functor_ty.clone(), str_ty.clone()),
    );
    let ty = type_fun(src_ty, dst_ty.clone());
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(
            TraitId::from_fullname(make_functor_name()),
            functor_ty,
        )],
        vec![],
        ty,
    );

    // Implementation of `act` function as AST.
    // The implementation as Fix source code is:
    // ```
    // |f, s| (
    //     let (unique, s) = s.unsafe_is_unique;
    //     if unique {
    //         let (x, ps) = s.#punch_{field}; // here, uniqueness is guaranteed.
    //         f(x).map(ps.#plug_in_fu_{field}) // here use "_fu" version: see the comment below.
    //     } else {
    //         f(s.@{field}).map(|e| s.set_{field}(e))
    //     }
    // );
    // ```
    // (Here, we cannot use the parser because we are using "#" is not allowed as value name)
    //
    // We should use `#plug_in_fu_{field}` here, not `#plug_in_{field}`:
    // `map` can call `#plug_in_fu_{field}(ps)` multiple times, so the argument to `#plug_in_fu_{field}` can be shared.
    let expr_unique = expr_let(
        PatternNode::make_struct(
            tycon(make_tuple_name_abs(2)),
            vec![
                ("0".to_string(), PatternNode::make_var(var_local("x"), None)),
                (
                    "1".to_string(),
                    PatternNode::make_var(var_local("ps"), None),
                ),
            ],
        ),
        expr_app(
            expr_var(
                FullName::new(
                    &struct_name.to_namespace(),
                    &format!("{}{}", STRUCT_PUNCH_SYMBOL, field_name),
                ),
                None,
            ),
            vec![expr_var(FullName::local("s"), None)],
            None,
        )
        .set_app_order(AppSourceCodeOrderType::XDotF),
        expr_app(
            expr_app(
                expr_var(
                    FullName::new(&make_functor_name().to_namespace(), "map"),
                    None,
                ),
                vec![expr_app(
                    expr_var(
                        FullName::new(
                            &struct_name.to_namespace(),
                            &format!("{}{}", STRUCT_PLUG_IN_FORCE_UNIQUE_SYMBOL, field_name),
                        ),
                        None,
                    ),
                    vec![expr_var(FullName::local("ps"), None)],
                    None,
                )
                .set_app_order(AppSourceCodeOrderType::XDotF)],
                None,
            )
            .set_app_order(AppSourceCodeOrderType::FX),
            vec![expr_app(
                expr_var(FullName::local("f"), None),
                vec![expr_var(FullName::local("x"), None)],
                None,
            )
            .set_app_order(AppSourceCodeOrderType::FX)],
            None,
        )
        .set_app_order(AppSourceCodeOrderType::XDotF),
        None,
    );

    let expr_shared = expr_app(
        expr_app(
            expr_var(
                FullName::new(&make_functor_name().to_namespace(), "map"),
                None,
            ),
            vec![expr_abs(
                vec![var_local("e")],
                expr_app(
                    expr_app(
                        expr_var(
                            FullName::new(
                                &struct_name.to_namespace(),
                                &format!("{}{}", STRUCT_SETTER_SYMBOL, field_name),
                            ),
                            None,
                        ),
                        vec![expr_var(FullName::local("e"), None)],
                        None,
                    )
                    .set_app_order(AppSourceCodeOrderType::FX),
                    vec![expr_var(FullName::local("s"), None)],
                    None,
                )
                .set_app_order(AppSourceCodeOrderType::XDotF),
                None,
            )],
            None,
        )
        .set_app_order(AppSourceCodeOrderType::FX),
        vec![expr_app(
            expr_var(FullName::local("f"), None),
            vec![expr_app(
                expr_var(
                    FullName::new(
                        &struct_name.to_namespace(),
                        &format!("{}{}", STRUCT_GETTER_SYMBOL, field_name),
                    ),
                    None,
                ),
                vec![expr_var(FullName::local("s"), None)],
                None,
            )
            .set_app_order(AppSourceCodeOrderType::XDotF)],
            None,
        )
        .set_app_order(AppSourceCodeOrderType::FX)],
        None,
    )
    .set_app_order(AppSourceCodeOrderType::XDotF);

    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(
            vec![var_local("s")],
            expr_let(
                PatternNode::make_struct(
                    tycon(make_tuple_name_abs(2)),
                    vec![
                        (
                            "0".to_string(),
                            PatternNode::make_var(var_local("unique"), None),
                        ),
                        ("1".to_string(), PatternNode::make_var(var_local("s"), None)),
                    ],
                ),
                expr_app(
                    expr_var(FullName::from_strs(&[STD_NAME], "unsafe_is_unique"), None),
                    vec![expr_var(FullName::local("s"), None)],
                    None,
                )
                .set_app_order(AppSourceCodeOrderType::XDotF),
                expr_if(
                    expr_var(FullName::local("unique"), None),
                    expr_unique,
                    expr_shared,
                    None,
                ),
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

// Field act function for a given struct, specialized for `Tuple2 X` functor for a given type `U`.
//
// If the struct is `S` and the field is `F`, then the function has the type `(F -> (U, F)) -> S -> (U, S)`.
//
// The implementation is optimized for the `Tuple2 U` functor:
// ```
// |f, x| (
//     let (x, p) = x.#punch_fu_{field}; // here, force uniqueness of x.
//     let (u, x) = f(x); // unwrap Tuple2
//     let p = #plug_in_{field}(p, x); // uniqueness of p is guaranteed here, so we do not use "_fu" version.
//     (u, p)
// )
// ```
pub fn struct_act_tuple2(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    let (_, field) = definition.get_field_by_name(field_name).unwrap();
    let str_ty = definition.applied_type();
    let field_ty = field.ty.clone();

    // Create type scheme: (F -> (U, F)) -> S -> (U, S)
    // We need a type variable for U
    let mut used_tyvar_names = Set::default();
    str_ty.collect_tyvar_names(&mut used_tyvar_names);
    field_ty.collect_tyvar_names(&mut used_tyvar_names);
    let used_tyvar_names: Set<FullName> = used_tyvar_names
        .into_iter()
        .map(|name| FullName::local(&name))
        .collect();
    let u_name = generate_new_names(&used_tyvar_names, 1)[0].name.clone();
    let u_ty = type_tyvar(&u_name, &kind_star());

    let tuple2_tycon = tycon(make_tuple_name_abs(2));
    // (U, F)
    let tuple2_u_f_ty = type_tyapp(
        type_tyapp(type_tycon(&tuple2_tycon), u_ty.clone()),
        field_ty.clone(),
    );
    // (U, S)
    let tuple2_u_s_ty = type_tyapp(
        type_tyapp(type_tycon(&tuple2_tycon), u_ty.clone()),
        str_ty.clone(),
    );

    let ty = type_fun(
        type_fun(field_ty.clone(), tuple2_u_f_ty.clone()),
        type_fun(str_ty.clone(), tuple2_u_s_ty.clone()),
    );

    // The implementation as AST:
    // |f, x| (
    //     let (x, p) = x.#punch_fu_{field};
    //     let (u, x) = f(x);
    //     let p = #plug_in_{field}(p, x);
    //     (u, p)
    // )

    // `#punch_fu_{field}`
    let punch_func = expr_var(
        FullName::new(
            &struct_name.to_namespace(),
            &format!("{}{}", STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL, field_name),
        ),
        None,
    );
    // `x.#punch_fu_{field}`
    let punch_expr = expr_app(punch_func, vec![expr_var(FullName::local("x"), None)], None);

    // `f(x)`
    let fx = expr_app(
        expr_var(FullName::local("f"), None),
        vec![expr_var(FullName::local("x"), None)],
        None,
    );

    // `#plug_in_{field}`
    let plug_in_func = expr_var(
        FullName::new(
            &struct_name.to_namespace(),
            &format!("{}{}", STRUCT_PLUG_IN_SYMBOL, field_name),
        ),
        None,
    );
    // `#plug_in_{field}(p, x)`
    let plug_in_expr = expr_app(
        expr_app(
            plug_in_func,
            vec![expr_var(FullName::local("p"), None)],
            None,
        ),
        vec![expr_var(FullName::local("x"), None)],
        None,
    );

    // `(u, p)`
    let wrap_tuple2 = expr_make_struct(
        tuple2_tycon.clone(),
        vec![
            ("0".to_string(), expr_var(FullName::local("u"), None)),
            ("1".to_string(), plug_in_expr),
        ],
    );

    // let (u, x) = f(x);
    // (u, #plug_in_{field}(p, x))
    let unwrap_tuple2 = expr_let(
        PatternNode::make_struct(
            tuple2_tycon.clone(),
            vec![
                ("0".to_string(), PatternNode::make_var(var_local("u"), None)),
                ("1".to_string(), PatternNode::make_var(var_local("x"), None)),
            ],
        ),
        fx,
        wrap_tuple2,
        None,
    );

    // let (x, p) = x.#punch_fu_{field};
    // let (u, x) = f(x);
    // let p = #plug_in_{field}(p, x);
    // (u, p)
    let let_expr = expr_let(
        PatternNode::make_struct(
            tuple2_tycon,
            vec![
                ("0".to_string(), PatternNode::make_var(var_local("x"), None)),
                ("1".to_string(), PatternNode::make_var(var_local("p"), None)),
            ],
        ),
        punch_expr,
        unwrap_tuple2,
        None,
    );

    // |f, x| ...
    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(vec![var_local("x")], let_expr, None),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// Field act function for a given struct, specialized for identity functor.
//
// If the struct is `S` and the field is `F`, then the function has the type `(F -> Std::Identity F) -> S -> Std::Identity S`.
//
// The implementation is optimized for the identity functor:
// ```
// |f, x| (
//     let (x, p) = x.#punch_fu_{field}; // here, force uniqueness of x.
//     let Identity { data : x } = f(x); // unwrap Identity
//     let p = #plug_in_{field}(p, x); // uniqueness of p is guaranteed here, so we do not use "_fu" version.
//     Identity { data : p }
// )
// ```
pub fn struct_act_identity(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    let (_, field) = definition.get_field_by_name(field_name).unwrap();
    let str_ty = definition.applied_type();
    let field_ty = field.ty.clone();

    // Type: (F -> Identity F) -> S -> Identity S
    let identity_tycon = tycon(FullName::from_strs(&[STD_NAME], IDENTITY_NAME));
    let identity_f_ty = type_tyapp(type_tycon(&identity_tycon), field_ty.clone());
    let identity_s_ty = type_tyapp(type_tycon(&identity_tycon), str_ty.clone());
    let ty = type_fun(
        type_fun(field_ty.clone(), identity_f_ty.clone()),
        type_fun(str_ty.clone(), identity_s_ty.clone()),
    );

    // The implementation as AST:
    // |f, x| (
    //     let (x, p) = x.#punch_fu_{field};
    //     let Identity { data : x } = f(x);
    //     let p = #plug_in_{field}(p, x);
    //     Identity { data : p }
    // )

    // `#punch_fu_{field}`
    let punch_func = expr_var(
        FullName::new(
            &struct_name.to_namespace(),
            &format!("{}{}", STRUCT_PUNCH_FORCE_UNIQUE_SYMBOL, field_name),
        ),
        None,
    );
    // `x.#punch_fu_{field}`
    let punch_expr = expr_app(punch_func, vec![expr_var(FullName::local("x"), None)], None);

    // `f(x)`
    let fx = expr_app(
        expr_var(FullName::local("f"), None),
        vec![expr_var(FullName::local("x"), None)],
        None,
    );

    // `#plug_in_{field}`
    let plug_in_func = expr_var(
        FullName::new(
            &struct_name.to_namespace(),
            &format!("{}{}", STRUCT_PLUG_IN_SYMBOL, field_name),
        ),
        None,
    );
    // `#plug_in_{field}(p, x)`
    let plug_in_expr = expr_app(
        expr_app(
            plug_in_func,
            vec![expr_var(FullName::local("p"), None)],
            None,
        ),
        vec![expr_var(FullName::local("x"), None)],
        None,
    );

    // `Identity { data : p }`
    let wrap_identity = expr_make_struct(
        identity_tycon.clone(),
        vec![("data".to_string(), plug_in_expr)],
    );

    // let Identity { data : x } = f(x);
    // Identity { data : #plug_in_{field}(p, x) }
    let unwrap_identity = expr_let(
        PatternNode::make_struct(
            identity_tycon,
            vec![(
                "data".to_string(),
                PatternNode::make_var(var_local("x"), None),
            )],
        ),
        fx,
        wrap_identity,
        None,
    );

    // let (x, p) = x.#punch_fu_{field};
    // let Identity { data : x } = f(x);
    // let p = #plug_in_{field}(p, x);
    // Identity { data : p }
    let let_expr = expr_let(
        PatternNode::make_struct(
            tycon(make_tuple_name_abs(2)),
            vec![
                ("0".to_string(), PatternNode::make_var(var_local("x"), None)),
                ("1".to_string(), PatternNode::make_var(var_local("p"), None)),
            ],
        ),
        punch_expr,
        unwrap_identity,
        None,
    );

    // |f, x| ...
    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(vec![var_local("x")], let_expr, None),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// Field act function for a given struct, specialized for constant functor.
//
// If the struct is `S` and the field is `F`, then the function has the type `(F -> Std::Const r F) -> S -> Std::Const r S`.
//
// The implementation is optimized for the constant functor:
// ```
// |f, s| (
//     let x = s.@{field}; // get the field value
//     let Const { data : r } = f(x); // unwrap `Const r F`
//     Const { data : r } // wrap back into `Const r S`
// )
// ```
pub fn struct_act_const(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    let (_, field) = definition.get_field_by_name(field_name).unwrap();
    let str_ty = definition.applied_type();
    let field_ty = field.ty.clone();

    // Create type scheme: (F -> Const r F) -> S -> Const r S
    // We need a type variable for r
    let mut used_tyvar_names = Set::default();
    str_ty.collect_tyvar_names(&mut used_tyvar_names);
    field_ty.collect_tyvar_names(&mut used_tyvar_names);
    let used_tyvar_names: Set<FullName> = used_tyvar_names
        .into_iter()
        .map(|name| FullName::local(&name))
        .collect();
    let r_name = generate_new_names(&used_tyvar_names, 1)[0].name.clone();
    let r_ty = type_tyvar(&r_name, &kind_star());

    let const_tycon = tycon(FullName::from_strs(&[STD_NAME], CONST_NAME));
    // Const r
    let const_r_ty = type_tyapp(type_tycon(&const_tycon), r_ty.clone());
    // Const r F
    let const_r_f_ty = type_tyapp(const_r_ty.clone(), field_ty.clone());
    // Const r S
    let const_r_s_ty = type_tyapp(const_r_ty, str_ty.clone());

    let ty = type_fun(
        type_fun(field_ty.clone(), const_r_f_ty.clone()),
        type_fun(str_ty.clone(), const_r_s_ty.clone()),
    );

    // The implementation as AST:
    // |f, s| (
    //     let x = s.@{field};
    //     let Const { data : r } = f(x);
    //     Const { data : r }
    // )

    // `@{field}`
    let getter_func = expr_var(
        FullName::new(
            &struct_name.to_namespace(),
            &format!("{}{}", STRUCT_GETTER_SYMBOL, field_name),
        ),
        None,
    );
    // `s.@{field}`
    let getter_expr = expr_app(
        getter_func,
        vec![expr_var(FullName::local("s"), None)],
        None,
    )
    .set_app_order(AppSourceCodeOrderType::XDotF);

    // `f(x)`
    let fx = expr_app(
        expr_var(FullName::local("f"), None),
        vec![expr_var(FullName::local("x"), None)],
        None,
    );

    // `Const { data : r }`
    let wrap_const = expr_make_struct(
        const_tycon.clone(),
        vec![("data".to_string(), expr_var(FullName::local("r"), None))],
    );

    // let Const { data : r } = f(x);
    // Const { data : r }
    let unwrap_const = expr_let(
        PatternNode::make_struct(
            const_tycon,
            vec![(
                "data".to_string(),
                PatternNode::make_var(var_local("r"), None),
            )],
        ),
        fx,
        wrap_const,
        None,
    );

    // let x = s.@{field};
    // let Const { data : r } = f(x);
    // Const { data : r }
    let let_expr = expr_let(
        PatternNode::make_var(var_local("x"), None),
        getter_expr,
        unwrap_const,
        None,
    );

    // |f, s| ...
    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(vec![var_local("s")], let_expr, None),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// Make struct object unique.
// If it is (unboxed or) unique, do nothing.
// If it is shared, clone the object.
fn make_struct_unique<'c, 'm>(gc: &mut Generator<'c, 'm>, str: Object<'c>) -> Object<'c> {
    make_struct_union_unique(gc, str)
}

// Make struct / union object unique.
// If it is (unboxed or) unique, do nothing.
// If it is shared, clone the object.
fn make_struct_union_unique<'c, 'm>(gc: &mut Generator<'c, 'm>, mut obj: Object<'c>) -> Object<'c> {
    assert!(obj.ty.is_union(gc.type_env()) || obj.ty.is_struct(gc.type_env()));

    let is_unbox = obj.ty.is_unbox(gc.type_env());
    if is_unbox {
        // In unboxed case, `obj` is always treated as unique object.
        return obj;
    }
    // In boxed case, `obj` should be replaced to cloned object if it is shared.

    // Branch by if refcnt is one.
    let obj_ptr = obj.value.into_pointer_value();
    let (unique_bb, shared_bb) = gc.build_branch_by_is_unique(obj_ptr);
    let end_bb = gc
        .context
        .append_basic_block(unique_bb.get_parent().unwrap(), "end_bb");

    // Implement shared_bb.
    gc.builder().position_at_end(shared_bb);

    // Create new object and clone fields.
    let cloned_obj = create_obj(obj.ty.clone(), &vec![], None, gc, Some("cloned_obj"));
    let cloned_obj = if obj.ty.is_struct(gc.type_env()) {
        ObjectFieldType::clone_struct(gc, &obj, cloned_obj)
    } else if obj.ty.is_union(gc.type_env()) {
        ObjectFieldType::clone_union(gc, &obj, cloned_obj)
    } else {
        unreachable!()
    };

    // Release the old object.
    gc.release(obj.clone());

    let cloned_obj_ptr = cloned_obj.value;
    let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
    gc.builder().build_unconditional_branch(end_bb).unwrap();

    // Implement unique_bb.
    gc.builder().position_at_end(unique_bb);
    // Jump to end_bb.
    gc.builder().build_unconditional_branch(end_bb).unwrap();

    // Implement end_bb.
    gc.builder().position_at_end(end_bb);
    // Build phi value.
    let obj_phi = gc
        .builder()
        .build_phi(obj_ptr.get_type(), "obj_phi")
        .unwrap();
    obj_phi.add_incoming(&[(&obj_ptr, unique_bb), (&cloned_obj_ptr, succ_of_shared_bb)]);

    obj = Object::new(obj_phi.as_basic_value(), obj.ty.clone(), gc);

    obj
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructSetBody {
    pub value_name: FullName,
    pub struct_name: FullName,
    field_count: u32,
    field_idx: u32,
    // When true, clone the struct first if it is shared, so the write lands in a uniquely owned
    // struct. Set false only where the struct is statically known to be unique.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMStructSetBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _str_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get arguments
        let value = gc.get_scoped_obj(&self.value_name);
        let str = gc.get_scoped_obj(&self.struct_name);

        // Make struct object unique.
        let str = if self.force_unique {
            make_struct_unique(gc, str)
        } else {
            str
        };

        // Release old value
        let old_value = ObjectFieldType::move_out_struct_field(gc, &str, self.field_idx as u32);
        gc.release(old_value);

        // Set new value
        ObjectFieldType::move_into_struct_field(gc, str, self.field_idx as u32, &value)
    }

    fn name(&self) -> String {
        format!(
            "struct_set_{}{}({}, {})",
            self.field_idx,
            if self.force_unique { "" } else { "[unique]" },
            self.value_name.to_string(),
            self.struct_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.value_name, &mut self.struct_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(STRUCT_SET_STRUCT_ARG, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        replaced_field_prov(
            result_ty,
            type_env,
            self.field_idx as usize,
            STRUCT_SET_STRUCT_ARG,
            STRUCT_SET_VALUE_ARG,
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The operand positions of a struct `set`: the new field value, then the struct.
const STRUCT_SET_VALUE_ARG: usize = 0;
const STRUCT_SET_STRUCT_ARG: usize = 1;

// `set` built-in function for a given struct.
pub fn struct_set(
    _struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const VALUE_NAME: &str = "val";
    const STRUCT_NAME: &str = "str";

    // Find the index of `field_name` in the given struct.
    let (field_idx, field) = definition.get_field_by_name(field_name).unwrap();
    let field_count = definition.fields().len() as u32;

    let str_ty = definition.applied_type();
    let expr = expr_abs(
        vec![var_local(VALUE_NAME)],
        expr_abs(
            vec![var_local(STRUCT_NAME)],
            expr_llvm(
                Box::new(InlineLLVMStructSetBody {
                    value_name: FullName::local(VALUE_NAME),
                    struct_name: FullName::local(STRUCT_NAME),
                    field_count,
                    field_idx,
                    force_unique: true,
                }),
                str_ty.clone(),
                None,
            ),
            None,
        ),
        None,
    );
    let ty = type_fun(field.ty.clone(), type_fun(str_ty.clone(), str_ty.clone()));
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMMakeUnionBody {
    field_name: FullName,
    generated_union_name: String,
    field_idx: usize,
}

impl InlineLLVMMakeUnionBody {
    /// The index of the variant this operation constructs.
    pub fn variant_index(&self) -> usize {
        self.field_idx
    }
}

#[typetag::serde]
impl LLVMGen for InlineLLVMMakeUnionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // Get field values.
        let field = gc.get_scoped_obj(&self.field_name);

        // Create union object.
        let obj = create_obj(
            ty.clone(),
            &vec![],
            None,
            gc,
            Some(&self.generated_union_name),
        );

        // Set tag value.
        let tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);
        let obj = ObjectFieldType::set_union_tag(gc, obj, tag_value);

        // Set value.
        ObjectFieldType::set_union_value(gc, obj, field)
    }

    fn name(&self) -> String {
        format!(
            "union_make_{}({})",
            self.field_idx,
            self.field_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.field_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // A boxed union is a fresh allocation (its single boxed leaf is at the root path). An unboxed
        // union lays out its variants: the constructed variant's boxed leaves carry the sole operand,
        // the other variants' leaves are bottom (an empty set). The path's head is the variant index,
        // its tail the position within that variant's payload.
        let active = self.variant_index();
        Provenance::build_shape(result_ty, type_env, &|path| match path.split_first() {
            None => Provenance::leaf(LeafOrigin::Fresh),
            Some((k, rest)) if *k == active => Provenance::leaf(LeafOrigin::Arg(0, rest.to_vec())),
            Some(_) => Set::default(),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// constructor function for a given union.
pub fn union_new_body(
    union_name: &FullName,
    union_defn: &TypeDefn,
    field_name: &Name,
    field_idx: usize,
) -> Arc<ExprNode> {
    let name = format!("new_{}({})", field_name, union_name.to_string());
    let name_cloned = name.clone();
    let field_name_cloned = FullName::local(field_name);
    expr_llvm(
        Box::new(InlineLLVMMakeUnionBody {
            field_name: field_name_cloned,
            generated_union_name: name_cloned,
            field_idx,
        }),
        union_defn.applied_type(),
        None,
    )
}

// `{field}` built-in function for a given union.
pub fn union_new(
    union_name: &FullName,
    field_name: &Name,
    union: &TypeDefn,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Get field index.
    let (field_idx, _) = union.get_field_by_name(field_name).unwrap();
    let field_idx = field_idx as usize;

    let expr = expr_abs(
        vec![var_local(field_name)],
        union_new_body(union_name, union, field_name, field_idx),
        None,
    );
    let union_ty = union.applied_type();
    let field_ty = union.fields()[field_idx].ty.clone();
    let ty = type_fun(field_ty, union_ty);
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// `as_{field}` built-in function for a given union.
pub fn union_as(field_name: &Name, union: &TypeDefn) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Get field index.
    let (field_idx, _) = union.get_field_by_name(field_name).unwrap();
    let field_idx = field_idx as usize;

    let union_arg_name = "union".to_string();
    let expr = expr_abs(
        vec![var_local(&union_arg_name)],
        union_as_body(
            &union_arg_name,
            field_idx,
            union.fields()[field_idx].ty.clone(),
        ),
        None,
    );
    let union_ty = union.applied_type();
    let field_ty = union.fields()[field_idx].ty.clone();
    let ty = type_fun(union_ty, field_ty);
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnionAsBody {
    union_arg_name: FullName,
    field_idx: usize,
}

impl InlineLLVMUnionAsBody {
    /// The index of the variant whose payload this operation reads.
    pub fn variant_index(&self) -> usize {
        self.field_idx
    }
}

#[typetag::serde]
impl LLVMGen for InlineLLVMUnionAsBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        // Get union object.
        let obj = gc.get_scoped_obj(&self.union_arg_name);

        let elem_ty = ty.clone();

        if gc.config.runtime_check() {
            let expected_tag = ObjectFieldType::UnionTag
                .to_basic_type(gc, vec![])
                .into_int_type()
                .const_int(self.field_idx as u64, false);

            // If tag mismatch, panic.
            ObjectFieldType::panic_if_union_tag_mismatch(gc, obj.clone(), expected_tag);
        }

        // If tag match, return the field value.
        ObjectFieldType::get_union_value(gc, obj, &elem_ty)
    }

    fn name(&self) -> String {
        format!(
            "union_as_{}({})",
            self.field_idx,
            self.union_arg_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.union_arg_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // From a boxed union the payload is `Unknown`; from an unboxed union it is a pure projection
        // carrying the scrutinee's leaf at that variant. `as` takes exactly the union, so
        // `arg_tys[0]` is it.
        let union_boxed = arg_tys[0].is_box(type_env);
        if union_boxed {
            Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)
        } else {
            let variant = self.variant_index();
            Provenance::build_shape(result_ty, type_env, &|sigma: &FieldPath| {
                let mut p = vec![variant];
                p.extend_from_slice(sigma);
                Provenance::leaf(LeafOrigin::Arg(0, p))
            })
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `as_{field}` built-in function for a given union.
pub fn union_as_body(
    union_arg_name: &Name,
    field_idx: usize,
    field_ty: Arc<TypeNode>,
) -> Arc<ExprNode> {
    let union_arg_name = FullName::local(union_arg_name);
    expr_llvm(
        Box::new(InlineLLVMUnionAsBody {
            union_arg_name,
            field_idx,
        }),
        field_ty,
        None,
    )
}

// `is_{field}` built-in function for a given union.
pub fn union_is(field_name: &Name, union: &TypeDefn) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Get field index.
    let (field_idx, _) = union.get_field_by_name(field_name).unwrap();
    let field_idx = field_idx as usize;

    let union_arg_name = "union".to_string();
    let expr = expr_abs(
        vec![var_local(&union_arg_name)],
        union_is_body(&union_arg_name, field_idx),
        None,
    );
    let union_ty = union.applied_type();
    let ty = type_fun(union_ty, make_bool_ty());
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnionIsBody {
    union_arg_name: FullName,
    field_idx: usize,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMUnionIsBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get union object.
        let obj = gc.get_scoped_obj_noretain(&self.union_arg_name);

        // Create specified tag value.
        let expected_tag = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);

        // Get tag value.
        let actual_tag = ObjectFieldType::get_union_tag(gc, &obj);

        // Compare tags and convert the boolean result to i8.
        let is_tag_match = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, expected_tag, actual_tag, "is_tag_match")
            .unwrap();
        let match_bool = gc
            .builder()
            .build_int_z_extend(is_tag_match, gc.context.i8_type(), "match_bool")
            .unwrap();

        // Return the value.
        let ret = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(format!("is_union_{}", self.field_idx).as_str()),
        );
        let ret = ret.insert_field(gc, 0, match_bool.as_basic_value_enum());
        ret
    }

    fn name(&self) -> String {
        format!(
            "union_is_{}({})",
            self.field_idx,
            self.union_arg_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.union_arg_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `is_{field}` built-in function for a given union.
pub fn union_is_body(union_arg_name: &Name, field_idx: usize) -> Arc<ExprNode> {
    expr_llvm(
        Box::new(InlineLLVMUnionIsBody {
            union_arg_name: FullName::local(union_arg_name),
            field_idx,
        }),
        make_bool_ty(),
        None,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnionModBody {
    union_name: FullName,
    modifier_name: FullName,
    field_idx: u32,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMUnionModBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, union_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get arguments
        let obj = gc.get_scoped_obj(&self.union_name);
        let modifier = gc.get_scoped_obj(&self.modifier_name);

        // Create specified tag value.
        let specified_tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);

        // Get tag value.
        let tag_value = ObjectFieldType::get_union_tag(gc, &obj);

        // Branch and store result to ret_ptr.
        let is_tag_match = gc
            .builder()
            .build_int_compare(
                IntPredicate::EQ,
                specified_tag_value,
                tag_value,
                "is_tag_match@union_mod_function",
            )
            .unwrap();
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let mut match_bb = gc.context.append_basic_block(current_func, "match_bb");
        let mut mismatch_bb = gc.context.append_basic_block(current_func, "mismatch_bb");
        let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");
        gc.builder()
            .build_conditional_branch(is_tag_match, match_bb, mismatch_bb)
            .unwrap();

        // Implement match_bb
        gc.builder().position_at_end(match_bb);
        let field_ty = union_ty.field_types(gc.type_env())[self.field_idx as usize].clone();
        let value = ObjectFieldType::get_union_value(gc, obj.clone(), &field_ty);
        let value = gc
            .apply_lambda(modifier.clone(), vec![value], false)
            .unwrap();
        // Prepare space for returned union object.
        let ret_obj = create_obj(
            union_ty.clone(),
            &vec![],
            None,
            gc,
            Some("create_obj@union_mod"),
        );
        // Set values of returned union object.
        let ret_obj = ObjectFieldType::set_union_tag(gc, ret_obj, specified_tag_value);
        let ret_obj = ObjectFieldType::set_union_value(gc, ret_obj, value);
        let match_val = ret_obj.value;
        match_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(cont_bb).unwrap();

        // Implement mismatch_bb
        gc.builder().position_at_end(mismatch_bb);
        gc.release(modifier);
        let mismatch_val = obj.value;
        mismatch_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(cont_bb).unwrap();

        // Return the value.
        gc.builder().position_at_end(cont_bb);
        let phi = gc
            .builder()
            .build_phi(match_val.get_type(), "phi@union_mod_function")
            .unwrap();
        phi.add_incoming(&[(&match_val, match_bb), (&mismatch_val, mismatch_bb)]);
        Object::new(phi.as_basic_value(), union_ty.clone(), gc)
    }

    fn name(&self) -> String {
        format!(
            "union_mod_{}({}, {})",
            self.field_idx,
            self.modifier_name.to_string(),
            self.union_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.union_name, &mut self.modifier_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn union_mod_function(
    _union_name: &FullName,
    field_name: &Name,
    union: &TypeDefn,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const UNION_NAME: &str = "union_value";
    const MODIFIER_NAME: &str = "modifier";

    let (field_idx, _) = union.get_field_by_name(&field_name).unwrap();

    let union_ty = union.applied_type();
    let field_ty = union.fields()[field_idx as usize].ty.clone();

    let expr = expr_abs(
        vec![var_local(MODIFIER_NAME)],
        expr_abs(
            vec![var_local(UNION_NAME)],
            expr_llvm(
                Box::new(InlineLLVMUnionModBody {
                    union_name: FullName::local(UNION_NAME),
                    modifier_name: FullName::local(MODIFIER_NAME),
                    field_idx,
                }),
                union_ty.clone(),
                None,
            ),
            None,
        ),
        None,
    );
    let ty = type_fun(
        type_fun(field_ty.clone(), field_ty),
        type_fun(union_ty.clone(), union_ty),
    );
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUndefinedInternalBody {
    msg_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMUndefinedInternalBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        if gc.config.runtime_check() {
            // Runtime check is enabled.
            // Print the message and abort the program.

            // Get the first argument.
            let msg = gc.get_scoped_obj(&self.msg_name);

            // Get the pointer to the message.
            let c_str = msg.gep_boxed(gc, ARRAY_BUF_IDX);

            // Write it to stderr, and flush.
            gc.call_runtime(RUNTIME_EPRINTLN, &[c_str.into()]);

            // Abort the program.
            gc.call_runtime(RUNTIME_ABORT, &[]);
        } else {
            // Runtime check is disabled.
            // Just generate unreachable instruction.

            // Generate unreachable instruction.
            gc.builder().build_unreachable().unwrap();

            // To satisfy LLVM, we need to create a valid control flow.
            let current_func = gc
                .builder()
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();
            let unreachable_bb = gc
                .context
                .append_basic_block(current_func, "unreachable_bb");
            // Following codes should be implemented in unreachable_bb.
            gc.builder().position_at_end(unreachable_bb);
        }

        // Return undefined value.
        Object::undef(ty.clone(), gc)
    }

    fn name(&self) -> String {
        format!("undefined({})", self.msg_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.msg_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // This op aborts, so the value it stands for never exists and its leaves have no source at
        // all — the bottom of the lattice, which is also the identity of the branch join. That is what
        // makes a guard clause transparent: `if bad { undefined(msg) }; value` says nothing about
        // `value`, where a source of unknown sharing would drag it down to unknown at the join and
        // hold on to a check the guarded value does not need.
        Provenance::uniform_bottom(result_ty, type_env)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `_undefined_internal` built-in function
pub fn undefined_internal_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const A_NAME: &str = "a";
    const UNDEFINED_ARG_NAME: &str = "msg";

    let expr = expr_abs(
        vec![var_local(UNDEFINED_ARG_NAME)],
        expr_llvm(
            Box::new(InlineLLVMUndefinedInternalBody {
                msg_name: FullName::local(UNDEFINED_ARG_NAME),
            }),
            type_tyvar_star(A_NAME),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            type_tyapp(make_array_ty(), make_u8_ty()),
            type_tyvar_star(A_NAME),
        ),
    );
    (expr, scm)
}

/// Inline-LLVM body for the `Std::#hole` builtin. Code generation
/// should be unreachable in practice because elaboration rejects any
/// program containing a hole; the body still emits an `unreachable`
/// instruction defensively so the LLVM module stays well-formed if
/// the diagnostic is somehow bypassed.
#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMHoleBody {}

#[typetag::serde]
impl LLVMGen for InlineLLVMHoleBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ty: &Arc<TypeNode>) -> Object<'c> {
        gc.builder().build_unreachable().unwrap();
        let current_func = gc
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let unreachable_bb = gc
            .context
            .append_basic_block(current_func, "unreachable_bb");
        gc.builder().position_at_end(unreachable_bb);
        Object::undef(ty.clone(), gc)
    }

    fn name(&self) -> String {
        "#hole".to_string()
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// `Std::#hole : a` — placeholder generated when the parser accepts
/// an empty expression position (e.g. `let x = 10; ` with no body).
/// Type-checks at any type via the generic `a`. A post-pass scans for
/// references to this name and emits ERR_HOLE.
pub fn hole_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const A_NAME: &str = "a";
    let expr = expr_llvm(
        Box::new(InlineLLVMHoleBody {}),
        type_tyvar_star(A_NAME),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], type_tyvar_star(A_NAME));
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMWithRetainedFunctionBody {
    f_name: FullName,
    x_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMWithRetainedFunctionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        // Get the argument "f".
        let f = gc.get_scoped_obj(&self.f_name);

        // Get the argument "x".
        let x = gc.get_scoped_obj(&self.x_name);

        // Retain "x" around the call so that "f" sees it as shared and cannot mutate it in place.
        gc.retain(x.clone());

        // Call "f" with "x".
        let ret = gc.apply_lambda(f, vec![x.clone()], false).unwrap();

        // Release "x".
        gc.release(x);

        // Return the result.
        ret
    }

    fn name(&self) -> String {
        format!(
            "with_retained({}, {})",
            self.x_name.to_string(),
            self.f_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.f_name, &mut self.x_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `with_retained : (a -> b) -> a -> b` built-in function
pub fn with_retained_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const A_NAME: &str = "a";
    const B_NAME: &str = "b";

    const WITH_RETAINED_F_ARG_NAME: &str = "f";
    const WITH_RETAINED_X_ARG_NAME: &str = "x";

    let expr = expr_abs(
        vec![var_local(WITH_RETAINED_F_ARG_NAME)],
        expr_abs(
            vec![var_local(WITH_RETAINED_X_ARG_NAME)],
            expr_llvm(
                Box::new(InlineLLVMWithRetainedFunctionBody {
                    f_name: FullName::local(WITH_RETAINED_F_ARG_NAME),
                    x_name: FullName::local(WITH_RETAINED_X_ARG_NAME),
                }),
                type_tyvar_star(B_NAME),
                None,
            ),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            type_fun(type_tyvar_star(A_NAME), type_tyvar_star(B_NAME)),
            type_fun(type_tyvar_star(A_NAME), type_tyvar_star(B_NAME)),
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIsUniqueFunctionBody {
    var_name: FullName,
    /// Set where the caller has proven the argument statically unique: the runtime uniqueness check
    /// is then known to succeed, so it is dropped and the returned flag is the constant `true`.
    pub(crate) assume_unique: bool,
}

/// The operand `is_unique` reports on: the value whose reference count it tests and hands back.
pub const IS_UNIQUE_VALUE_ARG: usize = 0;

#[typetag::serde]
impl LLVMGen for InlineLLVMIsUniqueFunctionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        let bool_ty = ObjectFieldType::I8
            .to_basic_type(gc, vec![])
            .into_int_type();

        // Get argument
        let obj = gc.get_scoped_obj(&self.var_name);

        // Prepare returned object.
        let ret = create_obj(ret_ty.clone(), &vec![], None, gc, Some("ret@is_unique"));

        // Get whether argument is unique.
        let is_unique = if !self.assume_unique && obj.is_box(gc.type_env()) {
            let obj_ptr = obj.value.into_pointer_value();
            let current_bb = gc.builder().get_insert_block().unwrap();
            let current_func = current_bb.get_parent().unwrap();

            let (unique_bb, shared_bb) = gc.build_branch_by_is_unique(obj_ptr);
            // Add continuing basic block.
            let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");

            // Implement unique_bb.
            gc.builder().position_at_end(unique_bb);
            let flag_unique_bb = bool_ty.const_int(1, false);
            // Jump to cont_bb.
            gc.builder().build_unconditional_branch(cont_bb).unwrap();

            // Implement shared_bb.
            gc.builder().position_at_end(shared_bb);
            let flag_shared_bb = bool_ty.const_int(0, false);
            // Jump to cont_bb.
            gc.builder().build_unconditional_branch(cont_bb).unwrap();

            // Implement cont_bb.
            gc.builder().position_at_end(cont_bb);
            let flag = gc.builder().build_phi(bool_ty, "phi@is_unique").unwrap();
            flag.add_incoming(&[(&flag_unique_bb, unique_bb), (&flag_shared_bb, shared_bb)]);
            flag.as_basic_value().into_int_value()
        } else {
            // An unboxed object is always unique, and where the caller proved the argument unique the
            // check is known to succeed; either way the flag is the constant `true`.
            bool_ty.const_int(1, false)
        };
        let bool_val = make_bool_ty().get_struct_type(gc, &vec![]).get_undef();
        let bool_val = gc
            .builder()
            .build_insert_value(bool_val, is_unique, 0, "insert@is_unique")
            .unwrap();

        // Store the result
        let ret = ret.insert_field(gc, 0, bool_val);
        let obj_val = obj.value;
        let ret = ret.insert_field(gc, 1, obj_val);

        ret
    }

    fn name(&self) -> String {
        let mark = if self.assume_unique { "[unique]" } else { "" };
        format!("is_unique{}({})", mark, self.var_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if self.assume_unique {
            return None;
        }
        unique_check_on_boxed_leaf(IS_UNIQUE_VALUE_ARG, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.assume_unique = true;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // `is_unique` returns `(Bool, a)` with the argument unchanged as the second component, yet
        // its result stays the conservative `Unknown` so the borrow pass treats the argument as
        // consumed. That consuming treatment is what makes `is_unique` detect sharing: a later use
        // of the argument forces a retain, so the container reads as shared at the check, and the
        // fold (which keys on the operand through `unique_check_operand_provs`) correctly stays off. Declaring
        // the argument a passthrough here would suppress that retain and report a shared container
        // as unique.
        Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Std::is_unique : a -> (Bool, a)
pub fn is_unique_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ret_type = make_tuple_ty(vec![make_bool_ty(), obj_type.clone()]);
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(obj_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMIsUniqueFunctionBody {
                var_name: FullName::local(VAR_NAME),
                assume_unique: false,
            }),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBoxedToRetainedPtrIOS {
    val_name: FullName,
    ios_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMBoxedToRetainedPtrIOS {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argument
        let obj = gc.get_scoped_obj(&self.val_name);
        assert!(obj.is_box(gc.type_env()));
        let ios = gc.get_scoped_obj(&self.ios_name);

        // Create the retained pointer.
        let ptr = create_obj(
            make_ptr_ty(),
            &vec![],
            None,
            gc,
            Some("ptr@boxed_to_retained_ptr_ios"),
        );
        let ptr = ptr.insert_field(gc, 0, obj.value);

        // Prepare returned object.
        let ret = create_obj(
            ret_ty.clone(),
            &vec![],
            None,
            gc,
            Some("ret@boxed_to_retained_ptr_ios"),
        );

        // Insert fields into returned object.
        let ret = ret.insert_field(gc, 0, ios.value);
        let ret = ret.insert_field(gc, 1, ptr.value);

        ret

        // Since the object should be retained by calling this function, we do not release `obj`.
    }

    fn name(&self) -> String {
        format!(
            "boxed_to_retained_ptr({}, {})",
            self.val_name.to_string(),
            self.ios_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.val_name, &mut self.ios_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn boxed_to_retained_ptr_ios() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAL_NAME: &str = "val";
    const IOS_NAME: &str = "ios";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ios_type = make_iostate_ty();
    let ret_type = make_tuple_ty(vec![ios_type.clone(), make_ptr_ty()]);
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), obj_type.clone())],
        vec![],
        type_fun(
            obj_type.clone(),
            type_fun(ios_type.clone(), ret_type.clone()),
        ),
    );
    let expr = expr_abs_many(
        vec![var_local(VAL_NAME), var_local(IOS_NAME)],
        expr_llvm(
            Box::new(InlineLLVMBoxedToRetainedPtrIOS {
                val_name: FullName::local(VAL_NAME),
                ios_name: FullName::local(IOS_NAME),
            }),
            ret_type,
            None,
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBoxedFromRetainedPtrIOS {
    ptr_name: FullName,
    ios_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMBoxedFromRetainedPtrIOS {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argument.
        let ptr = gc.get_scoped_obj(&self.ptr_name);
        let ios = gc.get_scoped_obj(&self.ios_name);
        let ptr = ptr.extract_field(gc, 0);

        // Prepare returned object.
        let ret = create_obj(
            ret_ty.clone(),
            &vec![],
            None,
            gc,
            Some("ret@boxed_from_retained_ptr_ios"),
        );

        // Insert fields into returned object.
        let ret = ret.insert_field(gc, 0, ios.value);
        let ret = ret.insert_field(gc, 1, ptr);

        ret
    }

    fn name(&self) -> String {
        format!(
            "boxed_from_retained_ptr_ios({}, {})",
            self.ptr_name.to_string(),
            self.ios_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.ptr_name, &mut self.ios_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn boxed_from_retained_ptr_ios() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const PTR_NAME: &str = "ptr";
    const IOS_NAME: &str = "ios";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ptr_type = make_ptr_ty();
    let ios_type = make_iostate_ty();
    let ret_type = make_tuple_ty(vec![ios_type.clone(), obj_type.clone()]);
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), obj_type.clone())],
        vec![],
        type_fun(
            ptr_type.clone(),
            type_fun(ios_type.clone(), ret_type.clone()),
        ),
    );
    let expr = expr_abs_many(
        vec![var_local(PTR_NAME), var_local(IOS_NAME)],
        expr_llvm(
            Box::new(InlineLLVMBoxedFromRetainedPtrIOS {
                ptr_name: FullName::local(PTR_NAME),
                ios_name: FullName::local(IOS_NAME),
            }),
            ret_type,
            None,
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
    var_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argument
        let arg = gc.get_scoped_obj_noretain(&self.var_name);

        // Get the target type.
        let arg_ty = arg.ty.clone();
        let target_ty = arg_ty.get_lambda_dst();
        assert!(target_ty.is_box(gc.type_env()));

        // Get function pointer to release function.
        let release_function_name = format!("release#{}", arg.ty.to_string_normalize());
        let func = if let Some(func) = gc.module.get_function(&release_function_name) {
            func
        } else {
            // Define release function.
            let release_function_ty = gc
                .context
                .void_type()
                .fn_type(&[gc.context.ptr_type(AddressSpace::from(0)).into()], false);
            let release_function = gc.module.add_function(
                &release_function_name,
                release_function_ty,
                Some(Linkage::Internal),
            );
            let bb = gc.context.append_basic_block(release_function, "entry");
            let _builder_guard = gc.push_builder();
            gc.builder().position_at_end(bb);

            // Get pointer to object.
            let obj_ptr = release_function.get_nth_param(0).unwrap();
            // Create object.
            let obj = Object::new(obj_ptr, target_ty.clone(), gc);
            // Release object.
            gc.release(obj);
            // Return.
            gc.builder().build_return(None).unwrap();

            release_function
        };
        let func_ptr = func.as_global_value().as_pointer_value();

        let ret = create_obj(
            make_ptr_ty(),
            &vec![],
            None,
            gc,
            Some("ret_val@get_funptr_release"),
        );
        ret.insert_field(gc, 0, func_ptr)
    }

    fn name(&self) -> String {
        format!("boxed_release_func_ptr({})", self.var_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn get_release_function_of_boxed_value() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TARGET_TY_NAME: &str = "a";
    const VAR_NAME: &str = "x";

    let target_type = type_tyvar_star(TARGET_TY_NAME);
    let arg_type = type_tyapp(make_lazy_ty(), target_type.clone());
    let ret_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), target_type.clone())],
        vec![],
        type_fun(arg_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
                var_name: FullName::local(VAR_NAME),
            }),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
    var_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argument
        let arg = gc.get_scoped_obj_noretain(&self.var_name);

        // Get the target type.
        let arg_ty = arg.ty.clone();
        let target_ty = arg_ty.get_lambda_dst();
        assert!(target_ty.is_box(gc.type_env()));

        // Get function pointer to retain function.
        let retain_function_name = format!("retain#{}", arg.ty.to_string_normalize());
        let func = if let Some(func) = gc.module.get_function(&retain_function_name) {
            func
        } else {
            // Define release function.
            let retain_function_ty = gc
                .context
                .void_type()
                .fn_type(&[gc.context.ptr_type(AddressSpace::from(0)).into()], false);
            let retain_function = gc.module.add_function(
                &retain_function_name,
                retain_function_ty,
                Some(Linkage::Internal),
            );
            let bb = gc.context.append_basic_block(retain_function, "entry");
            let _builder_guard = gc.push_builder();
            gc.builder().position_at_end(bb);

            // Get pointer to object.
            let obj_ptr = retain_function.get_nth_param(0).unwrap();
            // Create object.
            let obj = Object::new(obj_ptr, target_ty, gc);
            // retain object.
            gc.retain(obj);
            // Return.
            gc.builder().build_return(None).unwrap();

            retain_function
        };
        let func_ptr = func.as_global_value().as_pointer_value();

        let ret = create_obj(
            make_ptr_ty(),
            &vec![],
            None,
            gc,
            Some("ret_val@get_funptr_retain"),
        );
        ret.insert_field(gc, 0, func_ptr)
    }

    fn name(&self) -> String {
        format!("boxed_retain_func_ptr({})", self.var_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn get_retain_function_of_boxed_value() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TARGET_TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let target_type = type_tyvar_star(TARGET_TYPE_NAME);
    let arg_type = type_tyapp(make_lazy_ty(), target_type.clone());
    let ret_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), target_type.clone())],
        vec![],
        type_fun(arg_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
                var_name: FullName::local(VAR_NAME),
            }),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetBoxedDataPtrFunctionBody {
    var_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMGetBoxedDataPtrFunctionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get argument.
        let obj = gc.get_scoped_obj_noretain(&self.var_name);
        assert!(obj.ty.is_box(gc.type_env()));

        // Get data pointer.
        let data_ptr = get_data_pointer_from_boxed_value(gc, &obj);

        // Make returned object.
        let ret = create_obj(
            make_ptr_ty(),
            &vec![],
            None,
            gc,
            Some("ret_val@_get_boxed_ptr"),
        );
        ret.insert_field(gc, 0, data_ptr)
    }

    fn name(&self) -> String {
        format!("boxed_data_ptr({})", self.var_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    fn borrows_operand(&self, i: usize, _arg_tys: &[Arc<TypeNode>], _type_env: &TypeEnv) -> bool {
        i == 0
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn get_data_pointer_from_boxed_value<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    val: &Object<'c>,
) -> PointerValue<'c> {
    // Get the pointer to the data field.
    let data_field_idx = if val.ty.is_array() {
        ARRAY_BUF_IDX
    } else if val.ty.is_struct(gc.type_env()) {
        BOXED_TYPE_DATA_IDX
    } else {
        assert!(val.ty.is_union(gc.type_env()));
        BOXED_TYPE_DATA_IDX + UNION_DATA_IDX
    };

    // Get pointer
    let ptr = val.gep_boxed(gc, data_field_idx);
    ptr
}

pub fn get_get_boxed_ptr() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ret_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), obj_type.clone())],
        vec![],
        type_fun(obj_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMGetBoxedDataPtrFunctionBody {
                var_name: FullName::local(VAR_NAME),
            }),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnsafeMutateBoxedInternalFunctionBody {
    val_name: FullName,
    io_act_name: FullName,
    /// When true, clone the value first if it is shared, so the action writes into a uniquely owned
    /// one. Set false only where the value is statically known to be unique.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMUnsafeMutateBoxedInternalFunctionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get arguments.
        let io_act = gc.get_scoped_obj(&self.io_act_name);
        let val = gc.get_scoped_obj(&self.val_name);

        // If `val` is not boxed, error.
        assert!(val.is_box(gc.type_env()));

        // Before mutating the value, force uniqueness of the value.
        let val = force_unique_boxed(gc, val, self.force_unique);

        // Get the data pointer.
        let data_ptr = get_data_pointer_from_boxed_value(gc, &val);
        let data_ptr_obj = create_obj(make_ptr_ty(), &vec![], None, gc, Some("alloca_data_ptr"));
        let data_ptr_obj = data_ptr_obj.insert_field(gc, 0, data_ptr);

        // Run the IO action.
        let io_act = gc.apply_lambda(io_act, vec![data_ptr_obj], false).unwrap();
        let (_ios, io_res) = run_ios_runner(gc, &io_act, None);

        // Construct the return value.
        let res = create_obj(ret_ty.clone(), &vec![], None, gc, None);
        let res = ObjectFieldType::move_into_struct_field(gc, res, 0, &val);
        let res = ObjectFieldType::move_into_struct_field(gc, res, 1, &io_res);

        res
    }

    fn name(&self) -> String {
        format!(
            "mutate_boxed{}({}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.io_act_name.to_string(),
            self.val_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.val_name, &mut self.io_act_name]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(MUTATE_BOXED_VALUE_ARG, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // The result is `(value, action result)`. The value comes back uniquely owned, since this op
        // clones it when shared and is given it unique otherwise — the same reasoning as an array set,
        // and what lets an operation on the value that follows drop its check. The action's result
        // comes out of an indirect call and stays `Unknown`.
        Provenance::fresh_under(result_ty, type_env, &[MUTATE_BOXED_VALUE_FIELD])
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The operand position of the value a `mutate_boxed` writes into.
const MUTATE_BOXED_VALUE_ARG: usize = 0;
/// The path of that value in the result of `_mutate_boxed_internal`, `(value, action result)`.
const MUTATE_BOXED_VALUE_FIELD: usize = 0;

/// Clone a boxed value when it is shared, so that a write into it is not observed elsewhere. Does
/// nothing when `force_unique` is false, which is set only where the value is known to be unique.
fn force_unique_boxed<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    val: Object<'c>,
    force_unique: bool,
) -> Object<'c> {
    if !force_unique {
        return val;
    }
    if val.ty.is_array() {
        make_array_unique(gc, val)
    } else {
        make_struct_union_unique(gc, val)
    }
}

// _mutate_boxed_internal : (Ptr -> IOState -> (IOState, b)) -> a -> (a, b)
pub fn get_mutate_boxed_internal() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_A_NAME: &str = "a";
    const TYPE_B_NAME: &str = "b";
    const IO_ACT_NAME: &str = "a";
    const VAL_NAME: &str = "x";
    let a_ty = type_tyvar(TYPE_A_NAME, &kind_star());
    let b_ty = type_tyvar(TYPE_B_NAME, &kind_star());
    let ab_ty = make_tuple_ty(vec![a_ty.clone(), b_ty.clone()]);
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), a_ty.clone())],
        vec![],
        type_fun(
            type_fun(make_ptr_ty(), make_io_runner_ty(b_ty.clone())),
            type_fun(a_ty.clone(), ab_ty.clone()),
        ),
    );
    let expr = expr_abs(
        vec![var_local(IO_ACT_NAME)],
        expr_abs(
            vec![var_local(VAL_NAME)],
            expr_llvm(
                Box::new(InlineLLVMUnsafeMutateBoxedInternalFunctionBody {
                    val_name: FullName::local(VAL_NAME),
                    io_act_name: FullName::local(IO_ACT_NAME),
                    force_unique: true,
                }),
                ab_ty,
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnsafeMutateBoxedIOSInternalBody {
    val_name: FullName,
    io_act_name: FullName,
    iostate_name: FullName,
    /// As in `InlineLLVMUnsafeMutateBoxedInternalFunctionBody`.
    pub(crate) force_unique: bool,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMUnsafeMutateBoxedIOSInternalBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get arguments.
        let io_act = gc.get_scoped_obj(&self.io_act_name);
        let val = gc.get_scoped_obj(&self.val_name);
        let ios = gc.get_scoped_obj(&self.iostate_name);

        // If `val` is not boxed, error.
        assert!(val.is_box(gc.type_env()));

        // Before mutating the value, force uniqueness of the value.
        let val = force_unique_boxed(gc, val, self.force_unique);

        // Get the data pointer.
        let data_ptr = get_data_pointer_from_boxed_value(gc, &val);
        let data_ptr_obj = create_obj(make_ptr_ty(), &vec![], None, gc, Some("alloca_data_ptr"));
        let data_ptr_obj = data_ptr_obj.insert_field(gc, 0, data_ptr);

        // Run the IO action.
        let io_act = gc.apply_lambda(io_act, vec![data_ptr_obj], false).unwrap();
        let (ios, io_res) = run_ios_runner(gc, &io_act, Some(&ios));

        // Construct the return value.
        let pair_ab = create_obj(
            make_tuple_ty(vec![val.ty.clone(), io_res.ty.clone()]),
            &vec![],
            None,
            gc,
            Some("pair_ab"),
        );
        let pair_ab = ObjectFieldType::move_into_struct_field(gc, pair_ab, 0, &val);
        let pair_ab = ObjectFieldType::move_into_struct_field(gc, pair_ab, 1, &io_res);
        let res = create_obj(ret_ty.clone(), &vec![], None, gc, None);
        let res = ObjectFieldType::move_into_struct_field(gc, res, 0, &ios);
        let res = ObjectFieldType::move_into_struct_field(gc, res, 1, &pair_ab);

        res
    }

    fn name(&self) -> String {
        format!(
            "mutate_boxed_ios{}({}, {}, {})",
            if self.force_unique { "" } else { "[unique]" },
            self.io_act_name.to_string(),
            self.val_name.to_string(),
            self.iostate_name.to_string(),
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![
            &mut self.val_name,
            &mut self.io_act_name,
            &mut self.iostate_name,
        ]
    }

    fn unique_check_operand(
        &self,
        arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Option<UniqueCheckOperand> {
        if !self.force_unique {
            return None;
        }
        unique_check_on_boxed_leaf(MUTATE_BOXED_VALUE_ARG, vec![], arg_tys, type_env)
    }

    fn assuming_unique(&self) -> Box<dyn LLVMGen> {
        let mut c = self.clone();
        c.force_unique = false;
        Box::new(c)
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // As in `InlineLLVMUnsafeMutateBoxedInternalFunctionBody`, with the pair this op returns
        // wrapped in the `IOState` it threads.
        Provenance::fresh_under(
            result_ty,
            type_env,
            &[MUTATE_BOXED_IOS_PAIR_FIELD, MUTATE_BOXED_VALUE_FIELD],
        )
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The path of the returned pair in the result of `_mutate_boxed_ios_internal`, `(state, pair)`.
const MUTATE_BOXED_IOS_PAIR_FIELD: usize = 1;

// _mutate_boxed_internal : (Ptr -> IOState -> (IOState, b)) -> a -> IOState -> (IOState, (a, b))
pub fn get_mutate_boxed_ios_internal() -> (Arc<ExprNode>, Arc<Scheme>) {
    const A_TYPE_NAME: &str = "a";
    const B_TYPE_NAME: &str = "b";
    const IO_ACT_NAME: &str = "act";
    const VAL_NAME: &str = "x";
    const IOSTATE_NAME: &str = "ios";
    let a_ty = type_tyvar(A_TYPE_NAME, &kind_star());
    let iostate_ty = make_iostate_ty();
    let b_ty = type_tyvar(B_TYPE_NAME, &kind_star());
    let ab_ty = make_tuple_ty(vec![a_ty.clone(), b_ty.clone()]);
    let ret_ty = make_tuple_ty(vec![iostate_ty.clone(), ab_ty.clone()]);
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), a_ty.clone())],
        vec![],
        type_fun(
            type_fun(make_ptr_ty(), make_io_runner_ty(b_ty.clone())),
            type_fun(a_ty.clone(), make_io_runner_ty(ab_ty.clone())),
        ),
    );
    let expr = expr_abs_many(
        vec![
            var_local(IO_ACT_NAME),
            var_local(VAL_NAME),
            var_local(IOSTATE_NAME),
        ],
        expr_llvm(
            Box::new(InlineLLVMUnsafeMutateBoxedIOSInternalBody {
                io_act_name: FullName::local(IO_ACT_NAME),
                val_name: FullName::local(VAL_NAME),
                iostate_name: FullName::local(IOSTATE_NAME),
                force_unique: true,
            }),
            ret_ty,
            None,
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIOStateUnsafeCreate {}

#[typetag::serde]
impl LLVMGen for InlineLLVMIOStateUnsafeCreate {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ret_ty: &Arc<TypeNode>) -> Object<'c> {
        create_obj(make_iostate_ty(), &vec![], None, gc, Some("iostate"))
    }

    fn name(&self) -> String {
        "iostate_create".to_string()
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// IOState::_unsafe_create : IOState
pub fn make_iostate_unsafe_create() -> (Arc<ExprNode>, Arc<Scheme>) {
    let ios_ty = make_iostate_ty();
    let scm = Scheme::generalize(&[], vec![], vec![], ios_ty.clone());
    let expr = expr_llvm(Box::new(InlineLLVMIOStateUnsafeCreate {}), ios_ty, None);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMDestructorMake {
    value: FullName,
    dtor: FullName,
    ios: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMDestructorMake {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Get arguments.
        let value = gc.get_scoped_obj(&self.value); // a
        let dtor = gc.get_scoped_obj(&self.dtor); // a -> IOState -> (IOState, a)
        let ios = gc.get_scoped_obj(&self.ios); // IOState

        let ret_tys = ret_ty.field_types(gc.type_env()); // (IOState, Destructor a)
        let dtor_type = ret_tys[1].clone(); // Destructor a

        // Create destructor object.
        let dtor_obj = create_obj(dtor_type.clone(), &vec![], None, gc, Some("dtor_obj"));
        let dtor_obj = ObjectFieldType::move_into_struct_field(
            gc,
            dtor_obj,
            DESTRUCTOR_OBJECT_VALUE_FIELD_IDX,
            &value,
        );
        let dtor_obj = ObjectFieldType::move_into_struct_field(
            gc,
            dtor_obj,
            DESTRUCTOR_OBJECT_DTOR_FIELD_IDX,
            &dtor,
        );

        // Create returned object.
        let ret_obj = create_obj(
            ret_ty.clone(),
            &vec![],
            None,
            gc,
            Some("ret_obj@destructor_make"),
        );
        let ret_obj = ObjectFieldType::move_into_struct_field(gc, ret_obj, 0, &ios);
        let ret_obj = ObjectFieldType::move_into_struct_field(gc, ret_obj, 1, &dtor_obj);

        ret_obj
    }

    fn name(&self) -> String {
        format!(
            "destructor_make({}, {}, {})",
            self.value.to_string(),
            self.dtor.to_string(),
            self.ios.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.value, &mut self.dtor, &mut self.ios]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        Provenance::uniform(result_ty, type_env, LeafOrigin::Fresh)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Std::FFI::Destructor::_make : a -> (a -> IOState -> (IOState, a)) -> IOState -> (IOState, Destructor a);
pub fn destructor_make() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";

    const VAR_NAME: &str = "x";
    const DTOR_NAME: &str = "dtor";
    const IOS_NAME: &str = "ios";

    let a_ty = type_tyvar(TYPE_NAME, &kind_star());

    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            type_tyvar(TYPE_NAME, &kind_star()),
            type_fun(
                type_fun(a_ty.clone(), make_io_runner_ty(a_ty.clone())),
                make_io_runner_ty(make_destructor_ty(a_ty.clone())),
            ),
        ),
    );

    let expr = expr_abs_many(
        vec![
            var_local(VAR_NAME),
            var_local(DTOR_NAME),
            var_local(IOS_NAME),
        ],
        expr_llvm(
            Box::new(InlineLLVMDestructorMake {
                value: FullName::local(VAR_NAME),
                dtor: FullName::local(DTOR_NAME),
                ios: FullName::local(IOS_NAME),
            }),
            make_tuple_ty(vec![make_iostate_ty(), make_destructor_ty(a_ty.clone())]),
            None,
        ),
    );

    (expr, scm)
}

// Run either an IO or an IOState runner based on the type of the given value.
pub fn run_io_or_ios_runner<'b, 'm, 'c>(gc: &mut Generator<'c, 'm>, io: &Object<'c>) -> Object<'c> {
    if io.ty.toplevel_tycon().unwrap().name == make_io_tycon().name {
        run_io(gc, io)
    } else {
        run_ios_runner(gc, io, None).1
    }
}

// Run an IO runner in the IO monad and return the result.
pub fn run_io<'b, 'm, 'c>(gc: &mut Generator<'c, 'm>, io: &Object<'c>) -> Object<'c> {
    let res_ty = io.ty.collect_type_argments().into_iter().next().unwrap();
    let runner = io.extract_field(gc, 0);
    let runner_ty = type_fun(
        make_iostate_ty(),
        make_tuple_ty(vec![make_iostate_ty(), res_ty.clone()]),
    );
    let runner_obj = Object::new(runner, runner_ty, gc);
    run_ios_runner(gc, &runner_obj, None).1
}

// Given an value of type `IOState -> (IOState, a)`, run it with an initial IO state and return the result `IOState` and `a`.
pub fn run_ios_runner<'b, 'm, 'c>(
    gc: &mut Generator<'c, 'm>,
    runner: &Object<'c>,
    ios: Option<&Object<'c>>,
) -> (Object<'c>, Object<'c>) {
    let ios = if let Some(ios) = ios {
        ios.clone()
    } else {
        create_obj(make_iostate_ty(), &vec![], None, gc, Some("iostate"))
    };
    let ios_res_pair = gc.apply_lambda(runner.clone(), vec![ios], false).unwrap();
    let iostate_res = ObjectFieldType::get_struct_fields(gc, &ios_res_pair, &[0, 1]);
    let ios = iostate_res[0].clone();
    let res = iostate_res[1].clone();
    (ios, res)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMMarkThreadedFunctionBody {
    var_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMMarkThreadedFunctionBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ret_ty: &Arc<TypeNode>) -> Object<'c> {
        // Check if the `threaded` compiler flag is true.
        if !gc.config.threaded {
            panic_with_msg(
                "The `threaded` compiler flag must be set to true to use `Std::mark_threaded`.",
            );
        }

        let obj = gc.get_scoped_obj(&self.var_name);
        gc.mark_threaded(obj.clone());
        obj
    }

    fn name(&self) -> String {
        format!("mark_threaded({})", self.var_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    fn result_prov(
        &self,
        result_ty: &Arc<TypeNode>,
        _arg_tys: &[Arc<TypeNode>],
        type_env: &TypeEnv,
    ) -> Provenance {
        // This op returns the object it was given, yet its result is `Unknown` and its argument is
        // consumed, and it must stay that way. Unique-check elimination drops a check on the strength
        // of a value being uniquely owned, which for a value another thread can reach is only sound
        // because a threaded value can never be held through a `Fresh` handle: the two ways to make a
        // value threaded — this op and `boxed_from_retained_ptr` — both hand back a `Unknown` one.
        // Declaring the argument a passthrough would break that on both counts, since it also declares
        // the argument unconsumed: the caller would keep a `Fresh` handle to an object it has just
        // published to other threads, and a write through that handle would race.
        Provenance::uniform(result_ty, type_env, LeafOrigin::Unknown)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn mark_threaded_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(obj_type.clone(), obj_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            Box::new(InlineLLVMMarkThreadedFunctionBody {
                var_name: FullName::local(VAR_NAME),
            }),
            obj_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

// `infinity` built-in value
pub fn infinity_value(type_name: &str) -> (Arc<ExprNode>, Arc<Scheme>) {
    let ty = make_floating_ty(type_name).unwrap();
    let expr = expr_llvm(
        Box::new(InlineLLVMFloatLit { val: f64::INFINITY }),
        ty.clone(),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// `quiet_nan` built-in value
pub fn quiet_nan_value(type_name: &str) -> (Arc<ExprNode>, Arc<Scheme>) {
    let quiet_nan_bits = u64::MAX ^ (1 << 63);
    let nan_val: f64 = f64::from_bits(quiet_nan_bits);

    let ty = make_floating_ty(type_name).unwrap();
    let expr = expr_llvm(
        Box::new(InlineLLVMFloatLit { val: nan_val }),
        ty.clone(),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

const UNARY_OPERATOR_RHS_NAME: &str = "rhs";

pub fn unary_opeartor_instance(
    trait_id: TraitId,
    method_name: &Name,
    operand_ty: Arc<TypeNode>,
    result_ty: Arc<TypeNode>,
    generator: Box<dyn LLVMGen>,
) -> TraitImpl {
    TraitImpl {
        qual_pred: QualPred {
            pred_constraints: vec![],
            eq_constraints: vec![],
            kind_constraints: vec![],
            predicate: Predicate::make(trait_id, operand_ty),
        },
        members: make_map([(
            method_name.to_string(),
            expr_abs(
                vec![var_local(UNARY_OPERATOR_RHS_NAME)],
                expr_llvm(generator, result_ty, None),
                None,
            ),
        )]),
        member_lhs_srcs: Map::default(),
        assoc_types: Map::default(),
        define_module: STD_NAME.to_string(),
        source: None,
        is_user_defined: false,
        member_sigs: Map::default(),
    }
}

const BINARY_OPERATOR_LHS_NAME: &str = "lhs";
const BINARY_OPERATOR_RHS_NAME: &str = "rhs";

pub fn binary_opeartor_instance(
    trait_id: TraitId,
    method_name: &Name,
    operand_ty: Arc<TypeNode>,
    result_ty: Arc<TypeNode>,
    generator: Box<dyn LLVMGen>,
) -> TraitImpl {
    TraitImpl {
        qual_pred: QualPred {
            pred_constraints: vec![],
            eq_constraints: vec![],
            kind_constraints: vec![],
            predicate: Predicate::make(trait_id, operand_ty),
        },
        members: make_map([(
            method_name.to_string(),
            expr_abs(
                vec![var_local(BINARY_OPERATOR_LHS_NAME)],
                expr_abs(
                    vec![var_local(BINARY_OPERATOR_RHS_NAME)],
                    expr_llvm(generator, result_ty, None),
                    None,
                ),
                None,
            ),
        )]),
        member_lhs_srcs: Map::default(),
        assoc_types: Map::default(),
        define_module: STD_NAME.to_string(),
        source: None,
        is_user_defined: false,
        member_sigs: Map::default(),
    }
}

pub const EQ_TRAIT_NAME: &str = "Eq";
pub const EQ_TRAIT_EQ_NAME: &str = "eq";

pub fn eq_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], EQ_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntEqBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntEqBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs_obj = gc.get_scoped_obj(&self.lhs_name);
        let rhs_obj = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs_obj.extract_field(gc, 0).into_int_value();
        let rhs_val = rhs_obj.extract_field(gc, 0).into_int_value();
        let value = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, EQ_TRAIT_EQ_NAME)
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(
                value,
                ObjectFieldType::I8
                    .to_basic_type(gc, vec![])
                    .into_int_type(),
                "eq",
            )
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", EQ_TRAIT_EQ_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_eq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn eq_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        Box::new(InlineLLVMIntEqBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMPtrEqBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMPtrEqBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs_obj = gc.get_scoped_obj(&self.lhs_name);
        let rhs_obj = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs_obj.extract_field(gc, 0).into_pointer_value();
        let rhs_val = rhs_obj.extract_field(gc, 0).into_pointer_value();
        // let diff = gc
        //     .builder()
        //     .build_ptr_diff(lhs_val, rhs_val, "ptr_diff@eq_trait_instance_ptr")
        //     .unwrap();
        // let value = gc
        //     .builder()
        //     .build_int_compare(
        //         IntPredicate::EQ,
        //         diff,
        //         diff.get_type().const_zero(),
        //         EQ_TRAIT_EQ_NAME,
        //     )
        //     .unwrap();
        let value = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, EQ_TRAIT_EQ_NAME)
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(
                value,
                ObjectFieldType::I8
                    .to_basic_type(gc, vec![])
                    .into_int_type(),
                "eq_of_int",
            )
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", EQ_TRAIT_EQ_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "ptr_eq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn eq_trait_instance_ptr(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        Box::new(InlineLLVMPtrEqBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatEqBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatEqBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs_obj = gc.get_scoped_obj(&self.lhs_name);
        let rhs_obj = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs_obj.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs_obj.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_compare(FloatPredicate::OEQ, lhs_val, rhs_val, EQ_TRAIT_EQ_NAME)
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(
                value,
                ObjectFieldType::I8
                    .to_basic_type(gc, vec![])
                    .into_int_type(),
                "eq_of_float",
            )
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", EQ_TRAIT_EQ_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "float_eq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn eq_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        Box::new(InlineLLVMFloatEqBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const LESS_THAN_TRAIT_NAME: &str = "LessThan";
pub const LESS_THAN_TRAIT_LT_NAME: &str = "less_than";

pub fn less_than_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], LESS_THAN_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLessThanBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntLessThanBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs_obj = gc.get_scoped_obj(&self.lhs_name);
        let rhs_obj = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs_obj.extract_field(gc, 0).into_int_value();
        let rhs_val: IntValue = rhs_obj.extract_field(gc, 0).into_int_value();

        let is_singed = lhs_obj.ty.toplevel_tycon().unwrap().is_singned_intger();

        let value = gc
            .builder()
            .build_int_compare(
                if is_singed {
                    IntPredicate::SLT
                } else {
                    IntPredicate::ULT
                },
                lhs_val,
                rhs_val,
                LESS_THAN_TRAIT_LT_NAME,
            )
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(
                value,
                ObjectFieldType::I8
                    .to_basic_type(gc, vec![])
                    .into_int_type(),
                LESS_THAN_TRAIT_LT_NAME,
            )
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", LESS_THAN_TRAIT_LT_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_lt({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn less_than_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        less_than_trait_id(),
        &LESS_THAN_TRAIT_LT_NAME.to_string(),
        ty,
        make_bool_ty(),
        Box::new(InlineLLVMIntLessThanBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatLessThanBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatLessThanBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_compare(
                FloatPredicate::OLT,
                lhs_val,
                rhs_val,
                LESS_THAN_TRAIT_LT_NAME,
            )
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(
                value,
                ObjectFieldType::I8
                    .to_basic_type(gc, vec![])
                    .into_int_type(),
                LESS_THAN_TRAIT_LT_NAME,
            )
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", LESS_THAN_TRAIT_LT_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "float_lt({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn less_than_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        less_than_trait_id(),
        &LESS_THAN_TRAIT_LT_NAME.to_string(),
        ty,
        make_bool_ty(),
        Box::new(InlineLLVMFloatLessThanBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const LESS_THAN_OR_EQUAL_TO_TRAIT_NAME: &str = "LessThanOrEq";
pub const LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME: &str = "less_than_or_eq";

pub fn less_than_or_equal_to_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], LESS_THAN_OR_EQUAL_TO_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLessThanOrEqBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntLessThanOrEqBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let is_singed = lhs.ty.toplevel_tycon().unwrap().is_singned_intger();
        let lhs_val = lhs.extract_field(gc, 0).into_int_value();
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();
        let value = gc
            .builder()
            .build_int_compare(
                if is_singed {
                    IntPredicate::SLE
                } else {
                    IntPredicate::ULE
                },
                lhs_val,
                rhs_val,
                LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
            )
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(
                value,
                ObjectFieldType::I8
                    .to_basic_type(gc, vec![])
                    .into_int_type(),
                LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
            )
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_leq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn less_than_or_equal_to_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        less_than_or_equal_to_trait_id(),
        &LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME.to_string(),
        ty,
        make_bool_ty(),
        Box::new(InlineLLVMIntLessThanOrEqBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatLessThanOrEqBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatLessThanOrEqBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_compare(
                FloatPredicate::OLE,
                lhs_val,
                rhs_val,
                LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
            )
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(
                value,
                ObjectFieldType::I8
                    .to_basic_type(gc, vec![])
                    .into_int_type(),
                LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
            )
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "float_leq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn less_than_or_equal_to_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        less_than_or_equal_to_trait_id(),
        &LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME.to_string(),
        ty,
        make_bool_ty(),
        Box::new(InlineLLVMFloatLessThanOrEqBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const ADD_TRAIT_NAME: &str = "Add";
pub const ADD_TRAIT_ADD_NAME: &str = "add";

pub fn add_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], ADD_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntAddBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntAddBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_int_value();
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();
        let value = gc
            .builder()
            .build_int_add(lhs_val, rhs_val, ADD_TRAIT_ADD_NAME)
            .unwrap();
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", ADD_TRAIT_ADD_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_add({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn add_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        add_trait_id(),
        &ADD_TRAIT_ADD_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMIntAddBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatAddBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatAddBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_add(lhs_val, rhs_val, ADD_TRAIT_ADD_NAME)
            .unwrap();
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", ADD_TRAIT_ADD_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "float_add({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn add_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        add_trait_id(),
        &ADD_TRAIT_ADD_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMFloatAddBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const SUBTRACT_TRAIT_NAME: &str = "Sub";
pub const SUBTRACT_TRAIT_SUBTRACT_NAME: &str = "sub";

pub fn subtract_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], SUBTRACT_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntSubBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntSubBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_int_value();
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();
        let value = gc
            .builder()
            .build_int_sub(lhs_val, rhs_val, SUBTRACT_TRAIT_SUBTRACT_NAME)
            .unwrap();
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", SUBTRACT_TRAIT_SUBTRACT_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_sub({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn subtract_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        subtract_trait_id(),
        &SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMIntSubBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatSubBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatSubBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_sub(lhs_val, rhs_val, SUBTRACT_TRAIT_SUBTRACT_NAME)
            .unwrap();
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", SUBTRACT_TRAIT_SUBTRACT_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "float_sub({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn subtract_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        subtract_trait_id(),
        &SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMFloatSubBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const MULTIPLY_TRAIT_NAME: &str = "Mul";
pub const MULTIPLY_TRAIT_MULTIPLY_NAME: &str = "mul";

pub fn multiply_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], MULTIPLY_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntMulBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntMulBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_int_value();
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();
        let value = gc
            .builder()
            .build_int_mul(lhs_val, rhs_val, MULTIPLY_TRAIT_MULTIPLY_NAME)
            .unwrap();
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", MULTIPLY_TRAIT_MULTIPLY_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_mul({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn multiply_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        multiply_trait_id(),
        &MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMIntMulBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatMulBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatMulBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_mul(lhs_val, rhs_val, MULTIPLY_TRAIT_MULTIPLY_NAME)
            .unwrap();
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", MULTIPLY_TRAIT_MULTIPLY_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "float_mul({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn multiply_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        multiply_trait_id(),
        &MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMFloatMulBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const DIVIDE_TRAIT_NAME: &str = "Div";
pub const DIVIDE_TRAIT_DIVIDE_NAME: &str = "div";

pub fn divide_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], DIVIDE_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntDivBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntDivBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_int_value();
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();

        let is_singed = lhs.ty.toplevel_tycon().unwrap().is_singned_intger();

        let value = if is_singed {
            gc.builder()
                .build_int_signed_div(lhs_val, rhs_val, DIVIDE_TRAIT_DIVIDE_NAME)
                .unwrap()
        } else {
            gc.builder()
                .build_int_unsigned_div(lhs_val, rhs_val, DIVIDE_TRAIT_DIVIDE_NAME)
                .unwrap()
        };
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", DIVIDE_TRAIT_DIVIDE_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_div({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn divide_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        divide_trait_id(),
        &DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMIntDivBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatDivBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatDivBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_div(lhs_val, rhs_val, DIVIDE_TRAIT_DIVIDE_NAME)
            .unwrap();
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} lhs rhs", DIVIDE_TRAIT_DIVIDE_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "float_div({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn divide_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        divide_trait_id(),
        &DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMFloatDivBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const REMAINDER_TRAIT_NAME: &str = "Rem";
pub const REMAINDER_TRAIT_REMAINDER_NAME: &str = "rem";

pub fn remainder_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], REMAINDER_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntRemBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntRemBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_int_value();
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();

        let is_singed = lhs.ty.toplevel_tycon().unwrap().is_singned_intger();

        let value = if is_singed {
            gc.builder()
                .build_int_signed_rem(lhs_val, rhs_val, REMAINDER_TRAIT_REMAINDER_NAME)
                .unwrap()
        } else {
            gc.builder()
                .build_int_unsigned_rem(lhs_val, rhs_val, REMAINDER_TRAIT_REMAINDER_NAME)
                .unwrap()
        };
        let obj = create_obj(
            lhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{}(lhs, rhs)", REMAINDER_TRAIT_REMAINDER_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!(
            "int_rem({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn remainder_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    binary_opeartor_instance(
        remainder_trait_id(),
        &REMAINDER_TRAIT_REMAINDER_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMIntRemBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const NEGATE_TRAIT_NAME: &str = "Neg";
pub const NEGATE_TRAIT_NEGATE_NAME: &str = "neg";

pub fn negate_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], NEGATE_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntNegBody {
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMIntNegBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();
        let value = gc
            .builder()
            .build_int_neg(rhs_val, NEGATE_TRAIT_NEGATE_NAME)
            .unwrap();
        let obj = create_obj(
            rhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} rhs", NEGATE_TRAIT_NEGATE_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!("int_neg({})", self.rhs_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn negate_trait_instance_int(ty: Arc<TypeNode>) -> TraitImpl {
    unary_opeartor_instance(
        negate_trait_id(),
        &NEGATE_TRAIT_NEGATE_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMIntNegBody {
            rhs_name: FullName::local(UNARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatNegBody {
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMFloatNegBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();

        let value = gc
            .builder()
            .build_float_neg(rhs_val, NEGATE_TRAIT_NEGATE_NAME)
            .unwrap();
        let obj = create_obj(
            rhs.ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("{} rhs", NEGATE_TRAIT_NEGATE_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!("float_neg({})", self.rhs_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn negate_trait_instance_float(ty: Arc<TypeNode>) -> TraitImpl {
    unary_opeartor_instance(
        negate_trait_id(),
        &NEGATE_TRAIT_NEGATE_NAME.to_string(),
        ty.clone(),
        ty,
        Box::new(InlineLLVMFloatNegBody {
            rhs_name: FullName::local(UNARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const NOT_TRAIT_NAME: &str = "Not";
pub const NOT_TRAIT_OP_NAME: &str = "not";

pub fn not_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], NOT_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBoolNegBody {
    rhs_name: FullName,
}

#[typetag::serde]
impl LLVMGen for InlineLLVMBoolNegBody {
    fn generate<'c, 'm>(&self, gc: &mut Generator<'c, 'm>, _ty: &Arc<TypeNode>) -> Object<'c> {
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let rhs_val = rhs.extract_field(gc, 0).into_int_value();

        let bool_ty = ObjectFieldType::I8
            .to_basic_type(gc, vec![])
            .into_int_type();
        let false_val = bool_ty.const_zero();
        let value = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, rhs_val, false_val, NOT_TRAIT_OP_NAME)
            .unwrap();
        let value = gc
            .builder()
            .build_int_z_extend(value, bool_ty, NOT_TRAIT_OP_NAME)
            .unwrap();
        let obj = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("{} rhs", NOT_TRAIT_OP_NAME)),
        );
        obj.insert_field(gc, 0, value)
    }

    fn name(&self) -> String {
        format!("bool_neg({})", self.rhs_name.to_string())
    }

    fn free_vars_mut(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.rhs_name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn not_trait_instance_bool() -> TraitImpl {
    unary_opeartor_instance(
        not_trait_id(),
        &NOT_TRAIT_OP_NAME.to_string(),
        make_bool_ty(),
        make_bool_ty(),
        Box::new(InlineLLVMBoolNegBody {
            rhs_name: FullName::local(UNARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub fn boxed_trait_instance(ty: &Arc<TypeNode>) -> TraitImpl {
    let trait_id = make_boxed_trait();
    TraitImpl {
        qual_pred: QualPred {
            pred_constraints: vec![],
            eq_constraints: vec![],
            kind_constraints: vec![],
            predicate: Predicate::make(trait_id, ty.clone()),
        },
        members: Map::default(),
        member_lhs_srcs: Map::default(),
        assoc_types: Map::default(),
        define_module: STD_NAME.to_string(),
        source: None,
        is_user_defined: false,
        member_sigs: Map::default(),
    }
}
