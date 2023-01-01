use inkwell::{basic_block::BasicBlock, types::BasicType};

use super::*;

#[derive(Eq, PartialEq, Clone)]
pub enum ObjectFieldType {
    ControlBlock,
    DtorFunction,
    LambdaFunction(Arc<TypeNode>), // Specify type of lambda
    I64,
    Bool,
    SubObject(Arc<TypeNode>),
    UnionBuf,                    // pointer to buffer.
    UnionTag,                    // TODO: I should merge UnionTag and UnionBuf as like Array.
    ArraySizeBuf(Arc<TypeNode>), // [size, POINTER to buffer].
}

impl ObjectFieldType {
    pub fn to_basic_type<'c, 'm>(&self, gc: &GenerationContext<'c, 'm>) -> BasicTypeEnum<'c> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(gc.context).into(),
            ObjectFieldType::DtorFunction => ptr_to_dtor_type(gc.context).into(),
            ObjectFieldType::LambdaFunction(ty) => lambda_function_type(ty, gc)
                .ptr_type(AddressSpace::Generic)
                .into(),
            ObjectFieldType::SubObject(ty) => {
                get_object_type(ty, &vec![], gc.type_env()).to_embedded_type(gc)
            }
            ObjectFieldType::I64 => gc.context.i64_type().into(),
            ObjectFieldType::Bool => gc.context.i8_type().into(),
            ObjectFieldType::ArraySizeBuf(ty) => gc
                .context
                .struct_type(
                    &[
                        gc.context.i64_type().into(), // size
                        get_object_type(ty, &vec![], gc.type_env())
                            .to_embedded_type(gc)
                            .array_type(0)
                            .ptr_type(AddressSpace::Generic)
                            .into(), // buffer
                    ],
                    false,
                )
                .into(),
            ObjectFieldType::UnionTag => gc.context.i64_type().into(),
            ObjectFieldType::UnionBuf => ptr_to_object_type(gc.context).into(),
        }
    }

    // Get fields (size and buffer) from array.
    pub fn decompose_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        array: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
    ) -> (IntValue<'c>, PointerValue<'c>) {
        let array_struct = ObjectFieldType::ArraySizeBuf(elem_ty)
            .to_basic_type(gc)
            .into_struct_type();
        let size = gc.load_obj_field(array, array_struct, 0).into_int_value();
        let buffer = gc
            .load_obj_field(array, array_struct, 1)
            .into_pointer_value();
        (size, buffer)
    }

    // Get size of array.
    pub fn size_from_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        array: PointerValue<'c>,
    ) -> IntValue<'c> {
        let array_struct = ObjectFieldType::ArraySizeBuf(int_lit_ty() /* any will do */)
            .to_basic_type(gc)
            .into_struct_type();
        gc.load_obj_field(array, array_struct, 0).into_int_value()
    }

    // Take array and generate code iterating its elements.
    fn loop_over_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        loop_body: impl Fn(
            &mut GenerationContext<'c, 'm>,
            Object<'c>,       /* idx */
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
        let (size, buffer) = Self::decompose_array_size_buf(gc, size_buf, elem_ty);

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
        let stack_pos = gc.save_stack();

        // Generate code of loop body.
        loop_body(gc, Object::new(counter_ptr, int_lit_ty()), size, buffer);

        // Increment counter.
        let incremented_counter_val = gc.builder().build_int_add(
            counter_val,
            counter_type.const_int(1, false),
            "incremented_counter_val",
        );
        gc.builder()
            .build_store(counter_ptr, incremented_counter_val);

        // Jump back to loop_check bb.
        gc.restore_stack(stack_pos);
        gc.builder().build_unconditional_branch(loop_check_bb);

        // Generate code after loop.
        gc.builder().position_at_end(after_loop_bb);
        after_loop(gc, size, buffer);
    }

    // Take pointer to array_size_buf = [size, ptr_to_buffer], call release of ptr_to_bufer[i] for all i and free ptr_to_buffer.
    pub fn destruct_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
    ) {
        let elem_ty_clone = elem_ty.clone();

        // In loop body, release object of idx = counter_val.
        let loop_body = |gc: &mut GenerationContext<'c, 'm>,
                         idx: Object<'c>,
                         _size: IntValue<'c>,
                         ptr_to_buffer: PointerValue<'c>| {
            let idx = idx.load_field_nocap(gc, 0).into_int_value();
            let ptr = unsafe {
                gc.builder()
                    .build_gep(ptr_to_buffer, &[idx], "ptr_to_elem_of_array")
            };
            let obj = if elem_ty.is_box(gc.type_env()) {
                gc.builder()
                    .build_load(ptr, "elem_of_array")
                    .into_pointer_value()
            } else {
                ptr
            };
            gc.release(Object::new(obj, elem_ty.clone()));
        };

        // After loop, do nothing.
        fn after_loop<'c, 'm>(
            _gc: &mut GenerationContext<'c, 'm>,
            _size: IntValue<'c>,
            _ptr_to_buffer: PointerValue<'c>,
        ) {
        }

        // Generate loop.
        Self::loop_over_array_size_buf(gc, size_buf, elem_ty_clone, loop_body, after_loop);
    }

    // Initialize an array
    pub fn initialize_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        size: IntValue<'c>,
    ) {
        assert_eq!(size.get_type(), gc.context.i64_type());

        let array_struct = ObjectFieldType::ArraySizeBuf(elem_ty.clone())
            .to_basic_type(gc)
            .into_struct_type();

        // Set size.
        gc.store_obj_field(size_buf, array_struct, 0, size);

        // Allocate buffer and set it to array.
        let elem_type = get_object_type(&elem_ty, &vec![], gc.type_env()).to_embedded_type(gc);
        let buffer_ptr = gc
            .builder()
            .build_array_malloc(elem_type, size, "buffer_ptr")
            .unwrap();
        gc.store_obj_field(size_buf, array_struct, 1, buffer_ptr);
    }

    // Initialize an array by value.
    pub fn initialize_array_size_buf_by_value<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        size: IntValue<'c>,
        value: Object<'c>,
    ) {
        assert_eq!(value.ptr.get_type(), ptr_to_object_type(gc.context));

        Self::initialize_array_size_buf(gc, size_buf, elem_ty.clone(), size);

        // Initialize elements
        {
            // In loop body, retain value and store it at idx.
            let loop_body = |gc: &mut GenerationContext<'c, 'm>,
                             idx: Object<'c>,
                             _size: IntValue<'c>,
                             ptr_to_buffer: PointerValue<'c>| {
                let idx = idx.load_field_nocap(gc, 0).into_int_value();
                gc.retain(value.clone());
                let ptr_to_obj_ptr = unsafe {
                    gc.builder()
                        .build_gep(ptr_to_buffer, &[idx], "ptr_to_elem_of_array")
                };
                if value.is_box(gc.type_env()) {
                    gc.builder().build_store(ptr_to_obj_ptr, value.ptr);
                } else {
                    gc.builder()
                        .build_store(ptr_to_obj_ptr, value.load_nocap(gc));
                }
            };

            // After loop, release value.
            let after_loop = |gc: &mut GenerationContext<'c, 'm>,
                              _size: IntValue<'c>,
                              _ptr_to_buffer: PointerValue<'c>| {
                gc.release(value.clone());
            };

            // Generate loop.
            // NOTE: if you see error at here, try `cargo clean`.
            Self::loop_over_array_size_buf(gc, size_buf, elem_ty, loop_body, after_loop);
        }
    }

    // Initialize an array by map.
    pub fn initialize_array_size_buf_by_map<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        size: IntValue<'c>,
        map: Object<'c>,
    ) {
        // Initialize array.
        Self::initialize_array_size_buf(gc, size_buf, elem_ty.clone(), size);

        // Initialize elements
        {
            // In loop body, retain value and store it at idx.
            let loop_body = |gc: &mut GenerationContext<'c, 'm>,
                             idx: Object<'c>,
                             _size: IntValue<'c>,
                             ptr_to_buffer: PointerValue<'c>| {
                // Apply map to idx object to get initial value at this idx.
                gc.retain(map.clone());
                let value = gc.apply_lambda(map.clone(), idx.clone());

                // Store value.
                let idx_val = idx.load_field_nocap(gc, 0).into_int_value();
                let ptr_to_obj_ptr = unsafe {
                    gc.builder()
                        .build_gep(ptr_to_buffer, &[idx_val.into()], "ptr_to_elem_of_array")
                };
                if value.is_box(gc.type_env()) {
                    gc.builder().build_store(ptr_to_obj_ptr, value.ptr);
                } else {
                    gc.builder()
                        .build_store(ptr_to_obj_ptr, value.load_nocap(gc));
                }
            };

            // After loop, release map.
            let after_loop = |gc: &mut GenerationContext<'c, 'm>,
                              _size: IntValue<'c>,
                              _ptr_to_buffer: PointerValue<'c>| {
                gc.release(map.clone());
            };

            // Generate loop.
            // NOTE: if you see error at here, try `cargo clean`.
            Self::loop_over_array_size_buf(gc, size_buf, elem_ty, loop_body, after_loop);
        }
    }

    // Panic if idx is out_of_range for the array.
    pub fn panic_if_out_of_range<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        idx: IntValue<'c>,
    ) {
        let (size, _ptr_to_buffer) = Self::decompose_array_size_buf(gc, size_buf, elem_ty);
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
        gc.builder().position_at_end(in_range_bb);
    }

    // Read an element of array.
    // Returned object is already retained.
    pub fn read_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        idx: IntValue<'c>,
    ) -> Object<'c> {
        // Panic if out_of_range.
        Self::panic_if_out_of_range(gc, size_buf, elem_ty.clone(), idx);

        // Get fields (size, ptr_to_buffer).
        let (_size, ptr_to_buffer) = Self::decompose_array_size_buf(gc, size_buf, elem_ty.clone());

        // Get element.
        let ptr_to_elem = unsafe {
            gc.builder()
                .build_gep(ptr_to_buffer, &[idx.into()], "ptr_to_elem_of_array")
        };
        let elem_obj = Object::from_basic_value_enum(
            gc.builder().build_load(ptr_to_elem, "elem"),
            elem_ty,
            gc,
        );

        // Retain element and return it.
        gc.retain(elem_obj.clone());
        elem_obj
    }

    // Write an element into array.
    pub fn write_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size_buf: PointerValue<'c>,
        idx: IntValue<'c>,
        value: Object<'c>,
    ) {
        let elem_ty = value.ty.clone();

        // Panic if out_of_range.
        Self::panic_if_out_of_range(gc, size_buf, elem_ty.clone(), idx);

        // Get fields (size, ptr_to_buffer).
        let (_size, ptr_to_buffer) = Self::decompose_array_size_buf(gc, size_buf, elem_ty.clone());

        // Get ptr to the place at idx.
        let place = unsafe {
            gc.builder()
                .build_gep(ptr_to_buffer, &[idx.into()], "ptr_to_elem_of_array")
        };

        // Release element that is already at the place.
        let elem = if elem_ty.is_box(gc.type_env()) {
            gc.builder().build_load(place, "elem").into_pointer_value()
        } else {
            place
        };
        let elem_obj = Object::new(elem, elem_ty);
        gc.release(elem_obj);

        // Insert the given value to the place.
        let value = if value.is_box(gc.type_env()) {
            value.ptr.as_basic_value_enum()
        } else {
            value.load_nocap(gc).as_basic_value_enum()
        };
        gc.builder().build_store(place, value);
    }

    // Clone an array
    pub fn clone_array_size_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        src: PointerValue<'c>,
        dst: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
    ) {
        let array_struct = ObjectFieldType::ArraySizeBuf(elem_ty.clone())
            .to_basic_type(gc)
            .into_struct_type();

        // Get fields (size, ptr_to_buffer) of src.
        let (src_size, src_buffer) = Self::decompose_array_size_buf(gc, src, elem_ty.clone());

        // Copy size.
        gc.store_obj_field(dst, array_struct, 0, src_size);

        // Allocate buffer and set it to dst.
        let elem_str_type = get_object_type(&elem_ty, &vec![], gc.type_env()).to_struct_type(gc);
        let dst_buffer = gc
            .builder()
            .build_array_malloc(elem_str_type, src_size, "dst_buffer")
            .unwrap();
        gc.store_obj_field(dst, array_struct, 1, dst_buffer);

        let elem_ty_clone = elem_ty.clone();
        // Clone each elements.
        {
            // In loop body, retain value and store it at idx.
            let loop_body = |gc: &mut GenerationContext<'c, 'm>,
                             idx: Object<'c>,
                             _size: IntValue<'c>,
                             _ptr_to_buffer: PointerValue<'c>| {
                let idx = idx.load_field_nocap(gc, 0).into_int_value();
                let ptr_to_src_elem = unsafe {
                    gc.builder()
                        .build_gep(src_buffer, &[idx.into()], "ptr_to_src_elem")
                };
                let ptr_to_dst_elem = unsafe {
                    gc.builder()
                        .build_gep(dst_buffer, &[idx.into()], "ptr_to_dst_elem")
                };
                let src_elem = gc.builder().build_load(ptr_to_src_elem, "src_elem");
                gc.builder().build_store(ptr_to_dst_elem, src_elem);
                let src_elem = if elem_ty.is_box(gc.type_env()) {
                    src_elem.into_pointer_value()
                } else {
                    ptr_to_dst_elem
                };
                let src_obj = Object::new(src_elem, elem_ty.clone());
                gc.retain(src_obj);
            };

            // After loop, do nothing.
            let after_loop = |_gc: &mut GenerationContext<'c, 'm>,
                              _size: IntValue<'c>,
                              _ptr_to_buffer: PointerValue<'c>| {};

            Self::loop_over_array_size_buf(gc, src, elem_ty_clone, loop_body, after_loop);
        }
    }

    fn retain_release_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Arc<TypeNode>>,
        is_retain: bool,
    ) {
        // Retain or release field.
        let curr_func = gc
            .builder()
            .get_insert_block()
            .unwrap()
            .get_parent()
            .unwrap();
        let end_bb = gc.context.append_basic_block(curr_func, "end");
        let mut last_unmatch_bb: Option<BasicBlock> = None;
        for (i, field_ty) in field_types.iter().enumerate() {
            // Compare tag and jump.
            let match_bb = gc
                .context
                .append_basic_block(curr_func, &format!("match_tag{}", i));
            let unmatch_bb = gc
                .context
                .append_basic_block(curr_func, &format!("unmatch_tag{}", i));
            let expect_tag_val = gc.context.i64_type().const_int(i as u64, false);
            let is_match = gc.builder().build_int_compare(
                IntPredicate::EQ,
                tag,
                expect_tag_val,
                &format!("is_tag_{}", i),
            );
            gc.builder()
                .build_conditional_branch(is_match, match_bb, unmatch_bb);

            // Implement the case tag is match.
            gc.builder().position_at_end(match_bb);
            let value_ptr = if field_ty.is_box(gc.type_env()) {
                gc.builder()
                    .build_load(buf, "load_buf")
                    .into_pointer_value()
            } else {
                buf
            };
            let obj = Object::new(value_ptr, field_ty.clone());
            if is_retain {
                gc.retain(obj);
            } else {
                gc.release(obj);
            }
            gc.builder().build_unconditional_branch(end_bb);

            // Implement the case tag is unmatch.
            gc.builder().position_at_end(unmatch_bb);
            last_unmatch_bb = Some(unmatch_bb);
        }

        // Implement last unmatch bb.
        let last_unmatch_bb = last_unmatch_bb.unwrap();
        gc.builder().position_at_end(last_unmatch_bb);
        gc.builder().build_unreachable();
        gc.builder().build_unconditional_branch(end_bb);
    }

    pub fn retain_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Arc<TypeNode>>,
    ) {
        ObjectFieldType::retain_release_union_buf(gc, buf, tag, field_types, true);
    }

    pub fn release_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Arc<TypeNode>>,
    ) {
        ObjectFieldType::retain_release_union_buf(gc, buf, tag, field_types, false);
    }

    pub fn allocate_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        field_types: &Vec<Arc<TypeNode>>,
    ) -> PointerValue<'c> {
        if field_types.is_empty() {
            return ptr_to_object_type(gc.context).const_null();
        }
        let size = field_types
            .iter()
            .map(|ty| {
                let struct_ty = get_object_type(
                    ty,
                    &vec![], /* captured list desn't effect sizeof */
                    gc.type_env(),
                )
                .to_struct_type(gc);
                gc.sizeof(&struct_ty)
            })
            .max()
            .unwrap();
        let size = gc.context.i64_type().const_int(size, false);
        gc.builder()
            .build_array_malloc(gc.context.i8_type(), size, "alloc_union_buf")
            .unwrap()
    }

    pub fn set_value_to_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        val: Object<'c>,
    ) {
        let val = if val.is_box(gc.type_env()) {
            val.ptr.as_basic_value_enum()
        } else {
            val.load_nocap(gc).as_basic_value_enum()
        };
        let buf = gc.cast_pointer(buf, val.get_type().ptr_type(AddressSpace::Generic));
        gc.builder().build_store(buf, val);
    }

    pub fn get_value_from_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        elem_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let val = ObjectFieldType::get_basic_value_from_union_buf(gc, buf, elem_ty);
        let val = Object::from_basic_value_enum(val, elem_ty.clone(), gc);
        gc.retain(val.clone());
        val
    }

    pub fn get_basic_value_from_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        elem_ty: &Arc<TypeNode>,
    ) -> BasicValueEnum<'c> {
        let buf = gc.cast_pointer(
            buf,
            elem_ty
                .get_embedded_type(gc, &vec![])
                .ptr_type(AddressSpace::Generic),
        );
        gc.builder().build_load(buf, "value_at_union_buf")
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct ObjectType {
    pub field_types: Vec<ObjectFieldType>,
    pub is_unbox: bool,
}

