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
        // Array = [ControlBlock, PtrToArrayField], and ArrayField = [Size, PtrToBuffer].
        let size = gc
            .scope_get_field(&size_str, 1, int_type(gc.context))
            .into_int_value();
        let value = gc.scope_get(&value_str).ptr;
        let array_struct = ObjectType::array_type().to_struct_type(gc.context);
        let array = ObjectType::array_type().create_obj(gc, Some(name_cloned.as_str()));
        let array_field = ObjectFieldType::create_array(gc, size, value);
        gc.store_obj_field(array, array_struct, 1, array_field);
        array
    });
    lit(generator, free_vars, name)
}

// newArray built-in function.
pub fn newArray() -> Arc<ExprInfo> {
    lam(
        var_var("size"),
        lam(var_var("value"), new_array_lit("size", "value")),
    )
}
