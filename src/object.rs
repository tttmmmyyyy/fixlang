use std::sync::Arc;

use crate::error::error_exit;
use inkwell::{
    basic_block::BasicBlock,
    debug_info::{AsDIScope, DIType, DebugInfoBuilder},
    module::Linkage,
    types::{BasicMetadataTypeEnum, BasicType},
};

use super::*;

#[derive(Eq, PartialEq, Clone)]
pub enum ObjectFieldType {
    ControlBlock,
    TraverseFunction,
    LambdaFunction(Arc<TypeNode>), // Specify type of lambda
    Ptr,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    SubObject(Arc<TypeNode>, bool /* is_punched */),
    UnionBuf(Vec<Arc<TypeNode>>), // Embedded union.
    UnionTag,
    Array(Arc<TypeNode>), // field to store capacity (size) and buffer for elements.
}

impl ObjectFieldType {
    // Convert ObjectType to inkwell's BasicTypeEnum.
    // * `unboxed_path` -  See the comment for ObjectType::to_struct_type.
    pub fn to_basic_type<'c, 'm>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        unboxed_path: Vec<String>,
    ) -> BasicTypeEnum<'c> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(gc).into(),
            ObjectFieldType::TraverseFunction => ptr_to_traverser_type(gc.context).into(),
            ObjectFieldType::LambdaFunction(_ty) => {
                opaque_lambda_function_ptr_type(&gc.context).into()
            }
            ObjectFieldType::SubObject(ty, _is_punched) => {
                ty_to_object_ty(ty, &vec![], gc.type_env())
                    .to_embedded_type(gc, unboxed_path.clone())
            }
            ObjectFieldType::Ptr => gc.context.i8_type().ptr_type(AddressSpace::from(0)).into(),
            ObjectFieldType::I8 => gc.context.i8_type().into(),
            ObjectFieldType::U8 => gc.context.i8_type().into(),
            ObjectFieldType::I16 => gc.context.i16_type().into(),
            ObjectFieldType::U16 => gc.context.i16_type().into(),
            ObjectFieldType::I32 => gc.context.i32_type().into(),
            ObjectFieldType::U32 => gc.context.i32_type().into(),
            ObjectFieldType::I64 => gc.context.i64_type().into(),
            ObjectFieldType::U64 => gc.context.i64_type().into(),
            ObjectFieldType::F32 => gc.context.f32_type().into(),
            ObjectFieldType::F64 => gc.context.f64_type().into(),
            ObjectFieldType::Array(_) => gc.context.i64_type().into(), // Capacity field.
            ObjectFieldType::UnionTag => gc.context.i8_type().into(),
            ObjectFieldType::UnionBuf(field_tys) => {
                let mut size = 0;
                for field_ty in field_tys {
                    let struct_ty = ty_to_object_ty(
                        field_ty,
                        &vec![], /* captured list desn't effect sizeof */
                        gc.type_env(),
                    )
                    .to_embedded_type(gc, unboxed_path.clone());
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

    pub fn to_debug_type<'c, 'm>(&self, gc: &mut GenerationContext<'c, 'm>) -> DIType<'c> {
        match self {
            ObjectFieldType::ControlBlock => control_block_di_type(gc),
            ObjectFieldType::TraverseFunction => ptr_di_type("<ptr to traverser func>", gc),
            ObjectFieldType::LambdaFunction(_) => ptr_di_type("<ptr to closure func>", gc),
            ObjectFieldType::Ptr => ptr_di_type("Std::Ptr", gc),
            ObjectFieldType::I8 => gc
                .get_di_builder()
                .create_basic_type("Std::I8", 8, DW_ATE_SIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::U8 => gc
                .get_di_builder()
                .create_basic_type("Std::U8", 8, DW_ATE_UNSIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::I16 => gc
                .get_di_builder()
                .create_basic_type("Std::I16", 16, DW_ATE_SIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::U16 => gc
                .get_di_builder()
                .create_basic_type("Std::U16", 16, DW_ATE_UNSIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::I32 => gc
                .get_di_builder()
                .create_basic_type("Std::I32", 32, DW_ATE_SIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::U32 => gc
                .get_di_builder()
                .create_basic_type("Std::U32", 32, DW_ATE_UNSIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::I64 => gc
                .get_di_builder()
                .create_basic_type("Std::I64", 64, DW_ATE_SIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::U64 => gc
                .get_di_builder()
                .create_basic_type("Std::U64", 64, DW_ATE_UNSIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::F32 => gc
                .get_di_builder()
                .create_basic_type("Std::F32", 32, DW_ATE_FLOAT, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::F64 => gc
                .get_di_builder()
                .create_basic_type("Std::F64", 64, DW_ATE_FLOAT, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::SubObject(ty, _is_punched) => ty_to_debug_embedded_ty(ty.clone(), gc),
            ObjectFieldType::UnionBuf(tys) => {
                let basic_ty = self.to_basic_type(gc, vec![]);
                let size_in_bits = gc.target_data.get_bit_size(&basic_ty);
                let align_in_bits = gc.target_data.get_abi_alignment(&basic_ty) * 8;

                let mut elements = vec![];
                for (i, ty) in tys.iter().enumerate() {
                    let variant_ty = ty.get_embedded_type(gc, &vec![]);
                    let variant_debug_ty = ty_to_debug_embedded_ty(ty.clone(), gc);
                    let size_in_bits = gc.target_data.get_bit_size(&variant_ty);
                    let align_in_bits = gc.target_data.get_abi_alignment(&variant_ty) * 8;
                    let offset_in_bits = 0; // Union buffer has alignment 8.
                    let mem_ty = gc
                        .get_di_builder()
                        .create_member_type(
                            gc.get_di_compile_unit().as_debug_info_scope(),
                            &format!("<union variant {}>", i),
                            gc.create_di_file(None),
                            0,
                            size_in_bits,
                            align_in_bits,
                            offset_in_bits,
                            0,
                            variant_debug_ty,
                        )
                        .as_type();
                    elements.push(mem_ty);
                }
                let name = &format!(
                    "<union value {}>",
                    tys.iter()
                        .map(|ty| ty.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                // It seems that the second parameter of create_union_type (`name`, not `unique_id`) should vary depending on the element type, at least for lldb.
                gc.get_di_builder()
                    .create_union_type(
                        gc.get_di_compile_unit().as_debug_info_scope(),
                        &name,
                        gc.create_di_file(None),
                        0,
                        size_in_bits,
                        align_in_bits,
                        0,
                        &elements,
                        0,
                        &name,
                    )
                    .as_type()
            }
            ObjectFieldType::UnionTag => gc
                .get_di_builder()
                .create_basic_type("<union tag>", 8, DW_ATE_UNSIGNED, 0)
                .unwrap()
                .as_type(),
            ObjectFieldType::Array(elem_ty) => {
                // struct_ty = [capacity, element0]
                let struct_ty = ObjectType {
                    field_types: vec![self.clone()],
                    is_unbox: false,
                    name: "N/A".to_string(),
                }
                .to_struct_type(gc, vec![]);

                // Create element type for capacity field.
                let capacity_ty = self.to_basic_type(gc, vec![]);
                let capacity_debug_ty = ObjectFieldType::I64.to_debug_type(gc);
                let capacity_size_in_bits = gc.target_data.get_bit_size(&capacity_ty);
                let capacity_align_in_bits = gc.target_data.get_abi_alignment(&capacity_ty) * 8;
                let capacity_offset_in_bits = gc
                    .target_data
                    .offset_of_element(&struct_ty, ARRAY_CAP_IDX - ARRAY_CAP_IDX)
                    .unwrap()
                    * 8;
                let capacity_member_ty = gc
                    .get_di_builder()
                    .create_member_type(
                        gc.get_di_compile_unit().as_debug_info_scope(),
                        "<array capacity>",
                        gc.create_di_file(None),
                        0,
                        capacity_size_in_bits,
                        capacity_align_in_bits,
                        capacity_offset_in_bits as u64,
                        0,
                        capacity_debug_ty,
                    )
                    .as_type();

                // Create element type for buffer field.
                let element_ty =
                    ty_to_object_ty(elem_ty, &vec![], gc.type_env()).to_embedded_type(gc, vec![]);
                let element_debug_ty = ty_to_debug_embedded_ty(elem_ty.clone(), gc);
                let element_size_in_bits = gc.target_data.get_bit_size(&element_ty);
                let element_align_in_bits = gc.target_data.get_abi_alignment(&element_ty) * 8;
                let element_offset_in_bits = gc
                    .target_data
                    .offset_of_element(&struct_ty, ARRAY_BUF_IDX - ARRAY_CAP_IDX)
                    .unwrap()
                    * 8;
                let element_array_ty = gc
                    .get_di_builder()
                    .create_array_type(
                        element_debug_ty,
                        element_size_in_bits,
                        element_align_in_bits,
                        &[0..100],
                    )
                    .as_type();
                let element_member_ty = gc
                    .get_di_builder()
                    .create_member_type(
                        gc.get_di_compile_unit().as_debug_info_scope(),
                        "<array elements>",
                        gc.create_di_file(None),
                        0,
                        element_size_in_bits,
                        element_align_in_bits,
                        element_offset_in_bits as u64,
                        0,
                        element_array_ty,
                    )
                    .as_type();

                let size_in_bits = gc.target_data.get_bit_size(&struct_ty);
                let align_in_bits = gc.target_data.get_abi_alignment(&struct_ty) * 8;
                let name = format!("<array buffer of `{}`>", elem_ty.to_string());
                // It seems that the second parameter of create_struct_type (`name`, not `unique_id`) should vary depending on the element type, at least for lldb.
                gc.get_di_builder()
                    .create_struct_type(
                        gc.get_di_compile_unit().as_debug_info_scope(),
                        &name,
                        gc.create_di_file(None),
                        0,
                        size_in_bits,
                        align_in_bits,
                        0,
                        None,
                        &[capacity_member_ty, element_member_ty],
                        0,
                        None,
                        &name,
                    )
                    .as_type()
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

    pub fn release_or_mark_array_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        size: IntValue<'c>,
        buffer: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        work_type: TraverserWorkType,
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
            // Perform release or mark global or mark threaded.
            let obj = Object::new(obj, elem_ty.clone());
            gc.release_or_mark(obj, work_type);
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
                .build_int_compare(IntPredicate::UGE, idx, len, "is_out_of_range");
        let out_of_range_bb = gc.context.append_basic_block(curr_func, "out_of_range_bb");
        let in_range_bb = gc.context.append_basic_block(curr_func, "in_range_bb");
        gc.builder()
            .build_conditional_branch(is_out_of_range, out_of_range_bb, in_range_bb);
        gc.builder().position_at_end(out_of_range_bb);
        gc.panic("Index out of range.\n");
        gc.builder().build_unconditional_branch(in_range_bb);
        gc.builder().position_at_end(in_range_bb);
    }

    // Read an element of array.
    // Returned object is not retained.
    pub fn read_from_array_buf_noretain<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        len: Option<IntValue<'c>>, // If none, bounds checking is omitted.
        buffer: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
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
        elem_ty: Arc<TypeNode>,
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

        // Release element that is already at the place (if required).
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
        elem_ty: Arc<TypeNode>,
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

    // Perform retain or release or mark global or mark threaded on an object included in a union buffer.
    fn retain_release_mark_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Arc<TypeNode>>,
        work_type: Option<TraverserWorkType>, // None for retain, and Some for release or mark global threaded.
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
                .to_basic_type(gc, vec![])
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
            if work_type.is_none() {
                gc.retain(obj);
            } else {
                gc.release_or_mark(obj, work_type.unwrap());
            }
            gc.builder().build_unconditional_branch(end_bb);

            // Implement the case tag is unmatch.
            gc.builder().position_at_end(unmatch_bb);
            last_unmatch_bb = Some(unmatch_bb);
        }

        // Implement last unmatch bb.
        let last_unmatch_bb = last_unmatch_bb.unwrap();
        gc.builder().position_at_end(last_unmatch_bb);
        gc.panic("All union variants unmatch!\n"); // unreachable didn't work as I expected.
        gc.builder().build_unconditional_branch(end_bb);

        gc.builder().position_at_end(end_bb);
    }

    pub fn retain_union_buf<'c, 'm>(
        gc: &mut GenerationContext<'c, 'm>,
        buf: PointerValue<'c>,
        tag: IntValue<'c>,
        field_types: &Vec<Arc<TypeNode>>,
    ) {
        ObjectFieldType::retain_release_mark_union_buf(gc, buf, tag, field_types, None);
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
        elem_ty: &Arc<TypeNode>,
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
        elem_ty: &Arc<TypeNode>,
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
        gc.panic("Union variant unmatch!\n");
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
    ) {
        let field_offset = struct_field_idx(str.ty.is_unbox(gc.type_env()));
        let field_val = field.value(gc);
        str.store_field_nocap(gc, field_offset + field_idx as u32, field_val);
    }
}

#[derive(Eq, PartialEq, Clone)]
pub struct ObjectType {
    pub field_types: Vec<ObjectFieldType>,
    pub is_unbox: bool,
    pub name: Name,
}

impl ObjectType {
    // Convert ObjectType to inkwell's StructType.
    // * `unboxed_path` - When unboxed types are used recursively in each definition, this function can fall into infinite recursion. `unboxed_path` is an argument to detect this infinite loop and to generate a good error message. When you call to_struct_type from outside, specify an empty Vec. When to_struct_type calls itself (possibly via another function), unboxed_path contains the sequence of unboxed types that to_struct_type has been called on so far.
    pub fn to_struct_type<'c, 'm>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        mut unboxed_path: Vec<String>,
    ) -> StructType<'c> {
        if self.is_unbox {
            if unboxed_path.contains(&self.name) {
                // There is a loop of unboxed types.
                error_exit(&format!("Cannot determine the layout of type `{}`. There are circular definitions by unboxed types. Please change some types to boxed.", &self.name));
            }
            unboxed_path.push(self.name.clone());
        } else {
            unboxed_path.clear();
        }

        let mut fields: Vec<BasicTypeEnum<'c>> = vec![];
        for (i, field_type) in self.field_types.iter().enumerate() {
            fields.push(field_type.to_basic_type(gc, unboxed_path.clone()));
            match field_type {
                ObjectFieldType::Array(ty) => {
                    assert_eq!(i, self.field_types.len() - 1); // ArraySize must be the last field.
                    assert!(!self.is_unbox); // Array has to be boxed.

                    // Add space for one element.
                    // This is for:
                    // - to get the pointer to the first element by gep of this struct type.
                    // - used in implementation of size_of method.
                    // - in to_debug_type function.
                    fields.push(
                        ty.get_object_type(&vec![], gc.type_env())
                            .to_embedded_type(gc, unboxed_path.clone())
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
                .to_struct_type(gc, vec![])
                .size_of()
                .unwrap();
            let struct_ty = self.to_struct_type(gc, vec![]);
            let ptr_int_ty = gc.context.ptr_sized_int_type(&gc.target_data, None);
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
            self.to_struct_type(gc, vec![]).size_of().unwrap()
        }
    }

    // Get type used when this object is embedded.
    // i.e., for unboxed type, a pointer; for unboxed type, a struct.
    // * `unboxed_path` -  See the comment for ObjectType::to_struct_type.
    pub fn to_embedded_type<'c, 'm>(
        &self,
        gc: &mut GenerationContext<'c, 'm>,
        unboxed_path: Vec<String>,
    ) -> BasicTypeEnum<'c> {
        if self.is_unbox {
            let str_ty = self.to_struct_type(gc, unboxed_path);
            str_ty.into()
        } else {
            ptr_to_object_type(gc.context).into()
        }
    }
}

pub fn refcnt_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

pub fn refcnt_di_type<'ctx>(builder: &DebugInfoBuilder<'ctx>) -> DIType<'ctx> {
    builder
        .create_basic_type("<refcnt>", 64, DW_ATE_UNSIGNED, 0)
        .unwrap()
        .as_type()
}

// State for reference counting.
// Values of this fields are REFCNT_STATE_* constants.
pub fn refcnt_state_type<'c>(context: &'c Context) -> IntType<'c> {
    context.i8_type()
}

pub fn obj_id_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i64_type()
}

pub fn obj_id_di_type<'ctx>(builder: &DebugInfoBuilder<'ctx>) -> DIType<'ctx> {
    builder
        .create_basic_type("<object id>", 64, DW_ATE_UNSIGNED, 0)
        .unwrap()
        .as_type()
}

pub fn ptr_to_object_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    context.i8_type().ptr_type(AddressSpace::from(0))
}

fn traverser_type<'ctx>(context: &'ctx Context) -> FunctionType<'ctx> {
    context.void_type().fn_type(
        &[
            ptr_to_object_type(context).into(),  // Pointer to object.
            traverser_work_type(context).into(), // Data to specify work.
        ],
        false,
    )
}

pub fn traverser_work_type<'c>(context: &'c Context) -> IntType<'c> {
    context.i8_type()
}

