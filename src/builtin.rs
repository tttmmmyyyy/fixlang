use std::sync::Arc;

use crate::{error::panic_with_err, misc::Set};
use ast::name::{FullName, Name, NameSpace};
use inkwell::module::Linkage;
use misc::{Map, make_map};
use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

// Implement built-in functions, types, etc.
use super::*

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
            document: Some(include_str!("./docs/std_iostate.md").to_string()),
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
        TyCon::new(FullName::from_strs(&[STD_NAME], BOOL_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
            document: Some("The type of boolean values.".to_string()),
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
            fields: vec![Field {
                name: "array_elem".to_string(), // Unused
                ty: type_tyvar_star("a"),
                syn_ty: None,
                is_punched: false,
                source: None,
            }],
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
                    .map(|i| (make_tyvar(&format!("a{}", i), &kind_star())))
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

pub fn make_arrow_name() -> FullName {
    FullName::from_strs(&[STD_NAME], ARROW_NAME)
}

pub fn make_arrow_tycon() -> TyCon {
    TyCon::new(make_arrow_name())
}

pub fn make_dynamic_object_name() -> FullName {
    FullName::from_strs(&[STD_NAME], DYNAMIC_OBJECT_NAME)
}

pub fn make_dynamic_object_tycon() -> TyCon {
    TyCon::new(make_dynamic_object_name())
}

pub fn make_destructor_object_name() -> FullName {
    FullName::from_strs(&[STD_NAME, FFI_NAME], DESTRUCTOR_OBJECT_NAME)
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
    TyCon::new(FullName::from_strs(&[STD_NAME], ARRAY_NAME))
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
    tc.name == make_destructor_object_name()
}

// Returns whether given tycon is array
pub fn is_array_tycon(tc: &TyCon) -> bool {
    *tc == make_array_tycon()
}

// Make `Std::Boxed` trait.
pub fn make_boxed_trait() -> Trait {
    Trait::from_fullname(FullName::from_strs(&[STD_NAME], BOXED_TRAIT_NAME))
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
    let mut ty = type_tycon(&tycon(make_tuple_name(tys.len() as u32)));
    for field_ty in tys {
        ty = type_tyapp(ty, field_ty);
    }
    ty
}

// Make tuple name.
pub fn make_tuple_name(size: u32) -> FullName {
    let name = format!("{}{}", TUPLE_NAME, size);
    FullName::from_strs(&[STD_NAME], &name)
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
                .map(|i| Field {
                    name: i.to_string(),
                    ty: type_from_tyvar(tyvars[i as usize].clone()),
                    syn_ty: None,
                    is_punched: false,
                    source: None,
                })
                .collect(),
            is_unbox: TUPLE_UNBOX,
        }),
        source: None,
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLit {
    val: i64, // Since `serde_pickle` only supports i64 and not u64, we use i64 here.
}

impl InlineLLVMIntLit {
    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    pub fn name(&self) -> String {
        format!("int({})", self.val)
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn expr_int_lit(val: u64, ty: Arc<TypeNode>, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::IntLit(InlineLLVMIntLit { val: val as i64 }),
        ty,
        source,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatLit {
    val: f64,
}

impl InlineLLVMFloatLit {
    pub fn name(&self) -> String {
        format!("float({})", self.val)
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn expr_float_lit(val: f64, ty: Arc<TypeNode>, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::FloatLit(InlineLLVMFloatLit { val }),
        ty,
        source,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMNullPtrLit {}

impl InlineLLVMNullPtrLit {
    pub fn name(&self) -> String {
        "nullptr".to_string()
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let obj = create_obj(ty.clone(), &vec![], None, gc, Some("nullptr"));
        let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
        let value = ptr_ty.const_null();
        obj.insert_field(gc, 0, value)
    }
}

pub fn expr_nullptr_lit(source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::NullPtrLit(InlineLLVMNullPtrLit {}),
        make_ptr_ty(),
        source,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBoolLit {
    val: bool,
}

impl InlineLLVMBoolLit {
    pub fn name(&self) -> String {
        format!("bool({})", self.val)
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![]
    }
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let obj = create_obj(
            ty.clone(),
            &vec![],
            None,
            gc,
            Some(&format!("bool_lit_{}", self.val)),
        );
        let value = gc.context.i8_type().const_int(self.val as u64, false);
        obj.insert_field(gc, 0, value)
    }
}

pub fn expr_bool_lit(val: bool, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::BoolLit(InlineLLVMBoolLit { val }),
        make_bool_ty(),
        source,
    )
}

// Create a byte array by copying from given pointer.
pub fn make_byte_array_copy<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
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

impl InlineLLVMStringBuf {
    pub fn name(&self) -> String {
        format!("string_buf(\"{}\")", self.string)
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let string_ptr = gc.add_global_string(&self.string).as_pointer_value();
        let len_with_null_terminator = gc
            .context
            .i64_type()
            .const_int(self.string.as_bytes().len() as u64 + 1, false);
        make_byte_array_copy(gc, string_ptr, len_with_null_terminator)
    }
}

pub fn make_string_lit(string: String, source: Option<Span>) -> Arc<ExprNode> {
    expr_make_struct(
        make_string_tycon(),
        vec![(
            "_data".to_string(),
            expr_llvm(
                LLVMGenerator::StringBuf(InlineLLVMStringBuf { string }),
                type_tyapp(make_array_ty(), make_u8_ty()).set_source(source.clone()),
                source.clone(),
            ),
        )],
    )
    .set_source(source)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFixBody {
    x_str: FullName,
    f_str: FullName,
    cap_name: FullName,
}

impl InlineLLVMFixBody {
    pub fn name(&self) -> String {
        format!(
            "fix({}, {})",
            self.f_str.to_string(),
            self.x_str.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.x_str, &mut self.f_str, &mut self.cap_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
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
}

fn fix_body(b: &str, f: &str, x: &str) -> Arc<ExprNode> {
    let f_str = FullName::local(f);
    let x_str = FullName::local(x);
    let cap_name = FullName::local(CAP_NAME);
    expr_llvm(
        LLVMGenerator::FixBody(InlineLLVMFixBody {
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

impl InlineLLVMCastIntegralBody {
    pub fn name(&self) -> String {
        format!(
            "cast_int({}, {}, {})",
            self.from_name.to_string(),
            self.is_source_signed,
            self.is_target_signed
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
            LLVMGenerator::CastIntegralBody(InlineLLVMCastIntegralBody {
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

impl InlineLLVMCastFloatBody {
    pub fn name(&self) -> String {
        format!("cast_float({})", self.from_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
            LLVMGenerator::CastFloatBody(InlineLLVMCastFloatBody {
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

impl InlineLLVMCastIntToFloatBody {
    pub fn name(&self) -> String {
        format!(
            "cast_int_to_float({}, {})",
            self.from_name.to_string(),
            self.is_signed
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
            LLVMGenerator::CastIntToFloatBody(InlineLLVMCastIntToFloatBody {
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

impl InlineLLVMCastFloatToIntBody {
    pub fn name(&self) -> String {
        format!(
            "cast_float_to_int({}, {})",
            self.from_name.to_string(),
            self.is_signed
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.from_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
            LLVMGenerator::CastFloatToIntBody(InlineLLVMCastFloatToIntBody {
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

impl InlineLLVMShiftBody {
    pub fn name(&self) -> String {
        format!(
            "shift_{}({}, {})",
            if self.is_left { "left" } else { "right" },
            self.value_name.to_string(),
            self.n_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.value_name, &mut self.n_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
                LLVMGenerator::ShiftBody(InlineLLVMShiftBody {
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

impl InlineLLVMBitwiseOperationBody {
    pub fn name(&self) -> String {
        format!(
            "bit_{}({}, {})",
            self.op_type.to_string(),
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
                LLVMGenerator::BitwiseOperationBody(InlineLLVMBitwiseOperationBody {
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

impl InlineLLVMBitNotBody {
    pub fn name(&self) -> String {
        format!("bit_not({})", self.operand_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.operand_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
            LLVMGenerator::BitNotBody(InlineLLVMBitNotBody {
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
pub struct InlineLLVMFillArrayBody {
    size_name: FullName,
    value_name: FullName,
    array_name: String,
}

impl InlineLLVMFillArrayBody {
    pub fn name(&self) -> String {
        format!(
            "Array::fill({}, {})",
            self.size_name.to_string(),
            self.value_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.size_name, &mut self.value_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let size = gc.get_scoped_obj_field(&self.size_name, 0).into_int_value();
        let value = gc.get_scoped_obj(&self.value_name);
        let array = create_obj(
            ty.clone(),
            &vec![],
            Some(size),
            gc,
            Some(&self.array_name.as_str()),
        );
        let array = array.insert_field(gc, ARRAY_LEN_IDX, size);
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        ObjectFieldType::initialize_array_buf_by_value(gc, size, buf, value);
        array
    }
}

// Implementation of Array::fill built-in function.
fn fill_array_body(a: &str, size: &str, value: &str) -> Arc<ExprNode> {
    let size_name = FullName::local(size);
    let value_name = FullName::local(value);
    let name = format!("Array::fill({}, {})", size, value);
    let name_cloned = name.clone();
    expr_llvm(
        LLVMGenerator::FillArrayBody(InlineLLVMFillArrayBody {
            size_name,
            value_name,
            array_name: name_cloned,
        }),
        type_tyapp(make_array_ty(), type_tyvar_star(a)),
        None,
    )
}

// "Array::fill : I64 -> a -> Array a" built-in function.
// Creates an array with same capacity.
pub fn fill_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        vec![var_local("size")],
        expr_abs(
            vec![var_local("value")],
            fill_array_body("a", "size", "value"),
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
            type_fun(
                type_tyvar_star("a"),
                type_tyapp(make_array_ty(), type_tyvar_star("a")),
            ),
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMMakeEmptyArrayBody {
    capacity_name: FullName,
}

impl InlineLLVMMakeEmptyArrayBody {
    pub fn name(&self) -> String {
        format!("Array::empty({})", self.capacity_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.capacity_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        arr_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
            Some(&format!("Array::empty({})", self.capacity_name.to_string())),
        );

        // Set size to zero.
        let cap = gc.context.i64_type().const_zero();
        array.insert_field(gc, ARRAY_LEN_IDX, cap)
    }
}

// Make an empty array.
pub fn make_empty() -> (Arc<ExprNode>, Arc<Scheme>) {
    const CAPACITY_NAME: &str = "capacity";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar);

    let expr = expr_abs(
        vec![var_local(CAPACITY_NAME)],
        expr_llvm(
            LLVMGenerator::MakeEmptyArrayBody(InlineLLVMMakeEmptyArrayBody {
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
pub struct InlineLLVMArrayUnsafeSetBody {
    arr_name: FullName,
    idx_name: FullName,
    value_name: FullName,
}

impl InlineLLVMArrayUnsafeSetBody {
    pub fn name(&self) -> String {
        format!(
            "{}.Array::unsafe_set({}, {})",
            self.arr_name.to_string(),
            self.idx_name.to_string(),
            self.value_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name, &mut self.value_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

// Set an element to an array, with no uniqueness checking and without releasing the old value.
pub fn unsafe_set_array() -> (Arc<ExprNode>, Arc<Scheme>) {
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
                    LLVMGenerator::ArrayUnsafeSetBody(InlineLLVMArrayUnsafeSetBody {
                        arr_name: FullName::local(ARR_NAME),
                        idx_name: FullName::local(IDX_NAME),
                        value_name: FullName::local(VALUE_NAME),
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
            make_i64_ty(),
            type_fun(elem_tyvar.clone(), type_fun(array_ty.clone(), array_ty)),
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayUnsafeGetBody {
    arr_name: FullName,
    idx_name: FullName,
}

impl InlineLLVMArrayUnsafeGetBody {
    pub fn name(&self) -> String {
        format!(
            "{}.Array::unsafe_get({})",
            self.arr_name.to_string(),
            self.idx_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj_noretain(&self.arr_name);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();

        // Get array buffer
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Get element
        let elem = ObjectFieldType::read_from_array_buf_noretain(gc, None, buf, ty.clone(), idx);

        // Release the array.
        if !gc.is_var_used_later(&self.arr_name) {
            gc.release(array);
        }

        elem
    }
}

// Gets a value from an array, without bounds checking and retaining the returned value.
pub fn array_unsafe_get_function() -> (Arc<ExprNode>, Arc<Scheme>) {
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
                LLVMGenerator::ArrayUnsafeGetBody(InlineLLVMArrayUnsafeGetBody {
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
pub struct InlineLLVMArrayUnsafeGetLinearFunctionBody {
    arr_name: FullName,
    idx_name: FullName,
}

impl InlineLLVMArrayUnsafeGetLinearFunctionBody {
    pub fn name(&self) -> String {
        format!(
            "{}.Array::unsafe_get_linear({})",
            self.arr_name.to_string(),
            self.idx_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.arr_name);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();

        let elem_ty = ret_ty.collect_type_argments().get(1).unwrap().clone();

        // Get array buffer
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Get the element.
        let elem =
            ObjectFieldType::read_from_array_buf_noretain(gc, None, buf, elem_ty.clone(), idx);

        // Create the return value.
        let res = create_obj(
            ret_ty.clone(),
            &vec![],
            None,
            gc,
            Some("alloca@array_unsafe_get_linear"),
        );
        let res = ObjectFieldType::move_into_struct_field(gc, res, 0, &array);
        let res = ObjectFieldType::move_into_struct_field(gc, res, 1, &elem);

        res
    }
}

// Gets a value from an array, without bounds checking and retaining the returned value.
// Type: I64 -> Array a -> (Array a, a)
pub fn array_unsafe_get_linear_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const IDX_NAME: &str = "idx";
    const ARR_NAME: &str = "array";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());
    let res_ty = make_tuple_ty(vec![array_ty.clone(), elem_tyvar.clone()]);

    let expr = expr_abs_many(
        vec![var_local(IDX_NAME), var_local(ARR_NAME)],
        expr_llvm(
            LLVMGenerator::ArrayUnsafeGetLinearFunctionBody(
                InlineLLVMArrayUnsafeGetLinearFunctionBody {
                    arr_name: FullName::local(ARR_NAME),
                    idx_name: FullName::local(IDX_NAME),
                },
            ),
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
pub struct InlineLLVMArrayUnsafeSetSizeBody {
    arr_name: FullName,
    len_name: FullName,
}

impl InlineLLVMArrayUnsafeSetSizeBody {
    pub fn name(&self) -> String {
        format!(
            "{}.Array::unsafe_set_size({})",
            self.arr_name.to_string(),
            self.len_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.len_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.arr_name);
        let length = gc.get_scoped_obj_field(&self.len_name, 0).into_int_value();

        array.insert_field(gc, ARRAY_LEN_IDX, length)
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
                LLVMGenerator::ArrayUnsafeSetSizeBody(InlineLLVMArrayUnsafeSetSizeBody {
                    arr_name,
                    len_name,
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
pub struct InlineLLVMArrayGetBody {
    arr_name: FullName,
    idx_name: FullName,
}

impl InlineLLVMArrayGetBody {
    pub fn name(&self) -> String {
        format!(
            "{}.Array::@({})",
            self.arr_name.to_string(),
            self.idx_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name, &mut self.idx_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        // let array = gc.get_var_retained_if_used_later(&self.arr_name);
        let array = gc.get_scoped_obj_noretain(&self.arr_name);

        let len = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
        let buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();
        let elem = ObjectFieldType::read_from_array_buf(gc, Some(len), buf, ty.clone(), idx);

        if !gc.is_var_used_later(&self.arr_name) {
            gc.release(array);
        }
        elem
    }
}

// Implementation of Array::get built-in function.
fn get_array_body(a: &str, array: &str, idx: &str) -> Arc<ExprNode> {
    let elem_ty = type_tyvar_star(a);

    let arr_name = FullName::local(array);
    let idx_name = FullName::local(idx);

    expr_llvm(
        LLVMGenerator::ArrayGetBody(InlineLLVMArrayGetBody { arr_name, idx_name }),
        elem_ty,
        None,
    )
}

// "Array::@ : Array a -> I64 -> a" built-in function.
pub fn get_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        vec![var_local("idx")],
        expr_abs(
            vec![var_local("array")],
            get_array_body("a", "array", "idx"),
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
            type_fun(
                type_tyapp(make_array_ty(), type_tyvar_star("a")),
                type_tyvar_star("a"),
            ),
        ),
    );
    (expr, scm)
}

// Force array object to be unique.
// If it is unique, do nothing.
// If it is shared, clone the object.
fn make_array_unique<'c, 'm>(gc: &mut GenerationContext<'c, 'm>, array: Object<'c>) -> Object<'c> {
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
    ObjectFieldType::clone_array_buf(gc, array_len, array_buf, cloned_array_buf, elem_ty);
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
}

impl InlineLLVMArraySetBody {
    pub fn name(&self) -> String {
        format!(
            "{}.Array::set({}, {})",
            self.array_name.to_string(),
            self.idx_name.to_string(),
            self.value_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![
            &mut self.array_name,
            &mut self.idx_name,
            &mut self.value_name,
        ]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.array_name);
        let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();
        let value = gc.get_scoped_obj(&self.value_name);

        // Force array to be unique
        let array = make_array_unique(gc, array);

        // Perform write and return.
        let array_len = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
        let array_buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
        ObjectFieldType::write_to_array_buf(gc, Some(array_len), array_buf, idx, value, true);
        array
    }
}

// Implementation of Array::set built-in function.
// is_unique_mode - if true, generate code that calls abort when given array is shared.
fn set_array_body(a: &str, array: &str, idx: &str, value: &str) -> Arc<ExprNode> {
    let elem_ty = type_tyvar_star(a);

    let array_str = FullName::local(array);
    let idx_str = FullName::local(idx);
    let value_str = FullName::local(value);

    expr_llvm(
        LLVMGenerator::ArraySetBody(InlineLLVMArraySetBody {
            array_name: array_str,
            idx_name: idx_str,
            value_name: value_str,
        }),
        type_tyapp(make_array_ty(), elem_ty),
        None,
    )
}

// Array::set built-in function.
pub fn set_array_common() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        vec![var_local("idx")],
        expr_abs(
            vec![var_local("value")],
            expr_abs(
                vec![var_local("array")],
                set_array_body("a", "array", "idx", "value"),
                None,
            ),
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
            type_fun(type_tyvar_star("a"), type_fun(array_ty.clone(), array_ty)),
        ),
    );
    (expr, scm)
}

// `Array::set` built-in function.
pub fn set_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    set_array_common()
}

// #[derive(Clone, Serialize, Deserialize)]
// pub struct InlineLLVMArrayModBody {
//     array_name: FullName,
//     idx_name: FullName,
//     modifier_name: FullName,
// }

// impl InlineLLVMArrayModBody {
//     pub fn name(&self) -> String {
//         format!(
//             "{}.Array::mod({}, {})",
//             self.array_name.to_string(),
//             self.idx_name.to_string(),
//             self.modifier_name.to_string()
//         )
//     }

//     pub fn free_vars(&mut self) -> Vec<&mut FullName> {
//         vec![
//             &mut self.array_name,
//             &mut self.idx_name,
//             &mut self.modifier_name,
//         ]
//     }

//     pub fn generate<'c, 'm, 'b>(
//         &self,
//         gc: &mut GenerationContext<'c, 'm>,
//         _ty: &Arc<TypeNode>,
//     ) -> Object<'c> {
//         // Get argments
//         let array = gc.get_scoped_obj(&self.array_name);
//         let idx = gc.get_scoped_obj_field(&self.idx_name, 0).into_int_value();
//         let modifier = gc.get_scoped_obj(&self.modifier_name);

//         // Make array unique
//         let array = make_array_unique(gc, array);

//         // Get old element without retain.
//         let array_len = array.extract_field(gc, ARRAY_LEN_IDX).into_int_value();
//         let array_buf = array.gep_boxed(gc, ARRAY_BUF_IDX);
//         let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
//         let elem = ObjectFieldType::read_from_array_buf_noretain(
//             gc,
//             Some(array_len),
//             array_buf,
//             elem_ty,
//             idx,
//         );

//         // Apply modifier to get a new value.
//         let elem = gc.apply_lambda(modifier, vec![elem], false).unwrap();

//         // Perform write and return.
//         ObjectFieldType::write_to_array_buf(gc, None, array_buf, idx, elem, false);
//         array
//     }
// }

// pub fn mod_array() -> (Arc<ExprNode>, Arc<Scheme>) {
//     const MODIFIED_ARRAY_NAME: &str = "arr";
//     const MODIFIER_NAME: &str = "f";
//     const INDEX_NAME: &str = "idx";
//     const ELEM_TYPE: &str = "a";

//     let elem_tyvar = type_tyvar_star(ELEM_TYPE);
//     let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

//     let expr = expr_abs(
//         vec![var_local(INDEX_NAME)],
//         expr_abs(
//             vec![var_local(MODIFIER_NAME)],
//             expr_abs(
//                 vec![var_local(MODIFIED_ARRAY_NAME)],
//                 expr_llvm(
//                     LLVMGenerator::ArrayModBody(InlineLLVMArrayModBody {
//                         array_name: FullName::local(MODIFIED_ARRAY_NAME),
//                         idx_name: FullName::local(INDEX_NAME),
//                         modifier_name: FullName::local(MODIFIER_NAME),
//                     }),
//                     array_ty.clone(),
//                     None,
//                 ),
//                 None,
//             ),
//             None,
//         ),
//         None,
//     );
//     let scm = Scheme::generalize(
//         &[],
//         vec![],
//         vec![],
//         type_fun(
//             make_i64_ty(),
//             type_fun(
//                 type_fun(elem_tyvar.clone(), elem_tyvar),
//                 type_fun(array_ty.clone(), array_ty),
//             ),
//         ),
//     );
//     (expr, scm)
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayForceUniqueBody {
    arr_name: FullName,
}

impl InlineLLVMArrayForceUniqueBody {
    pub fn name(&self) -> String {
        format!("{}.Array::force_unique", self.arr_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argments
        let array = gc.get_scoped_obj(&self.arr_name);

        // Make array unique
        let array = make_array_unique(gc, array);

        array
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
            LLVMGenerator::ArrayForceUniqueBody(InlineLLVMArrayForceUniqueBody {
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
pub struct InlineLLVMArrayGetPtrBody {
    arr_name: FullName,
}

impl InlineLLVMArrayGetPtrBody {
    pub fn name(&self) -> String {
        format!("{}.Array::get_data_ptr", self.arr_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argment
        let array = gc.get_scoped_obj_noretain(&self.arr_name);

        // Get pointer
        let ptr = array.gep_boxed(gc, ARRAY_BUF_IDX);

        // Release array
        if !gc.is_var_used_later(&self.arr_name) {
            gc.release(array);
        }

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
            LLVMGenerator::ArrayGetPtrBody(InlineLLVMArrayGetPtrBody {
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

impl InlineLLVMArrayGetSizeBody {
    pub fn name(&self) -> String {
        format!("{}.Array::get_size", self.arr_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        let array_obj = gc.get_scoped_obj_noretain(&self.arr_name);
        let len = array_obj.extract_field(gc, ARRAY_LEN_IDX).into_int_value();

        if !gc.is_var_used_later(&self.arr_name) {
            gc.release(array_obj);
        }
        let int_obj = create_obj(make_i64_ty(), &vec![], None, gc, Some("length_of_arr"));
        int_obj.insert_field(gc, 0, len)
    }
}

// `get_size` built-in function for Array.
pub fn get_size_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            LLVMGenerator::ArrayGetSizeBody(InlineLLVMArrayGetSizeBody {
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

impl InlineLLVMArrayGetCapacityBody {
    pub fn name(&self) -> String {
        format!("{}.Array::get_capacity", self.arr_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.arr_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        let array_obj = gc.get_scoped_obj_noretain(&self.arr_name);
        let len = array_obj.extract_field(gc, ARRAY_CAP_IDX).into_int_value();

        if !gc.is_var_used_later(&self.arr_name) {
            gc.release(array_obj);
        }

        let int_obj = create_obj(make_i64_ty(), &vec![], None, gc, Some("cap_of_arr"));
        int_obj.insert_field(gc, 0, len)
    }
}

// `Array::get_capacity : Array a -> I64` built-in function.
pub fn get_capacity_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            LLVMGenerator::ArrayGetCapacityBody(InlineLLVMArrayGetCapacityBody {
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
    pub fn name(&self) -> String {
        format!("{}.@{}", self.var_name.to_string(), self.field_idx)
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get struct object.
        let str = gc.get_scoped_obj(&self.var_name);
        ObjectFieldType::get_struct_fields(gc, &str, &[self.field_idx as u32])[0].clone()
    }
}

// `get` built-in function for a given struct.
pub fn struct_get_body(var_name: &str, field_idx: usize, field_ty: Arc<TypeNode>) -> Arc<ExprNode> {
    let var_name_clone = FullName::local(var_name);
    expr_llvm(
        LLVMGenerator::StructGetBody(InlineLLVMStructGetBody {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructPunchBody {
    pub var_name: FullName,
    field_idx: usize,
    force_unique: bool,
}

impl InlineLLVMStructPunchBody {
    pub fn name(&self) -> String {
        format!(
            "{}.#punch{}_{}",
            self.var_name.to_string(),
            if self.force_unique { "_fu" } else { "" },
            self.field_idx
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

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
            LLVMGenerator::StructPunchBody(InlineLLVMStructPunchBody {
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
    force_unique: bool,
}

impl InlineLLVMStructPlugInBody {
    pub fn name(&self) -> String {
        format!(
            "{}.#plug_in_{}{}({})",
            self.punched_str_name.to_string(),
            if self.force_unique { "_fu" } else { "" },
            self.field_idx,
            self.field_name.to_string(),
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.punched_str_name, &mut self.field_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
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
                LLVMGenerator::StructPlugInBody(InlineLLVMStructPlugInBody {
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
            tycon(make_tuple_name(2)),
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
    let new_name = crate::optimization::rename::generate_new_names(&used_tyvar_names, 1)[0]
        .name
        .clone();
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
            Trait::from_fullname(make_functor_name()),
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
    // (Here, we cannot use the parser because we are using "#" is not allowed as value name)
    //
    // We should use `#plug_in_fu_{field}` here, not `#plug_in_{field}`:
    // `map` can call `#plug_in_fu_{field}(ps)` multiple times, so the argument to `#plug_in_fu_{field}` can be shared.
    let expr_unique = expr_let(
        PatternNode::make_struct(
            tycon(make_tuple_name(2)),
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
                vec![
                    expr_app(
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
                    .set_app_order(AppSourceCodeOrderType::XDotF),
                ],
                None,
            )
            .set_app_order(AppSourceCodeOrderType::FX),
            vec![
                expr_app(
                    expr_var(FullName::local("f"), None),
                    vec![expr_var(FullName::local("x"), None)],
                    None,
                )
                .set_app_order(AppSourceCodeOrderType::FX),
            ],
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
        vec![
            expr_app(
                expr_var(FullName::local("f"), None),
                vec![
                    expr_app(
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
                    .set_app_order(AppSourceCodeOrderType::XDotF),
                ],
                None,
            )
            .set_app_order(AppSourceCodeOrderType::FX),
        ],
        None,
    )
    .set_app_order(AppSourceCodeOrderType::XDotF);

    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(
            vec![var_local("s")],
            expr_let(
                PatternNode::make_struct(
                    tycon(make_tuple_name(2)),
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

// Make struct object unique.
// If it is (unboxed or) unique, do nothing.
// If it is shared, clone the object.
fn make_struct_unique<'c, 'm>(gc: &mut GenerationContext<'c, 'm>, str: Object<'c>) -> Object<'c> {
    make_struct_union_unique(gc, str)
}

// Make struct / union object unique.
// If it is (unboxed or) unique, do nothing.
// If it is shared, clone the object.
fn make_struct_union_unique<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    mut obj: Object<'c>,
) -> Object<'c> {
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
}

impl InlineLLVMStructSetBody {
    pub fn name(&self) -> String {
        format!(
            "{}.set_{}({})",
            self.struct_name.to_string(),
            self.field_idx,
            self.value_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.value_name, &mut self.struct_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _str_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get arguments
        let value = gc.get_scoped_obj(&self.value_name);
        let str = gc.get_scoped_obj(&self.struct_name);

        // Make struct object unique.
        let str = make_struct_unique(gc, str);

        // Release old value
        let old_value = ObjectFieldType::move_out_struct_field(gc, &str, self.field_idx as u32);
        gc.release(old_value);

        // Set new value
        ObjectFieldType::move_into_struct_field(gc, str, self.field_idx as u32, &value)
    }
}

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
                LLVMGenerator::StructSetBody(InlineLLVMStructSetBody {
                    value_name: FullName::local(VALUE_NAME),
                    struct_name: FullName::local(STRUCT_NAME),
                    field_count,
                    field_idx,
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
    pub fn name(&self) -> String {
        format!("union_{}({})", self.field_idx, self.field_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.field_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

// constructor function for a given union.
pub fn union_new_body(
    union_name: &FullName,
    union_defn: &TypeDefn,
    field_name: &Name,
    field_idx: usize,
) -> Arc<ExprNode> {
    let name = format!("{}.new_{}", union_name.to_string(), field_name);
    let name_cloned = name.clone();
    let field_name_cloned = FullName::local(field_name);
    expr_llvm(
        LLVMGenerator::MakeUnionBody(InlineLLVMMakeUnionBody {
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
    pub fn name(&self) -> String {
        format!("{}.as_{}", self.union_arg_name.to_string(), self.field_idx)
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.union_arg_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get union object.
        let obj = gc.get_scoped_obj(&self.union_arg_name);

        let elem_ty = ty.clone();

        // Create specified tag value.
        let specified_tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);

        // If tag unmatch, panic.
        ObjectFieldType::panic_if_union_tag_unmatch(gc, obj.clone(), specified_tag_value);

        // If tag match, return the field value.
        ObjectFieldType::get_union_value(gc, obj, &elem_ty)
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
        LLVMGenerator::UnionAsBody(InlineLLVMUnionAsBody {
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

impl InlineLLVMUnionIsBody {
    pub fn name(&self) -> String {
        format!("{}.is_{}", self.union_arg_name.to_string(), self.field_idx)
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.union_arg_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get union object.
        let obj = gc.get_scoped_obj_noretain(&self.union_arg_name);

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
                "is_tag_match",
            )
            .unwrap();
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let match_bb = gc.context.append_basic_block(current_func, "match_bb");
        let unmatch_bb = gc.context.append_basic_block(current_func, "unmatch_bb");
        let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");
        gc.builder()
            .build_conditional_branch(is_tag_match, match_bb, unmatch_bb)
            .unwrap();

        gc.builder().position_at_end(match_bb);
        let one = gc.context.i8_type().const_int(1 as u64, false);
        gc.builder().build_unconditional_branch(cont_bb).unwrap();

        gc.builder().position_at_end(unmatch_bb);
        let zero = gc.context.i8_type().const_int(0 as u64, false);
        gc.builder().build_unconditional_branch(cont_bb).unwrap();

        // Return the value.
        gc.builder().position_at_end(cont_bb);
        let phi = gc.builder().build_phi(gc.context.i8_type(), "phi").unwrap();
        phi.add_incoming(&[(&one, match_bb), (&zero, unmatch_bb)]);
        let ret = create_obj(
            make_bool_ty(),
            &vec![],
            None,
            gc,
            Some(format!("is_union_{}", self.field_idx).as_str()),
        );
        let ret = ret.insert_field(gc, 0, phi.as_basic_value());
        if !gc.is_var_used_later(&self.union_arg_name) {
            gc.release(obj);
        }
        ret
    }
}

// `is_{field}` built-in function for a given union.
pub fn union_is_body(union_arg_name: &Name, field_idx: usize) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::UnionIsBody(InlineLLVMUnionIsBody {
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

impl InlineLLVMUnionModBody {
    pub fn name(&self) -> String {
        format!(
            "{}.mod_{}({})",
            self.union_name.to_string(),
            self.field_idx,
            self.modifier_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.union_name, &mut self.modifier_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        union_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
        let mut unmatch_bb = gc.context.append_basic_block(current_func, "unmatch_bb");
        let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");
        gc.builder()
            .build_conditional_branch(is_tag_match, match_bb, unmatch_bb)
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

        // Implement unmatch_bb
        gc.builder().position_at_end(unmatch_bb);
        gc.release(modifier);
        let unmatch_val = obj.value;
        unmatch_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(cont_bb).unwrap();

        // Return the value.
        gc.builder().position_at_end(cont_bb);
        let phi = gc
            .builder()
            .build_phi(match_val.get_type(), "phi@union_mod_function")
            .unwrap();
        phi.add_incoming(&[(&match_val, match_bb), (&unmatch_val, unmatch_bb)]);
        Object::new(phi.as_basic_value(), union_ty.clone(), gc)
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
                LLVMGenerator::UnionModBody(InlineLLVMUnionModBody {
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

impl InlineLLVMUndefinedInternalBody {
    pub fn name(&self) -> String {
        format!("_undefined_internal({})", self.msg_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.msg_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get the first argument.
        let msg = gc.get_scoped_obj(&self.msg_name);

        // Get the pointer to the message.
        let c_str = msg.gep_boxed(gc, ARRAY_BUF_IDX);

        // Write it to stderr, and flush.
        gc.call_runtime(RUNTIME_EPRINTLN, &[c_str.into()]);

        // Abort the program.
        gc.call_runtime(RUNTIME_ABORT, &[]);

        // Return undefined value.
        let val = if ty.is_unbox(gc.type_env()) {
            ty.get_struct_type(gc, &vec![])
                .get_undef()
                .as_basic_value_enum()
        } else {
            gc.context
                .ptr_type(AddressSpace::from(0))
                .get_undef()
                .as_basic_value_enum()
        };
        Object::new(val, ty.clone(), gc)
    }
}

// `_undefined_internal` built-in function
pub fn undefined_internal_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const A_NAME: &str = "a";
    const UNDEFINED_ARG_NAME: &str = "msg";

    let expr = expr_abs(
        vec![var_local(UNDEFINED_ARG_NAME)],
        expr_llvm(
            LLVMGenerator::UndefinedFunctionBody(InlineLLVMUndefinedInternalBody {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMWithRetainedFunctionBody {
    f_name: FullName,
    x_name: FullName,
}

impl InlineLLVMWithRetainedFunctionBody {
    pub fn name(&self) -> String {
        format!(
            "{}.with_retained({})",
            self.f_name.to_string(),
            self.x_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.f_name, &mut self.x_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get the argument "f".
        let f = gc.get_scoped_obj(&self.f_name);

        // Get the argument "x".
        let x = gc.get_scoped_obj(&self.x_name);

        // Retain "x".
        if !gc.is_var_used_later(&self.x_name) {
            gc.retain(x.clone());
        }

        // Call "f" with "x".
        let ret = gc.apply_lambda(f, vec![x.clone()], false).unwrap();

        // Release "x".
        if !gc.is_var_used_later(&self.x_name) {
            gc.release(x);
        }

        // Return the result.
        ret
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
                LLVMGenerator::WithRetainedFunctionBody(InlineLLVMWithRetainedFunctionBody {
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
}

impl InlineLLVMIsUniqueFunctionBody {
    pub fn name(&self) -> String {
        format!("{}.is_unique", self.var_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let bool_ty = ObjectFieldType::I8
            .to_basic_type(gc, vec![])
            .into_int_type();

        // Get argument
        let obj = gc.get_scoped_obj(&self.var_name);

        // Prepare returned object.
        let ret = create_obj(ret_ty.clone(), &vec![], None, gc, Some("ret@is_unique"));

        // Get whether argument is unique.
        let is_unique = if obj.is_box(gc.type_env()) {
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
            // If the object is unboxed, it is always unique.
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
            LLVMGenerator::IsUniqueFunctionBody(InlineLLVMIsUniqueFunctionBody {
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
pub struct InlineLLVMGetRetainedPtrOfBoxedValueFunctionBody {
    var_name: FullName,
}

impl InlineLLVMGetRetainedPtrOfBoxedValueFunctionBody {
    pub fn name(&self) -> String {
        format!("{}.get_retained_ptr", self.var_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argument
        let obj = gc.get_scoped_obj(&self.var_name);
        assert!(obj.is_box(gc.type_env()));

        let ptr = obj.value;
        let ret = create_obj(
            make_ptr_ty(),
            &vec![],
            None,
            gc,
            Some("ret_val@get_ptr_of_boxed_value"),
        );
        ret.insert_field(gc, 0, ptr)
        // Since the object should be retained by calling this function, we do not release `obj`.
    }
}

pub fn get_retained_ptr_of_boxed_value_function() -> (Arc<ExprNode>, Arc<Scheme>) {
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
            LLVMGenerator::GetRetainedPtrOfBoxedValueFunctionBody(
                InlineLLVMGetRetainedPtrOfBoxedValueFunctionBody {
                    var_name: FullName::local(VAR_NAME),
                },
            ),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetBoxedValueFromRetainedPtrFunctionBody {
    var_name: FullName,
}

impl InlineLLVMGetBoxedValueFromRetainedPtrFunctionBody {
    pub fn name(&self) -> String {
        format!("boxed_from_retained_ptr({})", self.var_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        assert!(ret_ty.is_box(gc.type_env()));
        // Get argument.
        let ptr = gc.get_scoped_obj(&self.var_name);
        let ptr = ptr.extract_field(gc, 0);
        Object::new(ptr, ret_ty.clone(), gc)
    }
}

pub fn get_boxed_value_from_retained_ptr_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ptr_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![Predicate::make(make_boxed_trait(), obj_type.clone())],
        vec![],
        type_fun(ptr_type.clone(), obj_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            LLVMGenerator::GetBoxedValueFromRetainedPtrFunctionBody(
                InlineLLVMGetBoxedValueFromRetainedPtrFunctionBody {
                    var_name: FullName::local(VAR_NAME),
                },
            ),
            obj_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
    var_name: FullName,
}

impl InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
    pub fn name(&self) -> String {
        format!("{}.get_ptr_to_release_func", self.var_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argument
        let arg = gc.get_scoped_obj_noretain(&self.var_name);
        if !gc.is_var_used_later(&self.var_name) {
            gc.release(arg.clone());
        }

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
            LLVMGenerator::GetReleaseFunctionOfBoxedValueFunctionBody(
                InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
                    var_name: FullName::local(VAR_NAME),
                },
            ),
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

impl InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
    pub fn name(&self) -> String {
        format!("{}.get_ptr_to_retain_func", self.var_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argument
        let arg = gc.get_scoped_obj_noretain(&self.var_name);
        if !gc.is_var_used_later(&self.var_name) {
            gc.release(arg.clone());
        }

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
            LLVMGenerator::GetRetainFunctionOfBoxedValueFunctionBody(
                InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
                    var_name: FullName::local(VAR_NAME),
                },
            ),
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

impl InlineLLVMGetBoxedDataPtrFunctionBody {
    pub fn name(&self) -> String {
        format!("{}.get_data_ptr", self.var_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get argument.
        let obj = gc.get_scoped_obj_noretain(&self.var_name);
        assert!(obj.ty.is_box(gc.type_env()));

        // Get data pointer.
        let data_ptr = get_data_pointer_from_boxed_value(gc, &obj);

        // Relase the argument object.
        if !gc.is_var_used_later(&self.var_name) {
            gc.release(obj.clone());
        }

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
}

fn get_data_pointer_from_boxed_value<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
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
            LLVMGenerator::GetBoxedDataPtrFunctionBody(InlineLLVMGetBoxedDataPtrFunctionBody {
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
}

impl InlineLLVMUnsafeMutateBoxedInternalFunctionBody {
    pub fn name(&self) -> String {
        format!(
            "{}.mutate_boxed({})",
            self.val_name.to_string(),
            self.io_act_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.val_name, &mut self.io_act_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get arguments.
        let io_act = gc.get_scoped_obj(&self.io_act_name);
        let val = gc.get_scoped_obj(&self.val_name);

        // If `val` is not boxed, error.
        assert!(val.is_box(gc.type_env()));

        // Before mutating the value, force uniqueness of the value.
        let is_array = val.ty.is_array();
        let val = if is_array {
            make_array_unique(gc, val)
        } else {
            make_struct_union_unique(gc, val)
        };

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
                LLVMGenerator::UnsafeMutateBoxedInternalBody(
                    InlineLLVMUnsafeMutateBoxedInternalFunctionBody {
                        val_name: FullName::local(VAL_NAME),
                        io_act_name: FullName::local(IO_ACT_NAME),
                    },
                ),
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
}

impl InlineLLVMUnsafeMutateBoxedIOSInternalBody {
    pub fn name(&self) -> String {
        format!(
            "_mutate_boxed_ios_internal({}, {}, {})",
            self.io_act_name.to_string(),
            self.val_name.to_string(),
            self.iostate_name.to_string(),
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![
            &mut self.val_name,
            &mut self.io_act_name,
            &mut self.iostate_name,
        ]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Get arguments.
        let io_act = gc.get_scoped_obj(&self.io_act_name);
        let val = gc.get_scoped_obj(&self.val_name);
        let ios = gc.get_scoped_obj(&self.iostate_name);

        // If `val` is not boxed, error.
        assert!(val.is_box(gc.type_env()));

        // Before mutating the value, force uniqueness of the value.
        let is_array = val.ty.is_array();
        let val = if is_array {
            make_array_unique(gc, val)
        } else {
            make_struct_union_unique(gc, val)
        };

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
}

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
            LLVMGenerator::UnsafeMutateBoxedIOSInternalBody(
                InlineLLVMUnsafeMutateBoxedIOSInternalBody {
                    io_act_name: FullName::local(IO_ACT_NAME),
                    val_name: FullName::local(VAL_NAME),
                    iostate_name: FullName::local(IOSTATE_NAME),
                },
            ),
            ret_ty,
            None,
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIOStateUnsafeCreate {}

impl InlineLLVMIOStateUnsafeCreate {
    pub fn name(&self) -> String {
        "IOState::_unsafe_create".to_string()
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        create_obj(make_iostate_ty(), &vec![], None, gc, Some("iostate"))
    }
}

// IOState::_unsafe_create : IOState
pub fn make_iostate_unsafe_create() -> (Arc<ExprNode>, Arc<Scheme>) {
    let ios_ty = make_iostate_ty();
    let scm = Scheme::generalize(&[], vec![], vec![], ios_ty.clone());
    let expr = expr_llvm(
        LLVMGenerator::IOStateUnsafeCreate(InlineLLVMIOStateUnsafeCreate {}),
        ios_ty,
        None,
    );
    (expr, scm)
}

// Run either an IO or an IOState runner based on the type of the given value.
pub fn run_io_or_ios_runner<'b, 'm, 'c>(
    gc: &mut GenerationContext<'c, 'm>,
    io: &Object<'c>,
) -> Object<'c> {
    if io.ty.toplevel_tycon().unwrap().name == make_io_tycon().name {
        run_io(gc, io)
    } else {
        run_ios_runner(gc, io, None).1
    }
}

// Run an IO runner in the IO monad and return the result.
pub fn run_io<'b, 'm, 'c>(gc: &mut GenerationContext<'c, 'm>, io: &Object<'c>) -> Object<'c> {
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
    gc: &mut GenerationContext<'c, 'm>,
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

impl InlineLLVMMarkThreadedFunctionBody {
    pub fn name(&self) -> String {
        format!("{}.mark_threaded", self.var_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.var_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        // Check if the `threaded` compiler flag is true.
        if !gc.config.threaded {
            panic_with_err(
                "The `threaded` compiler flag must be set to true to use `Std::mark_threaded`.",
            );
        }

        let obj = gc.get_scoped_obj(&self.var_name);
        gc.mark_threaded(obj.clone());
        obj
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
            LLVMGenerator::MarkThreadedFunctionBody(InlineLLVMMarkThreadedFunctionBody {
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
        LLVMGenerator::FloatLit(InlineLLVMFloatLit { val: f64::INFINITY }),
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
        LLVMGenerator::FloatLit(InlineLLVMFloatLit { val: nan_val }),
        ty.clone(),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

const UNARY_OPERATOR_RHS_NAME: &str = "rhs";

pub fn unary_opeartor_instance(
    trait_id: Trait,
    method_name: &Name,
    operand_ty: Arc<TypeNode>,
    result_ty: Arc<TypeNode>,
    generator: LLVMGenerator,
) -> TraitInstance {
    TraitInstance {
        qual_pred: QualPredicate {
            pred_constraints: vec![],
            eq_constraints: vec![],
            kind_constraints: vec![],
            predicate: Predicate::make(trait_id, operand_ty),
        },
        methods: make_map([(
            method_name.to_string(),
            expr_abs(
                vec![var_local(UNARY_OPERATOR_RHS_NAME)],
                expr_llvm(generator, result_ty, None),
                None,
            ),
        )]),
        assoc_types: Map::default(),
        define_module: STD_NAME.to_string(),
        source: None,
        is_user_defined: false,
    }
}

const BINARY_OPERATOR_LHS_NAME: &str = "lhs";
const BINARY_OPERATOR_RHS_NAME: &str = "rhs";

pub fn binary_opeartor_instance(
    trait_id: Trait,
    method_name: &Name,
    operand_ty: Arc<TypeNode>,
    result_ty: Arc<TypeNode>,
    generator: LLVMGenerator,
) -> TraitInstance {
    TraitInstance {
        qual_pred: QualPredicate {
            pred_constraints: vec![],
            eq_constraints: vec![],
            kind_constraints: vec![],
            predicate: Predicate::make(trait_id, operand_ty),
        },
        methods: make_map([(
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
        assoc_types: Map::default(),
        define_module: STD_NAME.to_string(),
        source: None,
        is_user_defined: false,
    }
}

pub const EQ_TRAIT_NAME: &str = "Eq";
pub const EQ_TRAIT_EQ_NAME: &str = "eq";

pub fn eq_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], EQ_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntEqBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntEqBody {
    pub fn name(&self) -> String {
        format!(
            "int_eq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn eq_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::IntEqBody(InlineLLVMIntEqBody {
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

impl InlineLLVMPtrEqBody {
    pub fn name(&self) -> String {
        format!(
            "ptr_eq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn eq_trait_instance_ptr(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::PtrEqBody(InlineLLVMPtrEqBody {
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

impl InlineLLVMFloatEqBody {
    pub fn name(&self) -> String {
        format!(
            "float_eq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let lhs_obj = gc.get_scoped_obj(&self.lhs_name);
        let rhs_obj = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs_obj.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs_obj.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_compare(
                inkwell::FloatPredicate::OEQ,
                lhs_val,
                rhs_val,
                EQ_TRAIT_EQ_NAME,
            )
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
}

pub fn eq_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::FloatEqBody(InlineLLVMFloatEqBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const LESS_THAN_TRAIT_NAME: &str = "LessThan";
pub const LESS_THAN_TRAIT_LT_NAME: &str = "less_than";

pub fn less_than_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], LESS_THAN_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLessThanBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntLessThanBody {
    pub fn name(&self) -> String {
        format!(
            "int_lt({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn less_than_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_trait_id(),
        &LESS_THAN_TRAIT_LT_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::IntLessThanBody(InlineLLVMIntLessThanBody {
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

impl InlineLLVMFloatLessThanBody {
    pub fn name(&self) -> String {
        format!(
            "float_lt({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_compare(
                inkwell::FloatPredicate::OLT,
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
}

pub fn less_than_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_trait_id(),
        &LESS_THAN_TRAIT_LT_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::FloatLessThanBody(InlineLLVMFloatLessThanBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const LESS_THAN_OR_EQUAL_TO_TRAIT_NAME: &str = "LessThanOrEq";
pub const LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME: &str = "less_than_or_eq";

pub fn less_than_or_equal_to_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], LESS_THAN_OR_EQUAL_TO_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLessThanOrEqBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntLessThanOrEqBody {
    pub fn name(&self) -> String {
        format!(
            "int_leq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn less_than_or_equal_to_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_or_equal_to_trait_id(),
        &LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::IntLessThanOrEqBody(InlineLLVMIntLessThanOrEqBody {
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

impl InlineLLVMFloatLessThanOrEqBody {
    pub fn name(&self) -> String {
        format!(
            "float_leq({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let lhs = gc.get_scoped_obj(&self.lhs_name);
        let rhs = gc.get_scoped_obj(&self.rhs_name);
        let lhs_val = lhs.extract_field(gc, 0).into_float_value();
        let rhs_val = rhs.extract_field(gc, 0).into_float_value();
        let value = gc
            .builder()
            .build_float_compare(
                inkwell::FloatPredicate::OLE,
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
}

pub fn less_than_or_equal_to_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_or_equal_to_trait_id(),
        &LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::FloatLessThanOrEqBody(InlineLLVMFloatLessThanOrEqBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const ADD_TRAIT_NAME: &str = "Add";
pub const ADD_TRAIT_ADD_NAME: &str = "add";

pub fn add_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], ADD_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntAddBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntAddBody {
    pub fn name(&self) -> String {
        format!(
            "int_add({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn add_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        add_trait_id(),
        &ADD_TRAIT_ADD_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntAddBody(InlineLLVMIntAddBody {
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

impl InlineLLVMFloatAddBody {
    pub fn name(&self) -> String {
        format!(
            "float_add({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn add_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        add_trait_id(),
        &ADD_TRAIT_ADD_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatAddBody(InlineLLVMFloatAddBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const SUBTRACT_TRAIT_NAME: &str = "Sub";
pub const SUBTRACT_TRAIT_SUBTRACT_NAME: &str = "sub";

pub fn subtract_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], SUBTRACT_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntSubBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntSubBody {
    pub fn name(&self) -> String {
        format!(
            "int_sub({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn subtract_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        subtract_trait_id(),
        &SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntSubBody(InlineLLVMIntSubBody {
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

impl InlineLLVMFloatSubBody {
    pub fn name(&self) -> String {
        format!(
            "float_sub({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn subtract_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        subtract_trait_id(),
        &SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatSubBody(InlineLLVMFloatSubBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const MULTIPLY_TRAIT_NAME: &str = "Mul";
pub const MULTIPLY_TRAIT_MULTIPLY_NAME: &str = "mul";

pub fn multiply_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], MULTIPLY_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntMulBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntMulBody {
    pub fn name(&self) -> String {
        format!(
            "int_mul({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn multiply_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        multiply_trait_id(),
        &MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntMulBody(InlineLLVMIntMulBody {
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

impl InlineLLVMFloatMulBody {
    pub fn name(&self) -> String {
        format!(
            "float_mul({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn multiply_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        multiply_trait_id(),
        &MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatMulBody(InlineLLVMFloatMulBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const DIVIDE_TRAIT_NAME: &str = "Div";
pub const DIVIDE_TRAIT_DIVIDE_NAME: &str = "div";

pub fn divide_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], DIVIDE_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntDivBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntDivBody {
    pub fn name(&self) -> String {
        format!(
            "int_div({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn divide_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        divide_trait_id(),
        &DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntDivBody(InlineLLVMIntDivBody {
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

impl InlineLLVMFloatDivBody {
    pub fn name(&self) -> String {
        format!(
            "float_div({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn divide_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        divide_trait_id(),
        &DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatDivBody(InlineLLVMFloatDivBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const REMAINDER_TRAIT_NAME: &str = "Rem";
pub const REMAINDER_TRAIT_REMAINDER_NAME: &str = "rem";

pub fn remainder_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], REMAINDER_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntRemBody {
    lhs_name: FullName,
    rhs_name: FullName,
}

impl InlineLLVMIntRemBody {
    pub fn name(&self) -> String {
        format!(
            "int_rem({}, {})",
            self.lhs_name.to_string(),
            self.rhs_name.to_string()
        )
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.lhs_name, &mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn remainder_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        remainder_trait_id(),
        &REMAINDER_TRAIT_REMAINDER_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntRemBody(InlineLLVMIntRemBody {
            lhs_name: FullName::local(BINARY_OPERATOR_LHS_NAME),
            rhs_name: FullName::local(BINARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const NEGATE_TRAIT_NAME: &str = "Neg";
pub const NEGATE_TRAIT_NEGATE_NAME: &str = "neg";

pub fn negate_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], NEGATE_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntNegBody {
    rhs_name: FullName,
}

impl InlineLLVMIntNegBody {
    pub fn name(&self) -> String {
        format!("int_neg({})", self.rhs_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn negate_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    unary_opeartor_instance(
        negate_trait_id(),
        &NEGATE_TRAIT_NEGATE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntNegBody(InlineLLVMIntNegBody {
            rhs_name: FullName::local(UNARY_OPERATOR_RHS_NAME),
        }),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatNegBody {
    rhs_name: FullName,
}

impl InlineLLVMFloatNegBody {
    pub fn name(&self) -> String {
        format!("float_neg({})", self.rhs_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn negate_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    unary_opeartor_instance(
        negate_trait_id(),
        &NEGATE_TRAIT_NEGATE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatNegBody(InlineLLVMFloatNegBody {
            rhs_name: FullName::local(UNARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub const NOT_TRAIT_NAME: &str = "Not";
pub const NOT_TRAIT_OP_NAME: &str = "not";

pub fn not_trait_id() -> Trait {
    Trait {
        name: FullName::from_strs(&[STD_NAME], NOT_TRAIT_NAME),
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBoolNegBody {
    rhs_name: FullName,
}

impl InlineLLVMBoolNegBody {
    pub fn name(&self) -> String {
        format!("bool_neg({})", self.rhs_name.to_string())
    }

    pub fn free_vars(&mut self) -> Vec<&mut FullName> {
        vec![&mut self.rhs_name]
    }

    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
    ) -> Object<'c> {
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
}

pub fn not_trait_instance_bool() -> TraitInstance {
    unary_opeartor_instance(
        not_trait_id(),
        &NOT_TRAIT_OP_NAME.to_string(),
        make_bool_ty(),
        make_bool_ty(),
        LLVMGenerator::BoolNegBody(InlineLLVMBoolNegBody {
            rhs_name: FullName::local(UNARY_OPERATOR_RHS_NAME),
        }),
    )
}

pub fn boxed_trait_instance(ty: &Arc<TypeNode>) -> TraitInstance {
    let trait_id = make_boxed_trait();
    TraitInstance {
        qual_pred: QualPredicate {
            pred_constraints: vec![],
            eq_constraints: vec![],
            kind_constraints: vec![],
            predicate: Predicate::make(trait_id, ty.clone()),
        },
        methods: Map::default(),
        assoc_types: Map::default(),
        define_module: STD_NAME.to_string(),
        source: None,
        is_user_defined: false,
    }
}
