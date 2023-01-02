// Implement built-in functions, types, etc.
use super::*;

pub const STD_NAME: &str = "Std";

// Primitive types.
pub const INT_NAME: &str = "Int";
pub const BOOL_NAME: &str = "Bool";
pub const ARRAY_NAME: &str = "Array";

pub fn bulitin_tycons() -> HashMap<TyCon, TyConInfo> {
    let mut ret = HashMap::new();
    ret.insert(
        TyCon::new(NameSpacedName::from_strs(&[STD_NAME], INT_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            field_types: vec![],
        },
    );
    ret.insert(
        TyCon::new(NameSpacedName::from_strs(&[STD_NAME], BOOL_NAME)),
        TyConInfo {
            kind: kind_star(),
            variant: TyConVariant::Primitive,
            is_unbox: true,
            tyvars: vec![],
            field_types: vec![],
        },
    );
    ret.insert(
        TyCon::new(NameSpacedName::from_strs(&[STD_NAME], ARRAY_NAME)),
        TyConInfo {
            kind: kind_arrow(kind_star(), kind_star()),
            variant: TyConVariant::Array,
            is_unbox: false,
            tyvars: vec!["a".to_string()],
            field_types: vec![type_tyvar_star("a")],
        },
    );
    ret
}

// Following types are coustructed using primitive types.
pub const LOOP_RESULT_NAME: &str = "LoopResult";
pub const TUPLE_NAME: &str = "Tuple";

// Make name of tuples.
pub fn make_tuple_name(size: u32) -> Name {
    format!("{}{}", TUPLE_NAME, size)
}

// Get Int type.
pub fn int_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(NameSpacedName::from_strs(&[STD_NAME], INT_NAME)))
}

// Get Bool type.
pub fn bool_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(NameSpacedName::from_strs(&[STD_NAME], BOOL_NAME)))
}

// Get Array type.
pub fn array_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(NameSpacedName::from_strs(&[STD_NAME], ARRAY_NAME)))
}

// Get LoopResult type.
pub fn loop_result_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(NameSpacedName::from_strs(
        &[STD_NAME],
        LOOP_RESULT_NAME,
    )))
}

pub fn int(val: i64, source: Option<Span>) -> Arc<ExprNode> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        let obj = if rvo.is_none() {
            allocate_obj(ty.clone(), &vec![], gc, Some(&format!("int_lit_{}", val)))
        } else {
            rvo.unwrap()
        };
        let value = gc.context.i64_type().const_int(val as u64, false);
        obj.store_field_nocap(gc, 0, value);
        obj
    });
    expr_lit(generator, vec![], val.to_string(), int_lit_ty(), source)
}

pub fn bool(val: bool, source: Option<Span>) -> Arc<ExprNode> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        let obj = if rvo.is_none() {
            allocate_obj(ty.clone(), &vec![], gc, Some(&format!("bool_lit_{}", val)))
        } else {
            rvo.unwrap()
        };
        let value = gc.context.i8_type().const_int(val as u64, false);
        obj.store_field_nocap(gc, 0, value);
        obj
    });
    expr_lit(generator, vec![], val.to_string(), bool_lit_ty(), source)
}

fn fix_lit(b: &str, f: &str, x: &str) -> Arc<ExprNode> {
    let f_str = NameSpacedName::local(f);
    let x_str = NameSpacedName::local(x);
    let name = format!("fix {} {}", f_str.to_string(), x_str.to_string());
    let free_vars = vec![
        NameSpacedName::local(SELF_NAME),
        f_str.clone(),
        x_str.clone(),
    ];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, _ty, rvo| {
        let fixf = gc.get_var(&NameSpacedName::local(SELF_NAME)).ptr.get(gc);
        let x = gc.get_var(&x_str).ptr.get(gc);
        let f = gc.get_var(&f_str).ptr.get(gc);
        let f_fixf = gc.apply_lambda(f, fixf, None);
        let f_fixf_x = gc.apply_lambda(f_fixf, x, rvo);
        f_fixf_x
    });
    expr_lit(generator, free_vars, name, type_tyvar_star(b), None)
}

// fix = \f: ((a -> b) -> (a -> b)) -> \x: a -> fix_lit(b, f, x): b
pub fn fix() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("f", None),
        expr_abs(var_local("x", None), fix_lit("b", "f", "x"), None),
        None,
    );
    let fixed_ty = type_fun(type_tyvar_star("a"), type_tyvar_star("b"));
    let scm = Scheme::generalize(
        HashMap::from([
            ("a".to_string(), kind_star()),
            ("b".to_string(), kind_star()),
        ]),
        vec![],
        type_fun(type_fun(fixed_ty.clone(), fixed_ty.clone()), fixed_ty),
    );
    (expr, scm)
}

// Implementation of newArray built-in function.
fn new_array_lit(a: &str, size: &str, value: &str) -> Arc<ExprNode> {
    let size_str = NameSpacedName::local(size);
    let value_str = NameSpacedName::local(value);
    let name = format!("newArray {} {}", size, value);
    let name_cloned = name.clone();
    let free_vars = vec![size_str.clone(), value_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        // Array = [ControlBlock, ArrayField] where ArrayField = [Size, PtrToBuffer].
        let size = gc.get_var_field(&size_str, 0).into_int_value();
        gc.release(gc.get_var(&size_str).ptr.get(gc));
        let value = gc.get_var(&value_str).ptr.get(gc);
        assert!(rvo.is_none()); // Array is boxed, and we don't perform rvo for boxed values.
        let array = allocate_obj(ty.clone(), &vec![], gc, Some(name_cloned.as_str()));
        let array_field = array.ptr_to_field_nocap(gc, ARRAY_IDX);
        ObjectFieldType::initialize_array_size_buf_by_value(
            gc,
            array_field,
            ty.fields_types(gc.type_env())[0].clone(),
            size,
            value,
        );
        array
    });
    expr_lit(
        generator,
        free_vars,
        name,
        type_tyapp(array_lit_ty(), type_tyvar_star(a)),
        None,
    )
}