fn ptr_to_traverser_type<'ctx>(context: &'ctx Context) -> PointerType<'ctx> {
    traverser_type(context).ptr_type(AddressSpace::from(0))
}

pub fn control_block_type<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> StructType<'c> {
    let mut fields = vec![];
    assert_eq!(fields.len(), CTRL_BLK_REFCNT_IDX as usize);
    fields.push(refcnt_type(gc.context).into());
    assert_eq!(fields.len(), CTRL_BLK_REFCNT_STATE_IDX as usize);
    fields.push(refcnt_state_type(gc.context).into());
    if gc.config.sanitize_memory {
        assert_eq!(fields.len(), CTRL_BLK_OBJ_ID_IDX as usize);
        fields.push(obj_id_type(gc.context).into())
    }
    gc.context.struct_type(&fields, false)
}

pub fn control_block_di_type<'c, 'm>(gc: &mut GenerationContext<'c, 'm>) -> DIType<'c> {
    let str_type = control_block_type(gc);

    let refcnt_ty = refcnt_type(gc.context);
    let refcnt_size_in_bits = gc.target_data.get_bit_size(&refcnt_ty);
    let refcnt_align_in_bits = gc.target_data.get_abi_alignment(&refcnt_ty) * 8;
    let refcnt_offset_in_bits = gc.target_data.offset_of_element(&str_type, 0).unwrap();
    let refcnt_member = gc
        .get_di_builder()
        .create_member_type(
            gc.get_di_compile_unit().as_debug_info_scope(),
            "<refcnt>",
            gc.create_di_file(None),
            0,
            refcnt_size_in_bits,
            refcnt_align_in_bits,
            refcnt_offset_in_bits,
            0,
            refcnt_di_type(gc.get_di_builder()),
        )
        .as_type();
    let mut elements = vec![refcnt_member];
    if gc.config.sanitize_memory {
        let obj_id_ty = refcnt_type(gc.context);
        let obj_id_size_in_bits = gc.target_data.get_bit_size(&obj_id_ty);
        let obj_id_align_in_bits = gc.target_data.get_abi_alignment(&obj_id_ty) * 8;
        let obj_id_offset_in_bits = gc.target_data.offset_of_element(&str_type, 1).unwrap();
        let obj_id_member = gc
            .get_di_builder()
            .create_member_type(
                gc.get_di_compile_unit().as_debug_info_scope(),
                "<object id>",
                gc.create_di_file(None),
                0,
                obj_id_size_in_bits,
                obj_id_align_in_bits,
                obj_id_offset_in_bits,
                0,
                obj_id_di_type(gc.get_di_builder()),
            )
            .as_type();
        elements.push(obj_id_member);
    }

    let name = "<control block>";
    let size_in_bits = gc.target_data.get_bit_size(&str_type);
    let align_in_bits = gc.target_data.get_abi_alignment(&str_type) * 8;
    gc.get_di_builder()
        .create_struct_type(
            gc.get_di_compile_unit().as_debug_info_scope(),
            name,
            gc.create_di_file(None),
            0,
            size_in_bits,
            align_in_bits,
            0,
            None,
            &elements,
            0,
            None,
            name,
        )
        .as_type()
}

