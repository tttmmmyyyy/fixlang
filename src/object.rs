use inkwell::{
    basic_block::BasicBlock,
    module::Linkage,
    types::{BasicMetadataTypeEnum, BasicType},
};

use super::*;

#[derive(Eq, PartialEq, Clone)]
pub enum ObjectFieldType {
    ControlBlock,
    DtorFunction,
    LambdaFunction(Rc<TypeNode>), // Specify type of lambda
    Ptr,
    I8,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    SubObject(Rc<TypeNode>),
    UnionBuf(Vec<Rc<TypeNode>>), // Embedded union.
    UnionTag,
    Array(Rc<TypeNode>), // field to store capacity (size) and buffer for elements.
}

impl ObjectFieldType {
    pub fn to_basic_type<'c, 'm>(&self, gc: &mut GenerationContext<'c, 'm>) -> BasicTypeEnum<'c> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(gc).into(),
            ObjectFieldType::DtorFunction => ptr_to_dtor_type(gc.context).into(),
            ObjectFieldType::LambdaFunction(ty) => lambda_function_type(ty, gc)
                .ptr_type(AddressSpace::from(0))
                .into(),
            ObjectFieldType::SubObject(ty) => {
                get_object_type(ty, &vec![], gc.type_env()).to_embedded_type(gc)
            }
            ObjectFieldType::Ptr => gc.context.i8_type().ptr_type(AddressSpace::from(0)).into(),
            ObjectFieldType::I8 => gc.context.i8_type().into(),
            ObjectFieldType::I32 => gc.context.i32_type().into(),
            ObjectFieldType::U32 => gc.context.i32_type().into(),
            ObjectFieldType::I64 => gc.context.i64_type().into(),
            ObjectFieldType::U64 => gc.context.i64_type().into(),
            ObjectFieldType::F32 => gc.context.f32_type().into(),
            ObjectFieldType::F64 => gc.context.f64_type().into(),
            ObjectFieldType::Array(_) => gc.context.i64_type().into(),
            ObjectFieldType::UnionTag => gc.context.i8_type().into(),
            ObjectFieldType::UnionBuf(field_tys) => {
                let mut size = 0;
                for field_ty in field_tys {
                    let struct_ty = get_object_type(
                        field_ty,
                        &vec![], /* captured list desn't effect sizeof */
                        gc.type_env(),
                    )
                    .to_embedded_type(gc);
                    size = size.max(gc.sizeof(&struct_ty));
                }
                // Force align 8
                let num_of_i64 = size / 8 + if size % 8 == 0 { 0 } else { 1 };
                gc.context
                    .i64_type()
                    .array_type(num_of_i64 as u32)
                    .as_basic_type_enum()
            }
        }
    }

    // Take array and generate code iterating its elements.
    fn loop_over_array_buf<'c, 'm, F, G>(
        gc: &mut GenerationContext<'c, 'm>,
        size: IntValue<'c>,
        buffer: PointerValue<'c>,
        loop_body: F,
        after_loop: G,
    ) where
        for<'c2, 'm2> F: Fn(
            &mut GenerationContext<'c, 'm>,
            Object<'c>,       /* idx */
            IntValue<'c>,     /* size */
            PointerValue<'c>, /* buffer */
        ),
        for<'c2, 'm2> G: Fn(
            &mut GenerationContext<'c, 'm>,
            IntValue<'c>,     /* size */
            PointerValue<'c>, /* buffer */
        ),
    {
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
        let counter_ptr = gc.build_alloca_at_entry(counter_type, "release_loop_counter");
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
        loop_body(gc, Object::new(counter_ptr, make_i64_ty()), size, buffer);

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
        after_loop(gc, size, buffer);
    }

    // Release each element in array buffer.
    pub fn release_array_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size: IntValue<'c>,
        buffer: PointerValue<'c>,
        elem_ty: Rc<TypeNode>,
    ) {
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
        Self::loop_over_array_buf(gc, size, buffer, loop_body, after_loop);
    }

    // Initialize an array by value.
    pub fn initialize_array_buf_by_value<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size: IntValue<'c>,
        buffer: PointerValue<'c>,
        value: Object<'c>,
    ) {
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
                    gc.builder().build_store(ptr_to_obj_ptr, value.ptr(gc));
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
            Self::loop_over_array_buf(gc, size, buffer, loop_body, after_loop);
        }
    }

    // Panic if idx is out_of_range for the array.
    pub fn panic_if_out_of_range<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        len: IntValue<'c>,
        idx: IntValue<'c>,
    ) {
        let curr_bb = gc.builder().get_insert_block().unwrap();
        let curr_func = curr_bb.get_parent().unwrap();
        let is_out_of_range =
            gc.builder()
                .build_int_compare(IntPredicate::UGE, idx, len, "is_out_of_ramge");
        let out_of_range_bb = gc.context.append_basic_block(curr_func, "out_of_range_bb");
        let in_range_bb = gc.context.append_basic_block(curr_func, "in_range_bb");
        gc.builder()
            .build_conditional_branch(is_out_of_range, out_of_range_bb, in_range_bb);
        gc.builder().position_at_end(out_of_range_bb);
        gc.panic("Index out of range!");
        gc.builder().build_unconditional_branch(in_range_bb);
        gc.builder().position_at_end(in_range_bb);
    }

    // Read an element of array.
    // Returned object is not retained.
    pub fn read_from_array_buf_noretain<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        len: Option<IntValue<'c>>, // If none, bounds checking is omitted.
        buffer: PointerValue<'c>,
        elem_ty: Rc<TypeNode>,
        idx: IntValue<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        // Panic if out_of_range.
        if len.is_some() {
            Self::panic_if_out_of_range(gc, len.unwrap(), idx);
        }

        // Get element.
        let ptr_to_elem = unsafe {
            gc.builder()
                .build_gep(buffer, &[idx.into()], "ptr_to_elem_of_array")
        };

        // Get value
        let elem_val = gc.builder().build_load(ptr_to_elem, "elem");

        // Return value
        let elem_obj = if rvo.is_some() {
            let rvo = rvo.unwrap();
            rvo.store_unbox(gc, elem_val);
            rvo
        } else {
            Object::create_from_value(elem_val, elem_ty, gc)
        };
        elem_obj
    }

    // Read an element of array.
    // Returned object is retained.
    pub fn read_from_array_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        len: Option<IntValue<'c>>, // If none, bounds checking is omitted.
        buffer: PointerValue<'c>,
        elem_ty: Rc<TypeNode>,
        idx: IntValue<'c>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let elem =
            ObjectFieldType::read_from_array_buf_noretain(gc, len, buffer, elem_ty, idx, rvo);
        gc.retain(elem.clone());
        elem
    }

    // Write an element into array.
    pub fn write_to_array_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        len: Option<IntValue<'c>>,
        buffer: PointerValue<'c>,
        idx: IntValue<'c>,
        value: Object<'c>,
        release_old_value: bool,
    ) {
        let elem_ty = value.ty.clone();

        // Panic if out_of_range.
        if len.is_some() {
            Self::panic_if_out_of_range(gc, len.unwrap(), idx);
        }

        // Get ptr to the place at idx.
        let place = unsafe {
            gc.builder()
                .build_gep(buffer, &[idx.into()], "ptr_to_elem_of_array")
        };

        // Release element that is already at the place.
        if release_old_value {
            let elem = if elem_ty.is_box(gc.type_env()) {
                gc.builder().build_load(place, "elem").into_pointer_value()
            } else {
                place
            };
            let elem_obj = Object::new(elem, elem_ty);
            gc.release(elem_obj);
        }

        // Insert the given value to the place.
        gc.builder().build_store(place, value.value(gc));
    }

    // Clone an array
    pub fn clone_array_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        len: IntValue<'c>,
        src_buffer: PointerValue<'c>,
        dst_buffer: PointerValue<'c>,
        elem_ty: Rc<TypeNode>,
    ) {
        // Clone each elements.
        {
            // In loop body, retain value and store it at idx.
            let loop_body = |gc: &mut GenerationContext<'c, 'm>,
                             idx: Object<'c>,
                             _len: IntValue<'c>,
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
                              _len: IntValue<'c>,
                              _ptr_to_buffer: PointerValue<'c>| {};

            Self::loop_over_array_buf(gc, len, src_buffer, loop_body, after_loop);
        }
    }

    fn retain_release_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Rc<TypeNode>>,
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
            let expect_tag_val = ObjectFieldType::UnionTag
                .to_basic_type(gc)
                .into_int_type()
                .const_int(i as u64, false);
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
                let buf = gc.cast_pointer(
                    buf,
                    ptr_to_object_type(gc.context).ptr_type(AddressSpace::from(0)),
                );
                gc.builder()
                    .build_load(buf, "load_boxed_buf")
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
        gc.panic("all tags unmatch!"); // unreachable didn't work as I expected.
        gc.builder().build_unconditional_branch(end_bb);

        gc.builder().position_at_end(end_bb);
    }

    pub fn retain_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Rc<TypeNode>>,
    ) {
        ObjectFieldType::retain_release_union_buf(gc, buf, tag, field_types, true);
    }

    pub fn release_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Rc<TypeNode>>,
    ) {
        ObjectFieldType::retain_release_union_buf(gc, buf, tag, field_types, false);
    }

    pub fn set_value_to_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        val: Object<'c>,
    ) {
        let val = val.value(gc);
        let buf = gc.cast_pointer(buf, val.get_type().ptr_type(AddressSpace::from(0)));
        gc.builder().build_store(buf, val);
    }

    // Get field of union (with refcnt managed).
    pub fn get_union_field<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        union: Object<'c>,
        elem_ty: &Rc<TypeNode>,
        rvo: Option<Object<'c>>,
    ) -> Object<'c> {
        let is_unbox = union.ty.is_unbox(gc.type_env());
        let offset = if is_unbox { 0 } else { 1 };
        let buf = union.ptr_to_field_nocap(gc, 1 + offset);

        // Make return value by cloning the field in the union buffer,
        // because lifetime of returned value may be longer than that of union object itself.
        let field_val = ObjectFieldType::get_value_from_union_buf(gc, buf, elem_ty);
        let field = if rvo.is_none() {
            Object::create_from_value(field_val, elem_ty.clone(), gc)
        } else {
            let rvo = rvo.unwrap();
            rvo.store_unbox(gc, field_val);
            rvo
        };
        if union.is_box(gc.type_env()) {
            gc.retain(field.clone());
            gc.release(union);
        } else {
            // If unbox, retaining and releasing cancel each other out, so do nothing.
        }
        field
    }

    pub fn get_value_from_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        elem_ty: &Rc<TypeNode>,
    ) -> BasicValueEnum<'c> {
        let elem_ptr_ty = elem_ty
            .get_embedded_type(gc, &vec![])
            .ptr_type(AddressSpace::from(0));
        let buf = gc.cast_pointer(buf, elem_ptr_ty);
        gc.builder().build_load(buf, "value_at_union_buf")
    }

    pub fn panic_if_union_tag_unmatch<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        union: Object<'c>,
        expect_tag: IntValue<'c>,
    ) {
        let is_unbox = union.ty.is_unbox(gc.type_env());
        let offset = if is_unbox { 0 } else { 1 };

        // Get tag value.
        let tag_value = union.load_field_nocap(gc, 0 + offset).into_int_value();

        // If tag unmatch, panic.
        let is_tag_unmatch = gc.builder().build_int_compare(
            IntPredicate::NE,
            expect_tag,
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
        gc.builder().position_at_end(match_bb);
    }

    // Get field of struct as Object (no refcnt management and no cloned).
    pub fn get_struct_field_noclone<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        str: &Object<'c>,
        field_idx: u32,
    ) -> Object<'c> {
        let field_offset = struct_field_idx(str.ty.is_unbox(gc.type_env()));
        let field_ty = str.ty.field_types(gc.type_env())[field_idx as usize].clone();
        let field_ptr = if field_ty.is_box(gc.type_env()) {
            str.load_field_nocap(gc, field_idx + field_offset)
                .into_pointer_value()
        } else {
            str.ptr_to_field_nocap(gc, field_idx + field_offset)
        };
        Object::new(field_ptr, field_ty)
    }

    // Get field of struct as Objects (with refcnt managed).
    pub fn get_struct_fields<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        str: &Object<'c>,
        field_indices_rvo: Vec<(u32, Option<Object<'c>>)>,
    ) -> Vec<Object<'c>> {
        // Collect unretained (but cloned) fields.
        // We need clone here since lifetime of returned fields may be longer than that of struct object.
        let mut ret = vec![];
        for (field_idx, rvo) in &field_indices_rvo {
            // Get ptr to field.
            let field = ObjectFieldType::get_struct_field_noclone(gc, str, *field_idx);

            // Clone the field.
            let field_val = field.value(gc);
            let field = if rvo.is_none() {
                Object::create_from_value(field_val, field.ty, gc)
            } else {
                let rvo = rvo.as_ref().unwrap();
                rvo.store_unbox(gc, field_val);
                rvo.clone()
            };
            ret.push(field);
        }

        if str.is_box(gc.type_env()) {
            // If struct is boxed, simply retain fields and release the struct.
            for field in &ret {
                gc.retain(field.clone());
            }
            gc.release(str.clone());
        } else {
            // If the struct is unboxed, instead of retaining elements of `ret` and releasing the struct,
            // just release fields that are not not in `ret`.
            let field_indices: HashSet<u32> =
                HashSet::from_iter(field_indices_rvo.iter().map(|(i, _)| i.clone()));
            for field_idx in 0..str.ty.field_types(gc.type_env()).len() {
                let field_idx = field_idx as u32;
                if !field_indices.contains(&field_idx) {
                    let field = ObjectFieldType::get_struct_field_noclone(gc, str, field_idx);
                    gc.release(field);
                }
            }
        }

        ret
    }

    // Set an Object to field of struct. The old value isn't released in this function.
    pub fn set_struct_field_norelease<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        str: &Object<'c>,
        field_idx: u32,
        field: &Object<'c>,
    ) -> () {
        let field_offset = struct_field_idx(str.ty.is_unbox(gc.type_env()));
        let field_val = field.value(gc);
        str.store_field_nocap(gc, field_offset + field_idx as u32, field_val);
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct ObjectType {
    pub field_types: Vec<ObjectFieldType>,
    pub is_unbox: bool,
}

impl ObjectType {
    pub fn to_struct_type<'c, 'm>(&self, gc: &mut GenerationContext<'c, 'm>) -> StructType<'c> {
        let mut fields: Vec<BasicTypeEnum<'c>> = vec![];
        for (i, field_type) in self.field_types.iter().enumerate() {
            fields.push(field_type.to_basic_type(gc));
            match field_type {
                ObjectFieldType::Array(ty) => {
                    assert_eq!(i, self.field_types.len() - 1); // ArraySize must be the last field.
                    assert!(!self.is_unbox); // Array has to be boxed.

                    // Add space for one element.
                    // This is for:
                    // - to get the pointer to the first element by gep of this struct type.
                    // - used in implementation of size_of method.
                    fields.push(
                        ty.get_object_type(&vec![], gc.type_env())
                            .to_embedded_type(gc)
                            .into(),
                    );
                }
                _ => {}
            }
        }
        gc.context.struct_type(&fields, false)
    }

    pub fn size_of<'c, 'm>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        array_size: Option<IntValue<'c>>,
    ) -> IntValue<'c> {
        if array_size.is_some() {
            // Get pointer to the first element (which is properly aligned) and add it to sizeof(elem_ty) * size.

            // Calculate sizeof(elem_ty) * size.
            let elem_ty = match self.field_types.last().unwrap() {
                ObjectFieldType::Array(ty) => ty.clone(),
                _ => panic!(),
            };
            let elem_sizeof = elem_ty
                .get_object_type(&vec![], gc.type_env())
                .to_struct_type(gc)
                .size_of()
                .unwrap();
            let struct_ty = self.to_struct_type(gc);
            let ptr_int_ty = gc.context.ptr_sized_int_type(gc.target_data(), None);
            let size = array_size.unwrap();
            let size = gc
                .builder()
                .build_int_cast(size, ptr_int_ty, "size_as_ptr_int_ty");
            let elems_size = gc.builder().build_int_mul(elem_sizeof, size, "elems_size");

            // Get pointer to the first element
            let null = gc.cast_pointer(
                gc.context
                    .i8_type()
                    .ptr_type(AddressSpace::from(0))
                    .const_null(),
                struct_ty.ptr_type(AddressSpace::from(0)),
            );
            let ptr_to_first_elem = gc
                .builder()
                .build_struct_gep(null, ARRAY_BUF_IDX, "gep_first_elem_size_of")
                .unwrap();
            let ptr_to_first_elem =
                gc.builder()
                    .build_ptr_to_int(ptr_to_first_elem, ptr_int_ty, "size_with_one_elem");

            let size_with_elems =
                gc.builder()
                    .build_int_add(ptr_to_first_elem, elems_size, "size_with_elems");
            return size_with_elems;
        } else {
            self.to_struct_type(gc).size_of().unwrap()
        }
    }

    // Get type used when this object is embedded.
    // i.e., for unboxed type, a pointer; for unboxed type, a struct.
    pub fn to_embedded_type<'c, 'm>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
    ) -> BasicTypeEnum<'c> {
        if self.is_unbox {
            let str_ty = self.to_struct_type(gc);
            str_ty.into()
        } else {
            ptr_to_object_type(gc.context).into()
        }
    }
}