// "newArray" built-in function.
// newArray = for<a> \size: Int -> \value: a -> new_array_lit(a, size, value): Array<a>
pub fn new_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("size", None),
        expr_abs(
            var_local("value", None),
            new_array_lit("a", "size", "value"),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        HashMap::from([("a".to_string(), kind_star())]),
        vec![],
        type_fun(
            int_lit_ty(),
            type_fun(
                type_tyvar_star("a"),
                type_tyapp(array_lit_ty(), type_tyvar_star("a")),
            ),
        ),
    );
    (expr, scm)
}

// "Array.from_map" built-in function.
pub fn from_map_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let arr_ty = type_tyapp(array_lit_ty(), type_tyvar_star("a"));
    const SIZE_NAME: &str = "size";
    const MAP_NAME: &str = "map";
    let name = "Array.from_map size map".to_string();
    let name_cloned = name.clone();
    let size_name = NameSpacedName::local(SIZE_NAME);
    let map_name = NameSpacedName::local(MAP_NAME);
    let size_name_cloned = size_name.clone();
    let map_name_cloned = map_name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        let size = gc.get_var_field(&size_name_cloned, 0).into_int_value();
        gc.release(gc.get_var(&size_name_cloned).ptr.get(gc));
        let map = gc.get_var(&map_name_cloned).ptr.get(gc);
        assert!(rvo.is_none()); // Array is boxed, and we don't perform rvo for boxed values.
        let array = allocate_obj(ty.clone(), &vec![], gc, Some(name_cloned.as_str()));
        let array_field = array.ptr_to_field_nocap(gc, ARRAY_IDX);
        ObjectFieldType::initialize_array_size_buf_by_map(
            gc,
            array_field,
            ty.fields_types(gc.type_env())[0].clone(),
            size,
            map,
        );
        array
    });
    let expr = expr_abs(
        var_local(SIZE_NAME, None),
        expr_abs(
            var_local(MAP_NAME, None),
            expr_lit(generator, vec![size_name, map_name], name, arr_ty, None),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        HashMap::from([("a".to_string(), kind_star())]),
        vec![],
        type_fun(
            int_lit_ty(),
            type_fun(
                type_fun(int_lit_ty(), type_tyvar_star("a")),
                type_tyapp(array_lit_ty(), type_tyvar_star("a")),
            ),
        ),
    );
    (expr, scm)
}

// Implementation of readArray built-in function.
fn read_array_lit(a: &str, array: &str, idx: &str) -> Arc<ExprNode> {
    let elem_ty = type_tyvar_star(a);
    let array_str = NameSpacedName::local(array);
    let idx_str = NameSpacedName::local(idx);
    let name = format!("Array.get {} {}", idx, array);
    let free_vars = vec![array_str.clone(), idx_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].
        let array = gc.get_var(&array_str).ptr.get(gc);
        let array_field = array.ptr_to_field_nocap(gc, ARRAY_IDX);
        let idx = gc.get_var_field(&idx_str, 0).into_int_value();
        gc.release(gc.get_var(&idx_str).ptr.get(gc));
        let elem = ObjectFieldType::read_array_size_buf(gc, array_field, ty.clone(), idx, rvo);
        gc.release(array);
        elem
    });
    expr_lit(generator, free_vars, name, elem_ty, None)
}

// "readArray" built-in function.
// readArray = for<a> \arr: Array<a> -> \idx: Int -> (...read_array_lit(a, arr, idx)...): a
pub fn read_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("idx", None),
        expr_abs(
            var_local("array", None),
            read_array_lit("a", "array", "idx"),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        HashMap::from([("a".to_string(), kind_star())]),
        vec![],
        type_fun(
            int_lit_ty(),
            type_fun(
                type_tyapp(array_lit_ty(), type_tyvar_star("a")),
                type_tyvar_star("a"),
            ),
        ),
    );
    (expr, scm)
}

// Implementation of Array.set/Array.set! built-in function.
// is_unique_mode - if true, generate code that calls abort when given array is shared.
fn write_array_lit(
    a: &str,
    array: &str,
    idx: &str,
    value: &str,
    is_unique_version: bool,
) -> Arc<ExprNode> {
    let elem_ty = type_tyvar_star(a);
    let array_str = NameSpacedName::local(array);
    let idx_str = NameSpacedName::local(idx);
    let value_str = NameSpacedName::local(value);
    let func_name = String::from({
        if is_unique_version {
            "set!"
        } else {
            "set"
        }
    });
    let name = format!("{} {} {} {}", func_name, array, idx, value);
    let name_cloned = name.clone();
    let free_vars = vec![array_str.clone(), idx_str.clone(), value_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        assert!(rvo.is_none());
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].
        let elem_ty = ty.fields_types(gc.type_env())[0].clone();
        // Get argments
        let array = gc.get_var(&array_str).ptr.get(gc);
        let original_array_ptr = array.ptr(gc);
        let idx = gc.get_var_field(&idx_str, 0).into_int_value();
        gc.release(gc.get_var(&idx_str).ptr.get(gc));
        let value = gc.get_var(&value_str).ptr.get(gc);

        // Get array field.
        let array_field = array.ptr_to_field_nocap(gc, ARRAY_IDX);

        // Get refcnt.
        let refcnt = {
            let array_ptr = array.ptr(gc);
            gc.load_obj_field(array_ptr, control_block_type(gc.context), 0)
                .into_int_value()
        };

        // Add shared / cont bbs.
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let shared_bb = gc.context.append_basic_block(current_func, "shared_bb");
        let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");

        // Jump to shared_bb if refcnt > 1.
        let one = refcnt_type(gc.context).const_int(1, false);
        let is_unique = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, refcnt, one, "is_unique");
        gc.builder()
            .build_conditional_branch(is_unique, cont_bb, shared_bb);

        // In shared_bb, create new array and clone array field.
        gc.builder().position_at_end(shared_bb);
        if is_unique_version {
            // In case of unique version, panic in this case.
            gc.panic(format!("The argument of {} is shared!\n", func_name.as_str()).as_str());
        }
        let cloned_array = allocate_obj(ty.clone(), &vec![], gc, Some(name_cloned.as_str()));
        let cloned_array_field = cloned_array.ptr_to_field_nocap(gc, ARRAY_IDX);

        ObjectFieldType::clone_array_size_buf(gc, array_field, cloned_array_field, elem_ty);
        gc.release(array.clone()); // Given array should be released here.
        let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
        let cloned_array_ptr = cloned_array.ptr(gc);
        gc.builder().build_unconditional_branch(cont_bb);

        // Implement cont_bb
        gc.builder().position_at_end(cont_bb);

        // Build phi value of array and array_field.
        let array_phi = gc
            .builder()
            .build_phi(original_array_ptr.get_type(), "array_phi");
        array_phi.add_incoming(&[
            (&original_array_ptr, current_bb),
            (&cloned_array_ptr, succ_of_shared_bb),
        ]);
        let array = Object::new(array_phi.as_basic_value().into_pointer_value(), ty.clone());
        let array_field_phi = gc
            .builder()
            .build_phi(array_field.get_type(), "array_field_phi");
        assert_eq!(array_field.get_type(), cloned_array_field.get_type());
        array_field_phi.add_incoming(&[
            (&array_field, current_bb),
            (&cloned_array_field, succ_of_shared_bb),
        ]);
        let array_field = array_field_phi.as_basic_value().into_pointer_value();

        // Perform write and return.
        ObjectFieldType::write_array_size_buf(gc, array_field, idx, value);
        array
    });
    expr_lit(
        generator,
        free_vars,
        name,
        type_tyapp(array_lit_ty(), elem_ty),
        None,
    )
}

