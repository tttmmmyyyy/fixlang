use crate::misc::Map;
use crate::tbaa::MemoryRegion;
use crate::tests::test_util::{first_local_value, generated_llvm_ir_modules};
use std::sync::OnceLock;

/// A program that reaches every region of memory the compiler names. A boxed struct and a boxed
/// union give it fields and a payload buffer, an array gives it elements, a global gives it storage
/// of its own, and every boxed value it builds carries a control block that the program allocates,
/// counts references to and frees.
const MEMORY_ACCESS_SOURCE: &str = r#"
    module Main;

    type Pair = box struct { left : I64, right : Array I64 };
    type Choice = box union { one : I64, many : Array I64 };

    squares : Array I64;
    squares = Array::from_map(4, |i| i * i);

    width : Choice -> I64;
    width = |c| if c.is_one { c.as_one } else { c.as_many.@size };

    main : IO ();
    main = (
        let args = *get_args;
        let k = args.@size;
        let pair = Pair { left : k, right : Array::from_map(k + 3, |i| i) };
        let choice = if k == 1 { Choice::one(pair.@left) } else { Choice::many(pair.@right) };
        let offset = |i| pair.@left + squares.@(i);
        println $ (width(choice) + offset(1)).to_string
    );
"#;

/// The modules the compiler writes for `MEMORY_ACCESS_SOURCE`, built once and shared by every test
/// that reads them. The build is what these tests spend their time on.
///
/// Each module is read on its own, because a module numbers its metadata for itself: the `!0` of
/// one module and the `!0` of the next stand for different nodes.
fn memory_access_modules() -> &'static [String] {
    static MODULES: OnceLock<Vec<String>> = OnceLock::new();
    MODULES.get_or_init(|| generated_llvm_ir_modules(MEMORY_ACCESS_SOURCE, "none"))
}

/// Every memory access the compiler emits says which region of memory it reaches.
///
/// LLVM takes an access that says nothing to reach every region, so one such access among the
/// reference-count updates around it is enough to make the code read a value again that it already
/// holds. `Generator::build_load`, `build_store` and `build_atomicrmw` are what put the region on
/// an access, and an access built through the LLVM builder directly goes out bare.
#[test]
pub fn test_every_memory_access_says_which_region_it_reaches() {
    let mut accesses = 0;
    for module in memory_access_modules() {
        let module_accesses = memory_accesses(module);
        accesses += module_accesses.len();
        let bare = module_accesses
            .iter()
            .filter(|access| access.tag.is_none())
            .map(|access| access.line.to_string())
            .collect::<Vec<_>>();
        assert!(
            bare.is_empty(),
            "every memory access should carry a `!tbaa` tag, but {} do not:\n{}",
            bare.len(),
            bare.join("\n"),
        );
    }
    assert!(
        accesses > 0,
        "a program that builds and reads boxed values should reach memory, so that this test has \
         accesses to read",
    );
}

/// The reference count of a boxed object, the state beside it, the offset of the object within its
/// allocation, and the value the object holds are four regions, and every access reaches the region
/// of the pointer it is made through.
///
/// This is the promise the tags carry: a store to one region leaves every other region where a
/// reader already has it. Two accesses put in one region give that up, and two put in regions of
/// their own where they reach the same byte let LLVM reorder them.
#[test]
pub fn test_each_pointer_reaches_the_region_its_name_says() {
    let mut accesses: Map<&str, usize> = Map::default();
    for module in memory_access_modules() {
        let nodes = metadata_nodes(module);
        for access in memory_accesses(module) {
            let Some(pointer) = access.pointer.map(name_without_llvm_suffix) else {
                continue;
            };
            let Some((_, region)) = POINTERS_INTO_EACH_REGION
                .iter()
                .find(|(named, _)| *named == pointer)
            else {
                continue;
            };
            *accesses.entry(pointer).or_default() += 1;
            let tag = access.tag.unwrap_or_else(|| {
                panic!(
                    "an access through `{}` carries no tag: {}",
                    pointer, access.line
                )
            });
            assert_eq!(
                region_of_tag(&nodes, tag),
                region.name(),
                "an access through `{}` should reach the `{}` region: {}",
                pointer,
                region.name(),
                access.line,
            );
        }
    }
    for (pointer, _) in POINTERS_INTO_EACH_REGION {
        assert!(
            accesses.contains_key(pointer),
            "the program should reach memory through `{}`, so that this test has accesses to read",
            pointer,
        );
    }
}

