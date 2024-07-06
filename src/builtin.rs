use std::sync::Arc;

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};

// Implement built-in functions, types, etc.
use super::*;

pub fn bulitin_tycons() -> HashMap<TyCon, TyConInfo> {
    let mut ret = HashMap::new();
    ret.insert(
        TyCon::new(FullName::from_strs(&[STD_NAME], PTR_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            fields: vec![],
            source: None,
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
        },
    );
    // IO is defined in the source code of Std.

    ret.insert(
        make_array_tycon(),
        TyConInfo {
            kind: kind_arrow(kind_star(), kind_star()),
            variant: TyConVariant::Array,
            is_unbox: false,
            tyvars: vec![tyvar_from_name("a", &kind_star())],
            fields: vec![Field {
                name: "array_elem".to_string(), // Unused
                ty: type_tyvar_star("a"),
                is_punched: false,
            }],
            source: None,
        },
    );
    // String is defined in the source code of Std.

    // Function Pointers
    for arity in 1..=FUNPTR_ARGS_MAX {
        ret.insert(
            make_funptr_tycon(arity),
            TyConInfo {
                kind: make_kind_fun(arity),
                variant: TyConVariant::Primitive,
                is_unbox: true,
                tyvars: (0..arity)
                    .map(|i| (tyvar_from_name(&format!("a{}", i), &kind_star())))
                    .collect(),
                fields: vec![],
                source: None,
            },
        );
    }
    // Dynamic object
    ret.insert(
        TyCon::new(make_dynamic_object_name()),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::DynamicObject,
            is_unbox: false,
            tyvars: vec![],
            fields: vec![],
            source: None,
        },
    );

    ret
}

pub fn make_dynamic_object_name() -> FullName {
    FullName::from_strs(&[STD_NAME], DYNAMIC_OBJECT_NAME)
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

pub fn make_kind_fun(arity: u32) -> Arc<Kind> {
    let mut res = kind_star();
    for _ in 0..arity {
        res = kind_arrow(kind_star(), res);
    }
    res
}

// Following types are coustructed using primitive types.
pub const LOOP_RESULT_NAME: &str = "LoopResult";
pub const TUPLE_NAME: &str = "Tuple";

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

// Get String type.
pub fn make_string_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], STRING_NAME)))
}

// Get LoopResult type.
pub fn make_loop_result_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(FullName::from_strs(&[STD_NAME], LOOP_RESULT_NAME)))
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

// Make type `IO ()`
pub fn make_io_unit_ty() -> Arc<TypeNode> {
    type_tyapp(
        type_tycon(&tycon(FullName::from_strs(&[STD_NAME], IO_NAME))),
        make_unit_ty(),
    )
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
        .map(|i| tyvar_from_name(&("t".to_string() + &i.to_string()), &kind_star()))
        .collect::<Vec<_>>();
    TypeDefn {
        name: make_tuple_name(size),
        tyvars: tyvars.clone(),
        value: TypeDeclValue::Struct(Struct {
            fields: (0..size)
                .map(|i| Field {
                    name: i.to_string(),
                    ty: type_from_tyvar(tyvars[i as usize].clone()),
                    is_punched: false,
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
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let obj = if rvo.is_none() {
            allocate_obj(
                ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("int_lit_{}", self.val)),
            )
        } else {
            rvo.unwrap()
        };
        let int_ty = ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_int_type();
        let value = int_ty.const_int(self.val as u64, false);
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn expr_int_lit(val: u64, ty: Arc<TypeNode>, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::IntLit(InlineLLVMIntLit { val: val as i64 }),
        vec![],
        val.to_string(),
        ty,
        source,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatLit {
    val: f64,
}

impl InlineLLVMFloatLit {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let obj = if rvo.is_none() {
            allocate_obj(
                ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("float_lit_{}", self.val)),
            )
        } else {
            rvo.unwrap()
        };
        let float_ty = ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_float_type();
        let value = float_ty.const_float(self.val);
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn expr_float_lit(val: f64, ty: Arc<TypeNode>, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::FloatLit(InlineLLVMFloatLit { val }),
        vec![],
        val.to_string(),
        ty,
        source,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMNullPtrLit {}

impl InlineLLVMNullPtrLit {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let obj = if rvo.is_none() {
            allocate_obj(ty.clone(), &vec![], None, gc, Some("nullptr"))
        } else {
            rvo.unwrap()
        };
        let ptr_ty = gc.context.i8_type().ptr_type(AddressSpace::from(0));
        let value = ptr_ty.const_null();
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn expr_nullptr_lit(source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::NullPtrLit(InlineLLVMNullPtrLit {}),
        vec![],
        "nullptr_literal".to_string(),
        make_ptr_ty(),
        source,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBoolLit {
    val: bool,
}

impl InlineLLVMBoolLit {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let obj = if rvo.is_none() {
            allocate_obj(
                ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("bool_lit_{}", self.val)),
            )
        } else {
            rvo.unwrap()
        };
        let value = gc.context.i8_type().const_int(self.val as u64, false);
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn expr_bool_lit(val: bool, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::BoolLit(InlineLLVMBoolLit { val }),
        vec![],
        val.to_string(),
        make_bool_ty(),
        source,
    )
}

pub fn make_string_from_ptr<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    buf_with_null_terminator: PointerValue<'c>,
    len_with_null_terminator: IntValue<'c>,
    rvo: Option<Object<'c>>,
    _borrowed_vars: &Vec<FullName>,
) -> Object<'c> {
    // Create `Array U8` which contains null-terminated string.
    let array_ty = type_tyapp(make_array_ty(), make_u8_ty());
    let array = allocate_obj(
        array_ty,
        &vec![],
        Some(len_with_null_terminator),
        gc,
        Some("array@make_string_from_ptr"),
    );
    array.store_field_nocap(gc, ARRAY_LEN_IDX, len_with_null_terminator);
    let dst = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
    let len_ptr = gc.builder().build_int_cast(
        len_with_null_terminator,
        gc.context.ptr_sized_int_type(&gc.target_data, None),
        "len_ptr@make_string_from_ptr",
    );
    gc.builder()
        .build_memcpy(dst, 1, buf_with_null_terminator, 1, len_ptr)
        .ok()
        .unwrap();

    // Allocate String and store the array into it.
    let string = if rvo.is_none() {
        allocate_obj(
            make_string_ty(),
            &vec![],
            None,
            gc,
            Some(&format!("string@make_string_from_ptr")),
        )
    } else {
        rvo.unwrap()
    };
    assert!(string.is_unbox(gc.type_env()));

    // Store array to data.
    let array_val = array.value(gc);
    string.store_field_nocap(gc, 0, array_val);

    string
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStringLit {
    string: String,
}

impl InlineLLVMStringLit {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let string_ptr = gc
            .builder()
            .build_global_string_ptr(&self.string, "string_literal")
            .as_basic_value_enum()
            .into_pointer_value();
        let len_with_null_terminator = gc
            .context
            .i64_type()
            .const_int(self.string.as_bytes().len() as u64 + 1, false);
        make_string_from_ptr(
            gc,
            string_ptr,
            len_with_null_terminator,
            rvo,
            _borrowed_vars,
        )
    }
}

pub fn make_string_from_rust_string(string: String, source: Option<Span>) -> Arc<ExprNode> {
    expr_llvm(
        LLVMGenerator::StringLit(InlineLLVMStringLit { string }),
        vec![],
        "string_literal".to_string(),
        make_string_ty(),
        source,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFixBody {
    x_str: FullName,
    f_str: FullName,
}

impl InlineLLVMFixBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get arguments
        let x = gc.get_var(&self.x_str).ptr.get(gc);
        let f = gc.get_var(&self.f_str).ptr.get(gc);

        // Create "fix(f)" closure.
        let fixf_ty = f.ty.get_lambda_dst();
        let fixf = allocate_obj(fixf_ty.clone(), &vec![], None, gc, Some("fix(f)"));
        let fixf_funptr = gc
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap()
            .as_global_value()
            .as_pointer_value();
        fixf.store_field_nocap(gc, CLOSURE_FUNPTR_IDX, fixf_funptr);
        let cap_obj = gc.get_var(&FullName::local(CAP_NAME)).ptr.get(gc);
        let cap_obj_ptr = cap_obj.ptr(gc);
        fixf.store_field_nocap(gc, CLOSURE_CAPTURE_IDX, cap_obj_ptr);

        let f_fixf = gc.apply_lambda(f, vec![fixf], None);
        let f_fixf_x = gc.apply_lambda(f_fixf, vec![x], rvo);
        f_fixf_x
    }
}

fn fix_body(b: &str, f: &str, x: &str) -> Arc<ExprNode> {
    let f_str = FullName::local(f);
    let x_str = FullName::local(x);
    let name = format!("fix({}, {})", f_str.to_string(), x_str.to_string());
    let free_vars = vec![FullName::local(CAP_NAME), f_str.clone(), x_str.clone()];
    expr_llvm(
        LLVMGenerator::FixBody(InlineLLVMFixBody { x_str, f_str }),
        free_vars,
        name,
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
    from_name: String,
    is_source_signed: bool,
    is_target_signed: bool,
}

impl InlineLLVMCastIntegralBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get value
        let from_val = gc
            .get_var_field(&FullName::local(&self.from_name), 0)
            .into_int_value();
        gc.release(gc.get_var(&FullName::local(&self.from_name)).ptr.get(gc));

        // Get target type.
        let to_int = to_ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_int_type();

        // Perform cast.
        let to_val = gc.builder().build_int_cast_sign_flag(
            from_val,
            to_int,
            self.is_source_signed,
            "build_int_cast_sign_flag@cast_between_integral_function",
        );

        // Return result.
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                to_ty.clone(),
                &vec![],
                None,
                gc,
                Some("alloca@cast_between_integral_function"),
            )
        };
        obj.store_field_nocap(gc, 0, to_val);
        obj
    }
}

// Cast function of integrals
pub fn cast_between_integral_function(
    from: Arc<TypeNode>,
    to: Arc<TypeNode>,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const FROM_NAME: &str = "from";
    let is_source_signed = from.toplevel_tycon().unwrap().is_singned_intger();
    let is_target_signed = to.toplevel_tycon().unwrap().is_singned_intger();
    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(from.clone(), to.clone()),
    );
    let expr = expr_abs(
        vec![var_local(FROM_NAME)],
        expr_llvm(
            LLVMGenerator::CastIntegralBody(InlineLLVMCastIntegralBody {
                from_name: FROM_NAME.to_string(),
                is_target_signed,
                is_source_signed,
            }),
            vec![FullName::local(FROM_NAME)],
            format!(
                "cast_{}_to_{}({})",
                from.to_string(),
                to.to_string(),
                FROM_NAME
            ),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCastFloatBody {
    from_name: String,
}

impl InlineLLVMCastFloatBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get value
        let from_val = gc
            .get_var_field(&FullName::local(&self.from_name), 0)
            .into_float_value();
        gc.release(gc.get_var(&FullName::local(&self.from_name)).ptr.get(gc));

        // Get target type.
        let to_float = to_ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_float_type();

        // Perform cast.
        let to_val = gc.builder().build_float_cast(
            from_val,
            to_float,
            "float_cast@cast_between_float_function",
        );

        // Return result.
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                to_ty.clone(),
                &vec![],
                None,
                gc,
                Some("alloca@cast_between_float_function"),
            )
        };
        obj.store_field_nocap(gc, 0, to_val);
        obj
    }
}

// Cast function of integrals
pub fn cast_between_float_function(
    from: Arc<TypeNode>,
    to: Arc<TypeNode>,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const FROM_NAME: &str = "from";
    let scm = Scheme::generalize(
        Default::default(),
        vec![],
        vec![],
        type_fun(from.clone(), to.clone()),
    );
    let expr = expr_abs(
        vec![var_local(FROM_NAME)],
        expr_llvm(
            LLVMGenerator::CastFloatBody(InlineLLVMCastFloatBody {
                from_name: FROM_NAME.to_string(),
            }),
            vec![FullName::local(FROM_NAME)],
            format!(
                "cast_{}_to_{}({})",
                from.to_string(),
                to.to_string(),
                FROM_NAME
            ),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCastIntToFloatBody {
    from_name: String,
    is_signed: bool,
}

impl InlineLLVMCastIntToFloatBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get value
        let from_val = gc
            .get_var_field(&FullName::local(&self.from_name), 0)
            .into_int_value();
        gc.release(gc.get_var(&FullName::local(&self.from_name)).ptr.get(gc));

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
        };

        // Return result.
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                to_ty.clone(),
                &vec![],
                None,
                gc,
                Some("alloca@cast_int_to_float_function"),
            )
        };
        obj.store_field_nocap(gc, 0, to_val);
        obj
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
                from_name: FROM_NAME.to_string(),
                is_signed,
            }),
            vec![FullName::local(FROM_NAME)],
            format!(
                "cast_{}_to_{}({})",
                from.to_string(),
                to.to_string(),
                FROM_NAME
            ),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMCastFloatToIntBody {
    from_name: String,
    is_signed: bool,
}

impl InlineLLVMCastFloatToIntBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        to_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get value
        let from_val = gc
            .get_var_field(&FullName::local(&self.from_name), 0)
            .into_float_value();
        gc.release(gc.get_var(&FullName::local(&self.from_name)).ptr.get(gc));

        // Get target type.
        let to_int = to_ty
            .get_struct_type(gc, &vec![])
            .get_field_type_at_index(0)
            .unwrap()
            .into_int_type();

        // Perform cast.
        let to_val = if self.is_signed {
            gc.builder().build_float_to_signed_int(
                from_val,
                to_int,
                "float_to_signed_int@cast_float_to_int_function",
            )
        } else {
            gc.builder().build_float_to_unsigned_int(
                from_val,
                to_int,
                "float_to_unsigned_int@cast_float_to_int_function",
            )
        };

        // Return result.
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                to_ty.clone(),
                &vec![],
                None,
                gc,
                Some("alloca@cast_float_to_int_function"),
            )
        };
        obj.store_field_nocap(gc, 0, to_val);
        obj
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
                from_name: FROM_NAME.to_string(),
                is_signed,
            }),
            vec![FullName::local(FROM_NAME)],
            format!(
                "cast_{}_to_{}({})",
                from.to_string(),
                to.to_string(),
                FROM_NAME
            ),
            to,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMShiftBody {
    value_name: String,
    n_name: String,
    is_left: bool,
}

impl InlineLLVMShiftBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get value
        let val = gc
            .get_var_field(&FullName::local(&self.value_name), 0)
            .into_int_value();
        let n = gc
            .get_var_field(&FullName::local(&self.n_name), 0)
            .into_int_value();

        let is_signed = ty.toplevel_tycon().unwrap().is_singned_intger();

        // Perform cast.
        let to_val = if self.is_left {
            gc.builder()
                .build_left_shift(val, n, "left_shift@shift_function")
        } else {
            gc.builder()
                .build_right_shift(val, n, is_signed, "right_shift@shift_function")
        };

        // Return result.
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(ty.clone(), &vec![], None, gc, Some("alloca@shift_function"))
        };
        obj.store_field_nocap(gc, 0, to_val);
        obj
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
                    value_name: VALUE_NAME.to_string(),
                    n_name: N_NAME.to_string(),
                    is_left,
                }),
                vec![FullName::local(VALUE_NAME), FullName::local(N_NAME)],
                format!(
                    "shift_{}({},{})",
                    if is_left { "left" } else { "right" },
                    N_NAME,
                    VALUE_NAME
                ),
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
    lhs_name: String,
    rhs_name: String,
    op_type: BitOperationType,
}

impl InlineLLVMBitwiseOperationBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get value
        let lhs = gc
            .get_var_field(&FullName::local(&self.lhs_name), 0)
            .into_int_value();
        let rhs = gc
            .get_var_field(&FullName::local(&self.rhs_name), 0)
            .into_int_value();

        // Perform cast.
        let val = match self.op_type {
            BitOperationType::Xor => {
                gc.builder()
                    .build_xor(lhs, rhs, "xor@bitwise_operation_function")
            }
            BitOperationType::Or => {
                gc.builder()
                    .build_or(lhs, rhs, "or@bitwise_operation_function")
            }
            BitOperationType::And => {
                gc.builder()
                    .build_and(lhs, rhs, "and@bitwise_operation_function")
            }
        };

        // Return result.
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                ty.clone(),
                &vec![],
                None,
                gc,
                Some("alloca@bitwise_operation_function"),
            )
        };
        obj.store_field_nocap(gc, 0, val);
        obj
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
                    lhs_name: LHS_NAME.to_string(),
                    rhs_name: RHS_NAME.to_string(),
                    op_type,
                }),
                vec![FullName::local(LHS_NAME), FullName::local(RHS_NAME)],
                format!("bit_{}({},{})", op_type.to_string(), LHS_NAME, RHS_NAME),
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
pub struct InlineLLVMFillArrayBody {
    size_name: FullName,
    value_name: FullName,
    array_name: String,
}

impl InlineLLVMFillArrayBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let size = gc.get_var_field(&self.size_name, 0).into_int_value();
        gc.release(gc.get_var(&self.size_name).ptr.get(gc));
        let value = gc.get_var(&self.value_name).ptr.get(gc);
        assert!(rvo.is_none()); // Array is boxed, and we don't perform rvo for boxed values.
        let array = allocate_obj(
            ty.clone(),
            &vec![],
            Some(size),
            gc,
            Some(&self.array_name.as_str()),
        );
        array.store_field_nocap(gc, ARRAY_LEN_IDX, size);
        let buf = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
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
    let free_vars = vec![size_name.clone(), value_name.clone()];
    expr_llvm(
        LLVMGenerator::FillArrayBody(InlineLLVMFillArrayBody {
            size_name,
            value_name,
            array_name: name_cloned,
        }),
        free_vars,
        name,
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
    cap_name: String,
}

impl InlineLLVMMakeEmptyArrayBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        arr_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        assert!(rvo.is_none()); // Array is boxed, and we don't perform rvo for boxed values.

        // Get capacity
        let cap = gc
            .get_var_field(&FullName::local(&self.cap_name), 0)
            .into_int_value();

        // Allocate
        let array = allocate_obj(
            arr_ty.clone(),
            &vec![],
            Some(cap),
            gc,
            Some(&format!("Array::empty({})", self.cap_name)),
        );

        // Set size to zero.
        let cap = gc.context.i64_type().const_zero();
        array.store_field_nocap(gc, ARRAY_LEN_IDX, cap);

        array
    }
}

// Make an empty array.
pub fn make_empty() -> (Arc<ExprNode>, Arc<Scheme>) {
    const CAP_NAME: &str = "cap";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar);

    let expr = expr_abs(
        vec![var_local(CAP_NAME)],
        expr_llvm(
            LLVMGenerator::MakeEmptyArrayBody(InlineLLVMMakeEmptyArrayBody {
                cap_name: CAP_NAME.to_string(),
            }),
            vec![FullName::local(CAP_NAME)],
            format!("make_empty({})", CAP_NAME),
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
    arr_name: String,
    idx_name: String,
    value_name: String,
}

impl InlineLLVMArrayUnsafeSetBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        assert!(rvo.is_none()); // Array is boxed, and we don't perform rvo for boxed values.

        // Get argments
        let array = gc.get_var(&FullName::local(&self.arr_name)).ptr.get(gc);
        let idx = gc
            .get_var_field(&FullName::local(&self.idx_name), 0)
            .into_int_value();
        let value = gc.get_var(&FullName::local(&self.value_name)).ptr.get(gc);

        // Get array cap and buffer.
        let array_buf = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);

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
                        arr_name: ARR_NAME.to_string(),
                        idx_name: IDX_NAME.to_string(),
                        value_name: VALUE_NAME.to_string(),
                    }),
                    vec![
                        FullName::local(IDX_NAME),
                        FullName::local(VALUE_NAME),
                        FullName::local(ARR_NAME),
                    ],
                    format!("{}.unsafe_set({}, {})", ARR_NAME, IDX_NAME, VALUE_NAME),
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
    arr_name: String,
    idx_name: String,
}

impl InlineLLVMArrayUnsafeGetBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get argments
        let arr_name = FullName::local(&self.arr_name);
        let array = gc.get_var(&arr_name).ptr.get(gc);
        let idx = gc
            .get_var_field(&FullName::local(&self.idx_name), 0)
            .into_int_value();

        // Get array buffer
        let buf = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);

        // Get element
        let elem =
            ObjectFieldType::read_from_array_buf_noretain(gc, None, buf, ty.clone(), idx, rvo);

        // Release the array.
        if !borrowed_vars.contains(&arr_name) {
            gc.release(array);
        }

        elem
    }

    pub fn released_vars(&self) -> Vec<FullName> {
        vec![FullName::local(&self.arr_name)]
    }
}

// Gets a value from an array, without bounds checking and retaining the returned value.
pub fn unsafe_get_array() -> (Arc<ExprNode>, Arc<Scheme>) {
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
                    arr_name: ARR_NAME.to_string(),
                    idx_name: IDX_NAME.to_string(),
                }),
                vec![FullName::local(IDX_NAME), FullName::local(ARR_NAME)],
                format!("{}.get_array_noretain({})", ARR_NAME, IDX_NAME),
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
pub struct InlineLLVMArrayUnsafeSetSizeBody {
    arr_name: String,
    len_name: String,
}

impl InlineLLVMArrayUnsafeSetSizeBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        assert!(rvo.is_none()); // Array is boxed, and we don't perform rvo for boxed values.

        // Get argments
        let array = gc.get_var(&FullName::local(&self.arr_name)).ptr.get(gc);
        let length = gc
            .get_var_field(&FullName::local(&self.len_name), 0)
            .into_int_value();

        // Get pointer to length field.
        let ptr_to_length = array.ptr_to_field_nocap(gc, ARRAY_LEN_IDX);

        // Perform write and return.
        gc.builder().build_store(ptr_to_length, length);
        array
    }
}

// Set the length of an array, with no uniqueness checking, no validation of size argument.
pub fn unsafe_set_size_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "array";
    const LENGTH_NAME: &str = "length";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(LENGTH_NAME)],
        expr_abs(
            vec![var_local(ARR_NAME)],
            expr_llvm(
                LLVMGenerator::ArrayUnsafeSetSizeBody(InlineLLVMArrayUnsafeSetSizeBody {
                    arr_name: ARR_NAME.to_string(),
                    len_name: LENGTH_NAME.to_string(),
                }),
                vec![FullName::local(LENGTH_NAME), FullName::local(ARR_NAME)],
                format!("{}.unsafe_set_length({})", ARR_NAME, LENGTH_NAME),
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
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        let array = gc.get_var(&self.arr_name).ptr.get(gc);
        let len = array.load_field_nocap(gc, ARRAY_LEN_IDX).into_int_value();
        let buf = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
        let idx = gc.get_var_field(&self.idx_name, 0).into_int_value();
        gc.release(gc.get_var(&self.idx_name).ptr.get(gc));
        let elem = ObjectFieldType::read_from_array_buf(gc, Some(len), buf, ty.clone(), idx, rvo);
        if !borrowed_vars.contains(&self.arr_name) {
            gc.release(array);
        }
        elem
    }

    pub fn released_vars(&self) -> Vec<FullName> {
        vec![self.arr_name.clone()]
    }
}

// Implementation of Array::get built-in function.
fn read_array_body(a: &str, array: &str, idx: &str) -> Arc<ExprNode> {
    let elem_ty = type_tyvar_star(a);
    let array_str = FullName::local(array);
    let idx_str = FullName::local(idx);
    let name = format!("Array::@({}, {})", idx, array);
    let free_vars = vec![array_str.clone(), idx_str.clone()];
    expr_llvm(
        LLVMGenerator::ArrayGetBody(InlineLLVMArrayGetBody {
            arr_name: array_str.clone(),
            idx_name: idx_str.clone(),
        }),
        free_vars,
        name,
        elem_ty,
        None,
    )
}