pub fn refcnt_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

fn _ptr_to_refcnt_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    refcnt_type(context).ptr_type(AddressSpace::from(0))
}

pub fn obj_id_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

pub fn ptr_to_object_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    context.i8_type().ptr_type(AddressSpace::from(0))
}

fn dtor_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    context
        .void_type()
        .fn_type(&[ptr_to_object_type(context).into()], false)
}

fn ptr_to_dtor_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    dtor_type(context).ptr_type(AddressSpace::from(0))
}

pub fn control_block_type<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> StructType<'c> {
    let mut fields = vec![refcnt_type(gc.context).into()];
    if gc.config.sanitize_memory {
        fields.push(obj_id_type(gc.context).into())
    }
    gc.context.struct_type(&fields, false)
}

pub fn ptr_to_control_block_type<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> PointerType<'c> {
    control_block_type(gc).ptr_type(AddressSpace::from(0))
}

pub fn lambda_function_type<'c, 'm>(
    ty: &Rc<TypeNode>,
    gc: &mut GenerationContext<'c, 'm>,
) -> FunctionType<'c> {
    // Any lamba takes argments.
    // In addition, if the lambda is closure (in other words, not a function pointer), it takes CAP, which is dynamic object consists of captured objects.
    // In the last, if ret_ty is unboxed, it takes parameter for pointer to store return value and returns void.

    // Arguments.
    let mut arg_tys: Vec<BasicMetadataTypeEnum> = ty
        .get_lambda_srcs()
        .iter()
        .map(|src| src.get_embedded_type(gc, &vec![]).into())
        .collect::<Vec<_>>();

    // CAP for closure.
    if ty.is_closure() {
        arg_tys.push(ptr_to_object_type(gc.context).into());
    }
    if ty.get_lambda_dst().is_box(gc.type_env()) {
        ptr_to_object_type(gc.context).fn_type(&arg_tys, false)
    } else {
        // Add ptr to rvo.
        arg_tys.push(ptr_to_object_type(gc.context).into());
        gc.context.void_type().fn_type(&arg_tys, false)
    }
}