// Array.set built-in function.
pub fn write_array_common(is_unique_version: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("idx", None),
        expr_abs(
            var_local("value", None),
            expr_abs(
                var_local("array", None),
                write_array_lit("a", "array", "idx", "value", is_unique_version),
                None,
            ),
            None,
        ),
        None,
    );
    let array_ty = type_tyapp(array_lit_ty(), type_tyvar_star("a"));
    let scm = Scheme::generalize(
        HashMap::from([("a".to_string(), kind_star())]),
        vec![],
        type_fun(
            int_lit_ty(),
            type_fun(type_tyvar_star("a"), type_fun(array_ty.clone(), array_ty)),
        ),
    );
    (expr, scm)
}

// set built-in function.
pub fn write_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    write_array_common(false)
}

// set! built-in function.
pub fn write_array_unique() -> (Arc<ExprNode>, Arc<Scheme>) {
    write_array_common(true)
}

// `len` built-in function for Array.
pub fn length_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    const ARR_NAME: &str = "arr";

    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, _ty, rvo| {
        let arr_name = NameSpacedName::local(ARR_NAME);
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].
        let array_obj = gc.get_var(&arr_name).ptr.get(gc);
        let size_buf_ptr = array_obj.ptr_to_field_nocap(gc, ARRAY_IDX);
        let size = ObjectFieldType::size_from_array_size_buf(gc, size_buf_ptr);
        gc.release(array_obj);
        let int_obj = if rvo.is_none() {
            allocate_obj(int_lit_ty(), &vec![], gc, Some("length_of_arr"))
        } else {
            rvo.unwrap()
        };
        int_obj.store_field_nocap(gc, 0, size);
        int_obj
    });

    let expr = expr_abs(
        var_local(ARR_NAME, None),
        expr_lit(
            generator,
            vec![NameSpacedName::local(ARR_NAME)],
            "len arr".to_string(),
            int_lit_ty(),
            None,
        ),
        None,
    );
    let array_ty = type_tyapp(array_lit_ty(), type_tyvar_star("a"));
    let scm = Scheme::generalize(
        HashMap::from([("a".to_string(), kind_star())]),
        vec![],
        type_fun(array_ty, int_lit_ty()),
    );
    (expr, scm)
}

// `new` built-in function for a given struct.
pub fn struct_new_lit(
    struct_name: &NameSpacedName,
    struct_defn: &TypeDecl,
    field_names: Vec<String>,
) -> Arc<ExprNode> {
    let free_vars = field_names
        .iter()
        .map(|name| NameSpacedName::local(name))
        .collect();
    let name = format!("{}.new {}", struct_name.to_string(), field_names.join(" "));
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        // Get field values.
        let fields = field_names
            .iter()
            .map(|name| gc.get_var(&NameSpacedName::local(name)).ptr.get(gc))
            .collect::<Vec<_>>();

        // Create struct object.
        let obj = if rvo.is_none() {
            allocate_obj(ty.clone(), &vec![], gc, Some(&name_cloned))
        } else {
            rvo.unwrap()
        };

        // Set fields.
        for (i, field) in fields.iter().enumerate() {
            ObjectFieldType::set_struct_field(gc, &obj, i as u32, field);
        }

        obj
    });
    expr_lit(generator, free_vars, name, struct_defn.ty(), None)
}

// `new` built-in function for a given struct.
pub fn struct_new(
    struct_name: &NameSpacedName,
    definition: &TypeDecl,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    let mut expr = struct_new_lit(
        struct_name,
        definition,
        definition.fields().iter().map(|f| f.name.clone()).collect(),
    );
    let mut ty = definition.ty();
    for field in definition.fields().iter().rev() {
        expr = expr_abs(var_local(&field.name, None), expr, None);
        ty = type_fun(field.ty.clone(), ty);
    }
    let scm = Scheme::generalize(ty.free_vars(), vec![], ty);
    (expr, scm)
}

