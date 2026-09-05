use crate::ast::name::FullName;
use crate::ast::program::TypeEnv;
use crate::ast::types::{TyConVariant, TypeNode};
use crate::constants::{
    RefcntState, TraverserWorkType, ARRAY_ALIGNED_ALLOC_THRESHOLD, ARRAY_BUF_ALIGNMENT,
    ARRAY_CAP_IDX, ARRAY_SIZE_IDX, ARRAY_STORAGE_ALLOC_SLACK, ARRAY_STORAGE_IDX, BOOL_NAME,
    BOXED_TYPE_DATA_IDX, CTRL_BLK_ALLOC_OFFSET_IDX, CTRL_BLK_REFCNT_IDX, CTRL_BLK_REFCNT_STATE_IDX,
    DEBUG_ARRAY_ASSUMED_LEN, DW_ATE_ADDRESS, DW_ATE_BOOLEAN, DW_ATE_FLOAT, DW_ATE_SIGNED,
    DW_ATE_UNSIGNED, DYNAMIC_OBJ_CAP_IDX, DYNAMIC_OBJ_TRAVARSER_IDX, MAX_UNION_VARIANTS,
    PUNCHED_ARRAY_ARRAY_IDX, PUNCHED_ARRAY_HOLE_IDX, STD_NAME, STORAGE_BUF_IDX,
    TRAVERSER_WORK_MARK_GLOBAL, TRAVERSER_WORK_MARK_THREADED, TRAVERSER_WORK_RELEASE,
    UNION_DATA_IDX, UNION_TAG_BITS, UNION_TAG_IDX,
};
use crate::fixstd::builtin::{
    make_array_storage_ty, make_dynamic_object_ty, make_f32_ty, make_f64_ty, make_i16_ty,
    make_i32_ty, make_i64_ty, make_i8_ty, make_iostate_ty, make_ptr_ty, make_u16_ty, make_u32_ty,
    make_u64_ty, make_u8_ty,
};
use crate::fixstd::runtime::{
    RUNTIME_ARRAY_SIZE_OVERFLOW, RUNTIME_INDEX_OUT_OF_RANGE, RUNTIME_MALLOC,
    RUNTIME_NEGATIVE_ARRAY_SIZE,
};
use crate::generator::{is_const_one, Generator, Object};
use crate::misc::Map;
use crate::rc_ir::ast::RcState;
use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, FunctionType, IntType, StructType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use inkwell::{
    basic_block::BasicBlock,
    debug_info::{AsDIScope, DIType, DebugInfoBuilder},
    module::Linkage,
    types::{BasicMetadataTypeEnum, BasicType},
};
use inkwell::{AddressSpace, IntPredicate};
use std::sync::{Arc, OnceLock};

/// One field of the LLVM struct a Fix object is laid out as: either runtime machinery (the control
/// block, the traverse function, a union's tag) or a piece of the Fix value itself (a scalar, a
/// subobject, a union's payload buffer, an array).
// PROOF: P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
#[derive(Eq, PartialEq, Clone)]
pub enum ObjectFieldType {
    /// The reference count and the flags the runtime keeps for every boxed object, which a boxed
    /// object's layout begins with.
    ControlBlock,
    /// A pointer to the function that walks the fields following it, for an object whose fields the
    /// type alone leaves open (`#DynamicObject`).
    TraverseFunction,
    /// The function pointer of a closure of the given type, called with the closure's capture.
    LambdaFunction(Arc<TypeNode>),
    /// A `Std::Ptr`: an address the program carries and whose target it leaves alone.
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
    /// A value of the given type laid out in place. The flag marks a punched field, whose value has
    /// been moved out: the space stays, and traversal passes over it.
    SubObject(Arc<TypeNode>, bool /* is_punched */),
    /// The payload buffer of a union, sized and aligned to hold whichever of the given variant
    /// types the tag names.
    UnionBuf(Vec<Arc<TypeNode>>),
    /// The integer saying which variant a union carries.
    UnionTag,
    /// The tail of an `Array` object: a capacity slot followed by a flexible buffer of elements of
    /// the given type.
    Array(Arc<TypeNode>),
    /// The raw element buffer of an `#ArrayStorage` object: a flexible array member of the element
    /// type, like `Array` but with no length. It is reference-count-inert — the owning `Array`
    /// value's traverser drives element lifetime — so it is a no-op in retain / traverse and only
    /// contributes its element sizing to the object layout.
    ArrayStorageBuf(Arc<TypeNode>),
}

/// The opaque buffer holding a union's payload: an integer array sized and aligned to fit the
/// largest variant.
fn union_buf_type<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    variant_tys: &[Arc<TypeNode>],
) -> BasicTypeEnum<'c> {
    let mut max_size = 0;
    let mut max_align = 1;
    for variant_ty in variant_tys {
        let embedded_ty = gc.embedded_type_of(variant_ty);
        max_size = max_size.max(gc.sizeof(&embedded_ty));
        // The buffer needs the payloads' ABI alignment, not the preferred alignment: the
        // preferred alignment of a small or empty aggregate is 8, which would over-pad the union.
        max_align = max_align.max(gc.abi_alignment(&embedded_ty));
    }
    let max_align_int = match max_align {
        1 => gc.context.i8_type(),
        2 => gc.context.i16_type(),
        4 => gc.context.i32_type(),
        8 => gc.context.i64_type(),
        16 => gc.context.i128_type(),
        _ => panic!("Unsupported alignment: {}", max_align),
    };
    let num_of_ints = max_size.div_ceil(max_align);
    assert!(
        num_of_ints <= u32::MAX as u64,
        "A payload of {} bytes needs {} integers of {} bytes to cover it, more than an LLVM array type holds.",
        max_size,
        num_of_ints,
        max_align,
    );
    max_align_int.array_type(num_of_ints as u32).into()
}