pub const CLOSURE_FUNPTR_IDX: u32 = 0;
pub const CLOSURE_CAPTURE_IDX: u32 = CLOSURE_FUNPTR_IDX + 1;
pub const ARRAY_LEN_IDX: u32 = 1/* ControlBlock */;
pub const ARRAY_CAP_IDX: u32 = ARRAY_LEN_IDX + 1;
pub const ARRAY_BUF_IDX: u32 = ARRAY_CAP_IDX + 1;
pub const DYNAMIC_OBJ_DTOR_IDX: u32 = 1/* ControlBlock */;
pub const DYNAMIC_OBJ_CAP_IDX: u32 = DYNAMIC_OBJ_DTOR_IDX + 1;
pub fn struct_field_idx(is_unbox: bool) -> u32 {
    if is_unbox {
        0
    } else {
        1
    }
}

pub fn get_object_type(
    ty: &Rc<TypeNode>,
    capture: &Vec<Rc<TypeNode>>,
    type_env: &TypeEnv,
) -> ObjectType {
    assert!(ty.free_vars().is_empty());
    assert!(ty.is_dynamic() || capture.is_empty());
    let mut ret = ObjectType {
        field_types: vec![],
        is_unbox: true,
    };
    if ty.is_closure() {
        assert!(capture.is_empty());
        ret.is_unbox = true;
        ret.field_types
            .push(ObjectFieldType::LambdaFunction(ty.clone()));
        ret.field_types
            .push(ObjectFieldType::SubObject(make_dynamic_object_ty()));
    } else if ty.is_funptr() {
        assert!(capture.is_empty());
        ret.is_unbox = true;
        ret.field_types
            .push(ObjectFieldType::LambdaFunction(ty.clone()));
    } else {
        let tc = ty.toplevel_tycon().unwrap();
        let ti = type_env.tycons.get(&tc).unwrap();
        match ti.variant {
            TyConVariant::Primitive => {
                assert!(capture.is_empty());
                assert!(ti.is_unbox);
                ret.is_unbox = ti.is_unbox;
                if ty == &make_ptr_ty() {
                    ret.field_types.push(ObjectFieldType::Ptr);
                } else if ty == &make_bool_ty() {
                    ret.field_types.push(ObjectFieldType::I8);
                } else if ty == &make_u8_ty() {
                    ret.field_types.push(ObjectFieldType::I8);
                } else if ty == &make_i32_ty() {
                    ret.field_types.push(ObjectFieldType::I32);
                } else if ty == &make_u32_ty() {
                    ret.field_types.push(ObjectFieldType::U32);
                } else if ty == &make_i64_ty() {
                    ret.field_types.push(ObjectFieldType::I64);
                } else if ty == &make_u64_ty() {
                    ret.field_types.push(ObjectFieldType::U64);
                } else if ty == &make_f32_ty() {
                    ret.field_types.push(ObjectFieldType::F32);
                } else if ty == &make_f64_ty() {
                    ret.field_types.push(ObjectFieldType::F64);
                } else {
                    unreachable!()
                }
            }
            TyConVariant::Array => {
                assert!(capture.is_empty());
                let is_unbox = ti.is_unbox;
                assert!(!is_unbox);
                ret.is_unbox = is_unbox;
                ret.field_types.push(ObjectFieldType::ControlBlock);
                assert_eq!(ret.field_types.len(), ARRAY_LEN_IDX as usize);
                ret.field_types.push(ObjectFieldType::I64); // length
                assert_eq!(ret.field_types.len(), ARRAY_CAP_IDX as usize); // capacity
                ret.field_types
                    .push(ObjectFieldType::Array(ty.field_types(type_env)[0].clone()))
            }
            TyConVariant::Struct => {
                assert!(capture.is_empty());
                let is_unbox = ti.is_unbox;
                ret.is_unbox = is_unbox;
                if !is_unbox {
                    ret.field_types.push(ObjectFieldType::ControlBlock);
                }
                assert_eq!(ret.field_types.len(), struct_field_idx(is_unbox) as usize);
                let field_types = ty.field_types(type_env);
                if field_types.is_empty() {
                    // if this struct has no field, then this is unit `()`.
                    if is_unbox {
                        ret.field_types.push(ObjectFieldType::I8); // Avoid empty struct.
                    }
                } else {
                    for field_ty in field_types {
                        ret.field_types.push(ObjectFieldType::SubObject(field_ty));
                    }
                }
            }
            TyConVariant::Union => {
                assert!(capture.is_empty());
                let is_unbox = ti.is_unbox;
                ret.is_unbox = is_unbox;
                if !is_unbox {
                    ret.field_types.push(ObjectFieldType::ControlBlock);
                }
                ret.field_types.push(ObjectFieldType::UnionTag);
                ret.field_types
                    .push(ObjectFieldType::UnionBuf(ty.field_types(type_env)));
            }
            TyConVariant::DynamicObject => {
                let is_unbox = ti.is_unbox;
                assert_eq!(is_unbox, false);
                ret.is_unbox = false;
                ret.field_types.push(ObjectFieldType::ControlBlock);
                assert_eq!(ret.field_types.len(), DYNAMIC_OBJ_DTOR_IDX as usize);
                ret.field_types.push(ObjectFieldType::DtorFunction);
                assert_eq!(ret.field_types.len(), DYNAMIC_OBJ_CAP_IDX as usize);
                for cap in capture {
                    ret.field_types
                        .push(ObjectFieldType::SubObject(cap.clone()));
                }
            }
        }
    }
    ret
}