// `get` built-in function for a given struct.
pub fn struct_get_lit(
    var_name: &str,
    field_idx: usize,
    field_ty: Arc<TypeNode>,
    struct_name: &NameSpacedName,
    field_name: &str,
) -> Arc<ExprNode> {
    let var_name_clone = NameSpacedName::local(var_name);
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, _ty, rvo| {
        // Get struct object.
        let str = gc.get_var(&var_name_clone).ptr.get(gc);

        // Extract field.
        let field = ObjectFieldType::get_struct_field(gc, &str, field_idx as u32);
        let field_val = field.value(gc);
        let field = if rvo.is_none() {
            Object::create_from_value(field_val, field.ty, gc)
        } else {
            let rvo = rvo.unwrap();
            rvo.store_unbox(gc, field_val);
            rvo
        };

        // Retain field and release struct.
        gc.retain(field.clone());
        gc.release(str);

        field
    });
    let free_vars = vec![NameSpacedName::local(var_name)];
    let name = format!("{}.get_{}", struct_name.to_string(), field_name);
    expr_lit(generator, free_vars, name, field_ty, None)
}

// `get` built-in function for a given struct.
pub fn struct_get(
    struct_name: &NameSpacedName,
    definition: &TypeDecl,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition
        .fields()
        .iter()
        .enumerate()
        .find(|(_i, f)| f.name == field_name);
    if field.is_none() {
        error_exit(&format!(
            "error: no field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (field_idx, field) = field.unwrap();

    let str_ty = definition.ty();
    let expr = expr_abs(
        var_local("f", None),
        struct_get_lit("f", field_idx, field.ty.clone(), struct_name, field_name),
        None,
    );
    let ty = type_fun(str_ty, field.ty.clone());
    let scm = Scheme::generalize(ty.free_vars(), vec![], ty);
    (expr, scm)
}

// `mod` built-in function for a given struct.
pub fn struct_mod_lit(
    f_name: &str,
    x_name: &str,
    field_count: usize, // number of fields in this struct
    field_idx: usize,
    struct_name: &NameSpacedName,
    struct_defn: &TypeDecl,
    field_name: &str,
    is_unique_version: bool,
) -> Arc<ExprNode> {
    let name = format!(
        "{}.mod{}{} {} {}",
        struct_name.to_string(),
        field_name,
        if is_unique_version { "!" } else { "" },
        f_name,
        x_name
    );
    let f_name = NameSpacedName::local(f_name);
    let x_name = NameSpacedName::local(x_name);
    let free_vars = vec![f_name.clone(), x_name.clone()];
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        let is_unbox = ty.is_unbox(gc.type_env());

        // Get arguments
        let modfier = gc.get_var(&f_name).ptr.get(gc);
        let mut str = gc.get_var(&x_name).ptr.get(gc);

        if !is_unbox {
            // In boxed case, str should be replaced to cloned object if it is shared.
            // In unboxed case, str is always treated as unique object.
            assert!(rvo.is_none());

            // Get refcnt.
            let refcnt = {
                let str_ptr = str.ptr(gc);
                gc.load_obj_field(str_ptr, control_block_type(gc.context), 0)
                    .into_int_value()
            };

            // Add shared / cont bbs.
            let current_bb = gc.builder().get_insert_block().unwrap();
            let current_func = current_bb.get_parent().unwrap();
            let shared_bb = gc.context.append_basic_block(current_func, "shared_bb");
            let cont_bb = gc.context.append_basic_block(current_func, "cont_bb");

            let original_str_ptr = str.ptr(gc);

            // Jump to shared_bb if refcnt > 1.
            let one = refcnt_type(gc.context).const_int(1, false);
            let is_unique =
                gc.builder()
                    .build_int_compare(IntPredicate::EQ, refcnt, one, "is_unique");
            gc.builder()
                .build_conditional_branch(is_unique, cont_bb, shared_bb);

            // In shared_bb, create new struct and clone fields.
            gc.builder().position_at_end(shared_bb);
            if is_unique_version {
                // In case of unique version, panic in this case.
                gc.panic(&format!("The argument of mod! is shared!\n"));
            }
            let cloned_str = allocate_obj(str.ty.clone(), &vec![], gc, Some(name_cloned.as_str()));
            for i in 0..field_count {
                // Retain field.
                let field = ObjectFieldType::get_struct_field(gc, &str, i as u32);
                gc.retain(field.clone());
                // Clone field.
                ObjectFieldType::set_struct_field(gc, &cloned_str, i as u32, &field);
            }
            gc.release(str.clone());
            let cloned_str_ptr = cloned_str.ptr(gc);
            let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
            gc.builder().build_unconditional_branch(cont_bb);

            // Implement cont_bb
            gc.builder().position_at_end(cont_bb);

            // Build phi value
            let str_phi = gc.builder().build_phi(str.ptr(gc).get_type(), "str_phi");
            str_phi.add_incoming(&[
                (&original_str_ptr, current_bb),
                (&cloned_str_ptr, succ_of_shared_bb),
            ]);

            str = Object::new(str_phi.as_basic_value().into_pointer_value(), ty.clone());
        }

        // Modify field
        let field = ObjectFieldType::get_struct_field(gc, &str, field_idx as u32);
        let field = gc.apply_lambda(modfier, field, None);
        ObjectFieldType::set_struct_field(gc, &str, field_idx as u32, &field);

        if rvo.is_some() {
            assert!(is_unbox);
            // Move str to rvo.
            let rvo = rvo.unwrap();
            let str_val = str.load_nocap(gc);
            rvo.store_unbox(gc, str_val);
            str = rvo;
        }

        str
    });
    expr_lit(generator, free_vars, name, struct_defn.ty(), None)
}

// `mod` built-in function for a given struct.
pub fn struct_mod(
    struct_name: &NameSpacedName,
    definition: &TypeDecl,
    field_name: &str,
    is_unique_version: bool,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition
        .fields()
        .iter()
        .enumerate()
        .find(|(_i, f)| f.name == field_name);
    if field.is_none() {
        error_exit(&format!(
            "error: no field `{}` found in the struct `{}`.",
            &field_name,
            struct_name.to_string(),
        ));
    }
    let (field_idx, field) = field.unwrap();

    let field_count = definition.fields().len();
    let str_ty = definition.ty();
    let expr = expr_abs(
        var_local("f", None),
        expr_abs(
            var_local("x", None),
            struct_mod_lit(
                "f",
                "x",
                field_count,
                field_idx,
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
    let scm = Scheme::generalize(ty.free_vars(), vec![], ty);
    (expr, scm)
}

// `new_{field}` built-in function for a given union.
pub fn union_new(
    union_name: &NameSpacedName,
    field_name: &Name,
    union: &TypeDecl,
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
            "unknown field `{}` for union `{}`",
            field_name,
            union_name.to_string()
        ));
    }
    let expr = expr_abs(
        var_local(field_name, None),
        union_new_lit(union_name, union, field_name, field_idx),
        None,
    );
    let union_ty = union.ty();
    let field_ty = union.fields()[field_idx].ty.clone();
    let ty = type_fun(field_ty, union_ty);
    let scm = Scheme::generalize(ty.free_vars(), vec![], ty);
    (expr, scm)
}

// `new_{field}` built-in function for a given union.
pub fn union_new_lit(
    union_name: &NameSpacedName,
    union_defn: &TypeDecl,
    field_name: &Name,
    field_idx: usize,
) -> Arc<ExprNode> {
    let free_vars = vec![NameSpacedName::local(field_name)];
    let name = format!("{}.new_{}", union_name.to_string(), field_name);
    let name_cloned = name.clone();
    let field_name_cloned = field_name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        let is_unbox = ty.is_unbox(gc.type_env());
        let offset: u32 = if is_unbox { 0 } else { 1 };

        // Get field values.
        let field = gc
            .get_var(&NameSpacedName::local(&field_name_cloned))
            .ptr
            .get(gc);

        // Create union object.
        let obj = if rvo.is_none() {
            allocate_obj(ty.clone(), &vec![], gc, Some(&name_cloned))
        } else {
            rvo.unwrap()
        };

        // Set tag value.
        let tag_value = gc.context.i64_type().const_int(field_idx as u64, false);
        obj.store_field_nocap(gc, 0 + offset, tag_value);

        // Set value.
        let buf = obj.ptr_to_field_nocap(gc, offset + 1);
        ObjectFieldType::set_value_to_union_buf(gc, buf, field);

        obj
    });
    expr_lit(generator, free_vars, name, union_defn.ty(), None)
}