/// The name the code generator gives the pointer it reaches a region through, and the region those
/// accesses reach. A field of a boxed object and an element of an array are one region, so two of
/// the pointers name it.
///
/// The names come from `Generator::get_refcnt_ptr`, `Generator::get_refcnt_state_ptr`,
/// `build_gep_alloc_offset`, `Object::ptr_to_field_as` and `build_gep_array_elem`.
const POINTERS_INTO_EACH_REGION: [(&str, MemoryRegion); 5] = [
    ("%ptr_to_refcnt", MemoryRegion::Refcnt),
    ("%ptr_to_refcnt_state", MemoryRegion::RefcntState),
    ("%ptr_to_alloc_offset", MemoryRegion::AllocOffset),
    ("%gep2field", MemoryRegion::Value),
    ("%ptr_to_elem_of_array", MemoryRegion::Value),
];

/// One instruction of the generated code that reaches memory.
struct MemoryAccess<'a> {
    /// The instruction, as LLVM writes it.
    line: &'a str,
    /// The local value holding the address it reaches, absent where that address is a global.
    pointer: Option<&'a str>,
    /// The `!N` name of its `!tbaa` tag, absent where it carries none.
    tag: Option<&'a str>,
}

/// Every instruction of `module` that reaches memory.
///
/// LLVM indents the instructions of a function body, so the indentation is what tells them from the
/// declarations, global definitions and metadata around them.
fn memory_accesses(module: &str) -> Vec<MemoryAccess<'_>> {
    module
        .lines()
        .filter(|line| line.starts_with("  "))
        .map(str::trim)
        .filter(|line| reaches_memory(instruction_of(line)))
        .map(|line| MemoryAccess {
            line,
            pointer: accessed_pointer(line),
            tag: access_tag(line),
        })
        .collect()
}

/// The instruction on `line`, with the name LLVM writes its result under taken off.
///
/// # Examples
/// `instruction_of("%x = load i64, ptr %p, align 8")` is `"load i64, ptr %p, align 8"`, and
/// `instruction_of("store i64 0, ptr %p, align 8")` is the line itself.
fn instruction_of(line: &str) -> &str {
    line.split_once(" = ")
        .map_or(line, |(_, instruction)| instruction)
}

/// Whether `instruction` is one of the three the code generator emits that reach memory.
fn reaches_memory(instruction: &str) -> bool {
    ["load ", "store ", "atomicrmw "]
        .iter()
        .any(|opcode| instruction.starts_with(opcode))
}

/// The local value holding the address the access on `line` reaches, absent where that address is a
/// global. LLVM writes the pointer operand as the last `ptr` operand before the alignment.
fn accessed_pointer(line: &str) -> Option<&str> {
    let operands = line.split(", align ").next()?;
    let pointer_operand = operands.rfind("ptr ")?;
    first_local_value(&operands[pointer_operand..])
}

/// The `!N` name of the `!tbaa` tag the access on `line` carries, absent where it carries none.
fn access_tag(line: &str) -> Option<&str> {
    let (_, tag) = line.split_once("!tbaa ")?;
    Some(tag.split(',').next()?.trim())
}

/// The name the compiler gave a value, without the number LLVM appends to tell apart the values of
/// one function that the compiler named the same.
///
/// # Examples
/// `name_without_llvm_suffix("%ptr_to_refcnt20")` is `"%ptr_to_refcnt"`.
fn name_without_llvm_suffix(value: &str) -> &str {
    value.trim_end_matches(|c: char| c.is_ascii_digit())
}

/// The operands of each metadata node of `module`, by the `!N` name that stands for it, as the text
/// LLVM writes between the node's braces.
fn metadata_nodes(module: &str) -> Map<&str, &str> {
    let mut nodes = Map::default();
    for line in module.lines() {
        let Some((name, operands)) = line.split_once(" = !{") else {
            continue;
        };
        let Some(operands) = operands.strip_suffix('}') else {
            continue;
        };
        if name.starts_with('!') {
            nodes.insert(name, operands);
        }
    }
    nodes
}

/// The name of the region the access tag named `tag` reaches.
///
/// An access tag is `!{<region>, <region>, i64 <offset>}` and a region is
/// `!{!"<name>", <parent>, i64 <offset>}`, so the name is the first operand of the tag's first
/// operand.
fn region_of_tag(nodes: &Map<&str, &str>, tag: &str) -> String {
    let region = first_operand(node(nodes, tag));
    first_operand(node(nodes, region))
        .trim_start_matches('!')
        .trim_matches('"')
        .to_string()
}

/// The operands of the metadata node named `name`.
fn node<'a>(nodes: &Map<&str, &'a str>, name: &str) -> &'a str {
    nodes
        .get(name)
        .unwrap_or_else(|| panic!("the module should define the metadata node `{}`", name))
}

/// The first operand of a metadata node, as LLVM writes the operands between the node's braces.
fn first_operand(operands: &str) -> &str {
    operands.split(',').next().unwrap().trim()
}