impl ObjectType {
    pub fn to_struct_type<'c, 'm>(&self, gc: &GenerationContext<'c, 'm>) -> StructType<'c> {
        let mut fields: Vec<BasicTypeEnum<'c>> = vec![];
        for field_type in &self.field_types {
            fields.push(field_type.to_basic_type(gc));
        }
        gc.context.struct_type(&fields, false)
    }

    // Get type used when this object is embedded.
    // i.e., for unboxed type, a pointer; for unboxed type, a struct.
    pub fn to_embedded_type<'c, 'm>(&self, gc: &GenerationContext<'c, 'm>) -> BasicTypeEnum<'c> {
        let str_ty = self.to_struct_type(gc);
        if self.is_unbox {
            str_ty.into()
        } else {
            ptr_type(str_ty).into()
        }
    }

    // fn shared_obj_type(mut field_types: Vec<ObjectFieldType>) -> Self {
    //     let mut fields = vec![ObjectFieldType::ControlBlock];
    //     fields.append(&mut field_types);
    //     Self {
    //         field_types: fields,
    //     }
    // }

    // pub fn lam_obj_type() -> Self {
    //     Self::shared_obj_type(vec![ObjectFieldType::LambdaFunction]) // Other fields for captured objects may exist but omitted here.
    // }

    // pub fn int_obj_type() -> Self {
    //     Self::shared_obj_type(vec![ObjectFieldType::I64])
    // }

    // pub fn bool_obj_type() -> Self {
    //     Self::shared_obj_type(vec![ObjectFieldType::Bool])
    // }

    // pub fn array_type() -> Self {
    //     let fields = vec![ObjectFieldType::Array];
    //     Self::shared_obj_type(fields)
    // }

    // pub fn struct_type(field_count: usize) -> Self {
    //     let fields: Vec<ObjectFieldType> = iter::repeat(ObjectFieldType::Boxed)
    //         .take(field_count)
    //         .collect();
    //     Self::shared_obj_type(fields)
    // }

    // pub fn union_type() -> Self {
    //     let fields = vec![ObjectFieldType::Int /* tag */, ObjectFieldType::Boxed];
    //     Self::shared_obj_type(fields)
    // }
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
    let mut fields = vec![refcnt_type(context).into()];
    if SANITIZE_MEMORY {
        fields.push(obj_id_type(context).into())
    }
    context.struct_type(&fields, false)
}