// `as_{field}` built-in function for a given union.
pub fn union_as(
    union_name: &NameSpacedName,
    field_name: &Name,
    union: &TypeDecl,
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
            "unknown field `{}` for union `{}`",
            field_name,
            union_name.to_string()
        ));
    }
    let union_arg_name = "union".to_string();
    let expr = expr_abs(
        var_local(&union_arg_name, None),
        union_as_lit(
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
    let scm = Scheme::generalize(ty.free_vars(), vec![], ty);
    (expr, scm)
}

// `as_{field}` built-in function for a given union.
pub fn union_as_lit(
    union_name: &NameSpacedName,
    union_arg_name: &Name,
    field_name: &Name,
    field_idx: usize,
    field_ty: Arc<TypeNode>,
) -> Arc<ExprNode> {
    let name = format!("{}.as_{}", union_name.to_string(), field_name);
    let free_vars = vec![NameSpacedName::local(union_arg_name)];
    let union_arg_name = union_arg_name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        // Get union object.
        let obj = gc
            .get_var(&NameSpacedName::local(&union_arg_name))
            .ptr
            .get(gc);

        let is_unbox = obj.ty.is_unbox(gc.type_env());
        let offset = if is_unbox { 0 } else { 1 };
        let elem_ty = ty.clone();

        // Create specified tag value.
        let specified_tag_value = gc.context.i64_type().const_int(field_idx as u64, false);

        // Get tag value.
        let tag_value = obj.load_field_nocap(gc, 0 + offset).into_int_value();

        // If tag unmatch, panic.
        let is_tag_unmatch = gc.builder().build_int_compare(
            IntPredicate::NE,
            specified_tag_value,
            tag_value,
            "is_tag_unmatch",
        );
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let unmatch_bb = gc.context.append_basic_block(current_func, "unmatch_bb");
        let match_bb = gc.context.append_basic_block(current_func, "match_bb");
        gc.builder()
            .build_conditional_branch(is_tag_unmatch, unmatch_bb, match_bb);
        gc.builder().position_at_end(unmatch_bb);
        gc.panic("tag unmatch.");
        gc.builder().build_unconditional_branch(match_bb);

        // When match, return the value.
        gc.builder().position_at_end(match_bb);
        let buf = obj.ptr_to_field_nocap(gc, 1 + offset);
        let value = ObjectFieldType::get_object_from_union_buf(gc, buf, &elem_ty, rvo);

        gc.release(obj);
        value
    });
    expr_lit(generator, free_vars, name, field_ty, None)
}

// `is_{field}` built-in function for a given union.
pub fn union_is(
    union_name: &NameSpacedName,
    field_name: &Name,
    union: &TypeDecl,
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
            "unknown field `{}` for union `{}`",
            field_name,
            union_name.to_string()
        ));
    }
    let union_arg_name = "union".to_string();
    let expr = expr_abs(
        var_local(&union_arg_name, None),
        union_is_lit(union_name, &union_arg_name, field_name, field_idx),
        None,
    );
    let union_ty = union.ty();
    let ty = type_fun(union_ty, bool_lit_ty());
    let scm = Scheme::generalize(ty.free_vars(), vec![], ty);
    (expr, scm)
}

// `is_{field}` built-in function for a given union.
pub fn union_is_lit(
    union_name: &NameSpacedName,
    union_arg_name: &Name,
    field_name: &Name,
    field_idx: usize,
) -> Arc<ExprNode> {
    let name = format!("{}.is_{}", union_name.to_string(), field_name);
    let name_cloned = name.clone();
    let free_vars = vec![NameSpacedName::local(union_arg_name)];
    let union_arg_name = union_arg_name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        let is_unbox = ty.is_unbox(gc.type_env());
        let offset = if is_unbox { 0 } else { 1 };

        // Get union object.
        let obj = gc
            .get_var(&NameSpacedName::local(&union_arg_name))
            .ptr
            .get(gc);

        // Create specified tag value.
        let specified_tag_value = gc.context.i64_type().const_int(field_idx as u64, false);

        // Get tag value.
        let tag_value = obj.load_field_nocap(gc, 0 + offset).into_int_value();

        // Create returned value.
        let ret = if rvo.is_none() {
            allocate_obj(bool_lit_ty(), &vec![], gc, Some(&name_cloned))
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
    });
    expr_lit(generator, free_vars, name, bool_lit_ty(), None)
}