// "Array::get : Array a -> I64 -> a" built-in function.
pub fn read_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        vec![var_local("idx")],
        expr_abs(
            vec![var_local("array")],
            read_array_body("a", "array", "idx"),
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
// If it is shared, clone the object or panics if panic_if_shared is true.
fn make_array_unique<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    array: Object<'c>,
    panic_if_shared: bool,
) -> Object<'c> {
    let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
    let arr_ptr = array.ptr(gc);
    let current_bb = gc.builder().get_insert_block().unwrap();
    let current_func = current_bb.get_parent().unwrap();

    // Branch by whether the array is unique or not.
    let (unique_bb, shared_bb) = gc.build_branch_by_is_unique(arr_ptr);
    let end_bb = gc.context.append_basic_block(current_func, "end_bb");

    // Implement shared_bb.
    gc.builder().position_at_end(shared_bb);
    // Create new array and clone array field.
    if panic_if_shared {
        // In case of unique version, panic in this case.
        gc.panic("An array is asserted as unique but is shared!\n");
    }
    // Allocate cloned array.
    let array_cap = array.load_field_nocap(gc, ARRAY_CAP_IDX).into_int_value();
    let cloned_array = allocate_obj(
        array.ty.clone(),
        &vec![],
        Some(array_cap),
        gc,
        Some("cloned_array_for_uniqueness"),
    );
    // Set the length of the cloned array.
    let array_len = array.load_field_nocap(gc, ARRAY_LEN_IDX).into_int_value();
    cloned_array.store_field_nocap(gc, ARRAY_LEN_IDX, array_len);
    // Copy elements to the cloned array.
    let cloned_array_buf = cloned_array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
    let array_buf = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
    ObjectFieldType::clone_array_buf(gc, array_len, array_buf, cloned_array_buf, elem_ty);
    gc.release(array.clone()); // Given array should be released here.

    // Jump to the end_bb.
    let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
    let cloned_array_ptr = cloned_array.ptr(gc);
    gc.builder().build_unconditional_branch(end_bb);

    // Implement unique_bb
    gc.builder().position_at_end(unique_bb);
    // Jump to end_bb.
    gc.builder().build_unconditional_branch(end_bb);

    // Implement end_bb.
    gc.builder().position_at_end(end_bb);
    // Build phi value of array_ptr.
    let array_phi = gc.builder().build_phi(arr_ptr.get_type(), "array_phi");
    array_phi.add_incoming(&[
        (&arr_ptr, unique_bb),
        (&cloned_array_ptr, succ_of_shared_bb),
    ]);
    let array = Object::new(
        array_phi.as_basic_value().into_pointer_value(),
        array.ty.clone(),
    );

    array
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArraySetBody {
    array_name: FullName,
    idx_name: FullName,
    value_name: FullName,
    is_unique_version: bool,
}

impl InlineLLVMArraySetBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        assert!(rvo.is_none());

        // Get argments
        let array = gc.get_var(&self.array_name).ptr.get(gc);
        let idx = gc.get_var_field(&self.idx_name, 0).into_int_value();
        gc.release(gc.get_var(&self.idx_name).ptr.get(gc));
        let value = gc.get_var(&self.value_name).ptr.get(gc);

        // Force array to be unique
        let array = make_array_unique(gc, array, self.is_unique_version);

        // Perform write and return.
        let array_len = array.load_field_nocap(gc, ARRAY_LEN_IDX).into_int_value();
        let array_buf = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
        ObjectFieldType::write_to_array_buf(gc, Some(array_len), array_buf, idx, value, true);
        array
    }
}

// Implementation of Array::set/Array::set! built-in function.
// is_unique_mode - if true, generate code that calls abort when given array is shared.
fn set_array_body(
    a: &str,
    array: &str,
    idx: &str,
    value: &str,
    is_unique_version: bool,
) -> Arc<ExprNode> {
    let elem_ty = type_tyvar_star(a);
    let array_str = FullName::local(array);
    let idx_str = FullName::local(idx);
    let value_str = FullName::local(value);
    let func_name = String::from({
        if is_unique_version {
            "set!"
        } else {
            "set"
        }
    });
    let name = format!("{} {} {} {}", func_name, idx, value, array);
    let free_vars = vec![array_str.clone(), idx_str.clone(), value_str.clone()];
    expr_llvm(
        LLVMGenerator::ArraySetBody(InlineLLVMArraySetBody {
            array_name: array_str,
            idx_name: idx_str,
            value_name: value_str,
            is_unique_version,
        }),
        free_vars,
        name,
        type_tyapp(make_array_ty(), elem_ty),
        None,
    )
}

// Array::set built-in function.
pub fn set_array_common(is_unique_version: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        vec![var_local("idx")],
        expr_abs(
            vec![var_local("value")],
            expr_abs(
                vec![var_local("array")],
                set_array_body("a", "array", "idx", "value", is_unique_version),
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

// set built-in function.
pub fn write_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    set_array_common(false)
}

// set! built-in function.
pub fn write_array_unique() -> (Arc<ExprNode>, Arc<Scheme>) {
    set_array_common(true)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayModBody {
    array_name: String,
    idx_name: String,
    modifier_name: String,
    is_unique_version: bool,
}

impl InlineLLVMArrayModBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        assert!(rvo.is_none());

        // Get argments
        let array = gc.get_var(&FullName::local(&self.array_name)).ptr.get(gc);
        let idx = gc
            .get_var_field(&FullName::local(&self.idx_name), 0)
            .into_int_value();
        let modifier = gc
            .get_var(&FullName::local(&self.modifier_name))
            .ptr
            .get(gc);

        // Make array unique
        let array = make_array_unique(gc, array, self.is_unique_version);

        // Get old element without retain.
        let array_len = array.load_field_nocap(gc, ARRAY_LEN_IDX).into_int_value();
        let array_buf = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
        let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
        let elem = ObjectFieldType::read_from_array_buf_noretain(
            gc,
            Some(array_len),
            array_buf,
            elem_ty,
            idx,
            None,
        );

        // Apply modifier to get a new value.
        let elem = gc.apply_lambda(modifier, vec![elem], None);

        // Perform write and return.
        ObjectFieldType::write_to_array_buf(gc, None, array_buf, idx, elem, false);
        array
    }
}

pub fn mod_array(is_unique_version: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    const MODIFIED_ARRAY_NAME: &str = "arr";
    const MODIFIER_NAME: &str = "f";
    const INDEX_NAME: &str = "idx";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(INDEX_NAME)],
        expr_abs(
            vec![var_local(MODIFIER_NAME)],
            expr_abs(
                vec![var_local(MODIFIED_ARRAY_NAME)],
                expr_llvm(
                    LLVMGenerator::ArrayModBody(InlineLLVMArrayModBody {
                        array_name: MODIFIED_ARRAY_NAME.to_string(),
                        idx_name: INDEX_NAME.to_string(),
                        modifier_name: MODIFIER_NAME.to_string(),
                        is_unique_version,
                    }),
                    vec![
                        FullName::local(INDEX_NAME),
                        FullName::local(MODIFIER_NAME),
                        FullName::local(MODIFIED_ARRAY_NAME),
                    ],
                    format!(
                        "{}.mod{}({}, {})",
                        MODIFIED_ARRAY_NAME,
                        if is_unique_version { "!" } else { "" },
                        INDEX_NAME,
                        MODIFIER_NAME
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
            type_fun(
                type_fun(elem_tyvar.clone(), elem_tyvar),
                type_fun(array_ty.clone(), array_ty),
            ),
        ),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMArrayForceUniqueBody {
    arr_name: String,
    is_unique_version: bool,
}

impl InlineLLVMArrayForceUniqueBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        assert!(rvo.is_none());

        // Get argments
        let array = gc.get_var(&FullName::local(&self.arr_name)).ptr.get(gc);

        // Make array unique
        let array = make_array_unique(gc, array, self.is_unique_version);

        array
    }
}

pub fn force_unique_array(is_unique_version: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";
    const ELEM_TYPE: &str = "a";

    let elem_tyvar = type_tyvar_star(ELEM_TYPE);
    let array_ty = type_tyapp(make_array_ty(), elem_tyvar.clone());

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            LLVMGenerator::ArrayForceUniqueBody(InlineLLVMArrayForceUniqueBody {
                arr_name: ARR_NAME.to_string(),
                is_unique_version,
            }),
            vec![FullName::local(ARR_NAME)],
            format!(
                "{}.force_unique{}",
                ARR_NAME,
                if is_unique_version { "!" } else { "" },
            ),
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
    arr_name: String,
}

impl InlineLLVMArrayGetPtrBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get argment
        let arr_name = FullName::local(&self.arr_name);
        let array = gc.get_var(&arr_name).ptr.get(gc);

        // Get pointer
        let ptr = array.ptr_to_field_nocap(gc, ARRAY_BUF_IDX);
        let ptr_ty = ObjectFieldType::Ptr
            .to_basic_type(gc, vec![])
            .into_pointer_type();
        let ptr = gc.cast_pointer(ptr, ptr_ty);

        // Release array
        if !borrowed_vars.contains(&arr_name) {
            gc.release(array);
        }

        // Make returned object
        let obj = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                make_ptr_ty(),
                &vec![],
                None,
                gc,
                Some("alloca@get_ptr_array"),
            )
        };
        obj.store_field_nocap(gc, 0, ptr);

        obj
    }

    pub fn released_vars(&self) -> Vec<FullName> {
        vec![FullName::local(&self.arr_name)]
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
                arr_name: ARR_NAME.to_string(),
            }),
            vec![FullName::local(ARR_NAME)],
            format!("{}.get_ptr", ARR_NAME,),
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
    arr_name: String,
}

impl InlineLLVMArrayGetSizeBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let arr_name = FullName::local(&self.arr_name);
        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        let array_obj = gc.get_var(&arr_name).ptr.get(gc);
        let len = array_obj
            .load_field_nocap(gc, ARRAY_LEN_IDX)
            .into_int_value();
        if !borrowed_vars.contains(&arr_name) {
            gc.release(array_obj);
        }
        let int_obj = if rvo.is_none() {
            allocate_obj(make_i64_ty(), &vec![], None, gc, Some("length_of_arr"))
        } else {
            rvo.unwrap()
        };
        int_obj.store_field_nocap(gc, 0, len);
        int_obj
    }

    pub fn released_vars(&self) -> Vec<FullName> {
        vec![FullName::local(&self.arr_name)]
    }
}

// `get_size` built-in function for Array.
pub fn get_size_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            LLVMGenerator::ArrayGetSizeBody(InlineLLVMArrayGetSizeBody {
                arr_name: ARR_NAME.to_string(),
            }),
            vec![FullName::local(ARR_NAME)],
            "len arr".to_string(),
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
    arr_name: String,
}

impl InlineLLVMArrayGetCapacityBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let arr_name = FullName::local(&self.arr_name);

        // Array = [ControlBlock, Size, [Capacity, Element0, ...]]
        let array_obj = gc.get_var(&arr_name).ptr.get(gc);
        let len = array_obj
            .load_field_nocap(gc, ARRAY_CAP_IDX)
            .into_int_value();

        if !borrowed_vars.contains(&arr_name) {
            gc.release(array_obj);
        }

        let int_obj = if rvo.is_none() {
            allocate_obj(make_i64_ty(), &vec![], None, gc, Some("cap_of_arr"))
        } else {
            rvo.unwrap()
        };
        int_obj.store_field_nocap(gc, 0, len);
        int_obj
    }

    pub fn released_vars(&self) -> Vec<FullName> {
        vec![FullName::local(&self.arr_name)]
    }
}

// `Array::get_capacity : Array a -> I64` built-in function.
pub fn get_capacity_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";

    let expr = expr_abs(
        vec![var_local(ARR_NAME)],
        expr_llvm(
            LLVMGenerator::ArrayGetCapacityBody(InlineLLVMArrayGetCapacityBody {
                arr_name: ARR_NAME.to_string(),
            }),
            vec![FullName::local(ARR_NAME)],
            "arr.get_capacity".to_string(),
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
    var_name: FullName,
    field_idx: usize,
}

impl InlineLLVMStructGetBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get struct object.
        let str = gc.get_var(&self.var_name).ptr.get(gc);
        ObjectFieldType::get_struct_fields(gc, &str, vec![(self.field_idx as u32, rvo)])[0].clone()
    }
}

// `get` built-in function for a given struct.
pub fn struct_get_body(
    var_name: &str,
    field_idx: usize,
    field_ty: Arc<TypeNode>,
    struct_name: &FullName,
    field_name: &str,
) -> Arc<ExprNode> {
    let var_name_clone = FullName::local(var_name);
    let free_vars = vec![FullName::local(var_name)];
    let name = format!(
        "{}.get_{}({})",
        struct_name.to_string(),
        field_name,
        var_name
    );
    expr_llvm(
        LLVMGenerator::StructGetBody(InlineLLVMStructGetBody {
            var_name: var_name_clone,
            field_idx,
        }),
        free_vars,
        name,
        field_ty,
        None,
    )
}