pub fn ptr_to_control_block_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    control_block_type(context).ptr_type(AddressSpace::Generic)
}

pub fn lambda_function_type<'c, 'm>(
    ty: &Arc<TypeNode>,
    gc: &GenerationContext<'c, 'm>,
) -> FunctionType<'c> {
    // A function that takes argument and context (=lambda object itself).
    let arg_ty = ty.get_funty_src().get_embedded_type(gc, &vec![]);
    let ret_ty = ty.get_funty_dst().get_embedded_type(gc, &vec![]);
    ret_ty.fn_type(
        &[arg_ty.into(), ptr_to_object_type(gc.context).into()],
        false,
    )
}

// fn ptr_to_lambda_function_type<'ctx>(
//     ty: &Arc<TypeNode>,
//     context: &'ctx Context,
// ) -> PointerType<'ctx> {
//     lambda_function_type(ty, context).ptr_type(AddressSpace::Generic)
// }

// pub fn lambda_type<'c>(context: &'c Context) -> StructType<'c> {
//     ObjectType::lam_obj_type().to_struct_type(context)
// }

// pub fn int_type<'c>(context: &'c Context) -> StructType<'c> {
//     ObjectType::int_obj_type().to_struct_type(context)
// }

// pub fn bool_type<'c>(context: &'c Context) -> StructType<'c> {
//     ObjectType::bool_obj_type().to_struct_type(context)
// }