const LOOP_RESULT_CONTINUE_IDX: usize = 0;
pub fn loop_result_defn() -> TypeDecl {
    TypeDecl {
        name: NameSpacedName::from_strs(&[STD_NAME], LOOP_RESULT_NAME),
        tyvars: vec!["s".to_string(), "b".to_string()],
        value: TypeDeclValue::Union(Union {
            fields: vec![
                Field {
                    name: "continue".to_string(),
                    ty: type_tyvar("s", &kind_star()),
                },
                Field {
                    name: "break".to_string(),
                    ty: type_tyvar("b", &kind_star()),
                },
            ],
            is_unbox: true,
        }),
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
        HashMap::from([
            (S_NAME.to_string(), kind_star()),
            (B_NAME.to_string(), kind_star()),
        ]),
        vec![],
        type_fun(
            tyvar_s.clone(),
            type_fun(
                type_fun(
                    tyvar_s.clone(),
                    type_tyapp(type_tyapp(loop_result_ty(), tyvar_s), tyvar_b.clone()),
                ),
                tyvar_b,
            ),
        ),
    );

    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, ty, rvo| {
        let initial_state_name = NameSpacedName::local(INITIAL_STATE_NAME);
        let loop_body_name = NameSpacedName::local(LOOP_BODY_NAME);

        // Get argments.
        let init_state = gc.get_var(&initial_state_name).ptr.get(gc);
        let loop_body = gc.get_var(&loop_body_name).ptr.get(gc);

        // Allocate a variable to store loop state on stack.
        let state_ty = init_state.ty.clone();
        let state_ptr = gc
            .builder()
            .build_alloca(state_ty.get_embedded_type(gc, &vec![]), "loop_state");

        // Initialize state.
        let state_val = init_state.value(gc);
        gc.builder().build_store(state_ptr, state_val);

        // Create loop body bb and implement it.
        let current_bb = gc.builder().get_insert_block().unwrap();
        let current_func = current_bb.get_parent().unwrap();
        let loop_bb = gc.context.append_basic_block(current_func, "loop_bb");
        gc.builder().build_unconditional_branch(loop_bb);

        // Implement loop body.
        gc.builder().position_at_end(loop_bb);
        let stack_pos = gc.save_stack();

        fn get_loop_state<'c, 'm>(
            gc: &mut GenerationContext<'c, 'm>,
            state_ptr: PointerValue<'c>,
            state_ty: &Arc<TypeNode>,
        ) -> Object<'c> {
            Object::new(
                if state_ty.is_box(gc.type_env()) {
                    gc.builder()
                        .build_load(state_ptr, "loop_state")
                        .into_pointer_value()
                } else {
                    state_ptr
                },
                state_ty.clone(),
            )
        }
        let loop_state = get_loop_state(gc, state_ptr, &state_ty);

        // Run loop_body on init_state.
        gc.retain(loop_body.clone());
        let loop_res = gc.apply_lambda(loop_body.clone(), loop_state, None);

        // Branch due to loop_res.
        assert!(loop_res.ty.is_unbox(gc.type_env()));
        let tag_value = loop_res.load_field_nocap(gc, 0).into_int_value();
        let cont_tag_value = gc
            .context
            .i64_type()
            .const_int(LOOP_RESULT_CONTINUE_IDX as u64, false);
        let is_continue = gc.builder().build_int_compare(
            IntPredicate::EQ,
            tag_value,
            cont_tag_value,
            "is_continue",
        );
        let continue_bb = gc.context.append_basic_block(current_func, "continue_bb");
        let break_bb = gc.context.append_basic_block(current_func, "break_bb");
        gc.builder()
            .build_conditional_branch(is_continue, continue_bb, break_bb);

        // Implement continue.
        gc.builder().position_at_end(continue_bb);
        assert!(loop_res.is_unbox(gc.type_env()));
        let union_buf = loop_res.ptr_to_field_nocap(gc, 1);
        let next_state_val = ObjectFieldType::get_value_from_union_buf(gc, union_buf, &state_ty);
        gc.builder().build_store(state_ptr, next_state_val);
        let loop_state = get_loop_state(gc, state_ptr, &state_ty);
        gc.retain(loop_state);
        gc.release(loop_res.clone());

        gc.restore_stack(stack_pos);
        gc.builder().build_unconditional_branch(loop_bb);

        // Implement break.
        gc.builder().position_at_end(break_bb);
        gc.release(loop_body);
        assert!(loop_res.is_unbox(gc.type_env()));
        let union_buf = loop_res.ptr_to_field_nocap(gc, 1);
        let result = ObjectFieldType::get_object_from_union_buf(gc, union_buf, ty, rvo);
        gc.release(loop_res);
        result
    });

    let initial_state_name = NameSpacedName::local(INITIAL_STATE_NAME);
    let loop_body_name = NameSpacedName::local(LOOP_BODY_NAME);
    let expr = expr_abs(
        var_var(initial_state_name.clone(), None),
        expr_abs(
            var_var(loop_body_name.clone(), None),
            expr_lit(
                generator,
                vec![initial_state_name, loop_body_name],
                format!("loop {} {}", INITIAL_STATE_NAME, LOOP_BODY_NAME),
                type_tyvar_star(B_NAME),
                None,
            ),
            None,
        ),
        None,
    );
    (expr, scm)
}

pub fn tuple_defn(size: u32) -> TypeDecl {
    let tyvars = (0..size)
        .map(|i| "t".to_string() + &i.to_string())
        .collect::<Vec<_>>();
    TypeDecl {
        name: NameSpacedName::from_strs(&[STD_NAME], &make_tuple_name(size)),
        tyvars: tyvars.clone(),
        value: TypeDeclValue::Struct(Struct {
            fields: (0..size)
                .map(|i| Field {
                    name: i.to_string(),
                    ty: type_tyvar_star(&tyvars[i as usize]),
                })
                .collect(),
            is_unbox: true,
        }),
    }
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
                kind_preds: vec![],
                ty: type_fun(tv_type.clone(), tv_type.clone()),
            },
        )]),
        kind_predicates: vec![],
    }
}

