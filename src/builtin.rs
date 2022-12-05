// Implement built-in functions, types, etc.
use super::*;

pub const PRELUDE_NAME: &str = "Prelude";

const INT_NAME: &str = "Int";
const BOOL_NAME: &str = "Bool";
const ARRAY_NAME: &str = "Array";

pub fn bulitin_type_to_kind_map() -> HashMap<String, Arc<Kind>> {
    let mut ret = HashMap::new();
    ret.insert(INT_NAME.to_string(), kind_star());
    ret.insert(BOOL_NAME.to_string(), kind_star());
    ret.insert(ARRAY_NAME.to_string(), kind_arrow(kind_star(), kind_star()));
    ret
}

// Get Int type.
pub fn int_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(INT_NAME))
}

// Get Bool type.
pub fn bool_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(BOOL_NAME))
}

// Get Array type.
pub fn array_lit_ty() -> Arc<TypeNode> {
    type_tycon(&tycon(ARRAY_NAME))
}

pub fn int(val: i64, source: Option<Span>) -> Arc<ExprNode> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_int_obj =
            ObjectType::int_obj_type().create_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i64_type().const_int(val as u64, false);
        gc.store_obj_field(ptr_to_int_obj, int_type(gc.context), 1, value);
        ptr_to_int_obj
    });
    expr_lit(generator, vec![], val.to_string(), int_lit_ty(), source)
}

pub fn bool(val: bool, source: Option<Span>) -> Arc<ExprNode> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_obj = ObjectType::bool_obj_type().create_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i8_type().const_int(val as u64, false);
        gc.store_obj_field(ptr_to_obj, bool_type(gc.context), 1, value);
        ptr_to_obj
    });
    expr_lit(generator, vec![], val.to_string(), bool_lit_ty(), source)
}

fn add_lit(lhs: &str, rhs: &str) -> Arc<ExprNode> {
    let lhs_str = NameSpacedName::local(lhs);
    let rhs_str = NameSpacedName::local(rhs);
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let name = format!("add {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .get_var_field(&lhs_str, 1, int_type(gc.context))
            .into_int_value();
        let rhs_val = gc
            .get_var_field(&rhs_str, 1, int_type(gc.context))
            .into_int_value();
        let value = gc.builder().build_int_add(lhs_val, rhs_val, "add");
        let ptr_to_int_obj = ObjectType::int_obj_type().create_obj(gc, Some(name_cloned.as_str()));
        gc.store_obj_field(ptr_to_int_obj, int_type(gc.context), 1, value);
        gc.release(gc.get_var(&lhs_str).ptr.get(gc));
        gc.release(gc.get_var(&rhs_str).ptr.get(gc));
        ptr_to_int_obj
    });
    expr_lit(generator, free_vars, name, int_lit_ty(), None)
}

pub fn add() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("lhs", None, None),
        expr_abs(var_local("rhs", None, None), add_lit("lhs", "rhs"), None),
        None,
    );
    let scm = Scheme::from_type(type_fun(int_lit_ty(), type_fun(int_lit_ty(), int_lit_ty())));
    (expr, scm)
}

fn eq_lit(lhs: &str, rhs: &str) -> Arc<ExprNode> {
    let lhs_str = NameSpacedName::local(lhs);
    let rhs_str = NameSpacedName::local(rhs);
    let name = format!("eq {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .get_var_field(&lhs_str, 1, int_type(gc.context))
            .into_int_value();
        let rhs_val = gc
            .get_var_field(&rhs_str, 1, int_type(gc.context))
            .into_int_value();
        let value = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, lhs_val, rhs_val, "eq");
        let value = gc.builder().build_int_cast(
            value,
            ObjectFieldType::Bool
                .to_basic_type(gc.context)
                .into_int_type(),
            "eq_bool",
        );
        let ptr_to_obj = ObjectType::bool_obj_type().create_obj(gc, Some(name_cloned.as_str()));
        gc.store_obj_field(ptr_to_obj, bool_type(gc.context), 1, value);
        gc.release(gc.get_var(&lhs_str).ptr.get(gc));
        gc.release(gc.get_var(&rhs_str).ptr.get(gc));
        ptr_to_obj
    });
    expr_lit(generator, free_vars, name, bool_lit_ty(), None)
}