// field getter function for a given struct.
pub fn struct_get(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition.get_field_by_name(field_name);
    if field.is_none() {
        error_exit(&format!(
            "No field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (field_idx, field) = field.unwrap();

    let str_ty = definition.ty();
    const VAR_NAME: &str = "str_obj";
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        struct_get_body(
            VAR_NAME,
            field_idx as usize,
            field.ty.clone(),
            struct_name,
            field_name,
        ),
        None,
    );
    let ty = type_fun(str_ty, field.ty.clone());
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructPunchBody {
    var_name: FullName,
    field_idx: usize,
}

impl InlineLLVMStructPunchBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get the field type `F` and the punched struct type `PS` using `ty == (F, PS)`.
        // let field_type = ret_ty.field_types(gc.type_env())[0].clone();
        // let punched_type = ret_ty.field_types(gc.type_env())[1].clone();

        // Get the argument object (the struct value).
        let str = gc.get_var(&self.var_name).ptr.get(gc);

        // Move out struct field value without releaseing the struct itself.
        let field = ObjectFieldType::get_struct_field_noclone(gc, &str, self.field_idx as u32);

        // Create the return value.
        let pair = if rvo.is_none() {
            allocate_obj(ret_ty.clone(), &vec![], None, gc, Some("ret_of_punch"))
        } else {
            rvo.unwrap()
        };
        let field_val = field.value(gc);
        pair.store_field_nocap(gc, 0, field_val);
        let str_val = str.value(gc);
        pair.store_field_nocap(gc, 1, str_val);

        pair
    }
}

// Field punching function for a given struct.
// If the struct is `S` and the field is `F`, then the function has the type `S -> (F, PS)` where `PS` is the punched struct type.
pub fn struct_punch(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition.get_field_by_name(field_name);
    if field.is_none() {
        error_exit(&format!(
            "No field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (field_idx, field) = field.unwrap();

    let str_ty = definition.ty();
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
            }),
            vec![FullName::local(VAR_NAME)],
            format!(
                "{}::{}{}({})",
                struct_name.to_string(),
                STRUCT_PUNCH_SYMBOL,
                field_name,
                VAR_NAME
            ),
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
    field_name: FullName,
    field_idx: usize,
}

impl InlineLLVMStructPlugInBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        struct_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        assert!(struct_ty.is_box(gc.type_env()));
        assert!(rvo.is_none());

        // Get the first argument, a punched struct value, and the second argument, a field value.
        let punched_str = gc.get_var(&self.punched_str_name).ptr.get(gc);
        let field = gc.get_var(&self.field_name).ptr.get(gc);

        // Cast punched_str into the struct type.
        let str = Object::create_from_value(punched_str.value(gc), struct_ty.clone(), gc);

        // Move the field value into the struct value.
        ObjectFieldType::set_struct_field_norelease(gc, &str, self.field_idx as u32, &field);

        str
    }
}

// Field plugging-in function for a given struct.
// If the struct is `S` and the field is `F`, then the function has the type `PS -> F -> S` where `PS` is the punched struct type.
pub fn struct_plug_in(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition.get_field_by_name(field_name);
    if field.is_none() {
        error_exit(&format!(
            "No field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (field_idx, field) = field.unwrap();

    let str_ty = definition.ty();
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
                }),
                vec![
                    FullName::local(PUNCHED_STR_NAME),
                    FullName::local(FIELD_NAME),
                ],
                format!(
                    "{}::{}{}({}, {})",
                    struct_name.to_string(),
                    STRUCT_PLUG_IN_SYMBOL,
                    field_name,
                    PUNCHED_STR_NAME,
                    FIELD_NAME
                ),
                str_ty.clone(),
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructModBody {
    f_name: FullName,
    x_name: FullName,
    is_unique_version: bool,
    field_idx: usize,
    field_count: usize,
}

impl InlineLLVMStructModBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let is_unbox = ty.is_unbox(gc.type_env());

        // Get arguments
        let modfier = gc.get_var(&self.f_name).ptr.get(gc);
        let str = gc.get_var(&self.x_name).ptr.get(gc);

        let mut str = make_struct_unique(gc, str, self.field_count as u32, self.is_unique_version);

        // Modify field
        let field = ObjectFieldType::get_struct_field_noclone(gc, &str, self.field_idx as u32);
        let field = gc.apply_lambda(modfier, vec![field], None);
        ObjectFieldType::set_struct_field_norelease(gc, &str, self.field_idx as u32, &field);

        if rvo.is_some() {
            assert!(is_unbox);
            // Move str to rvo.
            let rvo = rvo.unwrap();
            let str_val = str.load_nocap(gc);
            rvo.store_unbox(gc, str_val);
            str = rvo;
        }

        str
    }
}

// `mod` built-in function for a given struct.
pub fn struct_mod_body(
    f_name: &str,
    x_name: &str,
    field_count: usize, // number of fields in this struct
    field_idx: usize,
    struct_name: &FullName,
    struct_defn: &TypeDefn,
    field_name: &str,
    is_unique_version: bool,
) -> Arc<ExprNode> {
    let name = format!(
        "{}.mod_{}{}({}, {})",
        struct_name.to_string(),
        field_name,
        if is_unique_version { "!" } else { "" },
        f_name,
        x_name
    );
    let f_name = FullName::local(f_name);
    let x_name = FullName::local(x_name);
    let free_vars = vec![f_name.clone(), x_name.clone()];
    expr_llvm(
        LLVMGenerator::StructModBody(InlineLLVMStructModBody {
            f_name,
            x_name,
            is_unique_version,
            field_idx,
            field_count,
        }),
        free_vars,
        name,
        struct_defn.ty(),
        None,
    )
}