pub fn unary_opeartor_instance(
    trait_id: TraitId,
    method_name: &Name,
    operand_ty: Arc<TypeNode>,
    result_ty: Arc<TypeNode>,
    generator: for<'c, 'm> fn(
        &mut GenerationContext<'c, 'm>, // gc
        Object<'c>,                     // rhs
        Option<Object<'c>>,             // rvo
    ) -> Object<'c>,
) -> TraitInstance {
    const RHS_NAME: &str = "rhs";
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, _ty, rvo| {
        let rhs_name = NameSpacedName::local(RHS_NAME);
        let rhs = gc.get_var(&rhs_name).ptr.get(gc);
        generator(gc, rhs, rvo)
    });
    TraitInstance {
        qual_pred: QualPredicate {
            context: vec![],
            kind_preds: vec![],
            predicate: Predicate {
                trait_id,
                ty: operand_ty,
            },
        },
        methods: HashMap::from([(
            method_name.to_string(),
            expr_abs(
                var_local(RHS_NAME, None),
                expr_lit(
                    generator,
                    vec![NameSpacedName::local(RHS_NAME)],
                    method_name.to_string(),
                    result_ty,
                    None,
                ),
                None,
            ),
        )]),
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
                kind_preds: vec![],
                ty: type_fun(tv_type.clone(), type_fun(tv_type.clone(), output_ty)),
            },
        )]),
        kind_predicates: vec![],
    }
}

pub fn binary_opeartor_instance(
    trait_id: TraitId,
    method_name: &Name,
    operand_ty: Arc<TypeNode>,
    result_ty: Arc<TypeNode>,
    generator: for<'c, 'm> fn(
        &mut GenerationContext<'c, 'm>, // gc
        Object<'c>,                     // lhs
        Object<'c>,                     // rhs
        Option<Object<'c>>,             // rvo
    ) -> Object<'c>,
) -> TraitInstance {
    const LHS_NAME: &str = "lhs";
    const RHS_NAME: &str = "rhs";
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc, _ty, rvo| {
        let lhs = NameSpacedName::local(LHS_NAME);
        let rhs = NameSpacedName::local(RHS_NAME);
        let lhs_val = gc.get_var(&lhs).ptr.get(gc);
        let rhs_val = gc.get_var(&rhs).ptr.get(gc);
        generator(gc, lhs_val, rhs_val, rvo)
    });
    TraitInstance {
        qual_pred: QualPredicate {
            context: vec![],
            kind_preds: vec![],
            predicate: Predicate {
                trait_id,
                ty: operand_ty,
            },
        },
        methods: HashMap::from([(
            method_name.to_string(),
            expr_abs(
                var_local(LHS_NAME, None),
                expr_abs(
                    var_local(RHS_NAME, None),
                    expr_lit(
                        generator,
                        vec![
                            NameSpacedName::local(LHS_NAME),
                            NameSpacedName::local(RHS_NAME),
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
    }
}

pub const EQ_TRAIT_NAME: &str = "Eq";
pub const EQ_TRAIT_EQ_NAME: &str = "eq";

pub fn eq_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], EQ_TRAIT_NAME),
    }
}

pub fn eq_trait() -> TraitInfo {
    binary_operator_trait(
        eq_trait_id(),
        EQ_TRAIT_EQ_NAME.to_string(),
        Some(bool_lit_ty()),
    )
}

pub fn eq_trait_instance_primitive(ty: Arc<TypeNode>) -> TraitInstance {
    fn generate_eq_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value =
            gc.builder()
                .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, EQ_TRAIT_EQ_NAME);
        let value = gc.builder().build_int_cast(
            value,
            ObjectFieldType::Bool.to_basic_type(gc).into_int_type(),
            "eq",
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                bool_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", EQ_TRAIT_EQ_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        eq_trait_id(),
        &EQ_TRAIT_EQ_NAME.to_string(),
        ty,
        bool_lit_ty(),
        generate_eq_int,
    )
}

pub const CMP_TRAIT_NAME: &str = "Cmp";
pub const CMP_TRAIT_LT_NAME: &str = "less_than";

pub fn cmp_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], CMP_TRAIT_NAME),
    }
}

pub fn cmp_trait() -> TraitInfo {
    binary_operator_trait(
        cmp_trait_id(),
        CMP_TRAIT_LT_NAME.to_string(),
        Some(bool_lit_ty()),
    )
}

pub fn cmp_trait_instance_int() -> TraitInstance {
    fn generate_cmp_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value =
            gc.builder()
                .build_int_compare(IntPredicate::SLT, lhs_val, rhs_val, CMP_TRAIT_LT_NAME);
        let value = gc.builder().build_int_cast(
            value,
            ObjectFieldType::Bool.to_basic_type(gc).into_int_type(),
            CMP_TRAIT_LT_NAME,
        );
        let obj = if rvo.is_none() {
            allocate_obj(
                bool_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", CMP_TRAIT_LT_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        cmp_trait_id(),
        &CMP_TRAIT_LT_NAME.to_string(),
        int_lit_ty(),
        bool_lit_ty(),
        generate_cmp_int,
    )
}

pub const ADD_TRAIT_NAME: &str = "Add";
pub const ADD_TRAIT_ADD_NAME: &str = "add";

pub fn add_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], ADD_TRAIT_NAME),
    }
}

pub fn add_trait() -> TraitInfo {
    binary_operator_trait(add_trait_id(), ADD_TRAIT_ADD_NAME.to_string(), None)
}

pub fn add_trait_instance_int() -> TraitInstance {
    fn generate_add_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_add(lhs_val, rhs_val, ADD_TRAIT_ADD_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                int_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", ADD_TRAIT_ADD_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        add_trait_id(),
        &ADD_TRAIT_ADD_NAME.to_string(),
        int_lit_ty(),
        int_lit_ty(),
        generate_add_int,
    )
}