pub fn ptr_to_control_block_type<'c, 'm>(gc: &GenerationContext<'c, 'm>) -> PointerType<'c> {
    control_block_type(gc).ptr_type(AddressSpace::from(0))
}

pub fn ptr_di_type<'c, 'm>(name: &str, gc: &mut GenerationContext<'c, 'm>) -> DIType<'c> {
    let ptr_ty = gc.context.i8_type().ptr_type(AddressSpace::from(0));
    let size_in_bits = gc.target_data.get_bit_size(&ptr_ty);
    gc.get_di_builder()
        .create_basic_type(name, size_in_bits, DW_ATE_ADDRESS, 0)
        .unwrap()
        .as_type()
}

pub fn lambda_function_type<'c, 'm>(
    ty: &Arc<TypeNode>,
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

// Opaque function pointer type used to handle type definition such as
// `type Foo = box struct { func : Foo -> Foo }`.
pub fn opaque_lambda_function_ptr_type<'c>(ctx: &'c Context) -> PointerType<'c> {
    ctx.i8_type().ptr_type(AddressSpace::from(0))
}

pub fn struct_field_idx(is_unbox: bool) -> u32 {
    if is_unbox {
        0
    } else {
        1
    }
}

pub fn ty_to_object_ty(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>,
    type_env: &TypeEnv,
) -> ObjectType {
    assert!(ty.free_vars().is_empty());
    assert!(ty.is_dynamic() || capture.is_empty());
    let mut ret = ObjectType {
        field_types: vec![],
        is_unbox: true,
        name: ty.to_string_normalize(),
    };
    if ty.is_closure() {
        assert!(capture.is_empty());
        ret.is_unbox = true;
        ret.field_types
            .push(ObjectFieldType::LambdaFunction(ty.clone()));
        ret.field_types
            .push(ObjectFieldType::SubObject(make_dynamic_object_ty(), false));
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
                    ret.field_types.push(ObjectFieldType::U8);
                } else if ty == &make_i8_ty() {
                    ret.field_types.push(ObjectFieldType::I8);
                } else if ty == &make_u8_ty() {
                    ret.field_types.push(ObjectFieldType::U8);
                } else if ty == &make_i16_ty() {
                    ret.field_types.push(ObjectFieldType::I16);
                } else if ty == &make_u16_ty() {
                    ret.field_types.push(ObjectFieldType::U16);
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
                for (field_idx, field_ty) in field_types.into_iter().enumerate() {
                    let punched = ti.fields[field_idx].is_punched;
                    ret.field_types
                        .push(ObjectFieldType::SubObject(field_ty, punched));
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
                assert_eq!(ret.field_types.len(), DYNAMIC_OBJ_TRAVARSER_IDX as usize);
                ret.field_types.push(ObjectFieldType::TraverseFunction);
                assert_eq!(ret.field_types.len(), DYNAMIC_OBJ_CAP_IDX as usize);
                for cap in capture {
                    ret.field_types
                        .push(ObjectFieldType::SubObject(cap.clone(), false));
                }
            }
        }
    }
    ret
}