pub const DTOR_IDX: u32 = 1/* ControlBlock */;
pub const LAMBDA_FUNCTION_IDX: u32 = DTOR_IDX + 1;
pub const CAPTURED_OBJECT_IDX: u32 = LAMBDA_FUNCTION_IDX + 1;
pub const ARRAY_IDX: u32 = 1;
pub fn struct_field_idx(is_unbox: bool) -> u32 {
    if is_unbox {
        0
    } else {
        1
    }
}

pub fn get_object_type(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>,
    type_env: &TypeEnv,
) -> ObjectType {
    assert!(ty.free_vars().is_empty());
    let mut ret = ObjectType {
        field_types: vec![],
        is_unbox: true,
    };
    if ty.is_function() {
        ret.is_unbox = false;
        ret.field_types.push(ObjectFieldType::ControlBlock);
        ret.field_types.push(ObjectFieldType::DtorFunction);
        assert_eq!(ret.field_types.len(), LAMBDA_FUNCTION_IDX as usize);
        ret.field_types
            .push(ObjectFieldType::LambdaFunction(ty.clone()));
        for cap in capture {
            ret.field_types
                .push(ObjectFieldType::SubObject(cap.clone()));
        }
    } else {
        assert!(capture.is_empty());
        let tc = ty.toplevel_tycon().unwrap();
        let ti = type_env.tycons.get(&tc).unwrap();
        match ti.variant {
            TyConVariant::Primitive => {
                assert!(ti.is_unbox);
                ret.is_unbox = ti.is_unbox;
                if ty == &int_lit_ty() {
                    ret.field_types.push(ObjectFieldType::I64);
                } else if ty == &bool_lit_ty() {
                    ret.field_types.push(ObjectFieldType::Bool);
                } else {
                    unreachable!()
                }
            }
            TyConVariant::Array => {
                let is_unbox = ti.is_unbox;
                assert!(!is_unbox);
                ret.is_unbox = is_unbox;
                ret.field_types.push(ObjectFieldType::ControlBlock);
                assert_eq!(ret.field_types.len(), ARRAY_IDX as usize);
                ret.field_types.push(ObjectFieldType::ArraySizeBuf(
                    ty.fields_types(type_env)[0].clone(),
                ))
            }
            TyConVariant::Struct => {
                let is_unbox = ti.is_unbox;
                ret.is_unbox = is_unbox;
                if !is_unbox {
                    ret.field_types.push(ObjectFieldType::ControlBlock);
                }
                assert_eq!(ret.field_types.len(), struct_field_idx(is_unbox) as usize);
                for field_ty in ty.fields_types(type_env) {
                    ret.field_types.push(ObjectFieldType::SubObject(field_ty));
                }
            }
            TyConVariant::Union => {
                let is_unbox = ti.is_unbox;
                ret.is_unbox = is_unbox;
                if !is_unbox {
                    ret.field_types.push(ObjectFieldType::ControlBlock);
                }
                ret.field_types.push(ObjectFieldType::UnionTag);
                ret.field_types.push(ObjectFieldType::UnionBuf);
            }
        }
    }
    ret
}