// Allocate an object.
pub fn allocate_obj<'c, 'm>(
    ty: Rc<TypeNode>,
    capture: &Vec<Rc<TypeNode>>,     // used in dynamic object
    array_cap: Option<IntValue<'c>>, // used in array
    gc: &mut GenerationContext<'c, 'm>,
    name: Option<&str>,
) -> Object<'c> {
    assert!(ty.free_vars().is_empty());
    assert!(ty.is_dynamic() || capture.is_empty());
    assert!(array_cap.is_some() == ty.is_array());
    let context = gc.context;
    let object_type = ty.get_object_type(capture, gc.type_env());
    let struct_type = object_type.to_struct_type(gc);

    // Allocate object
    let ptr_to_obj = if ty.is_array() {
        let sizeof = object_type.size_of(gc, array_cap);
        let ptr = gc
            .builder()
            .build_array_malloc(gc.context.i8_type(), sizeof, "malloc_array@allocate_obj")
            .unwrap();
        gc.cast_pointer(ptr, ptr_type(struct_type))
    } else {
        if object_type.is_unbox {
            gc.build_alloca_at_entry(struct_type, "alloca@allocate_obj")
        } else {
            gc.builder()
                .build_malloc(struct_type, "malloc@allocate_obj")
                .unwrap()
        }
    };

    // If sanitize memory, create object id.
    let mut object_id = obj_id_type(gc.context).const_int(0, false);
    if gc.config.sanitize_memory && !object_type.is_unbox {
        let string_ptr = name.unwrap_or("N/A");
        let string_ptr = gc
            .builder()
            .build_global_string_ptr(string_ptr, "name_of_obj");
        let string_ptr = string_ptr.as_pointer_value();
        let string_ptr = gc.builder().build_pointer_cast(
            string_ptr,
            gc.context.i8_type().ptr_type(AddressSpace::from(0)),
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
                if gc.config.sanitize_memory {
                    let ptr_to_obj_id = gc
                        .builder()
                        .build_struct_gep(ptr_to_control_block, 1, "ptr_to_obj_id")
                        .unwrap();
                    gc.builder().build_store(ptr_to_obj_id, object_id);
                }
            }
            ObjectFieldType::Ptr => {}
            ObjectFieldType::I8 => {}
            ObjectFieldType::I32 => {}
            ObjectFieldType::U32 => {}
            ObjectFieldType::I64 => {}
            ObjectFieldType::U64 => {}
            ObjectFieldType::F32 => {}
            ObjectFieldType::F64 => {}
            ObjectFieldType::SubObject(_) => {}
            ObjectFieldType::LambdaFunction(_) => {}
            ObjectFieldType::Array(_) => {
                assert_eq!(i, ARRAY_CAP_IDX as usize);
                // Set array size.
                let ptr_to_size_field = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, ARRAY_CAP_IDX, "ptr_to_size_field")
                    .unwrap();
                gc.builder()
                    .build_store(ptr_to_size_field, array_cap.unwrap());
            }
            ObjectFieldType::DtorFunction => {
                assert_eq!(i, DYNAMIC_OBJ_DTOR_IDX as usize);
                let ptr_to_dtor_field = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_dtor_field")
                    .unwrap();
                let dtor = get_dtor_ptr(&ty, capture, gc);
                gc.builder().build_store(ptr_to_dtor_field, dtor);
            }
            ObjectFieldType::UnionBuf(_) => {}
            ObjectFieldType::UnionTag => {}
        }
    }

    Object::new(ptr_to_obj, ty)
}