pub const SUBTRACT_TRAIT_NAME: &str = "Sub";
pub const SUBTRACT_TRAIT_SUBTRACT_NAME: &str = "sub";

pub fn subtract_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], SUBTRACT_TRAIT_NAME),
    }
}

pub fn subtract_trait() -> TraitInfo {
    binary_operator_trait(
        subtract_trait_id(),
        SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        None,
    )
}

pub fn subtract_trait_instance_int() -> TraitInstance {
    fn generate_subtract_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_sub(lhs_val, rhs_val, SUBTRACT_TRAIT_SUBTRACT_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                int_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", SUBTRACT_TRAIT_SUBTRACT_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        subtract_trait_id(),
        &SUBTRACT_TRAIT_SUBTRACT_NAME.to_string(),
        int_lit_ty(),
        int_lit_ty(),
        generate_subtract_int,
    )
}

pub const MULTIPLY_TRAIT_NAME: &str = "Mul";
pub const MULTIPLY_TRAIT_MULTIPLY_NAME: &str = "mul";

pub fn multiply_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], MULTIPLY_TRAIT_NAME),
    }
}

pub fn multiply_trait() -> TraitInfo {
    binary_operator_trait(
        multiply_trait_id(),
        MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        None,
    )
}

pub fn multiply_trait_instance_int() -> TraitInstance {
    fn generate_multiply_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_mul(lhs_val, rhs_val, MULTIPLY_TRAIT_MULTIPLY_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                int_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", MULTIPLY_TRAIT_MULTIPLY_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        multiply_trait_id(),
        &MULTIPLY_TRAIT_MULTIPLY_NAME.to_string(),
        int_lit_ty(),
        int_lit_ty(),
        generate_multiply_int,
    )
}

pub const DIVIDE_TRAIT_NAME: &str = "Div";
pub const DIVIDE_TRAIT_DIVIDE_NAME: &str = "div";

pub fn divide_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], DIVIDE_TRAIT_NAME),
    }
}

pub fn divide_trait() -> TraitInfo {
    binary_operator_trait(
        divide_trait_id(),
        DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        None,
    )
}

pub fn divide_trait_instance_int() -> TraitInstance {
    fn generate_divide_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_signed_div(lhs_val, rhs_val, DIVIDE_TRAIT_DIVIDE_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                int_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", DIVIDE_TRAIT_DIVIDE_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        divide_trait_id(),
        &DIVIDE_TRAIT_DIVIDE_NAME.to_string(),
        int_lit_ty(),
        int_lit_ty(),
        generate_divide_int,
    )
}

pub const REMAINDER_TRAIT_NAME: &str = "Rem";
pub const REMAINDER_TRAIT_REMAINDER_NAME: &str = "rem";

pub fn remainder_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], REMAINDER_TRAIT_NAME),
    }
}

pub fn remainder_trait() -> TraitInfo {
    binary_operator_trait(
        remainder_trait_id(),
        REMAINDER_TRAIT_REMAINDER_NAME.to_string(),
        None,
    )
}

pub fn remainder_trait_instance_int() -> TraitInstance {
    fn generate_remainder_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value =
            gc.builder()
                .build_int_signed_rem(lhs_val, rhs_val, REMAINDER_TRAIT_REMAINDER_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                int_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", REMAINDER_TRAIT_REMAINDER_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        remainder_trait_id(),
        &REMAINDER_TRAIT_REMAINDER_NAME.to_string(),
        int_lit_ty(),
        int_lit_ty(),
        generate_remainder_int,
    )
}

pub const AND_TRAIT_NAME: &str = "And";
pub const AND_TRAIT_AND_NAME: &str = "and";

pub fn and_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], AND_TRAIT_NAME),
    }
}

pub fn and_trait() -> TraitInfo {
    binary_operator_trait(and_trait_id(), AND_TRAIT_AND_NAME.to_string(), None)
}

pub fn and_trait_instance_bool() -> TraitInstance {
    fn generate_and_bool<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        lhs: Object<'c>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let lhs_val = lhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(lhs);
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc.builder().build_and(lhs_val, rhs_val, AND_TRAIT_AND_NAME);

        let obj = if rvo.is_none() {
            allocate_obj(
                bool_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} lhs rhs", AND_TRAIT_AND_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    binary_opeartor_instance(
        and_trait_id(),
        &AND_TRAIT_AND_NAME.to_string(),
        bool_lit_ty(),
        bool_lit_ty(),
        generate_and_bool,
    )
}

pub const NEGATE_TRAIT_NAME: &str = "Neg";
pub const NEGATE_TRAIT_NEGATE_NAME: &str = "neg";

pub fn negate_trait_id() -> TraitId {
    TraitId {
        name: NameSpacedName::from_strs(&[STD_NAME], NEGATE_TRAIT_NAME),
    }
}

pub fn negate_trait() -> TraitInfo {
    unary_operator_trait(negate_trait_id(), NEGATE_TRAIT_NEGATE_NAME.to_string())
}

pub fn negate_trait_instance_int() -> TraitInstance {
    fn generate_negate_int<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        rhs: Object<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let rhs_val = rhs.load_field_nocap(gc, 0).into_int_value();
        gc.release(rhs);
        let value = gc
            .builder()
            .build_int_neg(rhs_val, NEGATE_TRAIT_NEGATE_NAME);
        let obj = if rvo.is_none() {
            allocate_obj(
                int_lit_ty(),
                &vec![],
                gc,
                Some(&format!("{} rhs", NEGATE_TRAIT_NEGATE_NAME)),
            )
        } else {
            rvo.unwrap()
        };
        obj.store_field_nocap(gc, 0, value);
        obj
    }
    unary_opeartor_instance(
        negate_trait_id(),
        &NEGATE_TRAIT_NEGATE_NAME.to_string(),
        int_lit_ty(),
        int_lit_ty(),
        generate_negate_int,
    )
}