// `mod` built-in function for a given struct.
pub fn struct_mod(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
    is_unique_version: bool,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition.get_field_by_name(field_name);
    if field.is_none() {
        error_exit(&format!(
            "Error: no field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (field_idx, field) = field.unwrap();

    let field_count = definition.fields().len();
    let str_ty = definition.ty();
    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(
            vec![var_local("x")],
            struct_mod_body(
                "f",
                "x",
                field_count,
                field_idx as usize,
                struct_name,
                definition,
                field_name,
                is_unique_version,
            ),
            None,
        ),
        None,
    );
    let ty = type_fun(
        type_fun(field.ty.clone(), field.ty.clone()),
        type_fun(str_ty.clone(), str_ty.clone()),
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// Field act function for a given struct.
// If the struct is `S` and the field is `F`, then the function has the type `(F -> f F) -> S -> f S`.
// The implementation uses `#punch_{field}` and `#plug_in_{field}`.for the struct.
pub fn struct_act(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index and the `Field` instance of `field_name` in the given struct.
    let field = definition.get_field_by_name(field_name);
    if field.is_none() {
        error_exit(&format!(
            "No field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (_field_idx, field) = field.unwrap();

    // Create type scheme of this function.
    let str_ty = definition.ty();
    let field_ty = field.ty.clone();
    let functor_ty = type_tyvar("f", &kind_arrow(kind_star(), kind_star()));
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
            ty.clone(),
        )],
        vec![],
        ty,
    );

    // Implementation of `act` function as AST.
    // The implementation as Fix source code is:
    // ```
    // |f, s| (
    //     let (x, ps) = s.#punch_{field};
    //     f(x).map(ps.#plug_in_{field})
    // );
    // (Here, we cannot use the parser because we are using "#" is not allowed as value name)
    let expr = expr_abs(
        vec![var_local("f")],
        expr_abs(
            vec![var_local("s")],
            expr_let(
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
                        vec![expr_app(
                            expr_var(
                                FullName::local(&format!(
                                    "{}{}",
                                    STRUCT_PLUG_IN_SYMBOL, field_name
                                )),
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
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

// Make struct object to unique.
// If it is (unboxed or) unique, do nothing.
// If it is shared, clone the object or panics if panic_if_shared is true.
fn make_struct_unique<'c, 'm>(
    gc: &mut GenerationContext<'c, 'm>,
    mut str: Object<'c>,
    field_count: u32,
    panic_if_shared: bool,
) -> Object<'c> {
    let is_unbox = str.ty.is_unbox(gc.type_env());
    if !is_unbox {
        // In boxed case, `str` should be replaced to cloned object if it is shared.

        // Branch by refcnt is one.
        let str_ptr = str.ptr(gc);
        let (unique_bb, shared_bb) = gc.build_branch_by_is_unique(str_ptr);
        let end_bb = gc
            .context
            .append_basic_block(unique_bb.get_parent().unwrap(), "end_bb");

        // Implement shared_bb.
        gc.builder().position_at_end(shared_bb);
        if panic_if_shared {
            // In case of unique version, panic in this case.
            gc.panic("A struct object is asserted as unique but is shared!\n");
        }
        // Create new struct and clone fields.
        let cloned_str = allocate_obj(str.ty.clone(), &vec![], None, gc, Some("cloned_str"));
        for i in 0..field_count {
            // Retain field.
            let field = ObjectFieldType::get_struct_field_noclone(gc, &str, i as u32);
            gc.retain(field.clone());
            // Clone field.
            ObjectFieldType::set_struct_field_norelease(gc, &cloned_str, i as u32, &field);
        }
        gc.release(str.clone());
        let cloned_str_ptr = cloned_str.ptr(gc);
        let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(end_bb);

        // Implement unique_bb.
        gc.builder().position_at_end(unique_bb);
        // Jump to end_bb.
        gc.builder().build_unconditional_branch(end_bb);

        // Implement end_bb.
        gc.builder().position_at_end(end_bb);
        // Build phi value.
        let str_phi = gc.builder().build_phi(str.ptr(gc).get_type(), "str_phi");
        str_phi.add_incoming(&[(&str_ptr, unique_bb), (&cloned_str_ptr, succ_of_shared_bb)]);

        str = Object::new(
            str_phi.as_basic_value().into_pointer_value(),
            str.ty.clone(),
        );
    }
    // In unboxed case, str is always treated as unique object.
    str
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMStructSetBody {
    value_name: String,
    struct_name: String,
    field_count: u32,
    is_unique_version: bool,
    field_idx: u32,
}

impl InlineLLVMStructSetBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        str_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get arguments
        let value = gc.get_var(&FullName::local(&self.value_name)).ptr.get(gc);
        let str = gc.get_var(&FullName::local(&self.struct_name)).ptr.get(gc);

        // Make struct object unique.
        let mut str = make_struct_unique(gc, str, self.field_count, self.is_unique_version);

        // Release old value
        let old_value = ObjectFieldType::get_struct_field_noclone(gc, &str, self.field_idx as u32);
        gc.release(old_value);

        // Set new value
        ObjectFieldType::set_struct_field_norelease(gc, &str, self.field_idx as u32, &value);

        // If rvo, store the result to the rvo.
        if rvo.is_some() {
            assert!(str_ty.is_unbox(gc.type_env()));
            // Move str to rvo.
            let rvo = rvo.unwrap();
            let str_val = str.load_nocap(gc);
            rvo.store_unbox(gc, str_val);
            str = rvo;
        }

        str
    }
}

// `set` built-in function for a given struct.
pub fn struct_set(
    struct_name: &FullName,
    definition: &TypeDefn,
    field_name: &str,
    is_unique_version: bool,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const VALUE_NAME: &str = "val";
    const STRUCT_NAME: &str = "str";

    // Find the index of `field_name` in the given struct.
    let field = definition.get_field_by_name(field_name);
    if field.is_none() {
        error_exit(&format!(
            "No field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (field_idx, field) = field.unwrap();
    let field_count = definition.fields().len() as u32;

    let str_ty = definition.ty();
    let expr = expr_abs(
        vec![var_local(VALUE_NAME)],
        expr_abs(
            vec![var_local(STRUCT_NAME)],
            expr_llvm(
                LLVMGenerator::StructSetBody(InlineLLVMStructSetBody {
                    value_name: VALUE_NAME.to_string(),
                    struct_name: STRUCT_NAME.to_string(),
                    field_count,
                    is_unique_version,
                    field_idx,
                }),
                vec![FullName::local(VALUE_NAME), FullName::local(STRUCT_NAME)],
                format!(
                    "{}.{}{}{}({})",
                    STRUCT_NAME,
                    STRUCT_SETTER_SYMBOL,
                    field_name,
                    if is_unique_version { "!" } else { "" },
                    VALUE_NAME
                ),
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
    field_name: String,
    generated_union_name: String,
    field_idx: usize,
}

impl InlineLLVMMakeUnionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let is_unbox = ty.is_unbox(gc.type_env());
        let offset: u32 = if is_unbox { 0 } else { 1 };

        // Get field values.
        let field = gc.get_var(&FullName::local(&self.field_name)).ptr.get(gc);

        // Create union object.
        let obj = if rvo.is_none() {
            allocate_obj(
                ty.clone(),
                &vec![],
                None,
                gc,
                Some(&self.generated_union_name),
            )
        } else {
            rvo.unwrap()
        };

        // Set tag value.
        let tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);
        obj.store_field_nocap(gc, 0 + offset, tag_value);

        // Set value.
        let buf = obj.ptr_to_field_nocap(gc, offset + 1);
        ObjectFieldType::set_value_to_union_buf(gc, buf, field);

        obj
    }
}

// constructor function for a given union.
pub fn union_new_body(
    union_name: &FullName,
    union_defn: &TypeDefn,
    field_name: &Name,
    field_idx: usize,
) -> Arc<ExprNode> {
    let free_vars = vec![FullName::local(field_name)];
    let name = format!("{}.new_{}", union_name.to_string(), field_name);
    let name_cloned = name.clone();
    let field_name_cloned = field_name.clone();
    expr_llvm(
        LLVMGenerator::MakeUnionBody(InlineLLVMMakeUnionBody {
            field_name: field_name_cloned,
            generated_union_name: name_cloned,
            field_idx,
        }),
        free_vars,
        name,
        union_defn.ty(),
        None,
    )
}

// `new_{field}` built-in function for a given union.
pub fn union_new(
    union_name: &FullName,
    field_name: &Name,
    union: &TypeDefn,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Get field index.
    let mut field_idx = 0;
    for field in union.fields() {
        if *field_name == field.name {
            break;
        }
        field_idx += 1;
    }
    if field_idx == union.fields().len() {
        error_exit(&format!(
            "Unknown field `{}` for union `{}`",
            field_name,
            union_name.to_string()
        ));
    }
    let expr = expr_abs(
        vec![var_local(field_name)],
        union_new_body(union_name, union, field_name, field_idx),
        None,
    );
    let union_ty = union.ty();
    let field_ty = union.fields()[field_idx].ty.clone();
    let ty = type_fun(field_ty, union_ty);
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// `as_{field}` built-in function for a given union.
pub fn union_as(
    union_name: &FullName,
    field_name: &Name,
    union: &TypeDefn,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Get field index.
    let mut field_idx = 0;
    for field in union.fields() {
        if *field_name == field.name {
            break;
        }
        field_idx += 1;
    }
    if field_idx == union.fields().len() {
        error_exit(&format!(
            "Unknown field `{}` for union `{}`",
            field_name,
            union_name.to_string()
        ));
    }
    let union_arg_name = "union".to_string();
    let expr = expr_abs(
        vec![var_local(&union_arg_name)],
        union_as_body(
            union_name,
            &union_arg_name,
            field_name,
            field_idx,
            union.fields()[field_idx].ty.clone(),
        ),
        None,
    );
    let union_ty = union.ty();
    let field_ty = union.fields()[field_idx].ty.clone();
    let ty = type_fun(union_ty, field_ty);
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnionAsBody {
    union_arg_name: String,
    field_idx: usize,
}

impl InlineLLVMUnionAsBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get union object.
        let obj = gc
            .get_var(&FullName::local(&self.union_arg_name))
            .ptr
            .get(gc);

        let elem_ty = ty.clone();

        // Create specified tag value.
        let specified_tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);

        // If tag unmatch, panic.
        ObjectFieldType::panic_if_union_tag_unmatch(gc, obj.clone(), specified_tag_value);

        // If tag match, return the field value.
        ObjectFieldType::get_union_field(gc, obj, &elem_ty, rvo)
    }
}

// `as_{field}` built-in function for a given union.
pub fn union_as_body(
    union_name: &FullName,
    union_arg_name: &Name,
    field_name: &Name,
    field_idx: usize,
    field_ty: Arc<TypeNode>,
) -> Arc<ExprNode> {
    let name = format!("{}.as_{}", union_name.to_string(), field_name);
    let free_vars = vec![FullName::local(union_arg_name)];
    let union_arg_name = union_arg_name.clone();
    expr_llvm(
        LLVMGenerator::UnionAsBody(InlineLLVMUnionAsBody {
            union_arg_name,
            field_idx,
        }),
        free_vars,
        name,
        field_ty,
        None,
    )
}

// `is_{field}` built-in function for a given union.
pub fn union_is(
    union_name: &FullName,
    field_name: &Name,
    union: &TypeDefn,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Get field index.
    let mut field_idx = 0;
    for field in union.fields() {
        if *field_name == field.name {
            break;
        }
        field_idx += 1;
    }
    if field_idx == union.fields().len() {
        error_exit(&format!(
            "Unknown field `{}` for union `{}`",
            field_name,
            union_name.to_string()
        ));
    }
    let union_arg_name = "union".to_string();
    let expr = expr_abs(
        vec![var_local(&union_arg_name)],
        union_is_body(union_name, &union_arg_name, field_name, field_idx),
        None,
    );
    let union_ty = union.ty();
    let ty = type_fun(union_ty, make_bool_ty());
    let mut tvs = vec![];
    ty.free_vars_to_vec(&mut tvs);
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnionIsBody {
    union_arg_name: String,
    field_idx: usize,
    name_cloned: String,
}

impl InlineLLVMUnionIsBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get union object.
        let obj = gc
            .get_var(&FullName::local(&self.union_arg_name))
            .ptr
            .get(gc);

        let is_unbox = obj.is_unbox(gc.type_env());
        let offset = if is_unbox { 0 } else { 1 };

        // Create specified tag value.
        let specified_tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);

        // Get tag value.
        let tag_value = obj.load_field_nocap(gc, 0 + offset).into_int_value();

        // Create returned value.
        let ret = if rvo.is_none() {
            allocate_obj(make_bool_ty(), &vec![], None, gc, Some(&self.name_cloned))
        } else {
            rvo.unwrap()
        };

        // Branch and store result to ret_ptr.
        let is_tag_match = gc.builder().build_int_compare(
            IntPredicate::EQ,
            specified_tag_value,
            tag_value,
            "is_tag_match",
        );
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let match_bb = gc.context.append_basic_block(current_func, "match_bb");
        let unmatch_bb = gc.context.append_basic_block(current_func, "unmatch_bb");
        let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");
        gc.builder()
            .build_conditional_branch(is_tag_match, match_bb, unmatch_bb);

        gc.builder().position_at_end(match_bb);
        let value = gc.context.i8_type().const_int(1 as u64, false);
        ret.store_field_nocap(gc, 0, value);
        gc.builder().build_unconditional_branch(cont_bb);

        gc.builder().position_at_end(unmatch_bb);
        let value = gc.context.i8_type().const_int(0 as u64, false);
        ret.store_field_nocap(gc, 0, value);
        gc.builder().build_unconditional_branch(cont_bb);

        // Return the value.
        gc.builder().position_at_end(cont_bb);
        gc.release(obj);
        ret
    }
}

// `is_{field}` built-in function for a given union.
pub fn union_is_body(
    union_name: &FullName,
    union_arg_name: &Name,
    field_name: &Name,
    field_idx: usize,
) -> Arc<ExprNode> {
    let name = format!("{}.is_{}", union_name.to_string(), field_name);
    let name_cloned = name.clone();
    let free_vars = vec![FullName::local(union_arg_name)];
    let union_arg_name = union_arg_name.clone();
    expr_llvm(
        LLVMGenerator::UnionIsBody(InlineLLVMUnionIsBody {
            union_arg_name,
            field_idx,
            name_cloned,
        }),
        free_vars,
        name,
        make_bool_ty(),
        None,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMUnionModBody {
    union_name: String,
    modifier_name: String,
    field_idx: u32,
}

impl InlineLLVMUnionModBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        union_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get arguments
        let obj = gc.get_var(&FullName::local(&self.union_name)).ptr.get(gc);
        let modifier = gc
            .get_var(&FullName::local(&self.modifier_name))
            .ptr
            .get(gc);

        let is_unbox = obj.is_unbox(gc.type_env());
        let offset = if is_unbox { 0 } else { 1 };

        // Create specified tag value.
        let specified_tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(self.field_idx as u64, false);

        // Get tag value.
        let tag_value = obj.load_field_nocap(gc, 0 + offset).into_int_value();

        // Branch and store result to ret_ptr.
        let is_tag_match = gc.builder().build_int_compare(
            IntPredicate::EQ,
            specified_tag_value,
            tag_value,
            "is_tag_match@union_mod_function",
        );
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let match_bb = gc.context.append_basic_block(current_func, "match_bb");
        let unmatch_bb = gc.context.append_basic_block(current_func, "unmatch_bb");
        let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");
        gc.builder()
            .build_conditional_branch(is_tag_match, match_bb, unmatch_bb);

        // Implement match_bb
        gc.builder().position_at_end(match_bb);
        let field_ty = union_ty.field_types(gc.type_env())[self.field_idx as usize].clone();
        let value = ObjectFieldType::get_union_field(gc, obj.clone(), &field_ty, None);
        let value = gc.apply_lambda(modifier.clone(), vec![value], None);
        // Prepare space for returned union object.
        let ret_obj = allocate_obj(
            union_ty.clone(),
            &vec![],
            None,
            gc,
            Some("alloca@union_mod_function"),
        );
        // Set values of returned union object.
        ret_obj.store_field_nocap(gc, 0 + offset, specified_tag_value);
        let buf = ret_obj.ptr_to_field_nocap(gc, offset + 1);
        ObjectFieldType::set_value_to_union_buf(gc, buf, value);
        let match_ret_obj_ptr = ret_obj.ptr(gc);
        gc.builder().build_unconditional_branch(cont_bb);

        // Implement unmatch_bb
        gc.builder().position_at_end(unmatch_bb);
        gc.release(modifier);
        let unmatch_ret_obj_ptr = obj.ptr(gc);
        gc.builder().build_unconditional_branch(cont_bb);

        // Return the value.
        gc.builder().position_at_end(cont_bb);
        let phi = gc
            .builder()
            .build_phi(match_ret_obj_ptr.get_type(), "phi@union_mod_function");
        phi.add_incoming(&[
            (&match_ret_obj_ptr, match_bb),
            (&unmatch_ret_obj_ptr, unmatch_bb),
        ]);
        let ret_obj = Object::new(phi.as_basic_value().into_pointer_value(), union_ty.clone());
        if rvo.is_some() {
            assert!(union_ty.is_unbox(gc.type_env()));
            let rvo = rvo.unwrap();
            gc.builder().build_store(rvo.ptr(gc), ret_obj.value(gc));
            rvo
        } else {
            ret_obj
        }
    }
}

pub fn union_mod_function(
    union_name: &FullName,
    field_name: &Name,
    union: &TypeDefn,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    const UNION_NAME: &str = "union_value";
    const MODIFIER_NAME: &str = "modifier";

    let field_idx = if let Some((field_idx, _)) = union.get_field_by_name(&field_name) {
        field_idx
    } else {
        error_exit(&format!(
            "Unknown field `{}` for union `{}`",
            field_name,
            union_name.to_string()
        ));
    };

    let union_ty = union.ty();
    let field_ty = union.fields()[field_idx as usize].ty.clone();

    let expr = expr_abs(
        vec![var_local(MODIFIER_NAME)],
        expr_abs(
            vec![var_local(UNION_NAME)],
            expr_llvm(
                LLVMGenerator::UnionModBody(InlineLLVMUnionModBody {
                    union_name: UNION_NAME.to_string(),
                    modifier_name: MODIFIER_NAME.to_string(),
                    field_idx,
                }),
                vec![FullName::local(MODIFIER_NAME), FullName::local(UNION_NAME)],
                format!("mod_{}({}, {})", field_name, MODIFIER_NAME, UNION_NAME),
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

pub fn loop_result_defn() -> TypeDefn {
    TypeDefn {
        name: FullName::from_strs(&[STD_NAME], LOOP_RESULT_NAME),
        tyvars: vec![
            tyvar_from_name("s", &kind_star()),
            tyvar_from_name("b", &kind_star()),
        ],
        value: TypeDeclValue::Union(Union {
            fields: vec![
                Field {
                    name: "continue".to_string(),
                    ty: type_tyvar("s", &kind_star()),
                    is_punched: false,
                },
                Field {
                    name: "break".to_string(),
                    ty: type_tyvar("b", &kind_star()),
                    is_punched: false,
                },
            ],
            is_unbox: true,
        }),
        source: None,
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMLoopFunctionBody {
    initial_state_name: String,
    loop_body_name: String,
}

impl InlineLLVMLoopFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let initial_state_name = FullName::local(&self.initial_state_name);
        let loop_body_name = FullName::local(&self.loop_body_name);

        // Prepare constant.
        let cont_tag_value = ObjectFieldType::UnionTag
            .to_basic_type(gc, vec![])
            .into_int_type()
            .const_int(LOOP_RESULT_CONTINUE_IDX as u64, false);

        // Get argments.
        let init_state = gc.get_var(&initial_state_name).ptr.get(gc);
        let loop_body = gc.get_var(&loop_body_name).ptr.get(gc);

        // Collect types.
        let loop_state_ty = init_state.ty.clone();
        let loop_res_ty = loop_body.ty.get_lambda_dst();
        assert!(loop_res_ty.is_unbox(gc.type_env()));

        // Allocate a space to store LoopResult on stack.
        let loop_res = allocate_obj(loop_res_ty, &vec![], None, gc, Some("LoopResult_in_loop"));

        // If loop_state_ty is unboxed, allocate a space to store loop state on stack to avoid alloca in loop body.
        let loop_state_buf = if loop_state_ty.is_unbox(gc.type_env()) {
            let ty = loop_state_ty.get_embedded_type(gc, &vec![]);
            Some(Object::new(
                gc.build_alloca_at_entry(ty, "loop_state_in_loop"),
                loop_state_ty.clone(),
            ))
        } else {
            None
        };

        // Store the initial loop state to loop_res.
        let buf = loop_res.ptr_to_field_nocap(gc, 1);
        ObjectFieldType::set_value_to_union_buf(gc, buf, init_state.clone());

        // Create loop body bb and jump to it.
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let loop_bb = gc.context.append_basic_block(current_func, "loop_bb");
        gc.builder().build_unconditional_branch(loop_bb);

        // Implement loop body.
        gc.builder().position_at_end(loop_bb);

        // Run loop_body on loop state.
        gc.retain(loop_body.clone());
        let loop_state =
            ObjectFieldType::get_union_field(gc, loop_res.clone(), &loop_state_ty, loop_state_buf);
        let _ = gc.apply_lambda(loop_body.clone(), vec![loop_state], Some(loop_res.clone()));

        // Branch due to loop_res.
        let tag_value = loop_res.load_field_nocap(gc, 0).into_int_value();
        let is_continue = gc.builder().build_int_compare(
            IntPredicate::EQ,
            tag_value,
            cont_tag_value,
            "is_continue",
        );
        let break_bb = gc.context.append_basic_block(current_func, "break_bb");
        gc.builder()
            .build_conditional_branch(is_continue, loop_bb, break_bb);

        // Implement break_bb.
        gc.builder().position_at_end(break_bb);
        gc.release(loop_body);
        ObjectFieldType::get_union_field(gc, loop_res, ty, rvo)
    }
}

// `loop` built-in function.
// loop : s -> (s -> LoopResult s b) -> b;
pub fn state_loop() -> (Arc<ExprNode>, Arc<Scheme>) {
    const S_NAME: &str = "s";
    const B_NAME: &str = "b";
    const INITIAL_STATE_NAME: &str = "initial_state";
    const LOOP_BODY_NAME: &str = "loop_body";
    let tyvar_s = type_tyvar(S_NAME, &kind_star());
    let tyvar_b = type_tyvar(B_NAME, &kind_star());
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(
            tyvar_s.clone(),
            type_fun(
                type_fun(
                    tyvar_s.clone(),
                    type_tyapp(type_tyapp(make_loop_result_ty(), tyvar_s), tyvar_b.clone()),
                ),
                tyvar_b,
            ),
        ),
    );

    let initial_state_name = FullName::local(INITIAL_STATE_NAME);
    let loop_body_name = FullName::local(LOOP_BODY_NAME);
    let expr = expr_abs(
        vec![var_var(initial_state_name.clone())],
        expr_abs(
            vec![var_var(loop_body_name.clone())],
            expr_llvm(
                LLVMGenerator::LoopFunctionBody(InlineLLVMLoopFunctionBody {
                    initial_state_name: INITIAL_STATE_NAME.to_string(),
                    loop_body_name: LOOP_BODY_NAME.to_string(),
                }),
                vec![initial_state_name, loop_body_name],
                format!("loop({}, {})", INITIAL_STATE_NAME, LOOP_BODY_NAME),
                type_tyvar_star(B_NAME),
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMAbortFunctionBody {}

impl InlineLLVMAbortFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ty: &Arc<TypeNode>,
        _rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Abort
        gc.call_runtime(RUNTIME_ABORT, &[]);

        // Return
        Object::new(
            ty.get_struct_type(gc, &vec![])
                .ptr_type(AddressSpace::from(0))
                .const_null(),
            ty.clone(),
        )
    }
}

// `abort` built-in function
pub fn abort_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const A_NAME: &str = "a";
    const UNIT_NAME: &str = "unit";
    let expr = expr_abs(
        vec![var_local(UNIT_NAME)],
        expr_llvm(
            LLVMGenerator::AbortFunctionBody(InlineLLVMAbortFunctionBody {}),
            vec![],
            "abort".to_string(),
            type_tyvar_star(A_NAME),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_tyapp(make_lazy_ty(), type_tyvar_star(A_NAME)),
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIsUniqueFunctionBody {
    var_name: String,
}

impl InlineLLVMIsUniqueFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let bool_ty = ObjectFieldType::I8
            .to_basic_type(gc, vec![])
            .into_int_type();

        // Get argument
        let obj = gc.get_var(&FullName::local(&self.var_name)).ptr.get(gc);

        // Prepare returned object.
        let ret = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(ret_ty.clone(), &vec![], None, gc, Some("ret@is_unique"))
        };

        // Get whether argument is unique.
        let is_unique = if obj.is_box(gc.type_env()) {
            let obj_ptr = obj.ptr(gc);
            let current_bb = gc.builder().get_insert_block().unwrap();
            let current_func = current_bb.get_parent().unwrap();

            let (unique_bb, shared_bb) = gc.build_branch_by_is_unique(obj_ptr);
            // Add continuing basic block.
            let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");

            // Implement unique_bb.
            gc.builder().position_at_end(unique_bb);
            let flag_unique_bb = bool_ty.const_int(1, false);
            // Jump to cont_bb.
            gc.builder().build_unconditional_branch(cont_bb);

            // Implement shared_bb.
            gc.builder().position_at_end(shared_bb);
            let flag_shared_bb = bool_ty.const_int(0, false);
            // Jump to cont_bb.
            gc.builder().build_unconditional_branch(cont_bb);

            // Implement cont_bb.
            gc.builder().position_at_end(cont_bb);
            let flag = gc.builder().build_phi(bool_ty, "phi@is_unique");
            flag.add_incoming(&[(&flag_unique_bb, unique_bb), (&flag_shared_bb, shared_bb)]);
            flag.as_basic_value().into_int_value()
        } else {
            // If the object is boxed, it is always unique.
            bool_ty.const_int(1, false)
        };
        let bool_val = make_bool_ty().get_struct_type(gc, &vec![]).get_undef();
        let bool_val = gc
            .builder()
            .build_insert_value(bool_val, is_unique, 0, "insert@is_unique")
            .unwrap();

        // Store the result
        ret.store_field_nocap(gc, 0, bool_val);
        let obj_val = obj.value(gc);
        ret.store_field_nocap(gc, 1, obj_val);

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
                var_name: VAR_NAME.to_string(),
            }),
            vec![FullName::local(VAR_NAME)],
            format!("is_unique({})", VAR_NAME),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetRetainedPtrOfBoxedValueFunctionBody {
    var_name: String,
}

impl InlineLLVMGetRetainedPtrOfBoxedValueFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get argument
        let obj = gc.get_var(&FullName::local(&self.var_name)).ptr.get(gc);
        if !obj.is_box(gc.type_env()) {
            error_exit(
                "Std::FFI::unsafe_get_retained_ptr_of_boxed_value cannot be called on an unboxed value.",
            )
        }
        let ptr = obj.ptr(gc);
        let ret = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                make_ptr_ty(),
                &vec![],
                None,
                gc,
                Some("ret_val@get_ptr_of_boxed_value"),
            )
        };
        ret.store_field_nocap(gc, 0, ptr);
        ret
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
        vec![],
        vec![],
        type_fun(obj_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            LLVMGenerator::GetRetainedPtrOfBoxedValueFunctionBody(
                InlineLLVMGetRetainedPtrOfBoxedValueFunctionBody {
                    var_name: VAR_NAME.to_string(),
                },
            ),
            vec![FullName::local(VAR_NAME)],
            format!("unsafe_get_retained_ptr_of_boxed_value({})", VAR_NAME),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetBoxedValueFromRetainedPtrFunctionBody {
    var_name: String,
}

impl InlineLLVMGetBoxedValueFromRetainedPtrFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Check that return type is boxed.
        if !ret_ty.is_box(gc.type_env()) {
            error_exit(
                "Std::FFI::unsafe_get_boxed_value_from_retained_ptr cannot be called on an unboxed value.",
            )
        }
        assert!(rvo.is_none());

        // Get argument.
        let ptr = gc.get_var(&FullName::local(&self.var_name)).ptr.get(gc);
        let ptr = ptr.load_field_nocap(gc, 0).into_pointer_value();
        Object::new(ptr, ret_ty.clone())
    }
}

pub fn get_boxed_value_from_retained_ptr_function() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ptr_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(ptr_type.clone(), obj_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            LLVMGenerator::GetBoxedValueFromRetainedPtrFunctionBody(
                InlineLLVMGetBoxedValueFromRetainedPtrFunctionBody {
                    var_name: VAR_NAME.to_string(),
                },
            ),
            vec![FullName::local(VAR_NAME)],
            format!(
                "unsafe_get_boxed_value_from_retained_ptr_function({})",
                VAR_NAME
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
    var_name: String,
}

impl InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get argument
        let obj = gc.get_var(&FullName::local(&self.var_name)).ptr.get(gc);
        if !obj.is_box(gc.type_env()) {
            error_exit(
                "Std::FFI::unsafe_get_release_function_of_boxed_value cannot be called on an unboxed value.",
            )
        }
        gc.release(obj.clone());

        // Get function pointer to release function.
        let release_function_name = format!("release#{}", obj.ty.to_string_normalize());
        let func = if let Some(func) = gc.module.get_function(&release_function_name) {
            func
        } else {
            // Define release function.
            let release_function_ty = gc
                .context
                .void_type()
                .fn_type(&[ptr_to_object_type(gc.context).into()], false);
            let release_function =
                gc.module
                    .add_function(&release_function_name, release_function_ty, None);
            let bb = gc.context.append_basic_block(release_function, "entry");
            let _builder_guard = gc.push_builder();
            gc.builder().position_at_end(bb);

            // Get pointer to object.
            let obj_ptr = release_function
                .get_nth_param(0)
                .unwrap()
                .into_pointer_value();
            // Create object.
            let obj = Object::new(obj_ptr, obj.ty.clone());
            // Release object.
            gc.release(obj);
            // Return.
            gc.builder().build_return(None);

            release_function
        };
        let func_ptr = func.as_global_value().as_pointer_value();
        let func_ptr = gc.cast_pointer(func_ptr, ptr_to_object_type(gc.context));

        let ret = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                make_ptr_ty(),
                &vec![],
                None,
                gc,
                Some("ret_val@unsafe_get_release_function_of_boxed_value"),
            )
        };
        ret.store_field_nocap(gc, 0, func_ptr);
        ret
    }
}

pub fn get_release_function_of_boxed_value() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ret_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(obj_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            LLVMGenerator::GetReleaseFunctionOfBoxedValueFunctionBody(
                InlineLLVMGetReleaseFunctionOfBoxedValueFunctionBody {
                    var_name: VAR_NAME.to_string(),
                },
            ),
            vec![FullName::local(VAR_NAME)],
            format!("unsafe_get_release_function_of_boxed_value({})", VAR_NAME),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
    var_name: String,
}

impl InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get argument
        let obj = gc.get_var(&FullName::local(&self.var_name)).ptr.get(gc);
        if !obj.is_box(gc.type_env()) {
            error_exit(
                "Std::FFI::unsafe_get_retain_function_of_boxed_value cannot be called on an unboxed value.",
            )
        }
        gc.release(obj.clone());

        // Get function pointer to retain function.
        let retain_function_name = format!("retain#{}", obj.ty.to_string_normalize());
        let func = if let Some(func) = gc.module.get_function(&retain_function_name) {
            func
        } else {
            // Define release function.
            let retain_function_ty = gc
                .context
                .void_type()
                .fn_type(&[ptr_to_object_type(gc.context).into()], false);
            let retain_function =
                gc.module
                    .add_function(&retain_function_name, retain_function_ty, None);
            let bb = gc.context.append_basic_block(retain_function, "entry");
            let _builder_guard = gc.push_builder();
            gc.builder().position_at_end(bb);

            // Get pointer to object.
            let obj_ptr = retain_function
                .get_nth_param(0)
                .unwrap()
                .into_pointer_value();
            // Create object.
            let obj = Object::new(obj_ptr, obj.ty.clone());
            // retain object.
            gc.retain(obj);
            // Return.
            gc.builder().build_return(None);

            retain_function
        };
        let func_ptr = func.as_global_value().as_pointer_value();
        let func_ptr = gc.cast_pointer(func_ptr, ptr_to_object_type(gc.context));

        let ret = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                make_ptr_ty(),
                &vec![],
                None,
                gc,
                Some("ret_val@unsafe_get_retain_function_of_boxed_value"),
            )
        };
        ret.store_field_nocap(gc, 0, func_ptr);
        ret
    }
}

pub fn get_retain_function_of_boxed_value() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ret_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(obj_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            LLVMGenerator::GetRetainFunctionOfBoxedValueFunctionBody(
                InlineLLVMGetRetainFunctionOfBoxedValueFunctionBody {
                    var_name: VAR_NAME.to_string(),
                },
            ),
            vec![FullName::local(VAR_NAME)],
            format!("unsafe_get_retain_function_of_boxed_value({})", VAR_NAME),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMGetBoxedDataPtrFunctionBody {
    var_name: String,
}

impl InlineLLVMGetBoxedDataPtrFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get argument.
        let obj = gc.get_var(&FullName::local(&self.var_name)).ptr.get(gc);
        if !obj.is_box(gc.type_env()) {
            error_exit(
                "`Std::FFI::_unsafe_get_boxed_data_ptr` can only be called on a boxed value.",
            )
        }
        let ptr = obj.ptr(gc);
        gc.release(obj.clone());
        let struct_ty = obj.struct_ty(gc);
        let ptr = gc.cast_pointer(ptr, struct_ty.ptr_type(AddressSpace::from(0)));
        let data_ptr = gc.builder().build_struct_gep(ptr, 1, "elem_ptr").unwrap();
        let data_ptr = gc.cast_pointer(data_ptr, ptr_to_object_type(gc.context));

        let ret = if rvo.is_some() {
            rvo.unwrap()
        } else {
            allocate_obj(
                make_ptr_ty(),
                &vec![],
                None,
                gc,
                Some("ret_val@_unsafe_get_boxed_data_ptr"),
            )
        };
        ret.store_field_nocap(gc, 0, data_ptr);

        ret
    }
}

pub fn get_unsafe_get_boxed_ptr() -> (Arc<ExprNode>, Arc<Scheme>) {
    const TYPE_NAME: &str = "a";
    const VAR_NAME: &str = "x";
    let obj_type = type_tyvar(TYPE_NAME, &kind_star());
    let ret_type = make_ptr_ty();
    let scm = Scheme::generalize(
        &[],
        vec![],
        vec![],
        type_fun(obj_type.clone(), ret_type.clone()),
    );
    let expr = expr_abs(
        vec![var_local(VAR_NAME)],
        expr_llvm(
            LLVMGenerator::GetBoxedDataPtrFunctionBody(InlineLLVMGetBoxedDataPtrFunctionBody {
                var_name: VAR_NAME.to_string(),
            }),
            vec![FullName::local(VAR_NAME)],
            format!("_get_boxed_data_ptr({})", VAR_NAME),
            ret_type,
            None,
        ),
        None,
    );
    (expr, scm)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMMarkThreadedFunctionBody {
    var_name: String,
}

impl InlineLLVMMarkThreadedFunctionBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ret_ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        // Get argument
        let obj = gc.get_var(&FullName::local(&self.var_name)).ptr.get(gc);
        gc.mark_threaded(obj.clone());
        if rvo.is_some() {
            assert!(obj.is_unbox(gc.type_env()));
            let rvo = rvo.unwrap();
            let val = obj.load_nocap(gc);
            rvo.store_unbox(gc, val);
            rvo
        } else {
            obj
        }
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
                var_name: VAR_NAME.to_string(),
            }),
            vec![FullName::local(VAR_NAME)],
            format!("mark_threaded({})", VAR_NAME),
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
        vec![],
        format!("infinity_{}", type_name),
        ty.clone(),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

// `quiet_nan` built-in value
pub fn quiet_nan_value(type_name: &str) -> (Arc<ExprNode>, Arc<Scheme>) {
    let quet_nan_bits = u64::MAX ^ (1 << 63);
    let nan_val: f64 = unsafe { std::mem::transmute(quet_nan_bits) };

    let ty = make_floating_ty(type_name).unwrap();
    let expr = expr_llvm(
        LLVMGenerator::FloatLit(InlineLLVMFloatLit { val: nan_val }),
        vec![],
        format!("quiet_nan_{}", type_name),
        ty.clone(),
        None,
    );
    let scm = Scheme::generalize(&[], vec![], vec![], ty);
    (expr, scm)
}

pub fn unary_operator_trait(trait_id: TraitId, method_name: Name) -> TraitInfo {
    const TYVAR_NAME: &str = "a";
    let kind = kind_star();
    let tv_tyvar = tyvar_from_name(TYVAR_NAME, &kind);
    let tv_type = type_tyvar(TYVAR_NAME, &kind);
    TraitInfo {
        id: trait_id,
        type_var: tv_tyvar,
        methods: HashMap::from([(
            method_name,
            QualType {
                preds: vec![],
                kind_signs: vec![],
                eqs: vec![],
                ty: type_fun(tv_type.clone(), tv_type.clone()),
            },
        )]),
        assoc_types: HashMap::new(),
        kind_signs: vec![],
        source: None,
    }
}

const UNARY_OPERATOR_RHS_NAME: &str = "rhs";

pub fn unary_opeartor_instance(
    trait_id: TraitId,
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
        methods: HashMap::from([(
            method_name.to_string(),
            expr_abs(
                vec![var_local(UNARY_OPERATOR_RHS_NAME)],
                expr_llvm(
                    generator,
                    vec![FullName::local(UNARY_OPERATOR_RHS_NAME)],
                    method_name.to_string(),
                    result_ty,
                    None,
                ),
                None,
            ),
        )]),
        assoc_types: HashMap::new(),
        define_module: STD_NAME.to_string(),
        source: None,
    }
}