// eq = \lhs: a -> \rhs: a -> eq_lit(lhs, rhs): Bool
pub fn eq() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("lhs", None, None),
        expr_abs(var_local("rhs", None, None), eq_lit("lhs", "rhs"), None),
        None,
    );
    let scm = Scheme::generalize(
        HashMap::from([("a".to_string(), kind_star())]),
        vec![],
        type_fun(
            type_tyvar_star("a"),
            type_fun(type_tyvar_star("a"), bool_lit_ty()),
        ),
    );
    (expr, scm)
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
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let fixf = gc.get_var(&NameSpacedName::local(SELF_NAME)).ptr.get(gc);
        let x = gc.get_var(&x_str).ptr.get(gc);
        let f = gc.get_var(&f_str).ptr.get(gc);
        let f_fixf = gc.apply_lambda(f, fixf);
        let f_fixf_x = gc.apply_lambda(f_fixf, x);
        f_fixf_x
    });
    expr_lit(generator, free_vars, name, type_tyvar_star(b), None)
}

// fix = \f: ((a -> b) -> (a -> b)) -> \x: a -> fix_lit(b, f, x): b
pub fn fix() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("f", None, None),
        expr_abs(var_local("x", None, None), fix_lit("b", "f", "x"), None),
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
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, ArrayField] where ArrayField = [Size, PtrToBuffer].
        let size = gc
            .get_var_field(&size_str, 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.get_var(&size_str).ptr.get(gc));
        let value = gc.get_var(&value_str).ptr.get(gc);
        let array = ObjectType::array_type().create_obj(gc, Some(name_cloned.as_str()));
        let array_ptr_ty = ptr_type(ObjectType::array_type().to_struct_type(gc.context));
        let array = gc.cast_pointer(array, array_ptr_ty);
        let array_field = gc
            .builder()
            .build_struct_gep(array, 1, "array_field")
            .unwrap();
        ObjectFieldType::initialize_array(gc, array_field, size, value);
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
        var_local("size", None, None),
        expr_abs(
            var_local("value", None, None),
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

// Implementation of readArray built-in function.
fn read_array_lit(a: &str, array: &str, idx: &str) -> Arc<ExprNode> {
    let array_str = NameSpacedName::local(array);
    let idx_str = NameSpacedName::local(idx);
    let name = format!("readArray {} {}", array, idx);
    let free_vars = vec![array_str.clone(), idx_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].
        let array_ptr_ty = ptr_type(ObjectType::array_type().to_struct_type(gc.context));
        let array = gc.get_var(&array_str).ptr.get(gc);
        let array = gc.cast_pointer(array, array_ptr_ty);
        let array_field = gc
            .builder()
            .build_struct_gep(array, 1, "array_field")
            .unwrap();
        let idx = gc
            .get_var_field(&idx_str, 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.get_var(&idx_str).ptr.get(gc));
        let elem = ObjectFieldType::read_array(gc, array_field, idx);
        gc.release(array);
        elem
    });
    expr_lit(generator, free_vars, name, type_tyvar_star(a), None)
}

// "readArray" built-in function.
// readArray = for<a> \arr: Array<a> -> \idx: Int -> (...read_array_lit(a, arr, idx)...): a
pub fn read_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("array", None, None),
        expr_abs(
            var_local("idx", None, None),
            read_array_lit("a", "array", "idx"),
            None,
        ),
        None,
    );
    let scm = Scheme::generalize(
        HashMap::from([("a".to_string(), kind_star())]),
        vec![],
        type_fun(
            type_tyapp(array_lit_ty(), type_tyvar_star("a")),
            type_fun(int_lit_ty(), type_tyvar_star("a")),
        ),
    );
    (expr, scm)
}

