use crate::configuration::{Configuration, DeprecationMode};
use crate::fixstd::runtime::{RUNTIME_MALLOC, RUNTIME_REALLOC};
use crate::tests::test_util::{
    build_run_and_read_rc_ir, first_local_value, generated_llvm_ir, llvm_function_bodies,
    names_local_value, rc_ir_function_bodies, run_source_assert_failed, test_source,
};
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

/// A program that offsets a pointer each way and takes the distance between two pointers. The
/// arithmetic runs on `nullptr`, where the source states every answer outright, and on the elements
/// of an array, where it runs on an address the program owns.
const POINTER_ARITHMETIC_SOURCE: &str = r#"
    module Main;

    main : IO ();
    main = (
        assert_eq(|_|"an offset of zero", nullptr.add_offset(0).to_string, "0000000000000000");;
        assert_eq(|_|"an offset forward", nullptr.add_offset(16).to_string, "0000000000000010");;
        assert_eq(|_|"an offset and its opposite", nullptr.add_offset(16).add_offset(-16).to_string, "0000000000000000");;
        assert_eq(|_|"a distance forward", nullptr.add_offset(16).offset_from(nullptr), 16);;
        assert_eq(|_|"a distance backward", nullptr.offset_from(nullptr.add_offset(16)), -16);;
        assert_eq(|_|"an offset that wraps at the top", nullptr.add_offset(I64::maximum).add_offset(1).to_string, "8000000000000000");;
        assert_eq(|_|"an offset that wraps at the bottom", nullptr.add_offset(I64::minimum).add_offset(-1).to_string, "7fffffffffffffff");;
        assert_eq(|_|"a distance that wraps", nullptr.add_offset(I64::maximum).offset_from(nullptr.add_offset(I64::minimum)), -1);;

        let arr = Array::from_map(4, |i| i * 100);
        let distance = arr.borrow_elements(|elements| elements.add_offset(24).offset_from(elements));
        assert_eq(|_|"a distance within a buffer", distance, 24);;
        let no_distance = arr.borrow_elements(|elements| elements.offset_from(elements));
        assert_eq(|_|"a pointer is no distance from itself", no_distance, 0);;

        println("pointer arithmetic answered")
    );
"#;

/// The RC IR of `POINTER_ARITHMETIC_SOURCE` built at `opt_level`, read once the program it produces
/// has run and has given the answers the source states.
fn pointer_arithmetic_rc_ir(opt_level: &str) -> String {
    build_run_and_read_rc_ir(
        POINTER_ARITHMETIC_SOURCE,
        opt_level,
        "pointer arithmetic answered",
        "a program that offsets a pointer and takes the distance between two",
    )
}

/// A pointer is a number, and the two primitives are arithmetic on that number:
/// `Std::Ptr::add_offset` counts bytes from the address it is given, each way, and
/// `Std::Ptr::offset_from` answers with the signed count between two addresses. Both wrap at the
/// width of the address.
#[test]
pub fn test_pointer_arithmetic_counts_bytes_each_way() {
    test_source(POINTER_ARITHMETIC_SOURCE, Configuration::develop_mode());
}

/// Each pointer-arithmetic primitive computes its result with an operation of the compiler, so the
/// instructions stand in whichever unit names the primitive.
///
/// A runtime function's body is written into the main unit alone, so a primitive that calls one
/// leaves every other unit a call across a module boundary. Behind that boundary stand one to three
/// instructions, and an optimizer has to fold the surrounding address arithmetic through it.
#[test]
pub fn test_each_pointer_primitive_computes_its_result_itself() {
    let dump = pointer_arithmetic_rc_ir("none");
    for (primitive, operation) in [
        ("Std::Ptr::add_offset", "= add_offset("),
        ("Std::Ptr::offset_from", "= offset_from("),
    ] {
        let body = rc_ir_function_bodies(&dump, primitive).join("\n");
        assert!(
            !body.is_empty(),
            "the program reaches `{}`, so the dump holds its body",
            primitive,
        );
        assert!(
            body.contains(operation),
            "`{}` should compute its result with an operation of the compiler:\n{}",
            primitive,
            body,
        );
    }
}

/// The inliner carries a copy of each primitive's body to every place that names it, so at `max`,
/// the level a program is normally built at, the arithmetic stands in the function that writes it.
///
/// Inlining copies whatever the body holds: a body holding a foreign call is carried just the same,
/// and then every copy of it calls across a module boundary.
#[test]
pub fn test_the_inliner_carries_the_pointer_arithmetic_into_its_caller() {
    let dump = pointer_arithmetic_rc_ir("max");
    // A closure lifted out of `main` keeps its name, so this is the whole of what `main` writes.
    let written_in_main = rc_ir_function_bodies(&dump, "Main::main").join("\n");
    assert!(
        !written_in_main.is_empty(),
        "the program has an entry point, so the dump holds its body",
    );
    for operation in ["= add_offset(", "= offset_from("] {
        assert!(
            written_in_main.contains(operation),
            "`{}` should stand in what `main` writes:\n{}",
            operation,
            written_in_main,
        );
    }
}

/// `Std::Ptr::add_offset` counts on the integer address, which is what lets the address it answers
/// with lie outside the object the pointer points into.
///
/// `getelementptr inbounds` answers `poison` the moment the address it computes leaves the
/// allocation it started in, and `test_every_pointer_into_an_object_is_computed_inside_it` requires
/// every `getelementptr` the compiler emits to be `inbounds`. A primitive promising arithmetic on
/// the address therefore computes on the integer address, and the generated code says so. The
/// answers alone do not: LLVM leaves a `poison` address reaching `sprintf` where it was, so the
/// program prints the same text either way.
#[test]
pub fn test_the_offset_is_counted_on_the_integer_address() {
    // The property is about what the compiler emits, so it is read before LLVM has run.
    let ir = generated_llvm_ir(POINTER_ARITHMETIC_SOURCE, "none");
    let body = llvm_function_bodies(&ir, "Std::Ptr::add_offset").join("\n");
    assert!(
        !body.is_empty(),
        "the program reaches `Std::Ptr::add_offset`, so the module holds its body",
    );
    for instruction in [" = ptrtoint ", " = inttoptr "] {
        assert!(
            body.contains(instruction),
            "`Std::Ptr::add_offset` should compute its result with `{}`:\n{}",
            instruction.trim(),
            body,
        );
    }
}

/// `Std::Ptr::subtract_ptr` answers what `Std::Ptr::offset_from` answers, and naming it is reported
/// with the message its `DEPRECATED` pragma carries.
#[test]
pub fn test_the_deprecated_name_of_the_pointer_difference_answers_the_same() {
    const SOURCE: &str = r#"
        module Main;

        main : IO ();
        main = (
            let arr = Array::from_map(4, |i| i * 100);
            let (canonical, deprecated) = arr.borrow_elements(|elements| (
                elements.add_offset(24).offset_from(elements),
                elements.add_offset(24).subtract_ptr(elements)
            ));
            assert_eq(|_|"the deprecated name answers the same", deprecated, canonical);;
            assert_eq(|_|"the deprecated name counts backward", nullptr.subtract_ptr(nullptr.add_offset(16)), -16);;
            pure()
        );
    "#;
    test_source(SOURCE, Configuration::develop_mode());

    let mut config = Configuration::develop_mode();
    config.deprecation_mode = DeprecationMode::Deny;
    let report = run_source_assert_failed(SOURCE, config);
    assert!(
        report.contains("Use `Std::Ptr::offset_from` instead."),
        "naming `subtract_ptr` should be reported with the message its pragma carries:\n{}",
        report,
    );
}
