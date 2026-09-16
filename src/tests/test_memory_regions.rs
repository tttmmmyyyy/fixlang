use crate::misc::{Map, Set};
use crate::tbaa::MemoryRegion;
use crate::tests::test_util::{
    emitted_llvm_ir_modules, first_local_value, fix_build_source_command,
    generated_llvm_ir_modules, EmittedIr,
};
use std::sync::OnceLock;
use tempfile::TempDir;

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
fn single_threaded_modules() -> &'static [String] {
    static MODULES: OnceLock<Vec<String>> = OnceLock::new();
    MODULES.get_or_init(|| generated_llvm_ir_modules(MEMORY_ACCESS_SOURCE, "none"))
}

/// The modules the compiler writes for `MEMORY_ACCESS_SOURCE` with multi-threading on, built once
/// and shared the same way.
///
/// The reference count of a multi-threaded object is updated by an atomic read-modify-write, which
/// is the one access the code generator emits that no other build produces.
fn multi_threaded_modules() -> &'static [String] {
    static MODULES: OnceLock<Vec<String>> = OnceLock::new();
    MODULES.get_or_init(|| {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let dir = temp_dir.path();
        let build = fix_build_source_command(dir, MEMORY_ACCESS_SOURCE, "none")
            .arg("--emit-llvm")
            .arg("--threaded")
            .output()
            .expect("Failed to execute fix build");
        assert!(
            build.status.success(),
            "the build should succeed.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr),
        );
        emitted_llvm_ir_modules(dir, EmittedIr::BeforeOptimization)
    })
}

/// The builds these tests read, each with the words a failure names it by.
fn memory_access_builds() -> [(&'static str, &'static [String]); 2] {
    [
        ("in a single-threaded build", single_threaded_modules()),
        ("in a multi-threaded build", multi_threaded_modules()),
    ]
}

/// Every load, store and atomic read-modify-write the compiler emits says which region of memory it
/// reaches.
///
/// LLVM takes an access that says nothing to reach every region, so one such access among the
/// reference-count updates around it is enough to make the code read a value again that it already
/// holds. `Generator::build_load`, `build_store` and `build_atomicrmw` are what put the region on
/// an access, and an access built through the LLVM builder directly goes out bare.
#[test]
pub fn test_every_load_and_store_says_which_region_it_reaches() {
    for (build, modules) in memory_access_builds() {
        let mut accesses = 0;
        for module in modules {
            let module_accesses = memory_accesses(module);
            accesses += module_accesses.len();
            let bare = module_accesses
                .iter()
                .filter(|access| access.tag.is_none())
                .map(|access| access.line.to_string())
                .collect::<Vec<_>>();
            assert!(
                bare.is_empty(),
                "every memory access {} should carry a `!tbaa` tag, but {} do not:\n{}",
                build,
                bare.len(),
                bare.join("\n"),
            );
        }
        assert!(
            accesses > 0,
            "a program that builds and reads boxed values should reach memory {}, so that this \
             test has accesses to read",
            build,
        );
    }
    // A single-threaded build updates a reference count in place, so the atomic read-modify-write
    // is reached through the multi-threaded build alone. Counting them is what says the check above
    // read that access rather than none.
    let atomic_updates = multi_threaded_modules()
        .iter()
        .flat_map(|module| memory_accesses(module))
        .filter(|access| instruction_of(access.line).starts_with("atomicrmw "))
        .count();
    assert!(
        atomic_updates > 0,
        "a multi-threaded build should update a reference count atomically, so that the check \
         above reads those updates",
    );
}

/// Every access reaches the region of the pointer it is made through, and every region the compiler
/// names is reached.
///
/// This is the promise the tags carry: a store to one region leaves every other region where a
/// reader already has it. Two accesses put in one region give that up, and two put in regions of
/// their own where they reach the same byte let LLVM reorder them.
#[test]
pub fn test_each_access_reaches_the_region_of_its_pointer() {
    let mut regions_reached: Set<&str> = Set::default();
    for (build, modules) in memory_access_builds() {
        for module in modules {
            let nodes = metadata_nodes(module);
            for access in memory_accesses(module) {
                let pointer = accessed_pointer(access.line);
                let region = region_of_pointer(pointer).unwrap_or_else(|| {
                    panic!(
                        "the pointer `{}` {} belongs to no region of `REGION_OF_EACH_POINTER`: {}",
                        pointer.unwrap_or("<a global>"),
                        build,
                        access.line,
                    )
                });
                let tag = access.tag.unwrap_or_else(|| {
                    panic!("an access {} carries no tag: {}", build, access.line)
                });
                assert_eq!(
                    region_of_tag(&nodes, tag),
                    region.name(),
                    "an access {} through `{}` should reach the `{}` region: {}",
                    build,
                    pointer.unwrap_or("<a global>"),
                    region.name(),
                    access.line,
                );
                regions_reached.insert(region.name());
            }
        }
    }
    for (pointer, region) in REGION_OF_EACH_POINTER {
        assert!(
            regions_reached.contains(region.name()),
            "the program should reach the `{}` region, which `{}` is named as reaching, so that \
             this test has accesses to read there",
            region.name(),
            pointer,
        );
    }
}

