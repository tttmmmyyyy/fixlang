use std::iter;

use super::*;

#[derive(Eq, Hash, PartialEq, Clone)]
pub enum ObjectFieldType {
    ControlBlock,
    LambdaFunction,
    SubObject,
    Int,
    Bool,
    Array,
}

impl ObjectFieldType {
    pub fn to_basic_type<'ctx>(&self, context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(context).into(),
            ObjectFieldType::LambdaFunction => ptr_to_lambda_function_type(context).into(),
            ObjectFieldType::SubObject => ptr_to_object_type(context).into(),
            ObjectFieldType::Int => context.i64_type().into(),
            ObjectFieldType::Bool => context.i8_type().into(),
            ObjectFieldType::Array => context
                .struct_type(
                    &[
                        context.i64_type().into(), // size
                        ptr_to_object_type(context)
                            .ptr_type(AddressSpace::Generic)
                            .into(), // ptr to buffer
                    ],
                    false,
                )
                .into(),
        }
    }

    // Get fields (size and buffer) from array.
    pub fn get_size_and_buffer_of_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        array: PointerValue<'c>,
    ) -> (IntValue<'c>, PointerValue<'c>) {
        let array_struct = ObjectFieldType::Array
            .to_basic_type(gc.context)
            .into_struct_type();
        let size = gc.load_obj_field(array, array_struct, 0).into_int_value();
        let buffer = gc
            .load_obj_field(array, array_struct, 1)
            .into_pointer_value();
        // let array = gc.cast_pointer(array, ptr_type(array_struct));
        // let buffer = gc.builder().build_struct_gep(array, 1, "buffer").unwrap();
        (size, buffer)
    }

    // Take array and generate code iterating its elements.
    fn loop_over_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        ptr_to_array: PointerValue<'c>,
        loop_body: impl Fn(
            &mut GenerationContext<'c, 'm>,
            IntValue<'c>,     /* idx */
            IntValue<'c>,     /* size */
            PointerValue<'c>, /* buffer */
        ),
        after_loop: impl Fn(
            &mut GenerationContext<'c, 'm>,
            IntValue<'c>,     /* size */
            PointerValue<'c>, /* buffer */
        ),
    ) {
        // Get fields (size, ptr_to_buffer).
        let (size, ptr_to_buffer) = Self::get_size_and_buffer_of_array(gc, ptr_to_array);

        // Append blocks: loop_check, loop_body and after_loop.
        let current_bb = gc.builder().get_insert_block().unwrap();
        let dtor_func = current_bb.get_parent().unwrap();
        let loop_check_bb = gc
            .context
            .append_basic_block(dtor_func, "loop_release_array_elements");
        let loop_body_bb = gc.context.append_basic_block(dtor_func, "loop_body");
        let after_loop_bb = gc.context.append_basic_block(dtor_func, "after_loop");

        // Allocate and initialize loop counter.
        let counter_type = gc.context.i64_type();
        let counter_ptr = gc
            .builder()
            .build_alloca(counter_type, "release_loop_counter");
        gc.builder()
            .build_store(counter_ptr, counter_type.const_zero());

        // Jump to loop_check bb.
        gc.builder().build_unconditional_branch(loop_check_bb);

        // Implement loop_check bb.
        gc.builder().position_at_end(loop_check_bb);
        let counter_val = gc
            .builder()
            .build_load(counter_ptr, "counter_val")
            .into_int_value();
        let is_end = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, counter_val, size, "is_end");
        gc.builder()
            .build_conditional_branch(is_end, after_loop_bb, loop_body_bb);

        // Implement loop_body bb.
        gc.builder().position_at_end(loop_body_bb);

        // Generate code of loop body.
        loop_body(gc, counter_val, size, ptr_to_buffer);

        // Increment counter.
        let incremented_counter_val = gc.builder().build_int_add(
            counter_val,
            counter_type.const_int(1, false),
            "incremented_counter_val",
        );
        gc.builder()
            .build_store(counter_ptr, incremented_counter_val);

        // Jump back to loop_check bb.
        gc.builder().build_unconditional_branch(loop_check_bb);

        // Generate code after loop.
        gc.builder().position_at_end(after_loop_bb);
        after_loop(gc, size, ptr_to_buffer);
    }

    // Take pointer to array = [size, ptr_to_buffer], call release of ptr_to_bufer[i] for all i and free ptr_to_buffer.
    pub fn destruct_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        ptr_to_array: PointerValue<'c>,
    ) {
        // In loop body, release object of idx = counter_val.
        fn loop_body<'c, 'm>(
            gc: &mut GenerationContext<'c, 'm>,
            idx: IntValue<'c>,
            _size: IntValue<'c>,
            ptr_to_buffer: PointerValue<'c>,
        ) {
            let ptr_to_obj_ptr = unsafe {
                gc.builder()
                    .build_gep(ptr_to_buffer, &[idx.into()], "ptr_to_elem_of_array")
            };
            let obj_ptr = gc
                .builder()
                .build_load(ptr_to_obj_ptr, "elem_of_array")
                .into_pointer_value();
            gc.release(obj_ptr);
        }

        // After loop, free buffer.
        fn after_loop<'c, 'm>(
            gc: &mut GenerationContext<'c, 'm>,
            _size: IntValue<'c>,
            ptr_to_buffer: PointerValue<'c>,
        ) {
            gc.builder().build_free(ptr_to_buffer);
        }

        // Generate loop.
        Self::loop_over_array(gc, ptr_to_array, loop_body, after_loop);
    }

    // Initialize an array.
    pub fn initialize_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        array_ptr: PointerValue<'c>,
        size: IntValue<'c>,
        value: PointerValue<'c>,
    ) {
        assert_eq!(size.get_type(), gc.context.i64_type());
        assert_eq!(value.get_type(), ptr_to_object_type(gc.context));

        let array_struct = ObjectFieldType::Array
            .to_basic_type(gc.context)
            .into_struct_type();

        // Set size.
        gc.store_obj_field(array_ptr, array_struct, 0, size);

        // Allocate buffer and set it to array.
        let buffer_ptr = gc
            .builder()
            .build_array_malloc(ptr_to_object_type(gc.context), size, "buffer_ptr")
            .unwrap();
        gc.store_obj_field(array_ptr, array_struct, 1, buffer_ptr);

        // Initialize elements
        {
            // In loop body, retain value and store it at idx.
            let loop_body = |gc: &mut GenerationContext<'c, 'm>,
                             idx: IntValue<'c>,
                             _size: IntValue<'c>,
                             ptr_to_buffer: PointerValue<'c>| {
                gc.retain(value);
                let ptr_to_obj_ptr = unsafe {
                    gc.builder()
                        .build_gep(ptr_to_buffer, &[idx.into()], "ptr_to_elem_of_array")
                };
                gc.builder().build_store(ptr_to_obj_ptr, value);
            };

            // After loop, release value.
            let after_loop = |gc: &mut GenerationContext<'c, 'm>,
                              _size: IntValue<'c>,
                              _ptr_to_buffer: PointerValue<'c>| {
                gc.release(value);
            };

            // Generate loop.
            // NOTE: if you see error at here, try `cargo clean`.
            Self::loop_over_array(gc, array_ptr, loop_body, after_loop);
        }
    }

    // Panic if idx is out_of_range for the array.
    pub fn panic_if_out_of_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        array: PointerValue<'c>,
        idx: IntValue<'c>,
    ) {
        let (size, _ptr_to_buffer) = Self::get_size_and_buffer_of_array(gc, array);
        let curr_bb = gc.builder().get_insert_block().unwrap();
        let curr_func = curr_bb.get_parent().unwrap();
        let is_out_of_range =
            gc.builder()
                .build_int_compare(IntPredicate::UGE, idx, size, "is_out_of_ramge");
        let out_of_range_bb = gc.context.append_basic_block(curr_func, "out_of_range_bb");
        let in_range_bb = gc.context.append_basic_block(curr_func, "in_range_bb");
        gc.builder()
            .build_conditional_branch(is_out_of_range, out_of_range_bb, in_range_bb);
        gc.builder().position_at_end(out_of_range_bb);
        gc.panic("Index out of range!");
        gc.builder().build_unreachable();
        gc.builder().position_at_end(in_range_bb);
    }

    // Read an element of array.
    // Returned object is already retained.
    pub fn read_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        array: PointerValue<'c>,
        idx: IntValue<'c>,
    ) -> PointerValue<'c> {
        // Panic if out_of_range.
        Self::panic_if_out_of_array(gc, array, idx);

        // Get fields (size, ptr_to_buffer).
        let (_size, ptr_to_buffer) = Self::get_size_and_buffer_of_array(gc, array);

        // Get element.
        let ptr_to_elem = unsafe {
            gc.builder()
                .build_gep(ptr_to_buffer, &[idx.into()], "ptr_to_elem_of_array")
        };
        let elem = gc
            .builder()
            .build_load(ptr_to_elem, "elem")
            .into_pointer_value();

        // Retain element and return it.
        gc.retain(elem);
        elem
    }

    // Write an element into array.
    pub fn write_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        array: PointerValue<'c>,
        idx: IntValue<'c>,
        value: PointerValue<'c>,
    ) {
        // Panic if out_of_range.
        Self::panic_if_out_of_array(gc, array, idx);

        // Get fields (size, ptr_to_buffer).
        let (_size, ptr_to_buffer) = Self::get_size_and_buffer_of_array(gc, array);

        // Check if out_of_range.
        // TODO!

        // Get ptr to the place at idx.
        let place = unsafe {
            gc.builder()
                .build_gep(ptr_to_buffer, &[idx.into()], "ptr_to_elem_of_array")
        };

        // Release element that is already at the place.
        let elem = gc.builder().build_load(place, "elem").into_pointer_value();
        gc.release(elem);

        // Insert the given value to the place.
        gc.builder().build_store(place, value);
    }

    // Clone an array
    pub fn clone_array<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        src: PointerValue<'c>,
        dst: PointerValue<'c>,
    ) {
        let array_struct = ObjectFieldType::Array
            .to_basic_type(gc.context)
            .into_struct_type();

        // Get fields (size, ptr_to_buffer) of src.
        let (src_size, src_buffer) = Self::get_size_and_buffer_of_array(gc, src);

        // Copy size.
        gc.store_obj_field(dst, array_struct, 0, src_size);

        // Allocate buffer and set it to dst.
        let dst_buffer = gc
            .builder()
            .build_array_malloc(ptr_to_object_type(gc.context), src_size, "dst_buffer")
            .unwrap();
        gc.store_obj_field(dst, array_struct, 1, dst_buffer);

        // Clone each elements.
        {
            // In loop body, retain value and store it at idx.
            let loop_body = |gc: &mut GenerationContext<'c, 'm>,
                             idx: IntValue<'c>,
                             _size: IntValue<'c>,
                             _ptr_to_buffer: PointerValue<'c>| {
                let ptr_to_src_elem = unsafe {
                    gc.builder()
                        .build_gep(src_buffer, &[idx.into()], "ptr_to_src_elem")
                };
                let ptr_to_dst_elem = unsafe {
                    gc.builder()
                        .build_gep(dst_buffer, &[idx.into()], "ptr_to_dst_elem")
                };
                let src_elem = gc
                    .builder()
                    .build_load(ptr_to_src_elem, "src_elem")
                    .into_pointer_value();
                gc.retain(src_elem);
                gc.builder().build_store(ptr_to_dst_elem, src_elem);
            };

            // After loop, do nothing.
            let after_loop = |_gc: &mut GenerationContext<'c, 'm>,
                              _size: IntValue<'c>,
                              _ptr_to_buffer: PointerValue<'c>| {};

            Self::loop_over_array(gc, src, loop_body, after_loop);
        }
    }
}