// PROOF: P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
impl ObjectFieldType {
    /// The LLVM type this field occupies in the struct its object is laid out as.
    pub fn to_basic_type<'c, 'm>(&self, gc: &mut Generator<'c, 'm>) -> BasicTypeEnum<'c> {
        match self {
            ObjectFieldType::ControlBlock => control_block_type(gc).into(),
            ObjectFieldType::TraverseFunction => gc.context.ptr_type(AddressSpace::from(0)).into(),
            ObjectFieldType::LambdaFunction(_ty) => {
                gc.context.ptr_type(AddressSpace::from(0)).into()
            }
            ObjectFieldType::SubObject(ty, _is_punched) => gc.embedded_type_of(ty),
            ObjectFieldType::Ptr => gc.context.ptr_type(AddressSpace::from(0)).into(),
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
            ObjectFieldType::ArrayStorageBuf(ty) => gc.embedded_type_of(ty),
            ObjectFieldType::UnionTag => union_tag_type(gc.context).into(),
            ObjectFieldType::UnionBuf(field_tys) => union_buf_type(gc, field_tys),
        }
    }

    /// The debug-info type describing this field: the Fix type name and encoding a debugger displays
    /// it under. A union's payload buffer gets a synthetic member per variant, all at offset zero,
    /// so that every variant is readable from the one buffer.
    pub fn to_debug_type<'c, 'm>(&self, gc: &mut Generator<'c, 'm>) -> DIType<'c> {
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
                let basic_ty = self.to_basic_type(gc);
                let size_in_bits = gc.target_data.get_bit_size(&basic_ty);
                let align_in_bits = gc.target_data.get_abi_alignment(&basic_ty) * 8;

                let mut elements = vec![];
                for (i, ty) in tys.iter().enumerate() {
                    let variant_ty = ty.get_embedded_type(gc);
                    let variant_debug_ty = ty_to_debug_embedded_ty(ty.clone(), gc);
                    let size_in_bits = gc.target_data.get_bit_size(&variant_ty);
                    let align_in_bits = gc.target_data.get_abi_alignment(&variant_ty) * 8;
                    // Every variant starts at the beginning of the union buffer.
                    let offset_in_bits = 0;
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
                .create_basic_type("<union tag>", UNION_TAG_BITS as u64, DW_ATE_UNSIGNED, 0)
                .unwrap()
                .as_type(),
            // An array lays its elements out as an `ArrayStorageBuf`, which is the only array
            // field an object carries.
            ObjectFieldType::Array(_) => unreachable!(),
            ObjectFieldType::ArrayStorageBuf(elem_ty) => {
                // The storage buffer's debug type is an array of `DEBUG_ARRAY_ASSUMED_LEN`
                // elements. `#ArrayStorage`'s own declared size is stretched to cover them in
                // `ty_to_debug_struct_ty`, which builds the enclosing struct's debug layout.
                let element_ty = elem_ty.get_embedded_type(gc);
                let element_debug_ty = ty_to_debug_embedded_ty(elem_ty.clone(), gc);
                let element_size_in_bits = gc.target_data.get_bit_size(&element_ty);
                let element_align_in_bits = gc.target_data.get_abi_alignment(&element_ty) * 8;
                let elements_size_in_bits = DEBUG_ARRAY_ASSUMED_LEN * element_size_in_bits;
                gc.get_di_builder()
                    .create_array_type(
                        element_debug_ty,
                        elements_size_in_bits,
                        element_align_in_bits,
                        &[0..DEBUG_ARRAY_ASSUMED_LEN as i64],
                    )
                    .as_type()
            }
        }
    }

    /// Emit a loop that visits the indices `[0, size)` of `buffer`, calling `loop_body` at each
    /// index and `after_loop` once the loop ends. The builder is left positioned in the block
    /// after the loop.
    ///
    /// # Arguments
    /// * `size` — how many elements the walk covers, counted from `buffer`'s first element.
    /// * `loop_body` — receives the current index, `size` and `buffer`.
    /// * `after_loop` — receives `size` and `buffer`, and runs once, including when `size` is
    ///   zero.
    fn loop_over_array_buf<'c, 'm, F, G>(
        gc: &mut Generator<'c, 'm>,
        size: IntValue<'c>,
        buffer: PointerValue<'c>,
        loop_body: F,
        after_loop: G,
    ) where
        for<'c2, 'm2> F: Fn(
            &mut Generator<'c, 'm>,
            IntValue<'c>,     /* idx */
            IntValue<'c>,     /* size */
            PointerValue<'c>, /* buffer */
        ),
        for<'c2, 'm2> G: Fn(
            &mut Generator<'c, 'm>,
            IntValue<'c>,     /* size */
            PointerValue<'c>, /* buffer */
        ),
    {
        // Append blocks: loop_check, loop_body and after_loop.
        let current_func = gc.current_function();
        let loop_check_bb = gc
            .context
            .append_basic_block(current_func, "loop_release_array_elements");
        let loop_body_bb = gc.context.append_basic_block(current_func, "loop_body");
        let after_loop_bb = gc.context.append_basic_block(current_func, "after_loop");

        // Allocate and initialize loop counter.
        let counter_type = gc.context.i64_type();
        let counter_ptr = gc.build_alloca_at_entry(counter_type, "release_loop_counter");
        gc.builder()
            .build_store(counter_ptr, counter_type.const_zero())
            .unwrap();

        // Jump to loop_check bb.
        gc.builder()
            .build_unconditional_branch(loop_check_bb)
            .unwrap();

        // Implement loop_check bb.
        gc.builder().position_at_end(loop_check_bb);
        let counter_val = gc
            .builder()
            .build_load(counter_type, counter_ptr, "counter_val")
            .unwrap()
            .into_int_value();
        let is_end = gc
            .builder()
            .build_int_compare(IntPredicate::EQ, counter_val, size, "is_end")
            .unwrap();
        gc.builder()
            .build_conditional_branch(is_end, after_loop_bb, loop_body_bb)
            .unwrap();

        // Implement loop_body bb.
        gc.builder().position_at_end(loop_body_bb);

        // Generate code of loop body.
        let idx = gc
            .builder()
            .build_load(counter_type, counter_ptr, "idx")
            .unwrap()
            .into_int_value();
        loop_body(gc, idx, size, buffer);

        // Increment counter.
        let incremented_counter_val = gc
            .builder()
            .build_int_add(
                counter_val,
                counter_type.const_int(1, false),
                "incremented_counter_val",
            )
            .unwrap();
        gc.builder()
            .build_store(counter_ptr, incremented_counter_val)
            .unwrap();

        // Jump back to loop_check bb.
        gc.builder()
            .build_unconditional_branch(loop_check_bb)
            .unwrap();

        // Generate code after loop.
        gc.builder().position_at_end(after_loop_bb);
        after_loop(gc, size, buffer);
    }

    /// Locate the elements that follow a hole: elements `(hole, size)` live at
    /// `&buffer[hole + 1]`, and there are `size - hole - 1` of them.
    ///
    /// # Arguments
    /// * `hole` — the index of the slot whose element was moved out of the array
    ///   (`Std::PunchedArray`).
    ///
    /// # Returns
    /// The address of the first element after the hole, and how many elements follow it.
    fn array_buf_after_hole<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        elem_basic_ty: BasicTypeEnum<'c>,
        buffer: PointerValue<'c>,
        size: IntValue<'c>,
        hole: IntValue<'c>,
    ) -> (PointerValue<'c>, IntValue<'c>) {
        let one = gc.context.i64_type().const_int(1, false);
        let after_hole = gc.builder().build_int_add(hole, one, "after_hole").unwrap();
        let tail_buffer =
            build_gep_array_elem(gc, elem_basic_ty, buffer, after_hole, "buf_after_hole");
        let tail_count = gc
            .builder()
            .build_int_sub(size, after_hole, "count_after_hole")
            .unwrap();
        (tail_buffer, tail_count)
    }

    /// Perform `work_type`'s work — release, mark-global or mark-threaded — on each of `count`
    /// consecutive elements starting at `buffer`. An element type that is fully unboxed holds no
    /// reference, so nothing is emitted for one.
    ///
    /// # Arguments
    /// * `state` — what is known about the reference-counting state of the elements.
    // PROOF: P18c, P19, P20, P21, P22, P23, P24 (dev-docs/proof/rc_ir/borrow-cancel)
    fn traverse_array_range<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        buffer: PointerValue<'c>,
        count: IntValue<'c>,
        elem_ty: Arc<TypeNode>,
        work_type: TraverserWorkType,
        state: RcState,
    ) {
        // A fully unboxed element holds no reference, so an array of such elements has no element
        // the work reaches.
        if elem_ty.is_fully_unboxed(gc.type_env()) {
            return;
        }
        let value_ty = elem_ty.get_embedded_type(gc);

        // In loop body, release the element the buffer holds at `idx`.
        let loop_body = |gc: &mut Generator<'c, 'm>,
                         idx: IntValue<'c>,
                         _size: IntValue<'c>,
                         ptr_to_buffer: PointerValue<'c>| {
            let ptr =
                build_gep_array_elem(gc, value_ty, ptr_to_buffer, idx, "ptr_to_elem_of_array");
            let obj_val = gc
                .builder()
                .build_load(value_ty, ptr, "elem_of_array")
                .unwrap();
            // Perform the work on the element.
            let obj = Object::new(obj_val, elem_ty.clone(), gc);
            gc.build_traverser_work(obj, work_type, state);
        };

        /// Runs once the loop over the buffer ends. Each element's work happens in the loop
        /// body, so this stage has none of its own.
        fn after_loop<'c, 'm>(
            _gc: &mut Generator<'c, 'm>,
            _size: IntValue<'c>,
            _ptr_to_buffer: PointerValue<'c>,
        ) {
        }

        // Generate loop.
        Self::loop_over_array_buf(gc, count, buffer, loop_body, after_loop);
    }

    /// Perform `work_type`'s work on the elements in `[begin, end)` of an array's buffer.
    ///
    /// # Arguments
    /// * `begin`, `end` — element indices counted from `buffer`'s first element, a half-open
    ///   range.
    pub fn traverse_array_slice<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        buffer: PointerValue<'c>,
        begin: IntValue<'c>,
        end: IntValue<'c>,
        elem_ty: Arc<TypeNode>,
        work_type: TraverserWorkType,
        state: RcState,
    ) {
        let value_ty = elem_ty.get_embedded_type(gc);
        let slice_begin =
            build_gep_array_elem(gc, value_ty, buffer, begin, "array_buf_slice_begin");
        let count = gc
            .builder()
            .build_int_sub(end, begin, "array_slice_count")
            .unwrap();
        Self::traverse_array_range(gc, slice_begin, count, elem_ty, work_type, state);
    }

    /// Perform `work_type`'s work on every element of an array's buffer.
    ///
    /// # Arguments
    /// * `size` — the array's element count; the elements walked are `[0, size)`.
    /// * `hole` — `Some(idx)` names the slot whose element was moved out of the array
    ///   (`Std::PunchedArray`), which the storage therefore does not own, so it is skipped.
    // PROOF: D/A, P18c, P19, P20, P21, P22, P23, P24, P28 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn traverse_array_buf<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        size: IntValue<'c>,
        buffer: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        work_type: TraverserWorkType,
        hole: Option<IntValue<'c>>,
        state: RcState,
    ) {
        match hole {
            None => Self::traverse_array_range(gc, buffer, size, elem_ty, work_type, state),
            Some(hole) => {
                let value_ty = elem_ty.get_embedded_type(gc);
                Self::traverse_array_range(gc, buffer, hole, elem_ty.clone(), work_type, state);
                let (tail_buffer, tail_count) =
                    Self::array_buf_after_hole(gc, value_ty, buffer, size, hole);
                Self::traverse_array_range(gc, tail_buffer, tail_count, elem_ty, work_type, state);
            }
        }
    }

    /// Store `value` into every slot of `[0, size)` of `buffer`, giving each slot its own
    /// reference through one retain per slot, and consume the caller's own reference to `value`.
    /// The net change to `value`'s reference count is `size - 1`, correct at `size == 0`.
    ///
    /// # Arguments
    /// * `buffer` — allocated and uninitialized; every slot this writes is a first write.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn initialize_array_buf_by_value<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        size: IntValue<'c>,
        buffer: PointerValue<'c>,
        value: Object<'c>,
    ) {
        // Initialize elements
        {
            // In loop body, retain value and store it at idx.
            let loop_body = |gc: &mut Generator<'c, 'm>,
                             idx: IntValue<'c>,
                             _size: IntValue<'c>,
                             buf_ptr: PointerValue<'c>| {
                let value_ty = value.ty.get_embedded_type(gc);
                gc.retain(value.clone(), RcState::Unknown);
                let elem_ptr =
                    build_gep_array_elem(gc, value_ty, buf_ptr, idx, "ptr_to_elem_of_array");
                gc.builder().build_store(elem_ptr, value.value(gc)).unwrap();
            };

            // After loop, release value.
            let after_loop = |gc: &mut Generator<'c, 'm>,
                              _size: IntValue<'c>,
                              _ptr_to_buffer: PointerValue<'c>| {
                gc.release(value.clone(), RcState::Unknown);
            };

            // Generate loop.
            // NOTE: if you see error at here, try `cargo clean`.
            Self::loop_over_array_buf(gc, size, buffer, loop_body, after_loop);
        }
    }

    /// Store `value` into `[begin, begin + count)` of an array's buffer. Each slot is given its
    /// own reference through a single reference-count add of `count`, and the caller's own
    /// reference to `value` is then consumed. The net change to `value`'s reference count is
    /// `count - 1`, correct at `count == 0`.
    ///
    /// # Arguments
    /// * `begin` — the index the store starts at, counted from `buffer`'s first element. The
    ///   slots it covers are allocated and uninitialized, and each is a first write.
    // PROOF: P27, P28, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn append_value_into_array_buf<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        buffer: PointerValue<'c>,
        begin: IntValue<'c>,
        count: IntValue<'c>,
        value: Object<'c>,
        state: RcState,
    ) {
        // One reference per slot, in a single reference-count add.
        gc.build_retain(value.clone(), count, state);

        let value_ty = value.ty.get_embedded_type(gc);
        let dst = build_gep_array_elem(gc, value_ty, buffer, begin, "array_append_begin");
        let elem_val = value.value(gc);
        let loop_body = |gc: &mut Generator<'c, 'm>,
                         idx: IntValue<'c>,
                         _count: IntValue<'c>,
                         buf_ptr: PointerValue<'c>| {
            let slot = build_gep_array_elem(gc, value_ty, buf_ptr, idx, "array_append_slot");
            gc.builder().build_store(slot, elem_val).unwrap();
        };
        let after_loop =
            |_gc: &mut Generator<'c, 'm>, _count: IntValue<'c>, _buf: PointerValue<'c>| {};
        Self::loop_over_array_buf(gc, count, dst, loop_body, after_loop);

        // Hand off the op's own reference.
        gc.release(value, state);
    }

    /// Abort the program if `idx` falls outside `[0, len)`, the indices an array of `len` elements
    /// has. The comparison is unsigned, so a negative index is out of range as well.
    pub fn panic_if_out_of_range<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        len: IntValue<'c>,
        idx: IntValue<'c>,
    ) {
        let is_out_of_range = gc
            .builder()
            .build_int_compare(IntPredicate::UGE, idx, len, "is_out_of_range")
            .unwrap();
        build_abort_if(
            gc,
            is_out_of_range,
            RUNTIME_INDEX_OUT_OF_RANGE,
            &[idx.into(), len.into()],
            "out_of_range",
        );
    }

    /// Abort the program if the array size or capacity `size` is negative.
    pub fn panic_if_size_negative<'c, 'm>(gc: &mut Generator<'c, 'm>, size: IntValue<'c>) {
        let is_neg_size = gc
            .builder()
            .build_int_compare(
                IntPredicate::SLT,
                size,
                gc.context.i64_type().const_zero(),
                "is_neg_size",
            )
            .unwrap();
        build_abort_if(
            gc,
            is_neg_size,
            RUNTIME_NEGATIVE_ARRAY_SIZE,
            &[size.into()],
            "neg_size",
        );
    }

    /// Read the element at `idx` out of an array's buffer, borrowing the array's own reference to
    /// it: the caller has to retain the result to hold it past the array's lifetime.
    ///
    /// # Arguments
    /// * `len` - the number of elements the array holds, against which `idx` is bounds-checked.
    ///   `None` omits the check.
    pub fn read_from_array_buf_noretain<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        len: Option<IntValue<'c>>,
        buffer: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        idx: IntValue<'c>,
    ) -> Object<'c> {
        // Panic if out_of_range.
        if len.is_some() {
            Self::panic_if_out_of_range(gc, len.unwrap(), idx);
        }

        // Get element.
        let elem_basic_ty = elem_ty.get_embedded_type(gc);
        let elem_ptr = build_gep_array_elem(gc, elem_basic_ty, buffer, idx, "ptr_to_elem_of_array");

        // Get value
        let elem_val = gc
            .builder()
            .build_load(elem_basic_ty, elem_ptr, "elem")
            .unwrap();

        // Return value
        Object::new(elem_val, elem_ty, gc)
    }

    /// Read the element at `idx` out of an array's buffer and retain it, giving the caller a
    /// reference of its own.
    ///
    /// # Arguments
    /// * `len` - the number of elements the array holds, against which `idx` is bounds-checked.
    ///   `None` omits the check.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn read_from_array_buf<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        len: Option<IntValue<'c>>,
        buffer: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        idx: IntValue<'c>,
        state: RcState,
    ) -> Object<'c> {
        let elem = ObjectFieldType::read_from_array_buf_noretain(gc, len, buffer, elem_ty, idx);
        gc.retain(elem.clone(), state);
        elem
    }

    /// Store `value` into the slot at `idx` of an array's buffer, handing the caller's reference to
    /// it over to the array.
    ///
    /// # Arguments
    /// * `len` - the number of elements the array holds, against which `idx` is bounds-checked.
    ///   `None` omits the check.
    /// * `release_old_value` - `true` when the slot holds a live element, whose reference is
    ///   released before the store; `false` when the slot is uninitialized.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn write_to_array_buf<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        len: Option<IntValue<'c>>,
        buffer: PointerValue<'c>,
        idx: IntValue<'c>,
        value: Object<'c>,
        release_old_value: bool,
        state: RcState,
    ) {
        let elem_ty = value.ty.clone();

        // Panic if out_of_range.
        if len.is_some() {
            Self::panic_if_out_of_range(gc, len.unwrap(), idx);
        }

        // Get ptr to the place at idx.
        let elem_basic_ty = value.ty.get_embedded_type(gc);
        let elem_ptr = build_gep_array_elem(gc, elem_basic_ty, buffer, idx, "ptr_to_elem_of_array");

        // Release element that is already at the place (if required).
        if release_old_value {
            let elem_val = gc
                .builder()
                .build_load(elem_basic_ty, elem_ptr, "elem")
                .unwrap();
            let elem_obj = Object::new(elem_val, elem_ty, gc);
            gc.release(elem_obj, state);
        }

        // Insert the given value to the place.
        gc.builder().build_store(elem_ptr, value.value(gc)).unwrap();
    }

    /// Copy `count` consecutive elements from `src_buffer` into `dst_buffer`, starting at index 0
    /// of each, and retain every one so that both buffers own it.
    ///
    /// # Arguments
    /// * `dst_buffer` — allocated and uninitialized; every slot this writes is a first write.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    fn clone_array_range<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        src_buffer: PointerValue<'c>,
        dst_buffer: PointerValue<'c>,
        count: IntValue<'c>,
        elem_ty: Arc<TypeNode>,
        state: RcState,
    ) {
        let elem_basic_ty = elem_ty.get_embedded_type(gc);
        // In loop body, retain value and store it at idx. A fully unboxed element holds no
        // reference, so the copy of one is the store alone.
        let loop_body = |gc: &mut Generator<'c, 'm>,
                         idx: IntValue<'c>,
                         _len: IntValue<'c>,
                         _ptr_to_buffer: PointerValue<'c>| {
            let src_ptr =
                build_gep_array_elem(gc, elem_basic_ty, src_buffer, idx, "ptr_to_src_elem");
            let dst_ptr =
                build_gep_array_elem(gc, elem_basic_ty, dst_buffer, idx, "ptr_to_dst_elem");
            let src_elem = gc
                .builder()
                .build_load(elem_basic_ty, src_ptr, "src_elem")
                .unwrap();
            gc.builder().build_store(dst_ptr, src_elem).unwrap();
            if !elem_ty.is_fully_unboxed(gc.type_env()) {
                let src_obj = Object::new(src_elem, elem_ty.clone(), gc);
                gc.retain(src_obj, state);
            }
        };

        // After loop, do nothing.
        let after_loop =
            |_gc: &mut Generator<'c, 'm>, _len: IntValue<'c>, _ptr_to_buffer: PointerValue<'c>| {};

        Self::loop_over_array_buf(gc, count, src_buffer, loop_body, after_loop);
    }

    /// Copy the `len` elements of an array's buffer from `src_buffer` into `dst_buffer`,
    /// retaining each one so that both buffers own it.
    ///
    /// # Arguments
    /// * `dst_buffer` — allocated and uninitialized; every slot this writes is a first write.
    /// * `hole` — `Some(idx)` names the slot whose element was moved out of the array
    ///   (`Std::PunchedArray`), which the source therefore does not own; the copy skips it and
    ///   leaves `dst_buffer[idx]` uninitialized.
    // PROOF: P27, P28, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn clone_array_buf<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        len: IntValue<'c>,
        src_buffer: PointerValue<'c>,
        dst_buffer: PointerValue<'c>,
        elem_ty: Arc<TypeNode>,
        hole: Option<IntValue<'c>>,
        state: RcState,
    ) {
        match hole {
            None => Self::clone_array_range(gc, src_buffer, dst_buffer, len, elem_ty, state),
            Some(hole) => {
                let elem_basic_ty = elem_ty.get_embedded_type(gc);
                Self::clone_array_range(gc, src_buffer, dst_buffer, hole, elem_ty.clone(), state);
                let (tail_src, tail_count) =
                    Self::array_buf_after_hole(gc, elem_basic_ty, src_buffer, len, hole);
                let (tail_dst, _) =
                    Self::array_buf_after_hole(gc, elem_basic_ty, dst_buffer, len, hole);
                Self::clone_array_range(gc, tail_src, tail_dst, tail_count, elem_ty, state);
            }
        }
    }

    /// Copy the value-carrying fields of the struct object `src` into `dst`, retaining each boxed
    /// field so that both objects own it, and return `dst`. A punched field holds no value, so it
    /// is skipped. `src` is borrowed: it is left as it was.
    ///
    /// # Arguments
    /// * `dst` — an allocated but uninitialized struct object of the same type; every field this
    ///   writes is a first write.
    /// * `state` — the reference-counting state of the fields, which is what `src` reaches. An
    ///   operation whose annotation covers its clone path proves it local and passes it here.
    // PROOF: P27, P28, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn clone_struct<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        src: &Object<'c>,
        mut dst: Object<'c>,
        state: RcState,
    ) -> Object<'c> {
        for (i, _) in src.ty.unpunched_field_types(gc.type_env()) {
            // Retain the field.
            let field = ObjectFieldType::move_out_struct_field(gc, src, i as u32);
            gc.retain(field.clone(), state);

            // Clone the field.
            dst = ObjectFieldType::move_into_struct_field(gc, dst, i as u32, &field);
        }
        dst
    }

    /// Copy the tag and the payload of the union object `src` into `dst`, retaining the payload so
    /// that both objects own it, and return `dst`. `src` is borrowed: it is left as it was.
    ///
    /// # Arguments
    /// * `dst` — an allocated but uninitialized union object of the same type; the tag and payload
    ///   this writes are first writes.
    /// * `state` — the reference-counting state of the payload, which is what `src` reaches. An
    ///   operation whose annotation covers its clone path proves it local and passes it here.
    // PROOF: P27, P28, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn clone_union<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        src: &Object<'c>,
        dst: Object<'c>,
        state: RcState,
    ) -> Object<'c> {
        // Clone the tag.
        let tag = ObjectFieldType::get_union_tag(gc, &src);
        let dst = ObjectFieldType::set_union_tag(gc, dst, tag);

        // Clone the payload buffer.
        let union_buf_idx = ObjectFieldType::get_union_buf_idx(gc, src);
        let buf = src.extract_field(gc, union_buf_idx);
        let dst = dst.insert_field(gc, union_buf_idx, buf);

        // Retain the value.
        let one = gc.context.i64_type().const_int(1, false);
        ObjectFieldType::retain_union(gc, dst.clone(), one, state);

        dst
    }

    /// Emit the reference-counting work for the payload a union's buffer holds: a retain, a
    /// release, a mark global or a mark threaded, on the variant the tag names.
    // PROOF: D/A, P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    fn retain_release_mark_union<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: Object<'c>,
        work_type: Option<TraverserWorkType>, // `None` retains; `Some` gives the traverser work.
        amount: IntValue<'c>, // How many times to retain (retain path only); a constant 1 for release/mark.
        state: RcState,       // What is known about the payload's reference-counting state.
    ) {
        let variant_types = &union.ty.field_types(gc.type_env());
        // Retain or release field.
        let current_func = gc.current_function();
        let end_bb = gc.context.append_basic_block(current_func, "end");
        let mut last_mismatch_bb: Option<BasicBlock> = None;
        let actual_tag = ObjectFieldType::get_union_tag(gc, &union);
        for (i, variant_ty) in variant_types.iter().enumerate() {
            // Compare tag and jump.
            let match_bb = gc
                .context
                .append_basic_block(current_func, &format!("match_tag{}", i));
            let mismatch_bb = gc
                .context
                .append_basic_block(current_func, &format!("mismatch_tag{}", i));
            let expected_tag = union_tag_value(gc.context, i);
            let is_match = gc
                .builder()
                .build_int_compare(
                    IntPredicate::EQ,
                    actual_tag,
                    expected_tag,
                    &format!("is_tag_{}", i),
                )
                .unwrap();
            gc.builder()
                .build_conditional_branch(is_match, match_bb, mismatch_bb)
                .unwrap();

            // Implement the case tag is match.
            gc.builder().position_at_end(match_bb);
            let subobj =
                ObjectFieldType::get_union_value_noretain_norelease(gc, union.clone(), variant_ty);
            if work_type.is_none() {
                if is_const_one(amount) {
                    gc.retain(subobj, state);
                } else {
                    gc.build_retain(subobj, amount, state);
                }
            } else {
                gc.build_traverser_work(subobj, work_type.unwrap(), state);
            }
            gc.builder().build_unconditional_branch(end_bb).unwrap();

            // Implement the case tag mismatch.
            gc.builder().position_at_end(mismatch_bb);
            last_mismatch_bb = Some(mismatch_bb);
        }

        // Implement last mismatch bb.
        let last_mismatch_bb = last_mismatch_bb.unwrap();
        gc.builder().position_at_end(last_mismatch_bb);
        gc.builder().build_unreachable().unwrap();

        gc.builder().position_at_end(end_bb);
    }

    /// Increment the reference count of the payload a union buffer holds, `amount` times.
    // PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn retain_union<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: Object<'c>,
        amount: IntValue<'c>,
        state: RcState,
    ) {
        ObjectFieldType::retain_release_mark_union(gc, union, None, amount, state);
    }

    /// The index, among the fields of a union's struct type, of the tag telling which variant the
    /// value holds.
    fn get_union_tag_idx<'c, 'm>(gc: &mut Generator<'c, 'm>, union: &Object<'c>) -> u32 {
        struct_field_idx(union.is_unbox(gc.type_env())) + UNION_TAG_IDX
    }

    /// The tag of a union value: the index, among the union's variants, of the variant it holds.
    pub fn get_union_tag<'c, 'm>(gc: &mut Generator<'c, 'm>, union: &Object<'c>) -> IntValue<'c> {
        let union_tag_idx = ObjectFieldType::get_union_tag_idx(gc, union);
        union.extract_field(gc, union_tag_idx).into_int_value()
    }

    /// The union with its tag set to `tag`, the index of the variant it is to hold. The payload
    /// buffer keeps what it held, so the caller writes the variant's value into it.
    pub fn set_union_tag<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: Object<'c>,
        tag: IntValue<'c>,
    ) -> Object<'c> {
        let union_tag_idx = ObjectFieldType::get_union_tag_idx(gc, &union);
        union.insert_field(gc, union_tag_idx, tag)
    }

    /// The index, among the fields of a union's struct type, of the buffer holding the value of the
    /// variant it carries.
    pub fn get_union_buf_idx<'c, 'm>(gc: &mut Generator<'c, 'm>, union: &Object<'c>) -> u32 {
        struct_field_idx(union.is_unbox(gc.type_env())) + UNION_DATA_IDX
    }

    /// The contents of a union's payload buffer, still typed as the buffer, which is wide enough for
    /// whichever variant the union carries. Reading the variant's value out of it takes a bit cast.
    pub fn get_union_buf<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: &Object<'c>,
    ) -> BasicValueEnum<'c> {
        let union_buf_idx = ObjectFieldType::get_union_buf_idx(gc, union);
        union.extract_field(gc, union_buf_idx)
    }

    /// The value a union carries, read as `variant_ty`, owned by the caller: the value is retained
    /// and the union released, which cancel each other out for an unboxed union.
    // PROOF: D/A, P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn get_union_value<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: Object<'c>,
        variant_ty: &Arc<TypeNode>,
        state: RcState,
    ) -> Object<'c> {
        let value =
            ObjectFieldType::get_union_value_noretain_norelease(gc, union.clone(), variant_ty);
        if union.is_box(gc.type_env()) {
            // If the union is boxed, retain the value and release the union.
            gc.retain(value.clone(), state);
            gc.release(union, state);
        } else {
            // If the union is unbox, retaining and releasing cancel each other out, so does nothing.
        }
        value
    }

    /// The value a union carries, read as `variant_ty` and borrowed: the reference count of neither
    /// the value nor the union moves, so the value lives only as long as the union does.
    // PROOF: P7a, P7d, P7e (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn get_union_value_noretain_norelease<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: Object<'c>,
        variant_ty: &Arc<TypeNode>,
    ) -> Object<'c> {
        let buf = ObjectFieldType::get_union_buf(gc, &union);
        let value: BasicValueEnum<'_> =
            ObjectFieldType::get_value_from_union_buf(gc, buf, variant_ty);
        Object::new(value, variant_ty.clone(), gc)
    }

    /// The contents of a union's payload buffer read as `variant_ty`, by a bit cast: the buffer is
    /// at least as wide as the variant, and the variant starts at its beginning.
    // PROOF: P7a, P7d, P7e (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn get_value_from_union_buf<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        buf: BasicValueEnum<'c>,
        variant_ty: &Arc<TypeNode>,
    ) -> BasicValueEnum<'c> {
        let embedded_ty = variant_ty.get_embedded_type(gc);
        gc.bit_cast(buf, embedded_ty)
    }

    /// The union with `value` bit cast into its payload buffer. The tag is left to `set_union_tag`,
    /// which names the variant the payload is now to be read as.
    pub fn set_union_value<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: Object<'c>,
        value: Object<'c>,
    ) -> Object<'c> {
        let union_buf_idx = ObjectFieldType::get_union_buf_idx(gc, &union);
        let union_struct_ty = union.ty.get_struct_type(gc);
        let union_data_ty = union_struct_ty
            .get_field_type_at_index(union_buf_idx)
            .unwrap();
        let value_val = value.value(gc);
        let value = gc.bit_cast(value_val, union_data_ty);
        union.insert_field(gc, union_buf_idx, value)
    }

    /// Emit a check that the union carries the variant of `expected_tag`, aborting the program with
    /// a message where it carries another. Code after the call continues on the matching path.
    pub fn panic_if_union_tag_mismatch<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        union: Object<'c>,
        expected_tag: IntValue<'c>,
    ) {
        // Get tag value.
        let actual_tag = ObjectFieldType::get_union_tag(gc, &union);

        // If tag mismatch, panic.
        let is_tag_mismatch = gc
            .builder()
            .build_int_compare(
                IntPredicate::NE,
                expected_tag,
                actual_tag,
                "is_tag_mismatch",
            )
            .unwrap();
        let current_func = gc.current_function();
        let mismatch_bb = gc.context.append_basic_block(current_func, "mismatch_bb");
        let match_bb = gc.context.append_basic_block(current_func, "match_bb");
        gc.builder()
            .build_conditional_branch(is_tag_mismatch, mismatch_bb, match_bb)
            .unwrap();
        gc.builder().position_at_end(mismatch_bb);
        gc.panic("Union variant mismatch");
        gc.builder().build_unconditional_branch(match_bb).unwrap();
        gc.builder().position_at_end(match_bb);
    }

    /// The field of a struct at `field_idx`, taken at the struct's own reference to it: nothing is
    /// retained, so the caller either reads it while the struct is alive or takes the reference over
    /// by dropping the struct without releasing that field.
    // PROOF: D/A, P26, P27, P28, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn move_out_struct_field<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        struct_obj: &Object<'c>,
        field_idx: u32,
    ) -> Object<'c> {
        let field_offset = struct_field_idx(struct_obj.ty.is_unbox(gc.type_env()));
        let field_ty = struct_obj.ty.field_types(gc.type_env())[field_idx as usize].clone();
        struct_obj.extract_field_object(gc, field_idx + field_offset, field_ty)
    }

    /// The struct with `field` stored at `field_idx`, taking over the caller's reference to `field`.
    /// The value the field held before stays live and is the caller's to account for.
    // PROOF: D/A, P28 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn move_into_struct_field<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        struct_obj: Object<'c>,
        field_idx: u32,
        field: &Object<'c>,
    ) -> Object<'c> {
        let field_offset = struct_field_idx(struct_obj.ty.is_unbox(gc.type_env()));
        struct_obj.insert_field_object(gc, field_offset + field_idx, field)
    }

    /// Take the fields of `struct_obj` listed in `field_indices` out as owned objects, consuming
    /// the struct: each returned field owns its reference and so outlives the struct it came from,
    /// and the fields left behind are dropped.
    // PROOF: D/A, P7a, P7d, P7e, P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
    pub fn get_struct_fields<'c, 'm>(
        gc: &mut Generator<'c, 'm>,
        struct_obj: &Object<'c>,
        field_indices: &[u32],
        state: RcState,
    ) -> Vec<Object<'c>> {
        // Collect unretained (but cloned) fields.
        // We need clone here since lifetime of returned fields may be longer than that of struct object.
        let mut fields = vec![];
        for field_idx in field_indices {
            // Move the field out as an object; it carries its own parts, so it outlives the struct.
            let field = ObjectFieldType::move_out_struct_field(gc, struct_obj, *field_idx);
            fields.push(field);
        }

        if struct_obj.is_box(gc.type_env()) {
            // If struct is boxed, simply retain fields and release the struct.
            for field in &fields {
                gc.retain(field.clone(), state);
            }
            gc.release(struct_obj.clone(), state);
        } else {
            // The struct is unboxed, so the fields taken out are released by whoever receives them.
            // Releasing the fields left behind here accounts for the struct itself.
            for (field_idx, _) in struct_obj.ty.unpunched_field_types(gc.type_env()) {
                let field_idx = field_idx as u32;
                if !field_indices.iter().any(|i| *i == field_idx) {
                    let field = ObjectFieldType::move_out_struct_field(gc, struct_obj, field_idx);
                    gc.release(field, state);
                }
            }
        }

        fields
    }
}