/// The region an access reaches, for every name the code generator gives a pointer it reaches
/// memory through. The table is whole: an access through a pointer it does not name fails the test
/// above, so a new one has to be classified here.
///
/// The name in the emitted code carries a number where a function holds several values the compiler
/// named the same, and `compiler_given_name` takes that number off.
const REGION_OF_EACH_POINTER: [(&str, MemoryRegion); 13] = [
    // `Generator::get_refcnt_ptr`
    ("ptr_to_refcnt", MemoryRegion::Refcnt),
    // `Generator::get_refcnt_state_ptr`
    ("ptr_to_refcnt_state", MemoryRegion::RefcntState),
    // `build_gep_alloc_offset`
    ("ptr_to_alloc_offset", MemoryRegion::AllocOffset),
    // `Object::ptr_to_field_as`
    ("gep2field", MemoryRegion::Value),
    // `Object::gep_boxed`
    ("ptr_to_field_nocap", MemoryRegion::Value),
    // `build_gep_array_elem`
    ("ptr_to_elem_of_array", MemoryRegion::Value),
    ("ptr_to_src_elem", MemoryRegion::Value),
    ("ptr_to_dst_elem", MemoryRegion::Value),
    ("array_append_slot", MemoryRegion::Value),
    // `Generator::build_return_object` and `load_out_pointer_buffer`
    ("out_part_ptr", MemoryRegion::Value),
    // `Generator::bit_cast`
    ("alloca@bit_cast", MemoryRegion::Value),
    // `ObjectFieldType::loop_over_array_buf`
    ("release_loop_counter", MemoryRegion::Value),
    // `build_get_argv_function`
    ("elem_ptr", MemoryRegion::Value),
];

/// The region an access through `pointer` reaches, absent where `REGION_OF_EACH_POINTER` does not
/// name it. `None` for `pointer` is an access whose address is a global variable, and a global's
/// storage lies outside every control block.
fn region_of_pointer(pointer: Option<&str>) -> Option<MemoryRegion> {
    let Some(name) = pointer else {
        return Some(MemoryRegion::Value);
    };
    REGION_OF_EACH_POINTER
        .iter()
        .find(|(named, _)| *named == name)
        .map(|(_, region)| *region)
}

/// One instruction of the generated code that reaches memory.
struct MemoryAccess<'a> {
    /// The instruction, as LLVM writes it.
    line: &'a str,
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
///
/// A bulk transfer is a call to `llvm.memcpy` or `llvm.memmove` rather than one of these, and it
/// carries no region: the transfer that moves an `#ArrayStorage` to a new block moves the control
/// block along with the elements, so its bytes lie in two regions at once.
fn reaches_memory(instruction: &str) -> bool {
    ["load ", "store ", "atomicrmw "]
        .iter()
        .any(|opcode| instruction.starts_with(opcode))
}

/// The compiler's name for the address the access on `line` reaches, absent where that address is a
/// global variable. LLVM writes the pointer operand as the last `ptr` operand before the alignment.
fn accessed_pointer(line: &str) -> Option<&str> {
    let operands = line
        .split(", align ")
        .next()
        .expect("splitting a line yields its first part");
    let pointer_operand = operands
        .rfind("ptr ")
        .map(|at| operands[at + "ptr ".len()..].trim_start())
        .unwrap_or_else(|| panic!("an access takes a pointer operand: {}", line));
    if !pointer_operand.starts_with('%') {
        return None;
    }
    let named = first_local_value(pointer_operand)
        .unwrap_or_else(|| panic!("the pointer operand names a local value: {}", line));
    Some(compiler_given_name(named))
}

/// The `!N` name of the `!tbaa` tag the access on `line` carries, absent where it carries none.
fn access_tag(line: &str) -> Option<&str> {
    let (_, tag) = line.split_once("!tbaa ")?;
    Some(tag.split(',').next()?.trim())
}

/// The name the compiler gave a value: the text LLVM writes it under, without the `%` before it,
/// the quotes around a name holding a character a plain identifier cannot, and the number appended
/// to tell apart the values of one function that the compiler named the same.
///
/// # Examples
/// `compiler_given_name("%ptr_to_refcnt20")` is `"ptr_to_refcnt"`, and
/// `compiler_given_name("%\"alloca@bit_cast6\"")` is `"alloca@bit_cast"`.
fn compiler_given_name(value: &str) -> &str {
    value
        .trim_start_matches('%')
        .trim_matches('"')
        .trim_end_matches(|c: char| c.is_ascii_digit())
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