// Allocate an object.
pub fn allocate_obj<'c, 'm>(
    ty: Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>,         // used in dynamic object
    array_capacity: Option<IntValue<'c>>, // used in array
    gc: &mut GenerationContext<'c, 'm>,
    name: Option<&str>,
) -> Object<'c> {
    assert!(ty.free_vars().is_empty());
    assert!(ty.is_dynamic() || capture.is_empty());
    assert!(array_capacity.is_some() == ty.is_array());
    let context = gc.context;
    let object_type = ty.get_object_type(capture, gc.type_env());
    let struct_type = object_type.to_struct_type(gc, vec![]);

    // Allocate object
    let ptr_to_obj = if ty.is_array() {
        let sizeof = object_type.size_of(gc, array_capacity);
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
        let obj_id = gc.call_runtime(RUNTIME_REPORT_MALLOC, &[ptr.into(), string_ptr.into()]);
        object_id = obj_id.try_as_basic_value().unwrap_left().into_int_value();
    }

    // Initialize refcnt, refcnt_state and traverser for dynamic object.
    for (i, ft) in object_type.field_types.iter().enumerate() {
        match ft {
            ObjectFieldType::ControlBlock => {
                assert_eq!(i, 0);
                // Get pointer to control block.
                let ptr_to_control_block = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_control_block")
                    .unwrap();

                // Initialize the reference counter 1.
                let ptr_to_refcnt = gc
                    .builder()
                    .build_struct_gep(ptr_to_control_block, 0, "ptr_to_refcnt")
                    .unwrap();
                gc.builder()
                    .build_store(ptr_to_refcnt, refcnt_type(context).const_int(1, false));

                // Initialize the reference counter state to REFCNT_STATE_LOCAL.
                let ptr_to_refcnt_state = gc
                    .builder()
                    .build_struct_gep(ptr_to_control_block, 1, "ptr_to_refcnt_state")
                    .unwrap();
                gc.builder().build_store(
                    ptr_to_refcnt_state,
                    refcnt_state_type(context).const_int(REFCNT_STATE_LOCAL as u64, false),
                );

                // If sanitize memory, set object id.
                if gc.config.sanitize_memory {
                    let ptr_to_obj_id = gc
                        .builder()
                        .build_struct_gep(
                            ptr_to_control_block,
                            CTRL_BLK_OBJ_ID_IDX,
                            "ptr_to_obj_id",
                        )
                        .unwrap();
                    gc.builder().build_store(ptr_to_obj_id, object_id);
                }
            }
            ObjectFieldType::Ptr => {}
            ObjectFieldType::I8 => {}
            ObjectFieldType::U8 => {}
            ObjectFieldType::I16 => {}
            ObjectFieldType::U16 => {}
            ObjectFieldType::I32 => {}
            ObjectFieldType::U32 => {}
            ObjectFieldType::I64 => {}
            ObjectFieldType::U64 => {}
            ObjectFieldType::F32 => {}
            ObjectFieldType::F64 => {}
            ObjectFieldType::SubObject(_, _) => {}
            ObjectFieldType::LambdaFunction(_) => {}
            ObjectFieldType::Array(_) => {
                assert_eq!(i, ARRAY_CAP_IDX as usize);
                // Set array size.
                let ptr_to_size_field = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, ARRAY_CAP_IDX, "ptr_to_size_field")
                    .unwrap();
                gc.builder()
                    .build_store(ptr_to_size_field, array_capacity.unwrap());
            }
            ObjectFieldType::TraverseFunction => {
                assert_eq!(i, DYNAMIC_OBJ_TRAVARSER_IDX as usize);
                let ptr_to_dtor_field = gc
                    .builder()
                    .build_struct_gep(ptr_to_obj, i as u32, "ptr_to_dtor_field")
                    .unwrap();
                let dtor = get_traverser_ptr(&ty, capture, gc);
                gc.builder().build_store(ptr_to_dtor_field, dtor);
            }
            ObjectFieldType::UnionBuf(_) => {}
            ObjectFieldType::UnionTag => {}
        }
    }

    Object::new(ptr_to_obj, ty)
}