/// How an object that ends in an element buffer is laid out around that buffer.
struct ElementBufferLayout {
    /// The bytes of the fields laid out ahead of the buffer.
    header_size: u64,
    /// The bytes one element takes in the buffer, which is the stride every read and write of it
    /// uses.
    elem_stride: u64,
}

/// The layout the code generator gives a Fix type: the fields it is made of, and whether a value of
/// it is held in place or behind a pointer.
#[derive(Eq, PartialEq, Clone)]
pub struct ObjectType {
    /// The fields in the order they are laid out. A boxed type leads with its `ControlBlock`.
    pub field_types: Vec<ObjectFieldType>,
    /// Whether a value of this type is held in place. A boxed value is a pointer to a heap block
    /// whose contents this layout describes.
    pub is_unbox: bool,
    /// The Fix type this is the layout of.
    pub ty: Arc<TypeNode>,
}

impl ObjectType {
    /// The LLVM struct type this object is laid out as.
    ///
    /// The layout of an unboxed field is held in place, so this descends into it. A type reaching
    /// itself that way has no layout and no end to this descent; `Program::validate_layouts` rejects
    /// such a type before code generation begins.
    pub fn to_struct_type<'c, 'm>(&self, gc: &mut Generator<'c, 'm>) -> StructType<'c> {
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
                    // - in to_debug_type function.
                    fields.push(gc.embedded_type_of(ty));
                }
                _ => {}
            }
        }
        gc.context.struct_type(&fields, false)
    }

    /// The bytes laid out ahead of the element buffer, and the bytes one element takes in it, for an
    /// object type that ends in such a buffer (`Array`, `#ArrayStorage`).
    fn element_buffer_layout<'c, 'm>(&self, gc: &mut Generator<'c, 'm>) -> ElementBufferLayout {
        // The element buffer is the last field, of `Array` (with a preceding capacity slot) or of
        // `#ArrayStorage` (right after the control block).
        let elem_ty = match self.field_types.last().unwrap() {
            ObjectFieldType::Array(ty) => ty.clone(),
            ObjectFieldType::ArrayStorageBuf(ty) => ty.clone(),
            _ => panic!(
                "`{}` was given an array capacity, but its layout ends in no element buffer",
                self.ty.to_string()
            ),
        };
        let struct_ty = self.to_struct_type(gc);
        let buf_field_idx = struct_ty.count_fields() - 1;
        ElementBufferLayout {
            header_size: gc
                .target_data
                .offset_of_element(&struct_ty, buf_field_idx)
                .unwrap(),
            elem_stride: elem_stride(gc, &elem_ty),
        }
    }

    /// The size of this object in bytes: a constant, except for an object that ends in an element
    /// buffer, whose size takes a capacity the program computes at run time.
    ///
    /// The capacity is taken as one whose byte count fits in the address space;
    /// `build_capacity_check` is what establishes that.
    ///
    /// # Arguments
    /// * `array_capacity` - the number of elements the trailing element buffer is to hold, for an
    ///   object type that ends in one (`Array`, `#ArrayStorage`). For every other object type it is
    ///   `None` and the size is that of the struct alone.
    pub fn size_of<'c, 'm>(
        &self,
        gc: &mut Generator<'c, 'm>,
        array_capacity: Option<IntValue<'c>>,
    ) -> IntValue<'c> {
        if let Some(array_capacity) = array_capacity {
            // The size is the header -- the fields laid out ahead of the element buffer -- plus the
            // bytes the elements take.
            let layout = self.element_buffer_layout(gc);
            let ptr_int_ty = gc.context.ptr_sized_int_type(&gc.target_data, None);
            let cap = gc
                .builder()
                .build_int_cast(array_capacity, ptr_int_ty, "cap_as_ptr_int_ty")
                .unwrap();
            let elems_size = gc
                .builder()
                .build_int_mul(
                    ptr_int_ty.const_int(layout.elem_stride, false),
                    cap,
                    "elems_size",
                )
                .unwrap();
            return gc
                .builder()
                .build_int_add(
                    ptr_int_ty.const_int(layout.header_size, false),
                    elems_size,
                    "size_with_elems",
                )
                .unwrap();
        } else {
            self.to_struct_type(gc).size_of().unwrap()
        }
    }

    /// The type this object takes where it is embedded in another value: a struct for an unboxed
    /// type, a pointer for a boxed one.
    pub fn to_embedded_type<'c, 'm>(&self, gc: &mut Generator<'c, 'm>) -> BasicTypeEnum<'c> {
        if self.is_unbox {
            let struct_ty = self.to_struct_type(gc);
            struct_ty.into()
        } else {
            gc.context.ptr_type(AddressSpace::from(0)).into()
        }
    }
}