// Implementation of writeArray / writeArray! built-in function.
// is_unique_mode - if true, generate code that calls abort when given array is shared.
fn write_array_lit(
    a: &str,
    array: &str,
    idx: &str,
    value: &str,
    is_unique_version: bool,
) -> Arc<ExprNode> {
    let array_str = NameSpacedName::local(array);
    let idx_str = NameSpacedName::local(idx);
    let value_str = NameSpacedName::local(value);
    let func_name = String::from({
        if is_unique_version {
            "writeArray!"
        } else {
            "writeArray"
        }
    });
    let name = format!("{} {} {} {}", func_name, array, idx, value);
    let name_cloned = name.clone();
    let free_vars = vec![array_str.clone(), idx_str.clone(), value_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].

        // Get argments
        let array = gc.get_var(&array_str).ptr.get(gc);
        let idx = gc
            .get_var_field(&idx_str, 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.get_var(&idx_str).ptr.get(gc));
        let value = gc.get_var(&value_str).ptr.get(gc);

        // Get array field.
        let array_str_ty = ObjectType::array_type().to_struct_type(gc.context);
        let array = gc.cast_pointer(array, ptr_type(array_str_ty));
        let array_field = gc.builder().build_struct_gep(array, 1, "").unwrap();

        // Get refcnt.
        let refcnt = gc
            .load_obj_field(array, control_block_type(gc.context), 0)
            .into_int_value();

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
        let cloned_array = ObjectType::array_type().create_obj(gc, Some(name_cloned.as_str()));
        let cloned_array = gc.cast_pointer(cloned_array, ptr_type(array_str_ty));
        let cloned_array_field = gc.builder().build_struct_gep(cloned_array, 1, "").unwrap();
        ObjectFieldType::clone_array(gc, array_field, cloned_array_field);
        gc.release(array); // Given array should be released here.
        let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(cont_bb);

        // Implement cont_bb
        gc.builder().position_at_end(cont_bb);

        // Build phi value of array and array_field.
        let array_phi = gc.builder().build_phi(array.get_type(), "array_phi");
        assert_eq!(array.get_type(), cloned_array.get_type());
        array_phi.add_incoming(&[(&array, current_bb), (&cloned_array, succ_of_shared_bb)]);
        let array = array_phi.as_basic_value().into_pointer_value();
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
        ObjectFieldType::write_array(gc, array_field, idx, value);
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

// writeArray built-in function.
// writeArray = for<a> \arr: Array<a> -> \idx: Int -> \value: a -> (...write_array_lit(a, arr, idx)...): Array<a>
pub fn write_array_common(is_unique_version: bool) -> (Arc<ExprNode>, Arc<Scheme>) {
    let expr = expr_abs(
        var_local("array", None, None),
        expr_abs(
            var_local("idx", None, None),
            expr_abs(
                var_local("value", None, None),
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
            array_ty.clone(),
            type_fun(int_lit_ty(), type_fun(type_tyvar_star("a"), array_ty)),
        ),
    );
    (expr, scm)
}

// writeArray built-in function.
pub fn write_array() -> (Arc<ExprNode>, Arc<Scheme>) {
    write_array_common(false)
}

// writeArray! built-in function.
pub fn write_array_unique() -> (Arc<ExprNode>, Arc<Scheme>) {
    write_array_common(true)
}

// `new` built-in function for a given struct.
pub fn struct_new_lit(struct_name: &str, field_names: Vec<String>) -> Arc<ExprNode> {
    let free_vars = field_names
        .iter()
        .map(|name| NameSpacedName::local(name))
        .collect();
    let name = format!("{}.new {}", struct_name, field_names.join(" "));
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Get field values.
        let field_ptrs: Vec<PointerValue> = field_names
            .iter()
            .map(|name| gc.get_var(&NameSpacedName::local(name)).ptr.get(gc))
            .collect();

        // Create struct object.
        let obj_ty = ObjectType::struct_type(field_names.len());
        let str_ptr = obj_ty.create_obj(gc, Some(&name_cloned));

        // Set fields.
        let struct_ty = obj_ty.to_struct_type(gc.context);
        for (i, field_ptr) in field_ptrs.iter().enumerate() {
            gc.store_obj_field(
                str_ptr,
                struct_ty,
                i as u32 + 1,
                field_ptr.as_basic_value_enum(),
            );
        }

        str_ptr
    });
    expr_lit(
        generator,
        free_vars,
        name,
        type_tycon(&tycon(struct_name)),
        None,
    )
}