pub fn binary_operator_trait(
    trait_id: TraitId,
    method_name: Name,
    output_ty: Option<Arc<TypeNode>>,
) -> TraitInfo {
    const TYVAR_NAME: &str = "a";
    let kind = kind_star();
    let tv_tyvar = tyvar_from_name(TYVAR_NAME, &kind);
    let tv_type = type_tyvar(TYVAR_NAME, &kind);
    let output_ty = match output_ty {
        Some(t) => t,
        None => tv_type.clone(),
    };
    TraitInfo {
        id: trait_id,
        type_var: tv_tyvar,
        methods: HashMap::from([(
            method_name,
            QualType {
                preds: vec![],
                kind_signs: vec![],
                eqs: vec![],
                ty: type_fun(tv_type.clone(), type_fun(tv_type.clone(), output_ty)),
            },
        )]),
        assoc_types: HashMap::default(),
        kind_signs: vec![],
        source: None,
    }
}

const BINARY_OPERATOR_LHS_NAME: &str = "lhs";
const BINARY_OPERATOR_RHS_NAME: &str = "rhs";

pub fn binary_opeartor_instance(
    trait_id: TraitId,
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
        methods: HashMap::from([(
            method_name.to_string(),
            expr_abs(
                vec![var_local(BINARY_OPERATOR_LHS_NAME)],
                expr_abs(
                    vec![var_local(BINARY_OPERATOR_RHS_NAME)],
                    expr_llvm(
                        generator,
                        vec![
                            FullName::local(BINARY_OPERATOR_LHS_NAME),
                            FullName::local(BINARY_OPERATOR_RHS_NAME),
                        ],
                        method_name.to_string(),
                        result_ty,
                        None,
                    ),
                    None,
                ),
                None,
            ),
        )]),
        assoc_types: HashMap::new(),
        define_module: STD_NAME.to_string(),
        source: None,
    }
}