#[derive(Eq, Hash, PartialEq, Clone)]
pub struct ObjectType {
    pub field_types: Vec<ObjectFieldType>,
}

impl ObjectType {
    pub fn to_struct_type<'ctx>(&self, context: &'ctx Context) -> StructType<'ctx> {
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

    pub fn lam_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::LambdaFunction]) // Other fields for captured objects may exist but omitted here.
    }

    pub fn int_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::Int])
    }

    pub fn bool_obj_type() -> Self {
        Self::shared_obj_type(vec![ObjectFieldType::Bool])
    }

    pub fn array_type() -> Self {
        let fields = vec![ObjectFieldType::Array];
        Self::shared_obj_type(fields)
    }

    pub fn struct_type(field_count: usize) -> Self {
        let fields: Vec<ObjectFieldType> = iter::repeat(ObjectFieldType::SubObject)
            .take(field_count)
            .collect();
        Self::shared_obj_type(fields)
    }

    fn generate_func_dtor<'c, 'm>(&self, gc: &mut GenerationContext<'c, 'm>) -> FunctionValue<'c> {
        if gc
            .runtimes
            .contains_key(&RuntimeFunctions::Dtor(self.clone()))
        {
            return *gc
                .runtimes
                .get(&RuntimeFunctions::Dtor(self.clone()))
                .unwrap();
        }
        let struct_type = self.to_struct_type(gc.context);
        let func_type = dtor_type(gc.context);
        let func = gc.module.add_function("dtor", func_type, None);
        let bb = gc.context.append_basic_block(func, "entry");

        let _builder_guard = gc.push_builder();

        gc.builder().position_at_end(bb);
        let ptr_to_obj = func.get_first_param().unwrap().into_pointer_value();
        for (i, ft) in self.field_types.iter().enumerate() {
            match ft {
                ObjectFieldType::SubObject => {
                    let ptr_to_subobj = gc
                        .load_obj_field(ptr_to_obj, struct_type, i as u32)
                        .into_pointer_value();
                    gc.release(ptr_to_subobj);
                }
                ObjectFieldType::ControlBlock => {}
                ObjectFieldType::Int => {}
                ObjectFieldType::LambdaFunction => {}
                ObjectFieldType::Bool => {}
                ObjectFieldType::Array => {
                    let ptr_to_struct = gc.cast_pointer(ptr_to_obj, ptr_type(struct_type));
                    let ptr_to_array = gc
                        .builder()
                        .build_struct_gep(ptr_to_struct, i as u32, "ptr_to_array")
                        .unwrap();
                    ObjectFieldType::destruct_array(gc, ptr_to_array);
                }
            }
        }
        gc.builder().build_return(None);

        // gc.pop_builder();
        gc.runtimes
            .insert(RuntimeFunctions::Dtor(self.clone()), func);
        func
    }

    // Create an object.
    pub fn create_obj<'c, 'm>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        name: Option<&str>,
    ) -> PointerValue<'c> {
        let context = gc.context;
        let struct_type = self.to_struct_type(context);
        // NOTE: Only once allocation is needed since we don't implement weak_ptr
        let ptr_to_obj = gc
            .builder()
            .build_malloc(struct_type, "ptr_to_obj")
            .unwrap();

        let mut object_id = obj_id_type(gc.context).const_int(0, false);

        if SANITIZE_MEMORY {
            let string_ptr = name.unwrap_or("N/A");
            let string_ptr = gc
                .builder()
                .build_global_string_ptr(string_ptr, "name_of_obj");
            let string_ptr = string_ptr.as_pointer_value();
            let string_ptr = gc.builder().build_pointer_cast(
                string_ptr,
                gc.context.i8_type().ptr_type(AddressSpace::Generic),
                "name_of_obj_i8ptr",
            );
            let ptr = gc.cast_pointer(ptr_to_obj, ptr_to_object_type(gc.context));
            let obj_id = gc.call_runtime(
                RuntimeFunctions::ReportMalloc,
                &[ptr.into(), string_ptr.into()],
            );
            object_id = obj_id.try_as_basic_value().unwrap_left().into_int_value();
        }

        for (i, ft) in self.field_types.iter().enumerate() {
            match ft {
                ObjectFieldType::ControlBlock => {
                    let ptr_to_control_block = gc
                        .builder()
                        .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_control_block")
                        .unwrap();
                    let ptr_to_refcnt = gc
                        .builder()
                        .build_struct_gep(ptr_to_control_block, 0, "ptr_to_refcnt")
                        .unwrap();
                    // The initial value of refcnt should be one (as std::make_shared of C++ does).
                    gc.builder()
                        .build_store(ptr_to_refcnt, refcnt_type(context).const_int(1, false));
                    let ptr_to_dtor_field = gc
                        .builder()
                        .build_struct_gep(ptr_to_control_block, 1, "ptr_to_dtor_field")
                        .unwrap();
                    let dtor = self.generate_func_dtor(gc);
                    gc.builder()
                        .build_store(ptr_to_dtor_field, dtor.as_global_value().as_pointer_value());

                    if SANITIZE_MEMORY {
                        let ptr_to_obj_id = gc
                            .builder()
                            .build_struct_gep(ptr_to_control_block, 2, "ptr_to_obj_id")
                            .unwrap();
                        gc.builder().build_store(ptr_to_obj_id, object_id);
                    }
                }
                ObjectFieldType::Int => {}
                ObjectFieldType::SubObject => {}
                ObjectFieldType::LambdaFunction => {}
                ObjectFieldType::Bool => {}
                ObjectFieldType::Array => {}
            }
        }
        ptr_to_obj
    }
}