// `new` built-in function for a given struct.
pub fn struct_new(struct_name: &str, definition: &Struct) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Check there is no duplication of field names.
    let mut fields_set: HashMap<String, i32> = HashMap::new();
    for field in &definition.fields {
        if !fields_set.contains_key(&field.name) {
            fields_set.insert(field.name.clone(), 0);
        }
        *fields_set.get_mut(&field.name).unwrap() += 1;
        if fields_set[&field.name] >= 2 {
            error_exit(&format!(
                "error: in definition of struct `{}`, field `{}` is duplicated.",
                struct_name, &field.name
            ));
        }
    }
    let mut expr = struct_new_lit(
        struct_name,
        definition.fields.iter().map(|f| f.name.clone()).collect(),
    );
    let mut ty = type_tycon(&tycon(struct_name));
    for field in definition.fields.iter().rev() {
        expr = expr_abs(var_local(&field.name, None, None), expr, None);
        ty = type_fun(field.ty.clone(), ty);
    }
    let scm = Scheme::generalize(HashMap::new(), vec![], ty);
    (expr, scm)
}

// `get` built-in function for a given struct.
pub fn struct_get_lit(
    var_name: &str,
    field_count: usize, // number of fields in this struct
    field_idx: usize,
    field_ty: Arc<TypeNode>,
    struct_name: &str,
    field_name: &str,
) -> Arc<ExprNode> {
    let var_name_clone = NameSpacedName::local(var_name);
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Get struct object.
        let str_ptr = gc.get_var(&var_name_clone).ptr.get(gc);

        // Extract field.
        let str_ty = ObjectType::struct_type(field_count).to_struct_type(gc.context);
        let field_ptr = gc.load_obj_field(str_ptr, str_ty, field_idx as u32 + 1);
        let field_ptr = field_ptr.into_pointer_value();

        // Retain field and release struct.
        gc.retain(field_ptr);
        gc.release(str_ptr);

        field_ptr
    });
    let free_vars = vec![NameSpacedName::local(var_name)];
    let name = format!("{}.get{}", struct_name, capitalize_head(field_name));
    expr_lit(generator, free_vars, name, field_ty, None)
}

// `get` built-in function for a given struct.
pub fn struct_get(
    struct_name: &str,
    definition: &Struct,
    field_name: &str,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition
        .fields
        .iter()
        .enumerate()
        .find(|(_i, f)| f.name == field_name);
    if field.is_none() {
        error_exit(&format!(
            "error: no field `{}` found in the struct `{}`.",
            &field_name, struct_name,
        ));
    }
    let (field_idx, field) = field.unwrap();

    let field_count = definition.fields.len();
    let str_ty = type_tycon(&tycon(struct_name));
    let expr = expr_abs(
        var_local("f", None, None),
        struct_get_lit(
            "f",
            field_count,
            field_idx,
            field.ty.clone(),
            struct_name,
            field_name,
        ),
        None,
    );
    let ty = type_fun(str_ty, field.ty.clone());
    let scm = Scheme::generalize(HashMap::new(), vec![], ty);
    (expr, scm)
}

// `get` built-in function for a given struct.
pub fn struct_mod_lit(
    f_name: &str,
    x_name: &str,
    field_count: usize, // number of fields in this struct
    field_idx: usize,
    struct_name: &str,
    field_name: &str,
    is_unique_version: bool,
) -> Arc<ExprNode> {
    let name = format!(
        "{}.mod{}{} {} {}",
        struct_name,
        field_name,
        if is_unique_version { "!" } else { "" },
        f_name,
        x_name
    );
    let f_name = NameSpacedName::local(f_name);
    let x_name = NameSpacedName::local(x_name);
    let free_vars = vec![f_name.clone(), x_name.clone()];
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Make types
        let obj_ty = ObjectType::struct_type(field_count);
        let str_ty = obj_ty.to_struct_type(gc.context);

        // Get arguments
        let modfier = gc.get_var(&f_name).ptr.get(gc);
        let str = gc.get_var(&x_name).ptr.get(gc);
        let str = gc.cast_pointer(str, ptr_type(str_ty));

        // If str is not unique, then first clone it.
        let refcnt = gc
            .load_obj_field(str, control_block_type(gc.context), 0)
            .into_int_value();

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

        // In shared_bb, create new struct and clone fields.
        gc.builder().position_at_end(shared_bb);
        if is_unique_version {
            // In case of unique version, panic in this case.
            gc.panic(&format!("The argument of mod! is shared!\n"));
        }
        let cloned_str = obj_ty.create_obj(gc, Some(name_cloned.as_str()));
        let cloned_str = gc.cast_pointer(cloned_str, ptr_type(str_ty));
        for i in 0..field_count {
            let field_idx = 1 as u32 + i as u32;
            let field = gc
                .load_obj_field(str, str_ty, field_idx)
                .into_pointer_value();
            gc.retain(field);
            gc.store_obj_field(cloned_str, str_ty, field_idx, field);
        }
        gc.release(str); // Given struct should be released here.
        let succ_of_shared_bb = gc.builder().get_insert_block().unwrap();
        gc.builder().build_unconditional_branch(cont_bb);

        // Implement cont_bb
        gc.builder().position_at_end(cont_bb);

        // Build phi value
        let str_phi = gc.builder().build_phi(str.get_type(), "str_phi");
        assert_eq!(str.get_type(), cloned_str.get_type());
        str_phi.add_incoming(&[(&str, current_bb), (&cloned_str, succ_of_shared_bb)]);
        let str = str_phi.as_basic_value().into_pointer_value();

        // Modify field
        let field = gc
            .load_obj_field(str, str_ty, 1 + field_idx as u32)
            .into_pointer_value();
        let field = gc.apply_lambda(modfier, field);
        gc.store_obj_field(str, str_ty, 1 + field_idx as u32, field);

        str
    });
    expr_lit(
        generator,
        free_vars,
        name,
        type_tycon(&tycon(struct_name)),
        None,
    )
}