/// The integer type of the control block field holding an object's reference count. Its width is
/// what bounds the number of references to one object a program can hold, and `refcnt_di_type`
/// states that width a second time.
pub fn refcnt_type<'ctx>(context: &'ctx Context) -> IntType<'ctx> {
    context.i32_type()
}

/// The debug info type of an object's reference count, which presents it to a debugger session as an
/// unsigned integer. Its width is the width of `refcnt_type`, stated a second time.
pub fn refcnt_di_type<'ctx>(builder: &DebugInfoBuilder<'ctx>) -> DIType<'ctx> {
    builder
        .create_basic_type("<refcnt>", 32, DW_ATE_UNSIGNED, 0)
        .unwrap()
        .as_type()
}

/// The integer type of the control block field holding which reference-counting scheme an object is
/// under. Its values are the `REFCNT_STATE_*` constants.
pub fn refcnt_state_type<'c>(context: &'c Context) -> IntType<'c> {
    context.i8_type()
}

/// Type of the control block field holding how far the object sits above the base of its
/// allocation, in bytes. A byte holds every distance an object is placed by, which
/// `ARRAY_BUF_ALIGNMENT` bounds.
pub fn alloc_offset_type<'c>(context: &'c Context) -> IntType<'c> {
    assert!(ARRAY_BUF_ALIGNMENT <= u8::MAX as u64 + 1);
    context.i8_type()
}

/// The function type of a traverser, which walks an object's reference-counted leaves.
///
/// # Arguments
/// * `is_dynamic` - whether the traverser takes the work to do as a second argument, of
///   `traverser_work_type`, instead of having it fixed at the point the traverser is generated.
pub fn traverser_type<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    ty: &Arc<TypeNode>,
    is_dynamic: bool,
) -> FunctionType<'c> {
    // The object is passed as its parts, mirroring `lambda_function_type`: a boxed object is a
    // single pointer, an unbox struct is its fields spread out. This keeps the "materialize the
    // aggregate only at memory / foreign-ABI boundaries" invariant intact across the release / mark
    // path.
    let embedded = ty.get_embedded_type(gc);
    let mut arg_tys: Vec<BasicMetadataTypeEnum<'c>> = gc
        .type_parts(embedded)
        .into_iter()
        .map(|t| t.into())
        .collect();
    if is_dynamic {
        // Add argument for work type.
        arg_tys.push(gc.context.i8_type().into());
    }
    gc.context.void_type().fn_type(&arg_tys, false)
}

/// The integer type of the work argument a dynamic traverser takes, whose values are the
/// `TRAVERSER_WORK_*` constants.
pub fn traverser_work_type<'c>(context: &'c Context) -> IntType<'c> {
    context.i8_type()
}