pub fn get_traverser_ptr<'c, 'm>(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>, // used in destructor of lambda
    gc: &mut GenerationContext<'c, 'm>,
) -> PointerValue<'c> {
    match create_traverser(ty, capture, gc) {
        Some(fv) => fv.as_global_value().as_pointer_value(),
        None => ptr_to_traverser_type(gc.context).const_null(),
    }
}

// Traverser function is a function that traverses all fields of an object and does some work on them.
// Traverser function takes two arguments: a pointer to the object, and an 8-bit integer value called `work`.
// If `work` is 0, then traverser function works as destructor of an object.
// If `work` is 1, then traverser function marks all reachable objects as global.
// If `work` is 2, then traverser function marks all reachable objects as threaded.
// This function returns `None` if traverser function is empty.
pub fn create_traverser<'c, 'm>(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>, // used in destructor of dynamic object.
    gc: &mut GenerationContext<'c, 'm>,
) -> Option<FunctionValue<'c>> {
    assert!(ty.free_vars().is_empty());
    assert!(ty.is_dynamic() || capture.is_empty());
    if ty.is_dynamic() && capture.is_empty() {
        return None;
    }
    if ty.is_fully_unboxed(gc.type_env()) {
        return None;
    }
    let trav_name = ty.traverser_name(capture);
    match gc.module.get_function(&trav_name) {
        Some(fv) => Some(fv),
        None => {
            // Define traverser function.
            let object_type = ty_to_object_ty(ty, capture, gc.type_env());
            let struct_type = object_type.to_struct_type(gc, vec![]);
            let func_type = traverser_type(gc.context);
            let func = gc
                .module
                .add_function(&trav_name, func_type, Some(Linkage::Internal));
            let bb = gc.context.append_basic_block(func, "entry");

            let _builder_guard = gc.push_builder();

            gc.builder().position_at_end(bb);
            let ptr_to_obj = func.get_first_param().unwrap().into_pointer_value();
            let work = func.get_nth_param(1).unwrap().into_int_value();

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

            // Depending the value of `work`, do different works: destruction of objects (`work == 0`), or marking object as global (`work` == 1).
            let release_bb = gc.context.append_basic_block(func, "release_bb@traverser");
            let mark_global_bb = gc
                .context
                .append_basic_block(func, "mark_global_bb@traverser");
            let mut work_bbs = vec![
                (TRAVERSER_WORK_RELEASE, release_bb),
                (TRAVERSER_WORK_MARK_GLOBAL, mark_global_bb),
            ];
            if gc.config.threaded {
                let mark_threaded_bb = gc
                    .context
                    .append_basic_block(func, "mark_threaded_bb@traverser");
                work_bbs.push((TRAVERSER_WORK_MARK_THREADED, mark_threaded_bb))
            }
            let work_ty = traverser_work_type(gc.context);
            let mut switches = work_bbs
                .iter()
                .map(|(work_type, bb)| (work_ty.const_int(*work_type as u64, false), bb.clone()))
                .collect::<Vec<_>>();
            gc.builder()
                .build_switch(work, switches.pop().unwrap().1, &switches);

            for (work_type, work_bb) in work_bbs.iter() {
                let work_type = TraverserWorkType(*work_type);
                gc.builder().position_at_end(*work_bb);

                let mut union_tag: Option<IntValue<'c>> = None;
                for (i, ft) in object_type.field_types.iter().enumerate() {
                    match ft {
                        ObjectFieldType::SubObject(ty, is_punched) => {
                            if *is_punched {
                                continue;
                            }
                            let ptr_to_subobj = if ty.is_unbox(gc.type_env()) {
                                ptr_to_field(i as u32, gc)
                            } else {
                                gc.load_obj_field(ptr_to_obj, struct_type, i as u32)
                                    .into_pointer_value()
                            };
                            let obj = Object::new(ptr_to_subobj, ty.clone());
                            gc.release_or_mark(obj, work_type);
                        }
                        ObjectFieldType::ControlBlock => {}
                        ObjectFieldType::LambdaFunction(_) => {}
                        ObjectFieldType::Ptr => {}
                        ObjectFieldType::I8 => {}
                        ObjectFieldType::U8 => {}
                        ObjectFieldType::I16 => {}
                        ObjectFieldType::U16 => {}
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
                            ObjectFieldType::release_or_mark_array_buf(
                                gc,
                                size,
                                buffer,
                                ty.clone(),
                                work_type,
                            );
                        }
                        ObjectFieldType::UnionTag => {
                            union_tag = Some(
                                gc.load_obj_field(ptr_to_obj, struct_type, i as u32)
                                    .into_int_value(),
                            );
                        }
                        ObjectFieldType::UnionBuf(_) => {
                            let buf = ptr_to_field(i as u32, gc);
                            ObjectFieldType::retain_release_mark_union_buf(
                                gc,
                                buf,
                                union_tag.unwrap(),
                                &ty.field_types(gc.type_env()),
                                Some(work_type),
                            );
                        }
                        ObjectFieldType::TraverseFunction => {}
                    }
                }
                gc.builder().build_return(None);
            }

            Some(func)
        }
    }
}