pub fn refcnt_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

fn _ptr_to_refcnt_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    refcnt_type(context).ptr_type(AddressSpace::Generic)
}

pub fn obj_id_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

pub fn ptr_to_object_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    context.i8_type().ptr_type(AddressSpace::Generic)
}

fn dtor_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    context
        .void_type()
        .fn_type(&[ptr_to_object_type(context).into()], false)
}

fn ptr_to_dtor_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    dtor_type(context).ptr_type(AddressSpace::Generic)
}

pub fn control_block_type<'ctx>(context: &'ctx Context) -> StructType<'ctx> {
    let mut fields = vec![
        refcnt_type(context).into(),
        ptr_to_dtor_type(context).into(),
    ];
    if SANITIZE_MEMORY {
        fields.push(obj_id_type(context).into())
    }
    context.struct_type(&fields, false)
}

pub fn ptr_to_control_block_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    control_block_type(context).ptr_type(AddressSpace::Generic)
}

pub fn lambda_function_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
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

pub fn lambda_type<'c>(context: &'c Context) -> StructType<'c> {
    ObjectType::lam_obj_type().to_struct_type(context)
}

pub fn int_type<'c>(context: &'c Context) -> StructType<'c> {
    ObjectType::int_obj_type().to_struct_type(context)
}

pub fn bool_type<'c>(context: &'c Context) -> StructType<'c> {
    ObjectType::bool_obj_type().to_struct_type(context)
}