/// The LLVM struct type of the control block that heads every boxed object, holding the reference
/// count, the reference-counting state, and the distance the object sits above the base of its
/// allocation.
pub fn control_block_type<'c, 'm>(gc: &Generator<'c, 'm>) -> StructType<'c> {
    let mut fields = vec![];
    assert_eq!(fields.len(), CTRL_BLK_REFCNT_IDX as usize);
    fields.push(refcnt_type(gc.context).into());
    assert_eq!(fields.len(), CTRL_BLK_REFCNT_STATE_IDX as usize);
    fields.push(refcnt_state_type(gc.context).into());
    assert_eq!(fields.len(), CTRL_BLK_ALLOC_OFFSET_IDX as usize);
    fields.push(alloc_offset_type(gc.context).into());
    gc.context.struct_type(&fields, false)
}

/// The debug info type describing the control block that heads every boxed object. It presents the
/// reference counter alone, the one field a debugger session has use for.
pub fn control_block_di_type<'c, 'm>(gc: &mut Generator<'c, 'm>) -> DIType<'c> {
    let struct_type = control_block_type(gc);

    let refcnt_ty = refcnt_type(gc.context);
    let refcnt_size_in_bits = gc.target_data.get_bit_size(&refcnt_ty);
    let refcnt_align_in_bits = gc.target_data.get_abi_alignment(&refcnt_ty) * 8;
    let refcnt_offset_in_bits = gc
        .target_data
        .offset_of_element(&struct_type, CTRL_BLK_REFCNT_IDX)
        .unwrap()
        * 8;
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
    let elements = vec![refcnt_member];

    let name = "<control block>";
    let size_in_bits = gc.target_data.get_bit_size(&struct_type);
    let align_in_bits = gc.target_data.get_abi_alignment(&struct_type) * 8;
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

/// The debug info type of a pointer of the target's width, presented to a debugger under `name`.
pub fn ptr_di_type<'c, 'm>(name: &str, gc: &mut Generator<'c, 'm>) -> DIType<'c> {
    let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
    let size_in_bits = gc.target_data.get_bit_size(&ptr_ty);
    gc.get_di_builder()
        .create_basic_type(name, size_in_bits, DW_ATE_ADDRESS, 0)
        .unwrap()
        .as_type()
}

/// The type of a union's tag, an index into the union's variants.
fn union_tag_type<'c>(context: &'c Context) -> IntType<'c> {
    context.custom_width_int_type(UNION_TAG_BITS)
}

/// The tag of the variant a union declares at index `variant_idx`.
pub fn union_tag_value<'c>(context: &'c Context, variant_idx: usize) -> IntValue<'c> {
    assert!(
        variant_idx < MAX_UNION_VARIANTS,
        "A union declares at most {} variants, so index {} names no variant.",
        MAX_UNION_VARIANTS,
        variant_idx
    );
    union_tag_type(context).const_int(variant_idx as u64, false)
}

/// The parts a lambda of type `ty` returns, in `type_parts` order: a boxed result is the single
/// heap pointer, an unboxed one its parts. These are exactly the parts of the `Object` the lambda's
/// body returns, so a call site and a `return` agree on them.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn lambda_return_part_types<'c, 'm>(
    ty: &Arc<TypeNode>,
    gc: &mut Generator<'c, 'm>,
) -> Vec<BasicTypeEnum<'c>> {
    let ret_ty = ty.get_lambda_dst();
    if ret_ty.is_box(gc.type_env()) {
        return vec![gc.context.ptr_type(AddressSpace::from(0)).into()];
    }
    let embedded = ret_ty.get_embedded_type(gc);
    gc.type_parts(embedded)
}

/// The LLVM signature every lambda of type `ty` is defined and called with: the arguments, then the
/// CAP pointer when the lambda is a closure, and the result either returned directly or written
/// through an out-pointer that precedes them all.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn lambda_function_type<'c, 'm>(
    ty: &Arc<TypeNode>,
    gc: &mut Generator<'c, 'm>,
) -> FunctionType<'c> {
    // Arguments. An unbox-struct argument is passed as its parts rather than as one aggregate, so a
    // loop-carried field stays visible to LLVM (see `type_parts`).
    let mut arg_tys: Vec<BasicMetadataTypeEnum> = ty
        .get_lambda_srcs()
        .iter()
        .flat_map(|src| {
            let embedded = src.get_embedded_type(gc);
            gc.type_parts(embedded)
        })
        .map(|t| t.into())
        .collect::<Vec<_>>();

    // The pointer to the CAP (a dynamic object which contains captured values), if the lambda is closure.
    if ty.is_closure() {
        arg_tys.push(gc.context.ptr_type(AddressSpace::from(0)).into());
    }

    // A result too wide for the target's return registers is written through an out-pointer, which
    // precedes every other parameter, and the function returns `void`. Carrying the pointer as an
    // ordinary parameter is what keeps the function's tail calls turning into jumps; see
    // `return_abi`.
    let ret_part_tys = lambda_return_part_types(ty, gc);
    if gc.returns_through_out_pointer(&ret_part_tys) {
        let mut param_tys: Vec<BasicMetadataTypeEnum> =
            vec![gc.context.ptr_type(AddressSpace::from(0)).into()];
        param_tys.extend(arg_tys);
        return gc.context.void_type().fn_type(&param_tys, false);
    }

    // Otherwise the result is returned as its parts, mirroring how the arguments are passed (see
    // `type_parts`): no part returns `void`, a single part is returned bare, and several parts are
    // returned as a flat struct `{ part, ... }`. A later pass that splits the return value at a
    // control-flow merge then yields one phi per part instead of an aggregate phi, keeping a
    // loop-carried field visible to LLVM.
    match ret_part_tys.as_slice() {
        [] => gc.context.void_type().fn_type(&arg_tys, false),
        [single] => single.fn_type(&arg_tys, false),
        many => gc.context.struct_type(many, false).fn_type(&arg_tys, false),
    }
}

/// The index at which a value's own fields begin in its layout: those of a struct, and the tag and
/// the payload buffer of a union. A boxed value leads with its control block, which pushes them
/// along by one.
pub fn struct_field_idx(is_unbox: bool) -> u32 {
    if is_unbox {
        0
    } else {
        BOXED_TYPE_DATA_IDX
    }
}

/// The fields a primitive type is laid out as, by the primitive's name: one scalar, except for
/// `IOState`, which carries nothing.
fn primitive_field_types(name: &FullName) -> &'static [ObjectFieldType] {
    // PROOF: P2a, P15, P16, P17, P18 (dev-docs/proof/rc_ir/borrow-cancel)
    static FIELDS_BY_NAME: OnceLock<Map<FullName, Vec<ObjectFieldType>>> = OnceLock::new();
    let fields_by_name = FIELDS_BY_NAME.get_or_init(|| {
        [
            (make_iostate_ty(), vec![]),
            (make_ptr_ty(), vec![ObjectFieldType::Ptr]),
            (make_i8_ty(), vec![ObjectFieldType::I8]),
            (make_u8_ty(), vec![ObjectFieldType::U8]),
            (make_i16_ty(), vec![ObjectFieldType::I16]),
            (make_u16_ty(), vec![ObjectFieldType::U16]),
            (make_i32_ty(), vec![ObjectFieldType::I32]),
            (make_u32_ty(), vec![ObjectFieldType::U32]),
            (make_i64_ty(), vec![ObjectFieldType::I64]),
            (make_u64_ty(), vec![ObjectFieldType::U64]),
            (make_f32_ty(), vec![ObjectFieldType::F32]),
            (make_f64_ty(), vec![ObjectFieldType::F64]),
        ]
        .into_iter()
        .map(|(ty, fields)| (ty.toplevel_tycon().unwrap().name.clone(), fields))
        .collect()
    });
    fields_by_name
        .get(name)
        .unwrap_or_else(|| panic!("`{}` has no primitive layout", name.to_string()))
}

/// The layout of `ty`, derived from what its top-level type constructor is: a closure, a function
/// pointer, a primitive, or a struct, union, array or dynamic object declared in `type_env`.
///
/// # Arguments
/// * `capture` - the types a `#DynamicObject` holds captured, which become its trailing fields.
///   It is empty for every other type, whose fields follow from the type alone.
// PROOF: P7a, P7d, P7e, P27, P28, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn ty_to_object_ty(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>,
    type_env: &TypeEnv,
) -> ObjectType {
    assert!(ty.is_ground());
    assert!(ty.is_dynamic() || capture.is_empty());
    let mut object_ty = ObjectType {
        field_types: vec![],
        is_unbox: true,
        ty: ty.clone(),
    };
    if ty.is_closure() {
        assert!(capture.is_empty());
        object_ty.is_unbox = true;
        object_ty
            .field_types
            .push(ObjectFieldType::LambdaFunction(ty.clone()));
        object_ty
            .field_types
            .push(ObjectFieldType::SubObject(make_dynamic_object_ty(), false));
    } else if ty.is_funptr() {
        assert!(capture.is_empty());
        object_ty.is_unbox = true;
        object_ty
            .field_types
            .push(ObjectFieldType::LambdaFunction(ty.clone()));
    } else {
        let tc = ty.toplevel_tycon().unwrap();
        // A value of an unwrapped newtype is a value of its one field, so no value is laid out at
        // one. Reaching here with one means a type escaped the rewrite, and its values would
        // silently be laid out as the struct they were to stop being: an unboxed struct of one
        // field has the layout of that field, so nothing further down would notice.
        assert!(
            !type_env.is_unwrapped_newtype(&tc),
            "A value of `{}` is laid out, though a value of this newtype has become a value of its one field.",
            ty.to_string()
        );
        let ti = type_env.tycons().get(&tc).unwrap();
        match ti.variant {
            TyConVariant::Primitive => {
                assert!(capture.is_empty());
                assert!(ti.is_unbox);
                object_ty.is_unbox = ti.is_unbox;
                object_ty
                    .field_types
                    .extend_from_slice(primitive_field_types(&tc.name));
            }
            TyConVariant::Array => {
                assert!(capture.is_empty());
                assert!(ti.is_unbox);
                object_ty.is_unbox = true;
                // A pointer to the `#ArrayStorage` holding the elements, then the size and capacity.
                let elem_ty = ty.field_types(type_env)[0].clone();
                object_ty.field_types.push(ObjectFieldType::SubObject(
                    make_array_storage_ty(elem_ty),
                    false,
                ));
                assert_eq!(object_ty.field_types.len(), ARRAY_SIZE_IDX as usize);
                object_ty.field_types.push(ObjectFieldType::I64); // size
                assert_eq!(object_ty.field_types.len(), ARRAY_CAP_IDX as usize);
                object_ty.field_types.push(ObjectFieldType::I64); // capacity
            }
            TyConVariant::Struct => {
                assert!(capture.is_empty());
                let is_unbox = ti.is_unbox;
                object_ty.is_unbox = is_unbox;
                if !is_unbox {
                    object_ty.field_types.push(ObjectFieldType::ControlBlock);
                }
                assert_eq!(
                    object_ty.field_types.len(),
                    struct_field_idx(is_unbox) as usize
                );
                let field_types = ty.field_types(type_env);
                for (field_idx, field_ty) in field_types.into_iter().enumerate() {
                    let punched = ti.fields[field_idx].is_punched;
                    object_ty
                        .field_types
                        .push(ObjectFieldType::SubObject(field_ty, punched));
                }
            }
            TyConVariant::Union => {
                assert!(capture.is_empty());
                let is_unbox = ti.is_unbox;
                object_ty.is_unbox = is_unbox;
                if !is_unbox {
                    object_ty.field_types.push(ObjectFieldType::ControlBlock);
                }
                object_ty.field_types.push(ObjectFieldType::UnionTag);
                object_ty
                    .field_types
                    .push(ObjectFieldType::UnionBuf(ty.field_types(type_env)));
            }
            TyConVariant::DynamicObject => {
                let is_unbox = ti.is_unbox;
                assert_eq!(is_unbox, false);
                object_ty.is_unbox = false;
                object_ty.field_types.push(ObjectFieldType::ControlBlock);
                assert_eq!(
                    object_ty.field_types.len(),
                    DYNAMIC_OBJ_TRAVARSER_IDX as usize
                );
                object_ty
                    .field_types
                    .push(ObjectFieldType::TraverseFunction);
                assert_eq!(object_ty.field_types.len(), DYNAMIC_OBJ_CAP_IDX as usize);
                for cap in capture {
                    object_ty
                        .field_types
                        .push(ObjectFieldType::SubObject(cap.clone(), false));
                }
            }
            TyConVariant::ArrayStorage => {
                assert!(capture.is_empty());
                assert!(!ti.is_unbox);
                object_ty.is_unbox = false;
                object_ty.field_types.push(ObjectFieldType::ControlBlock);
                assert_eq!(object_ty.field_types.len(), STORAGE_BUF_IDX as usize);
                object_ty.field_types.push(ObjectFieldType::ArrayStorageBuf(
                    ty.field_types(type_env)[0].clone(),
                ));
            }
            TyConVariant::Arrow => {
                unreachable!() // Covered by `if ty.is_closure()` above.
            }
            TyConVariant::Opaque => {
                unreachable!() // Opaque types are resolved before code generation.
            }
        }
    }
    object_ty
}

