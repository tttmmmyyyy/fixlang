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

/// The IR the compiler writes for `ARRAY_ACCESS_SOURCE`, built once and shared by every test that
/// reads it. The build is what these tests spend their time on.
fn array_access_ir() -> &'static str {
    static IR: OnceLock<String> = OnceLock::new();
    IR.get_or_init(|| generated_llvm_ir(ARRAY_ACCESS_SOURCE, "none"))
}

/// Every pointer the compiler computes into an object is computed within that object's allocation,
/// and the generated code says so.
///
/// LLVM has to assume that a `getelementptr` without `inbounds` leaves the allocation it started
/// in. It then keeps address arithmetic it would otherwise fold into an addressing mode. It also
/// cannot bound the index a loop's bounds check reads, and that bound is what gives the loop a
/// trip count to unroll by.
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

/// The compiler marks the result of every allocator it declares `noalias`.
///
/// `malloc` returns a block that nothing else points to, and so does `realloc`: the pointer passed
/// in is dead from the call onward, whether the block moved or was resized in place. Without
/// `noalias` on the result, LLVM has to assume that a fresh block may be one a live pointer
/// already points to, so it cannot forward a load across an allocation. Growing an array runs
/// straight through such a call.
#[test]
pub fn test_the_allocators_say_their_result_is_the_callers_alone() {
    // The property is about what the compiler emits, so it is read before LLVM has run.
    let ir = array_access_ir();
    for allocator in [RUNTIME_MALLOC, RUNTIME_REALLOC] {
        let applied_name = name_before_arguments(allocator);
        let calls = ir
            .lines()
            .filter(|line| call_arguments(line, &applied_name).is_some())
            .count();
        assert!(
            calls > 0,
            "building and growing an array should reach `{}`, so that the declaration asserted on \
             below is one the program calls",
            allocator,
        );
        let declarations = ir
            .lines()
            .filter(|line| line.starts_with("declare ") && line.contains(&applied_name))
            .collect::<Vec<_>>();
        assert!(
            !declarations.is_empty(),
            "the program calls `{}`, so a module has to declare it",
            allocator,
        );
        // A return attribute stands before the name, and a parameter attribute after it.
        let declarations_without_noalias = declarations
            .iter()
            .filter(|line| {
                !line
                    .split(&applied_name)
                    .next()
                    .unwrap()
                    .contains("noalias")
            })
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

/// The text that joins `callee`'s name to its argument list, as LLVM writes it in a call and in a
/// declaration alike.
///
/// # Examples
/// `name_before_arguments("malloc")` is `"@malloc("`.
fn name_before_arguments(callee: &str) -> String {
    format!("@{}(", callee)
}

/// The text of the argument list that the call on `line` passes to the function named by
/// `name_before_arguments`.
fn call_arguments<'a>(line: &'a str, name_before_arguments: &str) -> Option<&'a str> {
    if !line.contains("call ") {
        return None;
    }
    let (_, arguments) = line.split_once(name_before_arguments)?;
    Some(arguments)
}

/// The name of the first local value in `text`, as LLVM writes one: `%name`, or `%"name"` when the
/// name holds a character that a plain identifier cannot.
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

/// Whether `text` names the local value `name`. The name has to stand whole: a longer name that
/// begins with `name` belongs to another value.
fn names_local_value(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        text[at + name.len()..]
            .chars()
            .next()
            .is_none_or(|c| !(c.is_alphanumeric() || c == '_' || c == '.'))
    })
}

/// The generated code stops reading the block it passes to `realloc` at the call.
///
/// The `noalias` on `realloc`'s result says that nothing else points to the block that comes back,
/// which holds only while the pointer passed in is dead from the call onward. LLVM is free to move
/// a load through that pointer across the call, where it would read a block the allocator has
/// already reused.
#[test]
pub fn test_nothing_reads_the_block_a_reallocation_was_given() {
    let ir = array_access_ir();
    let applied_name = name_before_arguments(RUNTIME_REALLOC);
    let mut calls = 0;
    for body in llvm_function_bodies(ir, "") {
        let lines = body.lines().map(|line| line.trim()).collect::<Vec<_>>();
        for (i, line) in lines.iter().enumerate() {
            let Some(arguments) = call_arguments(line, &applied_name) else {
                continue;
            };
            calls += 1;
            // The first value in the argument list is the block passed in, whatever attributes
            // stand beside it.
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
