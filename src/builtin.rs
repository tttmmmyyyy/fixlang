// Implement built-in functions, (constructor of) types, etc.
use super::*;

pub fn int(val: i64) -> Arc<ExprInfo> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_int_obj =
            ObjectType::int_obj_type().create_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i64_type().const_int(val as u64, false);
        gc.store_obj_field(ptr_to_int_obj, int_type(gc.context), 1, value);
        ptr_to_int_obj
    });
    lit(generator, vec![], val.to_string())
}

pub fn bool(val: bool) -> Arc<ExprInfo> {
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let ptr_to_obj = ObjectType::bool_obj_type().create_obj(gc, Some(val.to_string().as_str()));
        let value = gc.context.i8_type().const_int(val as u64, false);
        gc.store_obj_field(ptr_to_obj, bool_type(gc.context), 1, value);
        ptr_to_obj
    });
    lit(generator, vec![], val.to_string())
}

fn add_lit(lhs: &str, rhs: &str) -> Arc<ExprInfo> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let name = format!("add {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope_get_field(&lhs_str, 1, int_type(gc.context))
            .into_int_value();
        let rhs_val = gc
            .scope_get_field(&rhs_str, 1, int_type(gc.context))
            .into_int_value();
        let value = gc.builder().build_int_add(lhs_val, rhs_val, "add");
        let ptr_to_int_obj = ObjectType::int_obj_type().create_obj(gc, Some(name_cloned.as_str()));
        gc.store_obj_field(ptr_to_int_obj, int_type(gc.context), 1, value);
        gc.release(gc.scope_get(&lhs_str).ptr);
        gc.release(gc.scope_get(&rhs_str).ptr);
        ptr_to_int_obj
    });
    lit(generator, free_vars, name)
}

pub fn add() -> Arc<ExprInfo> {
    lam(var_var("lhs"), lam(var_var("rhs"), add_lit("lhs", "rhs")))
}

fn eq_lit(lhs: &str, rhs: &str) -> Arc<ExprInfo> {
    let lhs_str = String::from(lhs);
    let rhs_str = String::from(rhs);
    let name = format!("eq {} {}", lhs, rhs);
    let name_cloned = name.clone();
    let free_vars = vec![lhs_str.clone(), rhs_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let lhs_val = gc
            .scope_get_field(&lhs_str, 1, int_type(gc.context))
            .into_int_value();
        let rhs_val = gc
            .scope_get_field(&rhs_str, 1, int_type(gc.context))
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
        gc.release(gc.scope_get(&lhs_str).ptr);
        gc.release(gc.scope_get(&rhs_str).ptr);
        ptr_to_obj
    });
    lit(generator, free_vars, name)
}

pub fn eq() -> Arc<ExprInfo> {
    lam(var_var("lhs"), lam(var_var("rhs"), eq_lit("lhs", "rhs")))
}

fn fix_lit(f: &str, x: &str) -> Arc<ExprInfo> {
    let f_str = String::from(f);
    let x_str = String::from(x);
    let name = format!("fix {} {}", f_str, x_str);
    let free_vars = vec![String::from(SELF_NAME), f_str.clone(), x_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        let fixf = gc.scope_get(SELF_NAME).ptr;
        let x = gc.scope_get(&x_str).ptr;
        let f = gc.scope_get(&f_str).ptr;
        let f_fixf = gc.apply_lambda(f, fixf);
        let f_fixf_x = gc.apply_lambda(f_fixf, x);
        f_fixf_x
    });
    lit(generator, free_vars, name)
}

pub fn fix() -> Arc<ExprInfo> {
    lam(var_var("f"), lam(var_var("x"), fix_lit("f", "x")))
}

// Implementation of newArray built-in function.
fn new_array_lit(size: &str, value: &str) -> Arc<ExprInfo> {
    let size_str = String::from(size);
    let value_str = String::from(value);
    let name = format!("newArray {} {}", size, value);
    let name_cloned = name.clone();
    let free_vars = vec![size_str.clone(), value_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, ArrayField] where ArrayField = [Size, PtrToBuffer].
        let size = gc
            .scope_get_field(&size_str, 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.scope_get(&size_str).ptr);
        let value = gc.scope_get(&value_str).ptr;
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
    lit(generator, free_vars, name)
}

// newArray built-in function.
pub fn new_array() -> Arc<ExprInfo> {
    lam(
        var_var("size"),
        lam(var_var("value"), new_array_lit("size", "value")),
    )
}

// Implementation of readArray built-in function.
fn read_array_lit(array: &str, idx: &str) -> Arc<ExprInfo> {
    let array_str = String::from(array);
    let idx_str = String::from(idx);
    let name = format!("readArray {} {}", array, idx);
    let free_vars = vec![array_str.clone(), idx_str.clone()];
    let generator: Arc<LiteralGenerator> = Arc::new(move |gc| {
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].
        let array_ptr_ty = ptr_type(ObjectType::array_type().to_struct_type(gc.context));
        let array = gc.scope_get(array_str.as_str()).ptr;
        let array = gc.cast_pointer(array, array_ptr_ty);
        let array_field = gc
            .builder()
            .build_struct_gep(array, 1, "array_field")
            .unwrap();
        let idx = gc
            .scope_get_field(&idx_str, 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.scope_get(&idx_str).ptr);
        let elem = ObjectFieldType::read_array(gc, array_field, idx);
        gc.release(array);
        elem
    });
    lit(generator, free_vars, name)
}

// readArray built-in function.
pub fn read_array() -> Arc<ExprInfo> {
    lam(
        var_var("array"),
        lam(var_var("idx"), read_array_lit("array", "idx")),
    )
}

// Implementation of writeArray / writeArray! built-in function.
// is_unique_mode - if true, generate code that calls abort when given array is shared.
fn write_array_lit(array: &str, idx: &str, value: &str, is_unique_version: bool) -> Arc<ExprInfo> {
    let array_str = String::from(array);
    let idx_str = String::from(idx);
    let value_str = String::from(value);
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
        let array = gc.scope_get(array_str.as_str()).ptr;
        let idx = gc
            .scope_get_field(idx_str.as_str(), 1, int_type(gc.context))
            .into_int_value();
        gc.release(gc.scope_get(idx_str.as_str()).ptr);
        let value = gc.scope_get(value_str.as_str()).ptr;

        // Get array field.
        let array_str_ty = ObjectType::array_type().to_struct_type(gc.context);
        let array = gc.cast_pointer(array, ptr_type(array_str_ty));
        let array_field = gc.builder().build_struct_gep(array, 1, "").unwrap();

        // Get refcnt.
        let refcnt = gc
            .load_obj_field(array, control_block_type(gc.context), 0)
            .into_int_value();

        // Add unique / shared / cont bbs.
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
            gc.panic(format!("The argument of {} is shared!", func_name.as_str()).as_str());
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
    lit(generator, free_vars, name)
}

// writeArray built-in function.
pub fn write_array() -> Arc<ExprInfo> {
    lam(
        var_var("array"),
        lam(
            var_var("idx"),
            lam(
                var_var("value"),
                write_array_lit("array", "idx", "value", false),
            ),
        ),
    )
}

// writeArray! built-in function.
pub fn write_array_unique() -> Arc<ExprInfo> {
    lam(
        var_var("array"),
        lam(
            var_var("idx"),
            lam(
                var_var("value"),
                write_array_lit("array", "idx", "value", true),
            ),
        ),
    )
}