// Allocate an object.
pub fn allocate_obj<'c, 'm>(
    ty: Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>, // used in lambda
    gc: &mut GenerationContext<'c, 'm>,
    name: Option<&str>,
) -> Object<'c> {
    assert!(ty.free_vars().is_empty());
    let context = gc.context;
    let object_type = ty.get_object_type(capture, gc.type_env());
    let struct_type = object_type.to_struct_type(gc);

    // Allocate object
    let ptr_to_obj = if object_type.is_unbox {
        gc.builder()
            .build_alloca(struct_type, &format!("alloca_{}", ty.to_string_normalize()))
    } else {
        gc.malloc(struct_type)
    };

    // If sanitize memory, create object id.
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

    // Initialize refcnt and dtor field.
    for (i, ft) in object_type.field_types.iter().enumerate() {
        match ft {
            ObjectFieldType::ControlBlock => {
                assert_eq!(i, 0);
                // Set refcnt one.
                let ptr_to_control_block = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_control_block")
                    .unwrap();
                let ptr_to_refcnt = gc
                    .builder()
                    .build_struct_gep(ptr_to_control_block, 0, "ptr_to_refcnt")
                    .unwrap();
                gc.builder()
                    .build_store(ptr_to_refcnt, refcnt_type(context).const_int(1, false));

                // If sanitize memory, set object id.
                if SANITIZE_MEMORY {
                    let ptr_to_obj_id = gc
                        .builder()
                        .build_struct_gep(ptr_to_control_block, 1, "ptr_to_obj_id")
                        .unwrap();
                    gc.builder().build_store(ptr_to_obj_id, object_id);
                }
            }
            ObjectFieldType::I64 => {}
            ObjectFieldType::SubObject(_) => {}
            ObjectFieldType::LambdaFunction(_) => {
                assert_eq!(i, LAMBDA_FUNCTION_IDX as usize);
            }
            ObjectFieldType::Bool => {}
            ObjectFieldType::ArraySizeBuf(_) => {}
            ObjectFieldType::DtorFunction => {
                assert_eq!(i, DTOR_IDX as usize);
                let ptr_to_dtor_field = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_dtor_field")
                    .unwrap();
                let dtor = get_dtor_ptr(&ty, capture, gc);
                gc.builder().build_store(ptr_to_dtor_field, dtor);
            }
            ObjectFieldType::UnionBuf => {
                let field_types = ty.fields_types(gc.type_env());
                let ptr = ObjectFieldType::allocate_union_buf(gc, &field_types);
                let ptr_to_unionbuf_field = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_unionbuf_field")
                    .unwrap();
                gc.builder().build_store(ptr_to_unionbuf_field, ptr);
            }
            ObjectFieldType::UnionTag => {}
        }
    }

    Object::new(ptr_to_obj, ty)
}