pub const EQ_TRAIT_NAME: &str = "Eq";
pub const EQ_TRAIT_EQ_NAME: &str = "eq";

pub fn eq_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], EQ_TRAIT_NAME),
    }
}

pub fn eq_trait() -> TraitInfo {
    binary_operator_trait(
        eq_trait_id(),
        EQ_TRAIT_EQ_NAME.to_string(),
        Some(make_bool_ty()),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntEqBody {}

impl InlineLLVMIntEqBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs_obj = gc.get_var(&lhs).ptr.get(gc);
        let rhs_obj = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs_obj.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs_obj);
        let rhs_val = rhs_obj.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs_obj);
        let value =
            gc.builder()
                .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, EQ_TRAIT_EQ_NAME);
        let value = gc.builder().build_int_z_extend(
            value,
            ObjectFieldType::I8
                .to_basic_type(gc, vec![])
                .into_int_type(),
            "eq",
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", EQ_TRAIT_EQ_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn eq_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::IntEqBody(InlineLLVMIntEqBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMPtrEqBody {}

impl InlineLLVMPtrEqBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs_obj = gc.get_var(&lhs).ptr.get(gc);
        let rhs_obj = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs_obj.load_field_nocap(gc, 0).into_pointer_value();
        gc.release(lhs_obj);
        let rhs_val = rhs_obj.load_field_nocap(gc, 0).into_pointer_value();
        gc.release(rhs_obj);
        let diff = gc
            .builder()
            .build_ptr_diff(lhs_val, rhs_val, "ptr_diff@eq_trait_instance_ptr");
        let value = gc.builder().build_int_compare(
            IntPredicate::EQ,
            diff,
            diff.get_type().const_zero(),
            EQ_TRAIT_EQ_NAME,
        );
        let value = gc.builder().build_int_z_extend(
            value,
            ObjectFieldType::I8
                .to_basic_type(gc, vec![])
                .into_int_type(),
            "eq_of_int",
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", EQ_TRAIT_EQ_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn eq_trait_instance_ptr(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::PtrEqBody(InlineLLVMPtrEqBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatEqBody {}

impl InlineLLVMFloatEqBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs_obj = gc.get_var(&lhs).ptr.get(gc);
        let rhs_obj = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs_obj.load_field_nocap(gc, 0).into_float_value();
        gc.release(lhs_obj);
        let rhs_val = rhs_obj.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs_obj);
        let value = gc.builder().build_float_compare(
            inkwell::FloatPredicate::OEQ,
            lhs_val,
            rhs_val,
            EQ_TRAIT_EQ_NAME,
        );
        let value = gc.builder().build_int_z_extend(
            value,
            ObjectFieldType::I8
                .to_basic_type(gc, vec![])
                .into_int_type(),
            "eq_of_float",
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", EQ_TRAIT_EQ_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn eq_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::FloatEqBody(InlineLLVMFloatEqBody {}),
    )
}

pub const LESS_THAN_TRAIT_NAME: &str = "LessThan";
pub const LESS_THAN_TRAIT_LT_NAME: &str = "less_than";

pub fn less_than_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], LESS_THAN_TRAIT_NAME),
    }
}

pub fn less_than_trait() -> TraitInfo {
    binary_operator_trait(
        less_than_trait_id(),
        LESS_THAN_TRAIT_LT_NAME.to_string(),
        Some(make_bool_ty()),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLessThanBody {}

impl InlineLLVMIntLessThanBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs_obj = gc.get_var(&lhs).ptr.get(gc);
        let rhs_obj = gc.get_var(&rhs).ptr.get(gc);
        let is_singed = lhs_obj.ty.toplevel_tycon().unwrap().is_singned_intger();
        let lhs_val = lhs_obj.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs_obj);
        let rhs_val: IntValue = rhs_obj.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs_obj);
        let value = gc.builder().build_int_compare(
            if is_singed {
                IntPredicate::SLT
            } else {
                IntPredicate::ULT
            },
            lhs_val,
            rhs_val,
            LESS_THAN_TRAIT_LT_NAME,
        );
        let value = gc.builder().build_int_z_extend(
            value,
            ObjectFieldType::I8
                .to_basic_type(gc, vec![])
                .into_int_type(),
            LESS_THAN_TRAIT_LT_NAME,
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", LESS_THAN_TRAIT_LT_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn less_than_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_trait_id(),
        &LESS_THAN_TRAIT_LT_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::IntLessThanBody(InlineLLVMIntLessThanBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatLessThanBody {}

impl InlineLLVMFloatLessThanBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs);
        let value = gc.builder().build_float_compare(
            inkwell::FloatPredicate::OLT,
            lhs_val,
            rhs_val,
            LESS_THAN_TRAIT_LT_NAME,
        );
        let value = gc.builder().build_int_z_extend(
            value,
            ObjectFieldType::I8
                .to_basic_type(gc, vec![])
                .into_int_type(),
            LESS_THAN_TRAIT_LT_NAME,
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", LESS_THAN_TRAIT_LT_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn less_than_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_trait_id(),
        &LESS_THAN_TRAIT_LT_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::FloatLessThanBody(InlineLLVMFloatLessThanBody {}),
    )
}