pub fn get_dtor_ptr<'c, 'm>(
    ty: &Rc<TypeNode>,
    capture: &Vec<Rc<TypeNode>>, // used in destructor of lambda
    gc: &mut GenerationContext<'c, 'm>,
) -> PointerValue<'c> {
    match create_dtor(ty, capture, gc) {
        Some(fv) => fv.as_global_value().as_pointer_value(),
        None => ptr_to_dtor_type(gc.context).const_null(),
    }
}

pub fn create_dtor<'c, 'm>(
    ty: &Rc<TypeNode>,
    capture: &Vec<Rc<TypeNode>>, // used in destructor of dynamic object.
    gc: &mut GenerationContext<'c, 'm>,
) -> Option<FunctionValue<'c>> {
    assert!(ty.free_vars().is_empty());
    assert!(ty.is_dynamic() || capture.is_empty());
    if ty.is_dynamic() && capture.is_empty() {
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
            let func = gc
                .module
                .add_function(&dtor_name, func_type, Some(Linkage::Internal));
            let bb = gc.context.append_basic_block(func, "entry");

            let _builder_guard = gc.push_builder();

            gc.builder().position_at_end(bb);
            let ptr_to_obj = func.get_first_param().unwrap().into_pointer_value();

            // In this function, we need to access captured fields, so fundamentally we cannot use Object's methods to access fields.
            let ptr_to_field =
                |field_idx: u32, gc: &mut GenerationContext<'c, 'm>| -> PointerValue<'c> {
                    let ptr_to_struct = gc.cast_pointer(ptr_to_obj, ptr_type(struct_type));
                    gc.builder()
                        .build_struct_gep(
                            ptr_to_struct,
                            field_idx,
                            &format!("ptr_to_{}th_field", field_idx),
                        )
                        .unwrap()
                };

            let mut union_tag: Option<IntValue<'c>> = None;
            for (i, ft) in object_type.field_types.iter().enumerate() {
                match ft {
                    ObjectFieldType::SubObject(ty) => {
                        let ptr_to_subobj = if ty.is_unbox(gc.type_env()) {
                            ptr_to_field(i as u32, gc)
                        } else {
                            gc.load_obj_field(ptr_to_obj, struct_type, i as u32)
                                .into_pointer_value()
                        };
                        gc.release(Object::new(ptr_to_subobj, ty.clone()));
                    }
                    ObjectFieldType::ControlBlock => {}
                    ObjectFieldType::LambdaFunction(_) => {}
                    ObjectFieldType::Ptr => {}
                    ObjectFieldType::I8 => {}
                    ObjectFieldType::I32 => {}
                    ObjectFieldType::U32 => {}
                    ObjectFieldType::I64 => {}
                    ObjectFieldType::U64 => {}
                    ObjectFieldType::F32 => {}
                    ObjectFieldType::F64 => {}
                    ObjectFieldType::Array(ty) => {
                        assert_eq!(i, ARRAY_CAP_IDX as usize);
                        let size = gc
                            .load_obj_field(ptr_to_obj, struct_type, ARRAY_LEN_IDX)
                            .into_int_value();
                        let buffer = ptr_to_field(ARRAY_BUF_IDX, gc);
                        ObjectFieldType::release_array_buf(gc, size, buffer, ty.clone());
                    }
                    ObjectFieldType::UnionTag => {
                        union_tag = Some(
                            gc.load_obj_field(ptr_to_obj, struct_type, i as u32)
                                .into_int_value(),
                        );
                    }
                    ObjectFieldType::UnionBuf(_) => {
                        let buf = ptr_to_field(i as u32, gc);
                        ObjectFieldType::release_union_buf(
                            gc,
                            buf,
                            union_tag.unwrap(),
                            &ty.field_types(gc.type_env()),
                        );
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
