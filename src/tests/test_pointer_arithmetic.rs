use crate::fixstd::runtime::{RUNTIME_MALLOC, RUNTIME_REALLOC};
use crate::tests::test_util::{generated_llvm_ir, llvm_function_bodies};
use std::sync::OnceLock;

/// A program that reaches an array's elements every way the compiler computes a pointer into one:
/// reading a slot, writing a slot in place, writing one of a shared array (which clones the
/// buffer), growing an array, taking a range out of one, and moving an element out of one, which
/// leaves a hole that the walk over the rest of the buffer steps around.
///
/// The elements of the last array are themselves arrays, so the buffer owns references: releasing
/// it and cloning it walk the elements, which a buffer of a fully unboxed element type never does.
const ARRAY_ACCESS_SOURCE: &str = r#"
    module Main;

    sum_array : Array I64 -> I64;
    sum_array = |arr| Iterator::range(0, arr.@size).fold(0, |i, acc| acc + arr.@(i));

    main : IO ();
    main = (
        let arr = Array::from_map(8, |i| i);
        let shared = arr;
        let written = arr.set(3, 100);
        let grown = written.push_back(9);
        let taken = grown.get_sub(1, 5);
        let nested = Array::from_map(4, |i| Array::fill(i + 1, i));
        let acted : Option (Array (Array I64)) =
            nested.act(2, |xs| if xs.@size == 0 { none() } else { some(xs.push_back(0)) });
        println $ (sum_array(shared) + sum_array(written) + sum_array(grown) + sum_array(taken)
                       + sum_array(acted.as_some.@(2))).to_string
    );
"#;

/// The IR the compiler writes for `ARRAY_ACCESS_SOURCE`, built once however many tests read it.
/// Building it is what a test here spends its time on, so the tests share one build.
fn array_access_ir() -> &'static str {
    static IR: OnceLock<String> = OnceLock::new();
    IR.get_or_init(|| generated_llvm_ir(ARRAY_ACCESS_SOURCE, "none"))
}

/// Every pointer the compiler computes into an object is computed within that object's allocation,
/// and the generated code says so.
///
/// A `getelementptr` without `inbounds` is one LLVM has to assume may leave the allocation it
/// started in. It then keeps the address arithmetic it would otherwise fold into an addressing
/// mode, and it cannot bound an index that a loop's bounds check reads, which is what decides
/// whether that loop has a trip count it can unroll by.
#[test]
pub fn test_every_pointer_into_an_object_is_computed_inside_it() {
    // The property is about what the compiler emits, so it is read before LLVM has run: an
    // optimized module also holds the pointer arithmetic LLVM itself introduced.
    let ir = array_access_ir();
    // A `getelementptr` instruction is one the program computes an address with. The same syntax
    // also appears as a constant expression that walks off a null pointer to name the size of a
    // type; that expression yields a number, and it lies outside every allocation.
    let geps = ir
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.starts_with('%') && line.contains(" = getelementptr"))
        .collect::<Vec<_>>();
    assert!(
        !geps.is_empty(),
        "reading and writing an array should compute pointers into it"
    );
    let geps_without_inbounds = geps
        .iter()
        .filter(|line| !line.contains("= getelementptr inbounds"))
        .collect::<Vec<_>>();
    assert!(
        geps_without_inbounds.is_empty(),
        "every `getelementptr` instruction should be `inbounds`, but {} of {} are not:\n{}",
        geps_without_inbounds.len(),
        geps.len(),
        geps_without_inbounds
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    );
}