// `mod` built-in function for a given struct.
pub fn struct_mod(
    struct_name: &str,
    definition: &Struct,
    field_name: &str,
    is_unique_version: bool,
) -> (Arc<ExprNode>, Arc<Scheme>) {
    // Find the index of `field_name` in the given struct.
    let field = definition
        .fields
        .iter()
        .enumerate()
        .find(|(_i, f)| f.name == field_name);
    if field.is_none() {
        error_exit(&format!(
            "error: no field `{}` found in the struct `{}`.",
            &field_name, struct_name,
        ));
    }
    let (field_idx, field) = field.unwrap();

    let field_count = definition.fields.len();
    let str_ty = type_tycon(&tycon(struct_name));
    let expr = expr_abs(
        var_local("f", None, None),
        expr_abs(
            var_local("x", None, None),
            struct_mod_lit(
                "f",
                "x",
                field_count,
                field_idx,
                struct_name,
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
    let scm = Scheme::generalize(HashMap::new(), vec![], ty);
    (expr, scm)
}

// Add bult-in functions to a given ast.
pub fn add_builtin_symbols(program: &mut FixModule) {
    fn add_global(
        program: &mut FixModule,
        ns: &[&str],
        name: &str,
        (expr, scm): (Arc<ExprNode>, Arc<Scheme>),
    ) {
        program.add_global_object(NameSpacedName::from_strs(ns, name), (expr, scm));
    }
    add_global(program, &[PRELUDE_NAME], "add", add());
    add_global(program, &[PRELUDE_NAME], "eq", eq());
    add_global(program, &[PRELUDE_NAME], "fix", fix());
    add_global(program, &[PRELUDE_NAME], "newArray", new_array());
    add_global(program, &[PRELUDE_NAME], "readArray", read_array());
    add_global(program, &[PRELUDE_NAME], "writeArray", write_array());
    add_global(
        program,
        &[PRELUDE_NAME],
        "writeArray!",
        write_array_unique(),
    );
    for decl in &program.type_decls.clone() {
        match &decl.value {
            TypeDeclValue::Struct(str) => {
                let module_name = program.name.clone();
                let ns = vec![module_name.as_str(), decl.name.as_str()];
                add_global(program, ns.as_slice(), "new", struct_new(&decl.name, str));
                for field in &str.fields {
                    add_global(
                        program,
                        ns.as_slice(),
                        &format!("get{}", capitalize_head(&field.name)),
                        struct_get(&decl.name, str, &field.name),
                    );
                    for is_unique in [false, true] {
                        add_global(
                            program,
                            ns.as_slice(),
                            &format!(
                                "mod{}{}",
                                capitalize_head(&field.name),
                                if is_unique { "!" } else { "" }
                            ),
                            struct_mod(&decl.name, str, &field.name, is_unique),
                        );
                    }
                }
            }
        }
    }
}