pub const LESS_THAN_OR_EQUAL_TO_TRAIT_NAME: &str = "LessThanOrEq";
pub const LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME: &str = "less_than_or_eq";

pub fn less_than_or_equal_to_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], LESS_THAN_OR_EQUAL_TO_TRAIT_NAME),
    }
}

pub fn less_than_or_equal_to_trait() -> TraitInfo {
    binary_operator_trait(
        less_than_or_equal_to_trait_id(),
        LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME.to_string(),
        Some(make_bool_ty()),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntLessThanOrEqBody {}

impl InlineLLVMIntLessThanOrEqBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let is_singed = lhs.ty.toplevel_tycon().unwrap().is_singned_intger();

        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc.builder().build_int_compare(
            if is_singed {
                IntPredicate::SLE
            } else {
                IntPredicate::ULE
            },
            lhs_val,
            rhs_val,
            LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
        );
        let value = gc.builder().build_int_z_extend(
            value,
            ObjectFieldType::I8
                .to_basic_type(gc, vec![])
                .into_int_type(),
            LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn less_than_or_equal_to_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_or_equal_to_trait_id(),
        &LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::IntLessThanOrEqBody(InlineLLVMIntLessThanOrEqBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatLessThanOrEqBody {}

impl InlineLLVMFloatLessThanOrEqBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs);
        let value = gc.builder().build_float_compare(
            inkwell::FloatPredicate::OLE,
            lhs_val,
            rhs_val,
            LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
        );
        let value = gc.builder().build_int_z_extend(
            value,
            ObjectFieldType::I8
                .to_basic_type(gc, vec![])
                .into_int_type(),
            LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME,
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn less_than_or_equal_to_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        less_than_or_equal_to_trait_id(),
        &LESS_THAN_OR_EQUAL_TO_TRAIT_OP_NAME.to_string(),
        ty,
        make_bool_ty(),
        LLVMGenerator::FloatLessThanOrEqBody(InlineLLVMFloatLessThanOrEqBody {}),
    )
}

pub const ADD_TRAIT_NAME: &str = "Add";
pub const ADD_TRAIT_ADD_NAME: &str = "add";

pub fn add_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], ADD_TRAIT_NAME),
    }
}

pub fn add_trait() -> TraitInfo {
    binary_operator_trait(add_trait_id(), ADD_TRAIT_ADD_NAME.to_string(), None)
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntAddBody {}

impl InlineLLVMIntAddBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_add(lhs_val, rhs_val, ADD_TRAIT_ADD_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", ADD_TRAIT_ADD_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn add_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        add_trait_id(),
        &ADD_TRAIT_ADD_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntAddBody(InlineLLVMIntAddBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatAddBody {}

impl InlineLLVMFloatAddBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_float_add(lhs_val, rhs_val, ADD_TRAIT_ADD_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", ADD_TRAIT_ADD_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn add_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        add_trait_id(),
        &ADD_TRAIT_ADD_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatAddBody(InlineLLVMFloatAddBody {}),
    )
}

pub const SUBTRACT_TRAIT_NAME: &str = "Sub";
pub const SUBTRACT_TRAIT_SUBTRACT_NAME: &str = "sub";

pub fn subtract_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], SUBTRACT_TRAIT_NAME),
    }
}

pub fn subtract_trait() -> TraitInfo {
    binary_operator_trait(
        subtract_trait_id(),
        SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        None,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntSubBody {}

impl InlineLLVMIntSubBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_sub(lhs_val, rhs_val, SUBTRACT_TRAIT_SUBTRACT_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", SUBTRACT_TRAIT_SUBTRACT_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn subtract_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        subtract_trait_id(),
        &SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntSubBody(InlineLLVMIntSubBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatSubBody {}

impl InlineLLVMFloatSubBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_float_sub(lhs_val, rhs_val, SUBTRACT_TRAIT_SUBTRACT_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", SUBTRACT_TRAIT_SUBTRACT_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn subtract_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        subtract_trait_id(),
        &SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatSubBody(InlineLLVMFloatSubBody {}),
    )
}

pub const MULTIPLY_TRAIT_NAME: &str = "Mul";
pub const MULTIPLY_TRAIT_MULTIPLY_NAME: &str = "mul";

pub fn multiply_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], MULTIPLY_TRAIT_NAME),
    }
}

pub fn multiply_trait() -> TraitInfo {
    binary_operator_trait(
        multiply_trait_id(),
        MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        None,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntMulBody {}

impl InlineLLVMIntMulBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_mul(lhs_val, rhs_val, MULTIPLY_TRAIT_MULTIPLY_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", MULTIPLY_TRAIT_MULTIPLY_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn multiply_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        multiply_trait_id(),
        &MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntMulBody(InlineLLVMIntMulBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatMulBody {}

impl InlineLLVMFloatMulBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_float_mul(lhs_val, rhs_val, MULTIPLY_TRAIT_MULTIPLY_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", MULTIPLY_TRAIT_MULTIPLY_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn multiply_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        multiply_trait_id(),
        &MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatMulBody(InlineLLVMFloatMulBody {}),
    )
}

pub const DIVIDE_TRAIT_NAME: &str = "Div";
pub const DIVIDE_TRAIT_DIVIDE_NAME: &str = "div";

pub fn divide_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], DIVIDE_TRAIT_NAME),
    }
}

pub fn divide_trait() -> TraitInfo {
    binary_operator_trait(
        divide_trait_id(),
        DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        None,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntDivBody {}

impl InlineLLVMIntDivBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let is_singed = lhs.ty.toplevel_tycon().unwrap().is_singned_intger();

        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = if is_singed {
            gc.builder()
                .build_int_signed_div(lhs_val, rhs_val, DIVIDE_TRAIT_DIVIDE_NAME)
        } else {
            gc.builder()
                .build_int_unsigned_div(lhs_val, rhs_val, DIVIDE_TRAIT_DIVIDE_NAME)
        };
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", DIVIDE_TRAIT_DIVIDE_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn divide_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        divide_trait_id(),
        &DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntDivBody(InlineLLVMIntDivBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatDivBody {}

impl InlineLLVMFloatDivBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let lhs_val = lhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_float_div(lhs_val, rhs_val, DIVIDE_TRAIT_DIVIDE_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", DIVIDE_TRAIT_DIVIDE_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn divide_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        divide_trait_id(),
        &DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatDivBody(InlineLLVMFloatDivBody {}),
    )
}

pub const REMAINDER_TRAIT_NAME: &str = "Rem";
pub const REMAINDER_TRAIT_REMAINDER_NAME: &str = "rem";

pub fn remainder_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], REMAINDER_TRAIT_NAME),
    }
}

pub fn remainder_trait() -> TraitInfo {
    binary_operator_trait(
        remainder_trait_id(),
        REMAINDER_TRAIT_REMAINDER_NAME.to_string(),
        None,
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntRemBody {}

impl InlineLLVMIntRemBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let lhs = FullName::local(BINARY_OPERATOR_LHS_NAME);
        let rhs = FullName::local(BINARY_OPERATOR_RHS_NAME);
        let lhs = gc.get_var(&lhs).ptr.get(gc);
        let rhs = gc.get_var(&rhs).ptr.get(gc);
        let is_singed = lhs.ty.toplevel_tycon().unwrap().is_singned_intger();

        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs.clone());
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = if is_singed {
            gc.builder()
                .build_int_signed_rem(lhs_val, rhs_val, REMAINDER_TRAIT_REMAINDER_NAME)
        } else {
            gc.builder()
                .build_int_unsigned_rem(lhs_val, rhs_val, REMAINDER_TRAIT_REMAINDER_NAME)
        };
        let obj = if rvo.is_none() {
            allocate_obj(
                lhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} lhs rhs", REMAINDER_TRAIT_REMAINDER_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn remainder_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    binary_opeartor_instance(
        remainder_trait_id(),
        &REMAINDER_TRAIT_REMAINDER_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntRemBody(InlineLLVMIntRemBody {}),
    )
}

pub const NEGATE_TRAIT_NAME: &str = "Neg";
pub const NEGATE_TRAIT_NEGATE_NAME: &str = "neg";

pub fn negate_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], NEGATE_TRAIT_NAME),
    }
}

pub fn negate_trait() -> TraitInfo {
    unary_operator_trait(negate_trait_id(), NEGATE_TRAIT_NEGATE_NAME.to_string())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMIntNegBody {}

impl InlineLLVMIntNegBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let rhs_name = FullName::local(UNARY_OPERATOR_RHS_NAME);
        let rhs = gc.get_var(&rhs_name).ptr.get(gc);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs.clone());
        let value = gc
            .builder()
            .build_int_neg(rhs_val, NEGATE_TRAIT_NEGATE_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                rhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} rhs", NEGATE_TRAIT_NEGATE_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn negate_trait_instance_int(ty: Arc<TypeNode>) -> TraitInstance {
    unary_opeartor_instance(
        negate_trait_id(),
        &NEGATE_TRAIT_NEGATE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::IntNegBody(InlineLLVMIntNegBody {}),
    )
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMFloatNegBody {}

impl InlineLLVMFloatNegBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let rhs_name = FullName::local(UNARY_OPERATOR_RHS_NAME);
        let rhs = gc.get_var(&rhs_name).ptr.get(gc);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_float_value();
        gc.release(rhs.clone());
        let value = gc
            .builder()
            .build_float_neg(rhs_val, NEGATE_TRAIT_NEGATE_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                rhs.ty.clone(),
                &vec![],
                None,
                gc,
                Some(&format!("{} rhs", NEGATE_TRAIT_NEGATE_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn negate_trait_instance_float(ty: Arc<TypeNode>) -> TraitInstance {
    unary_opeartor_instance(
        negate_trait_id(),
        &NEGATE_TRAIT_NEGATE_NAME.to_string(),
        ty.clone(),
        ty,
        LLVMGenerator::FloatNegBody(InlineLLVMFloatNegBody {}),
    )
}

pub const NOT_TRAIT_NAME: &str = "Not";
pub const NOT_TRAIT_OP_NAME: &str = "not";

pub fn not_trait_id() -> TraitId {
    TraitId {
        name: FullName::from_strs(&[STD_NAME], NOT_TRAIT_NAME),
    }
}

pub fn not_trait() -> TraitInfo {
    unary_operator_trait(not_trait_id(), NOT_TRAIT_OP_NAME.to_string())
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InlineLLVMBoolNegBody {}

impl InlineLLVMBoolNegBody {
    pub fn generate<'c, 'm, 'b>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        _ty: &Arc<TypeNode>,
        rvo: Option<Object<'c>>,
        _borrowed_vars: &Vec<FullName>,
    ) -> Object<'c> {
        let rhs_name = FullName::local(UNARY_OPERATOR_RHS_NAME);
        let rhs = gc.get_var(&rhs_name).ptr.get(gc);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let bool_ty = ObjectFieldType::I8
            .to_basic_type(gc, vec![])
            .into_int_type();
        let false_val = bool_ty.const_zero();
        let value =
            gc.builder()
                .build_int_compare(IntPredicate::EQ, rhs_val, false_val, NOT_TRAIT_OP_NAME);
        let value = gc
            .builder()
            .build_int_z_extend(value, bool_ty, NOT_TRAIT_OP_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                make_bool_ty(),
                &vec![],
                None,
                gc,
                Some(&format!("{} rhs", NOT_TRAIT_OP_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
}

pub fn not_trait_instance_bool() -> TraitInstance {
    unary_opeartor_instance(
        not_trait_id(),
        &NOT_TRAIT_OP_NAME.to_string(),
        make_bool_ty(),
        make_bool_ty(),
        LLVMGenerator::BoolNegBody(InlineLLVMBoolNegBody {}),
    )
}