pub fn get_dtor_ptr<'c, 'm>(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>, // used in destructor of lambda
    gc: &mut GenerationContext<'c, 'm>,
) -> PointerValue<'c> {
    match create_dtor(ty, capture, gc) {
        Some(fv) => fv.as_global_value().as_pointer_value(),
        None => ptr_to_dtor_type(gc.context).const_null(),
    }
}

pub fn create_dtor<'c, 'm>(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>, // used in destructor of lambda
    gc: &mut GenerationContext<'c, 'm>,
) -> Option<FunctionValue<'c>> {
    assert!(ty.free_vars().is_empty());
    if ty.is_function() && capture.is_empty() {
        return None;
    }
    let dtor_name = ty.dtor_name(capture);
    match gc.module.get_function(&dtor_name) {
        Some(fv) => {
            if fv.is_null() {
                None
            } else {
                Some(fv)
            }
        }
        None => {
            // Define dtor function.
            let object_type = get_object_type(ty, capture, gc.type_env());
            let struct_type = object_type.to_struct_type(gc);
            let func_type = dtor_type(gc.context);
            let func = gc.module.add_function(&dtor_name, func_type, None);
            let bb = gc.context.append_basic_block(func, "entry");

            let _builder_guard = gc.push_builder();

            gc.builder().position_at_end(bb);
            let ptr_to_obj = func.get_first_param().unwrap().into_pointer_value();
            let mut union_tag: Option<IntValue<'c>> = None;
            for (i, ft) in object_type.field_types.iter().enumerate() {
                match ft {
                    ObjectFieldType::SubObject(ty) => {
                        let is_unbox = ty.is_unbox(gc.type_env());
                        let ptr_to_subobj = if is_unbox {
                            let ptr_to_struct = gc.cast_pointer(ptr_to_obj, ptr_type(struct_type));
                            gc.builder()
                                .build_struct_gep(
                                    ptr_to_struct,
                                    i as u32,
                                    &format!("ptr_to_{}nd_field", i),
                                )
                                .unwrap()
                        } else {
                            gc.load_obj_field(ptr_to_obj, struct_type, i as u32)
                                .into_pointer_value()
                        };
                        gc.release(Object {
                            ptr: ptr_to_subobj,
                            ty: ty.clone(),
                        });
                    }
                    ObjectFieldType::ControlBlock => {}
                    ObjectFieldType::I64 => {}
                    ObjectFieldType::LambdaFunction(_) => {}
                    ObjectFieldType::Bool => {}
                    ObjectFieldType::ArraySizeBuf(ty) => {
                        let ptr = gc
                            .load_obj_field(ptr_to_obj, struct_type, i as u32)
                            .into_pointer_value();
                        ObjectFieldType::destruct_array_size_buf(gc, ptr, ty.clone());
                        gc.builder().build_free(ptr);
                    }
                    ObjectFieldType::UnionTag => {
                        union_tag = Some(
                            gc.load_obj_field(ptr_to_obj, struct_type, i as u32)
                                .into_int_value(),
                        );
                    }
                    ObjectFieldType::UnionBuf => {
                        let buf = gc
                            .load_obj_field(ptr_to_obj, struct_type, i as u32)
                            .into_pointer_value();
                        ObjectFieldType::release_union_buf(
                            gc,
                            buf,
                            union_tag.unwrap(),
                            &ty.fields_types(gc.type_env()),
                        );
                        gc.builder().build_free(buf);
                    }
                    ObjectFieldType::DtorFunction => {}
                }
            }
            gc.builder().build_return(None);
            if func.is_null() {
                None
            } else {
                Some(func)
            }
        }
    }
}