/// The compiler tells LLVM that the block an allocator answers with is the caller's alone.
///
/// `malloc` hands back a block nothing else holds, and so does `realloc`: the block it was given is
/// over, whether the block moved or grew where it stood. Without `noalias` on the result, LLVM has
/// to assume a fresh buffer may be one a live pointer already names, and it keeps across every
/// allocation the loads it would otherwise forward — which is what an array's growth sits in the
/// middle of.
#[test]
pub fn test_the_allocators_say_their_result_is_the_callers_alone() {
    // The property is about what the compiler emits, so it is read before LLVM has run.
    let ir = array_access_ir();
    for allocator in [RUNTIME_MALLOC, RUNTIME_REALLOC] {
        let prefix = call_prefix(allocator);
        let calls = ir
            .lines()
            .filter(|line| call_arguments(line, &prefix).is_some())
            .count();
        assert!(
            calls > 0,
            "building and growing an array should reach `{}`, so that the declaration asserted on \
             below is one the program calls",
            allocator,
        );
        let declarations = ir
            .lines()
            .filter(|line| line.starts_with("declare ") && line.contains(&prefix))
            .collect::<Vec<_>>();
        assert!(
            !declarations.is_empty(),
            "the program calls `{}`, so a module has to declare it",
            allocator,
        );
        // A return attribute stands before the name, where a parameter attribute stands after it.
        let declarations_without_noalias = declarations
            .iter()
            .filter(|line| !line.split(&prefix).next().unwrap().contains("noalias"))
            .collect::<Vec<_>>();
        assert!(
            declarations_without_noalias.is_empty(),
            "every declaration of `{}` should give its result `noalias`, but {} of {} do not:\n{}",
            allocator,
            declarations_without_noalias.len(),
            declarations.len(),
            declarations_without_noalias
                .iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
}

/// The text a call to `callee` writes in front of the arguments it passes. A declaration of the
/// same function holds the name followed by its parameter types, so reading a line against this
/// finds the calls alone.
fn call_prefix(callee: &str) -> String {
    format!("@{}(", callee)
}

/// The arguments the call on `line` passes, where `line` is a call to the function whose
/// `call_prefix` is `prefix`, and `None` where it is not such a call.
fn call_arguments<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    if !line.contains("call ") {
        return None;
    }
    let (_, arguments) = line.split_once(prefix)?;
    Some(arguments)
}

/// The name of the first local value `text` names, as LLVM writes one: `%name`, or `%"name"` where
/// the name holds a character an identifier cannot.
///
/// # Examples
/// `first_local_value("(ptr %x, i64 3)")` is `Some("%x")`, and
/// `first_local_value("(ptr %\"a@b\", i64 3)")` is `Some("%\"a@b\"")`.
fn first_local_value(text: &str) -> Option<&str> {
    let start = text.find('%')?;
    let rest = &text[start + 1..];
    let length = match rest.strip_prefix('"') {
        Some(quoted) => quoted.find('"')? + 2,
        None => rest
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
            .unwrap_or(rest.len()),
    };
    Some(&text[start..start + 1 + length])
}

/// Whether `text` names the local value `name`, rather than one whose name begins with it.
fn names_local_value(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        text[at + name.len()..]
            .chars()
            .next()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '.'))
    })
}

/// Nothing reads the block a reallocation was given once the call has answered.
///
/// The `noalias` on `realloc`'s result says the block that comes back is the caller's alone, and
/// that holds only while the pointer handed in is dead from the call onward: a load through it,
/// which LLVM is then free to move across the call, would read a block the allocator has already
/// reused.
#[test]
pub fn test_nothing_reads_the_block_a_reallocation_was_given() {
    let ir = array_access_ir();
    let prefix = call_prefix(RUNTIME_REALLOC);
    let mut calls = 0;
    for body in llvm_function_bodies(ir, "") {
        let lines = body.lines().map(|line| line.trim()).collect::<Vec<_>>();
        for (i, line) in lines.iter().enumerate() {
            let Some(arguments) = call_arguments(line, &prefix) else {
                continue;
            };
            calls += 1;
            // The first value the call names is the block it is given, whatever attributes stand
            // beside it.
            let old_block = first_local_value(arguments).unwrap_or_else(|| {
                panic!("`realloc` takes a pointer as its first argument: {}", line)
            });
            let later_uses = lines[i + 1..]
                .iter()
                .filter(|later_line| names_local_value(later_line, old_block))
                .map(|later_line| later_line.to_string())
                .collect::<Vec<_>>();
            assert!(
                later_uses.is_empty(),
                "`{}` is the block handed to a reallocation, and it is named after the call:\n{}",
                old_block,
                later_uses.join("\n"),
            );
        }
    }
    assert!(
        calls > 0,
        "growing an array should reach `realloc`, so that this test has a call to read",
    );
}