/// The `#ArrayStorage` object a flipped `Array` value points to, wrapped as an `Object` of its real
/// type so the reference-count helpers and buffer GEPs operate on it directly.
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn get_array_storage<'c, 'm>(gc: &mut Generator<'c, 'm>, array: &Object<'c>) -> Object<'c> {
    let elem_ty = array.ty.field_types(gc.type_env())[0].clone();
    let storage_ty = make_array_storage_ty(elem_ty);
    let storage_ptr = array.extract_field(gc, ARRAY_STORAGE_IDX);
    Object::new(storage_ptr, storage_ty, gc)
}

/// A pointer to the first element of a flipped `Array`'s element buffer.
pub fn get_array_storage_buf<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    array: &Object<'c>,
) -> PointerValue<'c> {
    get_array_storage(gc, array).gep_boxed(gc, STORAGE_BUF_IDX)
}

/// Whether an allocation checks the capacity it is given against the address space.
///
/// Every allocation asks for one, so each allocation site states which of the two it is: the check
/// is what keeps a wrapped-around byte count from reaching `malloc`, and a site that skipped it
/// silently would corrupt the heap.
#[derive(Clone, Copy)]
pub enum CapacityCheck {
    /// Nothing has checked this capacity, so this allocation checks it.
    Run,
    /// The capacity is already within the bound -- read off an array that holds it, or checked by
    /// the caller ahead of a branch whose arms both allocate -- so checking it here would only add
    /// a branch to the emitted code.
    Skip,
}

/// Allocate a fresh `#ArrayStorage` object for element type `elem_ty` with room for `cap` elements,
/// its control block initialized to a reference count of one and its buffer left uninitialized.
pub fn alloc_array_storage<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    elem_ty: Arc<TypeNode>,
    cap: IntValue<'c>,
    capacity_check: CapacityCheck,
) -> Object<'c> {
    build_capacity_check(gc, &elem_ty, cap, capacity_check);
    let storage_ty = make_array_storage_ty(elem_ty);
    create_obj(storage_ty, &vec![], Some(cap), gc, Some("array_storage"))
}

/// Emit a call to `malloc(sizeof)`.
///
/// We bypass inkwell's `build_malloc` / `build_array_malloc` because they declare `@malloc` with an
/// i32 size parameter and truncate the size, which breaks allocations >= 4 GiB. Instead we call our
/// own `@malloc(i64)` declaration registered in `runtime.rs`.
///
/// The result is used without a test for null, so an allocation the system cannot supply ends the
/// program with SIGSEGV. That failure is deterministic because `create_obj` initializes the control
/// block before it hands the object on: the first access to a block that was never allocated is
/// within the header, at a fixed low address, and no capacity or index the program computed reaches
/// it. Deferring that initialization would put a value the program chose into the faulting address
/// and turn this into a wild write.
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
fn build_malloc<'c, 'm>(
    gc: &Generator<'c, 'm>,
    sizeof: IntValue<'c>,
    name: &str,
) -> PointerValue<'c> {
    let malloc_fn = gc
        .module
        .get_function(RUNTIME_MALLOC)
        .expect("malloc is not declared");
    gc.builder()
        .build_call(malloc_fn, &[sizeof.into()], name)
        .unwrap()
        .try_as_basic_value()
        .left()
        .unwrap()
        .into_pointer_value()
}

/// The number of bytes one element of `elem_ty` occupies in an element buffer.
///
/// The buffer holds elements as they are embedded -- a pointer where the element type is boxed --
/// which is the stride every read and write of it uses.
fn elem_stride<'c, 'm>(gc: &mut Generator<'c, 'm>, elem_ty: &Arc<TypeNode>) -> u64 {
    let embedded_elem_ty = elem_ty.get_embedded_type(gc);
    gc.target_data.get_abi_size(&embedded_elem_ty)
}

/// The number of bytes `count` elements of `elem_ty` occupy in an element buffer, as a value of
/// `count`'s type named `name`.
pub fn build_elems_bytes<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    elem_ty: &Arc<TypeNode>,
    count: IntValue<'c>,
    name: &str,
) -> IntValue<'c> {
    let stride = elem_stride(gc, elem_ty);
    gc.builder()
        .build_int_mul(count.get_type().const_int(stride, false), count, name)
        .unwrap()
}

/// Where an `#ArrayStorage` object is placed in a block starting at `base`, as a distance from that
/// base, so that its element buffer starts on `ARRAY_BUF_ALIGNMENT`. The distance is below
/// `ARRAY_BUF_ALIGNMENT`, which is the slack a block needs to hold an object placed this way.
pub fn build_array_storage_shift<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    struct_type: StructType<'c>,
    base: PointerValue<'c>,
) -> IntValue<'c> {
    // The mask below is the distance to the next boundary only for a power-of-two boundary, which
    // is also what makes `ARRAY_BUF_ALIGNMENT - 1` the slack a block needs to hold an object placed
    // off its base.
    assert!(
        ARRAY_BUF_ALIGNMENT.is_power_of_two(),
        "ARRAY_BUF_ALIGNMENT must be a power of two, but is {}",
        ARRAY_BUF_ALIGNMENT
    );
    let i64_ty = gc.context.i64_type();
    let buf_offset = gc
        .target_data
        .offset_of_element(&struct_type, STORAGE_BUF_IDX)
        .unwrap();
    let buf_addr_at_base = gc
        .builder()
        .build_int_add(
            gc.builder()
                .build_ptr_to_int(base, i64_ty, "base_addr")
                .unwrap(),
            i64_ty.const_int(buf_offset, false),
            "buf_addr_at_base",
        )
        .unwrap();
    gc.builder()
        .build_and(
            gc.builder()
                .build_int_neg(buf_addr_at_base, "neg_buf_addr_at_base")
                .unwrap(),
            i64_ty.const_int(ARRAY_BUF_ALIGNMENT - 1, false),
            "storage_alloc_offset",
        )
        .unwrap()
}

/// Whether an `#ArrayStorage` object of `sizeof` bytes has its element buffer aligned, which it does
/// from `ARRAY_ALIGNED_ALLOC_THRESHOLD` bytes up.
pub fn build_storage_is_aligned<'c, 'm>(
    gc: &Generator<'c, 'm>,
    sizeof: IntValue<'c>,
) -> IntValue<'c> {
    gc.builder()
        .build_int_compare(
            IntPredicate::UGE,
            sizeof,
            gc.context
                .i64_type()
                .const_int(ARRAY_ALIGNED_ALLOC_THRESHOLD, false),
            "storage_is_aligned",
        )
        .unwrap()
}

/// Emit the check `capacity_check` asks for: the program aborts unless an `#ArrayStorage` holding
/// `cap` elements of `elem_ty` fits in the address space.
///
/// Left unchecked, a capacity whose byte count wraps around asks `malloc` for a small block, gets
/// one, and leaves an object claiming a capacity its block has no room for; the first write past the
/// block corrupts the heap. The elements must therefore leave room for the header in front of them
/// and for the padding that puts the element buffer on `ARRAY_BUF_ALIGNMENT`.
///
/// The bound is the widest capacity whose byte count cannot wrap, and a constant, so the whole check
/// is one unsigned comparison. A byte count within the bound that the system cannot supply is a
/// separate matter: `malloc` answers null and the program faults on the store that initializes the
/// object.
///
/// Every allocation given a capacity nothing has checked runs this, which is what makes it an
/// invariant that an array's capacity field is within the bound. `capacity_check` says whether this
/// allocation is one of those; `CapacityCheck::Skip` is for the capacities that invariant already
/// covers.
pub fn build_capacity_check<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    elem_ty: &Arc<TypeNode>,
    cap: IntValue<'c>,
    capacity_check: CapacityCheck,
) {
    if matches!(capacity_check, CapacityCheck::Skip) || !gc.config.runtime_check() {
        return;
    }
    let storage_ty = make_array_storage_ty(elem_ty.clone());
    let layout = storage_ty
        .get_object_type(&vec![], gc.type_env())
        .element_buffer_layout(gc);
    // A buffer of elements of no size is no bytes long, however many of them the capacity asks for.
    if layout.elem_stride == 0 {
        return;
    }
    // The bound below is the widest byte count of a 64-bit address space, so it bounds a capacity
    // of that width.
    assert_eq!(cap.get_type().get_bit_width(), 64);
    let max_cap = (u64::MAX - ARRAY_STORAGE_ALLOC_SLACK - layout.header_size) / layout.elem_stride;
    let is_capacity_overflow = gc
        .builder()
        .build_int_compare(
            IntPredicate::UGT,
            cap,
            cap.get_type().const_int(max_cap, false),
            "is_capacity_overflow",
        )
        .unwrap();
    build_abort_if(
        gc,
        is_capacity_overflow,
        RUNTIME_ARRAY_SIZE_OVERFLOW,
        &[cap.into()],
        "capacity_overflow",
    );
}

/// Emit a call to the runtime function `func_name` with `args` for the case that `cond` holds, and
/// leave the builder at the block reached when it does not.
///
/// The runtime function ends the program, so the call is followed by a branch to the continuation
/// only to close its basic block. `bb_name` names that pair of blocks in the emitted IR.
fn build_abort_if<'c, 'm>(
    gc: &Generator<'c, 'm>,
    cond: IntValue<'c>,
    func_name: &str,
    args: &[BasicMetadataValueEnum<'c>],
    bb_name: &str,
) {
    let current_func = gc.current_function();
    let abort_bb = gc
        .context
        .append_basic_block(current_func, &format!("{}_bb", bb_name));
    let continue_bb = gc
        .context
        .append_basic_block(current_func, &format!("{}_continue_bb", bb_name));
    gc.builder()
        .build_conditional_branch(cond, abort_bb, continue_bb)
        .unwrap();
    gc.builder().position_at_end(abort_bb);
    gc.call_runtime(func_name, args);
    gc.builder()
        .build_unconditional_branch(continue_bb)
        .unwrap();
    gc.builder().position_at_end(continue_bb);
}

/// The address of the slot at `idx` in an array's element buffer, whose elements occupy
/// `elem_basic_ty` as they are embedded.
///
/// The address is computed inside the buffer's allocation, and the emitted code says so. Saying so
/// lets LLVM fold the address arithmetic into an addressing mode, and bound the index where a
/// loop's bounds check reads it.
///
/// `idx` names a slot of the buffer, or the one past its last, and who owes that varies with the
/// caller. An index the compiler itself counts runs to the number of elements. An index a program
/// supplies is bounds-checked against the array's size on the way here, except through the
/// `_unsafe_` primitives and in a build whose runtime checks are off, where the obligation is the
/// Fix program's own -- the same obligation those two already carry for the read or the write that
/// follows.
pub fn build_gep_array_elem<'c, 'm>(
    gc: &Generator<'c, 'm>,
    elem_basic_ty: BasicTypeEnum<'c>,
    buffer: PointerValue<'c>,
    idx: IntValue<'c>,
    name: &str,
) -> PointerValue<'c> {
    unsafe {
        gc.builder()
            .build_in_bounds_gep(elem_basic_ty, buffer, &[idx], name)
    }
    .unwrap()
}

