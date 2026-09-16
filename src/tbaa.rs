//! The type-based alias analysis metadata the generated code carries.
//!
//! LLVM has no way to tell a boxed object's reference count from the value the object holds: both
//! are reached through pointers a program loaded from somewhere. A reference count written between
//! two reads of a field therefore reaches, as far as LLVM knows, the field itself, and the second
//! read has to happen again.
//!
//! A `!tbaa` tag on a load or a store names the region of memory that access reaches. LLVM takes
//! two accesses naming different regions never to reach the same byte, which is what lets a
//! reference count be written between two reads of a field without the field being read twice.
//!
//! Every access the code generator emits reaches memory the compiler itself laid out, which is why
//! each one names a region. An address the program computed can land on any byte, a control block
//! included, and the compiler hands such an address to foreign code rather than reading through
//! it.

use inkwell::context::Context;
use inkwell::values::{InstructionValue, MetadataValue};

/// The region of memory a load or store the code generator emits reaches.
///
/// The regions partition the memory the generated code touches: every byte a tagged access reaches
/// belongs to one of them, and no byte belongs to two. That is what the tags promise LLVM, so a
/// byte that belonged to two regions would let LLVM reorder the accesses that reach it.
///
/// A bulk transfer carries no region, and so reaches every one of them. The transfer that moves an
/// `#ArrayStorage` into a new block covers the control block and the elements together, which lie
/// in two regions at once.
#[derive(Clone, Copy)]
pub enum MemoryRegion {
    /// The reference count of a boxed object.
    Refcnt,
    /// The `RefcntState` of a boxed object, which says how its reference count is maintained.
    RefcntState,
    /// How far a boxed object sits above the base of its allocation.
    AllocOffset,
    /// Every byte outside a boxed object's control block: the fields of an object, the elements of
    /// an array, a union's tag and payload, the storage of a global, a stack slot the compiler
    /// allocated, the array of arguments the C runtime hands the program.
    ///
    /// One region covers them all, so a value written under one LLVM type and read back under
    /// another — which a union's payload buffer is, and `Generator::bit_cast` — stays a pair of
    /// accesses that LLVM takes to reach the same byte.
    Data,
}

impl MemoryRegion {
    /// The name the type tree gives this region, which is the name the generated code carries.
    pub fn name(self) -> &'static str {
        match self {
            MemoryRegion::Refcnt => "refcnt",
            MemoryRegion::RefcntState => "refcnt state",
            MemoryRegion::AllocOffset => "alloc offset",
            MemoryRegion::Data => "data",
        }
    }
}

/// The name the type tree of the generated code is rooted in.
///
/// LLVM relates two access tags only when their trees share a root, and takes tags from two roots
/// to reach the same memory. A root of Fix's own is therefore what keeps these tags from being read
/// against those of the C code a program links with, which is rooted in a name of its own.
const ROOT_NAME: &str = "Fix TBAA";

/// The `!tbaa` access tag of each region, built once per LLVM context and attached to every access
/// the code generator emits into it.
pub struct TbaaTags<'c> {
    refcnt: MetadataValue<'c>,
    refcnt_state: MetadataValue<'c>,
    alloc_offset: MetadataValue<'c>,
    data: MetadataValue<'c>,
    /// The id LLVM knows the metadata kind `tbaa` by, which an access carries its tag under.
    kind_id: u32,
}

impl<'c> TbaaTags<'c> {
    /// The tags of `context`: one type node per region, each under the tree's root, and the access
    /// tag naming it.
    pub fn new(context: &'c Context) -> Self {
        let root = context.metadata_node(&[context.metadata_string(ROOT_NAME).into()]);
        let region_tag = |region: MemoryRegion| {
            // A type node names the region, the node it sits under, and the offset it begins at
            // within that node. A region is a leaf of the tree, so it begins where the root does.
            let zero = context.i64_type().const_zero();
            let region_node = context.metadata_node(&[
                context.metadata_string(region.name()).into(),
                root.into(),
                zero.into(),
            ]);
            // An access tag names the region the access lands in, the region of the value it moves,
            // and the offset of that value within the first. An access to a whole region moves the
            // region itself, from its beginning.
            context.metadata_node(&[region_node.into(), region_node.into(), zero.into()])
        };
        TbaaTags {
            refcnt: region_tag(MemoryRegion::Refcnt),
            refcnt_state: region_tag(MemoryRegion::RefcntState),
            alloc_offset: region_tag(MemoryRegion::AllocOffset),
            data: region_tag(MemoryRegion::Data),
            kind_id: context.get_kind_id("tbaa"),
        }
    }

    /// State that `instruction` reaches `region`, so that LLVM can tell it from the accesses that
    /// reach the others.
    pub fn tag_access(&self, instruction: InstructionValue<'c>, region: MemoryRegion) {
        let tag = match region {
            MemoryRegion::Refcnt => self.refcnt,
            MemoryRegion::RefcntState => self.refcnt_state,
            MemoryRegion::AllocOffset => self.alloc_offset,
            MemoryRegion::Data => self.data,
        };
        instruction
            .set_metadata(tag, self.kind_id)
            .expect("a `!tbaa` tag is a metadata node");
    }
}