pub fn ty_to_debug_embedded_ty<'c, 'm>(
    ty: Arc<TypeNode>,
    gc: &mut GenerationContext<'c, 'm>,
) -> DIType<'c> {
    let debug_str_ty = ty_to_debug_struct_ty(ty.clone(), gc);
    if ty.is_box(&gc.type_env()) {
        let ptr_ty = gc.context.i8_type().ptr_type(AddressSpace::from(0));
        let size_in_bits = gc.target_data.get_bit_size(&ptr_ty);
        let align_in_bits = gc.target_data.get_abi_alignment(&ptr_ty) * 8;
        gc.get_di_builder()
            .create_pointer_type(
                "<pointer to boxed value>",
                debug_str_ty,
                size_in_bits,
                align_in_bits,
                AddressSpace::from(0),
            )
            .as_type()
    } else {
        debug_str_ty
    }
}

pub fn ty_to_debug_struct_ty<'c, 'm>(
    ty: Arc<TypeNode>,
    gc: &mut GenerationContext<'c, 'm>,
) -> DIType<'c> {
    let name = &ty.to_string();
    let obj_type = ty_to_object_ty(&ty, &vec![], gc.type_env());
    let is_primitive = !ty.is_closure()
        && ty.toplevel_tycon_info(gc.type_env()).variant == TyConVariant::Primitive;
    if is_primitive {
        // Primitive case
        if ty.toplevel_tycon().unwrap().is_boolean() {
            return gc
                .get_di_builder()
                .create_basic_type(
                    &format!("{}::{}", STD_NAME, BOOL_NAME),
                    8,
                    DW_ATE_BOOLEAN,
                    0,
                )
                .unwrap()
                .as_type();
        }
        // Do not wrap the element type into struct type.
        obj_type.field_types[0].to_debug_type(gc)
    } else {
        // NOTE: Maybe we should use llvm's DataLayout::getStructLayout instead of get_abi_alignment, but it seems that the function isn't wrapped in llvm-sys.
        let str_type = obj_type.to_struct_type(gc, vec![]);
        let size_in_bits = gc.target_data.get_bit_size(&str_type);
        let align_in_bits = gc.target_data.get_abi_alignment(&str_type) * 8;

        let mut subelement_names = vec![];
        if !ty.is_closure() {
            let tc_info = ty.toplevel_tycon_info(gc.type_env());
            subelement_names = tc_info
                .fields
                .iter()
                .map(|field| field.name.clone())
                .collect();
        }

        let mut elements = vec![];
        for (i, field) in obj_type.field_types.iter().enumerate() {
            let mut member_name = match field {
                ObjectFieldType::SubObject(ty, _) => {
                    if !subelement_names.is_empty() {
                        subelement_names.remove(0)
                    } else {
                        format!("<subelement of type {}>", ty.to_string())
                    }
                }
                ObjectFieldType::ControlBlock => "<control block>".to_string(),
                ObjectFieldType::TraverseFunction => "<ptr to traverser function>".to_string(),
                ObjectFieldType::LambdaFunction(_) => "<ptr to lambda function>".to_string(),
                ObjectFieldType::Ptr => "<Ptr member>".to_string(),
                ObjectFieldType::I8 => "<I8 member>".to_string(),
                ObjectFieldType::U8 => "<U8 member>".to_string(),
                ObjectFieldType::I16 => "<I16 member>".to_string(),
                ObjectFieldType::U16 => "<U16 member>".to_string(),
                ObjectFieldType::I32 => "<I32 member>".to_string(),
                ObjectFieldType::U32 => "<U32 member>".to_string(),
                ObjectFieldType::I64 => "<I64 member>".to_string(),
                ObjectFieldType::U64 => "<U64 member>".to_string(),
                ObjectFieldType::F32 => "<F32 member>".to_string(),
                ObjectFieldType::F64 => "<F64 member>".to_string(),
                ObjectFieldType::UnionBuf(_) => "<union value>".to_string(),
                ObjectFieldType::UnionTag => "<union tag>".to_string(),
                ObjectFieldType::Array(_) => "<array>".to_string(),
            };
            if ty.is_array() && i as u32 == ARRAY_LEN_IDX {
                member_name = "<array size>".to_string();
            }

            let element_di_ty = field.to_debug_type(gc);
            let elemet_ty = field.to_basic_type(gc, vec![]);
            let size_in_bits = gc.target_data.get_bit_size(&elemet_ty);
            let align_in_bits = gc.target_data.get_abi_alignment(&elemet_ty) * 8;
            let offset_in_bits = gc
                .target_data
                .offset_of_element(&str_type, i as u32)
                .unwrap()
                * 8;
            let mem_ty = gc
                .get_di_builder()
                .create_member_type(
                    gc.get_di_compile_unit().as_debug_info_scope(),
                    &member_name,
                    gc.create_di_file(None),
                    0,
                    size_in_bits,
                    align_in_bits,
                    offset_in_bits,
                    0,
                    element_di_ty,
                )
                .as_type();
            elements.push(mem_ty);
        }

        gc.get_di_builder()
            .create_struct_type(
                gc.get_di_compile_unit().as_debug_info_scope(),
                name,
                gc.create_di_file(None),
                0,
                size_in_bits,
                align_in_bits,
                0,
                None,
                &elements,
                0,
                None,
                name,
            )
            .as_type()
    }
}