/// The address `offset` bytes from `ptr`, within the allocation `ptr` points into.
///
/// An `#ArrayStorage` can sit above the base of the block it was allocated in (see
/// `build_alloc_array_storage`), so the base and the object are reached from one another by an
/// offset in bytes. Both ends lie in the one allocation, and the emitted code says so.
pub fn build_gep_within_allocation<'c, 'm>(
    gc: &Generator<'c, 'm>,
    ptr: PointerValue<'c>,
    offset: IntValue<'c>,
    name: &str,
) -> PointerValue<'c> {
    unsafe {
        gc.builder()
            .build_in_bounds_gep(gc.context.i8_type(), ptr, &[offset], name)
    }
    .unwrap()
}

/// Allocate the block of an `#ArrayStorage` object of `sizeof` bytes and return the object's address
/// within it, together with the distance from the base of the block to that address.
///
/// From `ARRAY_ALIGNED_ALLOC_THRESHOLD` bytes up, the block carries room to slide the object off its
/// base, and the object is placed where the element buffer starts on `ARRAY_BUF_ALIGNMENT`. A
/// smaller block takes the whole allocation and whatever alignment `malloc` gives.
///
/// The returned distance is what the object was placed by, which its address alone does not tell:
/// an allocator is free to hand out any alignment the requested size can hold, and mimalloc, for
/// one, aligns an 8-byte allocation -- the size of an empty array's storage -- to 8 bytes.
///
/// The threshold is applied by masking, so the allocation stays a single basic block: an array
/// allocation is a handful of instructions that many callers inline, and extra blocks in every one
/// of them cost more in inlining decisions downstream than the arithmetic the mask spends.
fn build_alloc_array_storage<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    struct_type: StructType<'c>,
    sizeof: IntValue<'c>,
) -> (PointerValue<'c>, IntValue<'c>) {
    let i64_ty = gc.context.i64_type();
    let is_aligned = build_storage_is_aligned(gc, sizeof);
    let aligned_mask = gc
        .builder()
        .build_int_s_extend(is_aligned, i64_ty, "aligned_mask@alloc_array_storage")
        .unwrap();

    // A storage worth aligning asks for room to be placed off the base of its block; one below the
    // threshold asks for its own size and stays at the base.
    let slack = gc
        .builder()
        .build_and(
            aligned_mask,
            i64_ty.const_int(ARRAY_STORAGE_ALLOC_SLACK, false),
            "slack@alloc_array_storage",
        )
        .unwrap();
    let alloc_size = gc
        .builder()
        .build_int_add(sizeof, slack, "alloc_size@alloc_array_storage")
        .unwrap();
    let base = build_malloc(gc, alloc_size, "malloc_storage@alloc_array_storage");
    let alloc_offset = gc
        .builder()
        .build_and(
            build_array_storage_shift(gc, struct_type, base),
            aligned_mask,
            "alloc_offset@alloc_array_storage",
        )
        .unwrap();
    let ptr =
        build_gep_within_allocation(gc, base, alloc_offset, "storage_ptr@alloc_array_storage");
    (ptr, alloc_offset)
}

/// Free the allocation a boxed object of type `ty` lives in.
// PROOF: D/A, P28 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn build_free_boxed<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    ptr: PointerValue<'c>,
    ty: &Arc<TypeNode>,
) {
    // An `#ArrayStorage` can sit above the base of its allocation, so step back by the distance its
    // control block records.
    let base = if ty.is_array_storage() {
        let alloc_offset = read_alloc_offset(gc, ptr);
        let neg_alloc_offset = gc
            .builder()
            .build_int_neg(alloc_offset, "neg_alloc_offset")
            .unwrap();
        build_gep_within_allocation(gc, ptr, neg_alloc_offset, "alloc_base")
    } else {
        ptr
    };
    gc.builder().build_free(base).unwrap();
}

/// A pointer to the field of the control block of `ptr` recording how far the object sits above the
/// base of its allocation.
fn build_gep_alloc_offset<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    ptr: PointerValue<'c>,
) -> PointerValue<'c> {
    let ctrl_blk_ty = control_block_type(gc);
    gc.builder()
        .build_struct_gep(
            ctrl_blk_ty,
            ptr,
            CTRL_BLK_ALLOC_OFFSET_IDX,
            "ptr_to_alloc_offset",
        )
        .unwrap()
}

/// How far the object at `ptr` sits above the base of its allocation, as a pointer-sized integer.
pub fn read_alloc_offset<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    ptr: PointerValue<'c>,
) -> IntValue<'c> {
    let ptr_to_alloc_offset = build_gep_alloc_offset(gc, ptr);
    let alloc_offset = gc
        .builder()
        .build_load(
            alloc_offset_type(gc.context),
            ptr_to_alloc_offset,
            "alloc_offset",
        )
        .unwrap()
        .into_int_value();
    gc.builder()
        .build_int_z_extend(alloc_offset, gc.context.i64_type(), "alloc_offset_as_i64")
        .unwrap()
}

/// Record how far the object at `ptr` sits above the base of its allocation.
pub fn write_alloc_offset<'c, 'm>(
    gc: &mut Generator<'c, 'm>,
    ptr: PointerValue<'c>,
    alloc_offset: IntValue<'c>,
) {
    let ptr_to_alloc_offset = build_gep_alloc_offset(gc, ptr);
    let alloc_offset = gc
        .builder()
        .build_int_truncate(
            alloc_offset,
            alloc_offset_type(gc.context),
            "alloc_offset_as_byte",
        )
        .unwrap();
    gc.builder()
        .build_store(ptr_to_alloc_offset, alloc_offset)
        .unwrap();
}

/// A fresh object of type `ty`, with its control block initialized and its remaining fields left
/// undefined for the caller to fill in. A boxed type is allocated on the heap and comes back as a
/// pointer to it; an unboxed type comes back as an undefined aggregate value.
// PROOF: D/A, P7a, P7d, P7e, P26, P27, P28, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn create_obj<'c, 'm>(
    ty: Arc<TypeNode>,
    // Captured values. Used only for creating dynamic object.
    capture: &Vec<Arc<TypeNode>>,
    // Capacity of array. Used only for creating array object.
    array_capacity: Option<IntValue<'c>>,
    gc: &mut Generator<'c, 'm>,
    _name: Option<&str>,
) -> Object<'c> {
    // Validate arguments. The capacity is for the `#ArrayStorage` object, which carries the element
    // buffer; a flipped `Array` value itself is an unboxed aggregate created without one.
    assert!(ty.is_ground());
    assert!(ty.is_dynamic() || capture.is_empty());
    assert!(array_capacity.is_some() == ty.is_array_storage());
    assert!(!ty.is_funptr()); // Funptr type is not supported, Currently, there is no need to create an object for funptr.

    let context = gc.context;
    let object_type = ty.get_object_type(capture, gc.type_env());
    let struct_type = object_type.to_struct_type(gc);

    // Allocate object. An array storage can be placed above the base of its allocation, so it
    // carries the distance it was placed by; every other object starts at the base.
    let alloc_offset_at_base = gc.context.i64_type().const_zero();
    let (obj, alloc_offset) = if ty.is_array_storage() {
        // When the object is the array storage (a control block and a flexible element buffer),
        let sizeof = object_type.size_of(gc, array_capacity);
        let (ptr, alloc_offset) = build_alloc_array_storage(gc, struct_type, sizeof);
        (
            Object::new(ptr.as_basic_value_enum(), ty.clone(), gc),
            alloc_offset,
        )
    } else {
        if object_type.is_unbox {
            // When the object is unboxed (not a funptr),
            (
                Object::new(
                    struct_type.get_undef().as_basic_value_enum(),
                    ty.clone(),
                    gc,
                ),
                alloc_offset_at_base,
            )
        } else {
            // When the object is boxed,
            let sizeof = struct_type.size_of().unwrap();
            let ptr = build_malloc(gc, sizeof, "malloc@create_obj");
            (
                Object::new(ptr.as_basic_value_enum(), ty.clone(), gc),
                alloc_offset_at_base,
            )
        }
    };

    // Initialize refcnt, refcnt_state and traverser for dynamic object.
    for (i, ft) in object_type.field_types.iter().enumerate() {
        match ft {
            ObjectFieldType::ControlBlock => {
                // Initialize the control block.
                assert_eq!(i, 0);
                // Get pointer to control block.
                let ptr_to_ctrl_blk = obj.gep_boxed(gc, i as u32);

                // Initialize the reference counter 1.
                let ptr_to_refcnt = gc.get_refcnt_ptr(ptr_to_ctrl_blk);
                gc.builder()
                    .build_store(ptr_to_refcnt, refcnt_type(context).const_int(1, false))
                    .unwrap();

                // A fresh object is reachable from the thread that made it alone.
                gc.set_refcnt_state(ptr_to_ctrl_blk, RefcntState::LOCAL);

                // Record how far the object was placed above the base of its allocation.
                write_alloc_offset(gc, ptr_to_ctrl_blk, alloc_offset);
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
                // Initialize the capacity of the array.
                assert_eq!(i, ARRAY_CAP_IDX as usize);
                let ptr_to_cap = obj.gep_boxed(gc, i as u32);
                gc.builder()
                    .build_store(ptr_to_cap, array_capacity.unwrap())
                    .unwrap();
            }
            // The storage buffer is left uninitialized; there is no capacity field to set.
            ObjectFieldType::ArrayStorageBuf(_) => {}
            ObjectFieldType::TraverseFunction => {
                // Initialize the traverser function.
                assert_eq!(i, DYNAMIC_OBJ_TRAVARSER_IDX as usize);
                let ptr_to_trav = obj.gep_boxed(gc, i as u32);
                let trav = get_traverser_ptr(&ty, capture, gc, None);
                gc.builder().build_store(ptr_to_trav, trav).unwrap();
            }
            ObjectFieldType::UnionBuf(_) => {}
            ObjectFieldType::UnionTag => {}
        }
    }

    obj
}

/// The address of the traverser function for an object of type `ty`, for a dynamic object to store
/// and call indirectly.
///
/// # Arguments
/// * `capture` — the captured types of a dynamic object, whose traverser disposes of them.
/// * `work` — the job the traverser performs: `TraverserWorkType::release` selects the object's
///   destructor, `mark_global` and `mark_threaded` the corresponding markers. `None` selects the
///   dynamic traverser, which takes the job as a second argument and dispatches on it at run time.
///
/// # Returns
/// Where the type leaves the traverser no work to do, the address of an empty function, so that a
/// caller holding this pointer always has one to call.
// PROOF: P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn get_traverser_ptr<'c, 'm>(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>, // used in destructor of lambda
    gc: &mut Generator<'c, 'm>,
    work: Option<TraverserWorkType>,
) -> PointerValue<'c> {
    // The pointer is stored in a dynamic object and called indirectly at reference count zero, so
    // nothing is known about the state of what it traverses.
    match create_traverser(ty, capture, gc, work, RcState::Unknown) {
        Some(fv) => fv.as_global_value().as_pointer_value(),
        None => {
            let is_dynamic = work.is_none();
            let func_name = if is_dynamic {
                "fixruntime_empty_traverser_dynamic"
            } else {
                "fixruntime_empty_traverser"
            };

            // Define an empty function (if there is none) and return its pointer.
            let fv = if let Some(fv) = gc.module.get_function(func_name) {
                fv
            } else {
                let func_type = traverser_type(gc, ty, work.is_none());
                let func = gc
                    .module
                    .add_function(func_name, func_type, Some(Linkage::Internal));
                let _builder_guard = gc.push_builder();
                let bb = gc.context.append_basic_block(func, "entry");
                gc.builder().position_at_end(bb);
                gc.builder().build_return(None).unwrap();
                func
            };
            fv.as_global_value().as_pointer_value()
        }
    }
}

/// Generate the traverser function for an object of type `ty`: a function taking a pointer to the
/// object, which walks its fields and performs one reference-counting job on each.
///
/// # Arguments
/// * `capture` — the captured types of a dynamic object, whose traverser disposes of them.
/// * `work` — the job to compile in: `TraverserWorkType::release` makes the object's destructor,
///   `mark_global` and `mark_threaded` make the corresponding markers. `None` makes the dynamic
///   traverser, which takes the job as a second argument and dispatches on it at run time.
///
/// # Returns
/// `None` where the traverser would have no work to do, which lets a caller emit no call at all.
// PROOF: P26, P27, P29, P30 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn create_traverser<'c, 'm>(
    ty: &Arc<TypeNode>,
    capture: &Vec<Arc<TypeNode>>,
    gc: &mut Generator<'c, 'm>,
    work: Option<TraverserWorkType>,
    state: RcState,
) -> Option<FunctionValue<'c>> {
    assert!(ty.is_ground());
    assert!(ty.is_dynamic() || capture.is_empty());
    if ty.is_dynamic() && capture.is_empty() {
        return None;
    }
    if ty.is_fully_unboxed(gc.type_env()) {
        return None;
    }

    // If the function already exists, return it.
    let trav_name = ty.traverser_name(capture, work, state);
    if let Some(fv) = gc.module.get_function(&trav_name) {
        return Some(fv);
    }

    // Define traverser function.
    let func_type = traverser_type(gc, ty, work.is_none());
    let func = gc
        .module
        .add_function(&trav_name, func_type, Some(Linkage::Internal));

    let bb = gc.context.append_basic_block(func, "entry");

    // Implement traverser function.
    let _builder_guard = gc.push_builder();
    gc.builder().position_at_end(bb);

    // Reassemble the object from its part parameters (see `traverser_type`).
    let part_count = {
        let embedded = ty.get_embedded_type(gc);
        gc.part_count(embedded)
    };
    let parts = (0..part_count)
        .map(|i| func.get_nth_param(i as u32).unwrap())
        .collect::<Vec<_>>();
    let obj = Object::from_parts(parts, ty.clone(), gc);

    match work {
        Some(work) => {
            // Static traverser case.
            build_traverse(obj, capture, work, gc, state);
            gc.builder().build_return(None).unwrap();
        }
        None => {
            // Dynamic traverser case.

            // The work-type argument follows the object's part parameters.
            let work = func
                .get_nth_param(part_count as u32)
                .unwrap()
                .into_int_value();

            // Branch to the block of the work asked for: destruction of the objects it owns
            // (`work == 0`), marking them global (`work == 1`), or marking them threaded
            // (`work == 2`, compiled only into a program that runs on several threads).
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
            let cases = work_bbs
                .iter()
                .map(|(work, bb)| (work_ty.const_int(*work as u64, false), bb.clone()))
                .collect::<Vec<_>>();

            // Every call passes a work this traverser was generated for, so the block reached by
            // any other value ends the program instead of standing in for one of them.
            let unknown_work_bb = gc
                .context
                .append_basic_block(func, "unknown_work_bb@traverser");
            gc.builder()
                .build_switch(work, unknown_work_bb, &cases)
                .unwrap();
            gc.builder().position_at_end(unknown_work_bb);
            if gc.config.develop_mode {
                gc.panic("A traverser was called with a work it was not generated for.\n");
            }
            gc.builder().build_unreachable().unwrap();

            for (work, work_bb) in work_bbs.iter() {
                let work = TraverserWorkType(*work);
                gc.builder().position_at_end(*work_bb);
                build_traverse(obj.clone(), capture, work, gc, state);
                gc.builder().build_return(None).unwrap();
            }
        }
    }

    Some(func)
}

/// Emit the body of a traverser: perform `work` on every boxed object `obj` directly owns, walking
/// through its unboxed structure to reach them.
// PROOF: D/A, P26, P28 (dev-docs/proof/rc_ir/borrow-cancel)
fn build_traverse<'c, 'm>(
    obj: Object<'c>,
    capture: &Vec<Arc<TypeNode>>, // used in destructor of dynamic object.
    work: TraverserWorkType,
    gc: &mut Generator<'c, 'm>,
    state: RcState, // What is known about the state of the boxed leaves this traverser reaches.
) {
    // `Array a` = unbox { SubObject(#ArrayStorage a), size, cap }: the storage's own destructor is
    // free-only, so the array value drives element release. Work on the storage through its
    // refcount bookkeeping, and when that drops it to zero release its `[0, size)` elements. Doing
    // the element release inside `traverse_refs` (called only at rc 1 -> 0) keeps a shared array's
    // elements alive.
    if obj.ty.is_array() {
        let elem_ty = obj.ty.field_types(gc.type_env())[0].clone();
        let size = obj.extract_field(gc, ARRAY_SIZE_IDX).into_int_value();
        let storage = get_array_storage(gc, &obj);
        let buffer = storage.gep_boxed(gc, STORAGE_BUF_IDX);
        gc.build_traverser_work_nonnull_boxed_with(&storage, work, state, |gc| {
            ObjectFieldType::traverse_array_buf(
                gc,
                size,
                buffer,
                elem_ty,
                work,
                None,
                RcState::Unknown,
            );
        });
        return;
    }

    // `PunchedArray a` = unbox { Array a, I64 idx }: work on the inner array's elements
    // while skipping the hole at `idx` (the moved-out element), reusing the storage's refcount
    // bookkeeping.
    if obj.ty.is_punched_array() {
        let inner_array_ty = obj.ty.field_types(gc.type_env())[0].clone();
        let elem_ty = inner_array_ty.field_types(gc.type_env())[0].clone();
        let inner_array = Object::new(
            obj.extract_field(gc, PUNCHED_ARRAY_ARRAY_IDX),
            inner_array_ty,
            gc,
        );
        let idx = Object::new(
            obj.extract_field(gc, PUNCHED_ARRAY_HOLE_IDX),
            make_i64_ty(),
            gc,
        )
        .extract_field(gc, 0)
        .into_int_value();
        let size = inner_array
            .extract_field(gc, ARRAY_SIZE_IDX)
            .into_int_value();
        let storage = get_array_storage(gc, &inner_array);
        let buffer = storage.gep_boxed(gc, STORAGE_BUF_IDX);
        gc.build_traverser_work_nonnull_boxed_with(&storage, work, state, |gc| {
            ObjectFieldType::traverse_array_buf(
                gc,
                size,
                buffer,
                elem_ty,
                work,
                Some(idx),
                RcState::Unknown,
            );
        });
        return;
    }

    // In this function, we need to access captured fields, which is not possible by `obj` only.
    let object_type = ty_to_object_ty(&obj.ty, capture, gc.type_env());
    let struct_type = object_type.to_struct_type(gc);

    for (i, ft) in object_type.field_types.iter().enumerate() {
        match ft {
            ObjectFieldType::SubObject(subty, is_punched) => {
                if *is_punched {
                    continue;
                }
                let subval = if capture.is_empty() {
                    obj.extract_field(gc, i as u32)
                } else {
                    obj.extract_field_as(gc, struct_type, i as u32)
                };
                let subobj = Object::new(subval, subty.clone(), gc);
                gc.build_traverser_work(subobj, work, state);
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
            // The `is_array` branch of this function walks an array's elements through its
            // `#ArrayStorage`, whose buffer is an `ArrayStorageBuf`: that is the only array field
            // an object holds.
            ObjectFieldType::Array(_) => unreachable!(),
            // Reference-count-inert: the storage buffer has no length, and the owning `Array` value's
            // traverser drives element lifetime. Traversing the storage itself touches no element.
            ObjectFieldType::ArrayStorageBuf(_) => {}
            ObjectFieldType::UnionTag => {}
            ObjectFieldType::UnionBuf(_) => {
                // The amount is unused on the release/mark path; pass a constant 1.
                let one = gc.context.i64_type().const_int(1, false);
                ObjectFieldType::retain_release_mark_union(gc, obj.clone(), Some(work), one, state);
            }
            ObjectFieldType::TraverseFunction => {}
        }
    }
}

/// The debug type for how `ty` is embedded in a field: a pointer to the boxed layout when `ty` is
/// boxed, and the layout itself when it is unboxed.
pub fn ty_to_debug_embedded_ty<'c, 'm>(
    ty: Arc<TypeNode>,
    gc: &mut Generator<'c, 'm>,
) -> DIType<'c> {
    let debug_struct_ty = ty_to_debug_struct_ty(ty.clone(), gc);
    if ty.is_box(&gc.type_env()) {
        let ptr_ty = gc.context.ptr_type(AddressSpace::from(0));
        let size_in_bits = gc.target_data.get_bit_size(&ptr_ty);
        let align_in_bits = gc.target_data.get_abi_alignment(&ptr_ty) * 8;
        gc.get_di_builder()
            .create_pointer_type(
                "<pointer to boxed value>",
                debug_struct_ty,
                size_in_bits,
                align_in_bits,
                AddressSpace::from(0),
            )
            .as_type()
    } else {
        debug_struct_ty
    }
}

/// The debug type describing `ty`'s in-memory layout, caching each type by name so recursive types
/// terminate.
pub fn ty_to_debug_struct_ty<'c, 'm>(ty: Arc<TypeNode>, gc: &mut Generator<'c, 'm>) -> DIType<'c> {
    let key = ty.to_string();
    gc.get_or_build_di_type(key, |gc| ty_to_debug_struct_ty_body(ty, gc))
}

/// The debug type describing `ty`'s in-memory layout, built by expanding its fields. The caching
/// and recursion-breaking that keep this finite on recursive types live in
/// `ty_to_debug_struct_ty`.
fn ty_to_debug_struct_ty_body<'c, 'm>(ty: Arc<TypeNode>, gc: &mut Generator<'c, 'm>) -> DIType<'c> {
    let name = &ty.to_string();
    let obj_type = ty_to_object_ty(&ty, &vec![], gc.type_env());
    // Bool is a union type bit-identical to i8, but its debug type is `DW_ATE_BOOLEAN`. It is
    // checked before the primitive gate because Bool's variant is `Union`.
    if ty.is_boolean() {
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
    let is_primitive = !ty.is_closure()
        && ty.toplevel_tycon_info(gc.type_env()).variant == TyConVariant::Primitive;
    if is_primitive {
        // Primitive case
        if obj_type.field_types.len() == 0 {
            // Empty type case
            gc.get_di_builder()
                .create_struct_type(
                    gc.get_di_compile_unit().as_debug_info_scope(),
                    name,
                    gc.create_di_file(None),
                    0,
                    0,
                    0,
                    0,
                    None,
                    &[],
                    0,
                    None,
                    name,
                )
                .as_type()
        } else {
            // General primitive case
            assert!(obj_type.field_types.len() == 1);
            // Unwrap the element type from the struct type.
            obj_type.field_types[0].to_debug_type(gc)
        }
    } else {
        let struct_type = gc.struct_type_of(&ty);
        let size_in_bits = gc.target_data.get_bit_size(&struct_type);
        let align_in_bits = gc.target_data.get_abi_alignment(&struct_type) * 8;

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
                ObjectFieldType::ArrayStorageBuf(_) => "<array elements>".to_string(),
            };
            if ty.is_array() {
                // Name the flipped `Array` value's three members so a debugger can reach the
                // elements: `_storage` is bracket-free so gdb's expression parser accepts
                // `arr._storage` (the pointer to the `#ArrayStorage` holding the elements), while
                // the size and capacity are register-readable scalars.
                member_name = match i as u32 {
                    ARRAY_STORAGE_IDX => "_storage".to_string(),
                    ARRAY_SIZE_IDX => "<array size>".to_string(),
                    ARRAY_CAP_IDX => "<array capacity>".to_string(),
                    _ => unreachable!("unexpected Array value member at index {}", i),
                };
            }

            let element_di_ty = field.to_debug_type(gc);
            let element_ty = field.to_basic_type(gc);
            let size_in_bits = element_di_ty.get_size_in_bits();
            let align_in_bits = gc.target_data.get_abi_alignment(&element_ty) * 8;
            let offset_in_bits = gc
                .target_data
                .offset_of_element(&struct_type, i as u32)
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

        // `#ArrayStorage`'s declared size must cover the `DEBUG_ARRAY_ASSUMED_LEN` elements claimed
        // by its `<array elements>` member: debuggers read member values only within a type's
        // declared byte size, so without this the elements past the real (flexible, zero-length)
        // buffer would not be displayed. The flipped `Array` value keeps its true 24-byte layout —
        // only the storage it points to is inflated.
        let size_in_bits = if ty.is_array_storage() {
            let buffer_member = elements.last().unwrap();
            buffer_member.get_offset_in_bits() + buffer_member.get_size_in_bits()
        } else {
            size_in_bits
        };

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
