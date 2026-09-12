use crate::env_vars::MAX_OPT_LEVEL_VAR;
use crate::tests::test_util::{
    build_run_and_read_rc_ir, emitted_llvm_ir, emitted_llvm_ir_modules, fix_build_source_command,
    llvm_function_bodies, EmittedIr,
};
use std::path::Path;
use tempfile::TempDir;

/// A program with two globals the compiler decides differently about.
///
/// `table` is read from a loop, so its accessor is shared and its initializer is long enough that
/// an inlining decided by size would leave the accessor a call. `read_once` is read from one place.
const TWO_GLOBALS_SOURCE: &str = r#"
    module Main;

    // The characters the table's initializer reads.
    alphabet : Array U8;
    alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".get_bytes;

    // A table built by a fold over another global.
    table : Array I64;
    table = Iterator::range(0, 256).fold(Array::fill(256, 3), |i, entries|
        entries.set(alphabet.@(i.bit_and(63)).to_I64, 3)
    );

    read_once : I64;
    read_once = Iterator::range(0, 256).fold(0, |i, acc| acc + i * 3);

    main : IO ();
    main = (
        let total = Iterator::range(0, 1000).fold(0, |i, total| total + table.@(i.bit_and(255)));
        println((total + read_once).to_string)
    );
"#;

/// The names the compiler gives the parts of `table`, as the emitted LLVM IR quotes them.
const TABLE_ACCESSOR: &str = "@\"Get#Main::table#";
const TABLE_INITIALIZER: &str = "@\"InitValue#Main::table#";
const TABLE_STORAGE: &str = "@\"GlobalVar#Main::table#";
const TABLE_FLAG: &str = "@\"InitFlag#Main::table#";

/// The same, for `read_once`.
const READ_ONCE_ACCESSOR: &str = "@\"Get#Main::read_once#";
const READ_ONCE_INITIALIZER: &str = "@\"InitValue#Main::read_once#";

/// A program whose global is read by the C function an `FFI_EXPORT` statement builds.
///
/// `counter` is read by `main` and by the exported C function. The exported function is emitted
/// after every symbol of the program is, so its read arrives after the rest.
const EXPORTED_GLOBAL_SOURCE: &str = r#"
    module Main;

    counter : I64;
    counter = Iterator::range(0, 256).fold(0, |i, acc| acc + i * 3);
    FFI_EXPORT[counter, c_counter];

    main : IO ();
    main = println(counter.to_string);
"#;

/// The name the compiler gives the accessor of `counter`, as the emitted LLVM IR quotes it.
const COUNTER_ACCESSOR: &str = "@\"Get#Main::counter#";

/// Build `source` with `--emit-llvm` in a directory of its own, and return that directory.
///
/// The build works at `-O max` whatever level the suite runs at, which is where a program is
/// divided into compilation units optimized one by one.
fn build_emitting_llvm_ir(source: &str) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    let build = fix_build_source_command(dir, source, "max")
        .env(MAX_OPT_LEVEL_VAR, "max")
        .arg("--emit-llvm")
        .output()
        .expect("Failed to execute fix build");
    assert!(
        build.status.success(),
        "the build should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );
    temp_dir
}

/// The generated module that defines the function whose name starts with `name`.
///
/// A compilation unit is optimized on its own, so what LLVM decides about a function is decided in
/// the module defining it: that module holds the function's callers, and it numbers the attribute
/// groups the function's own signature names.
fn module_defining(dir: &Path, name: &str) -> String {
    let mut modules = emitted_llvm_ir_modules(dir, EmittedIr::BeforeOptimization)
        .into_iter()
        .filter(|ir| {
            ir.lines()
                .any(|line| line.starts_with("define ") && line.contains(name))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        modules.len(),
        1,
        "one compilation unit should define `{}`, and {} do",
        name,
        modules.len()
    );
    modules.remove(0)
}

/// The single body of the function whose name starts with `name`, out of `ir`.
fn sole_body(ir: &str, name: &str) -> String {
    let bodies = llvm_function_bodies(ir, name);
    assert_eq!(bodies.len(), 1, "the build should emit one `{}`", name);
    bodies[0].clone()
}

/// Whether the function whose name starts with `name` carries the `noinline` attribute.
fn stays_out_of_its_callers(ir: &str, name: &str) -> bool {
    let signature = sole_body(ir, name)
        .lines()
        .next()
        .expect("a function body starts with its signature")
        .to_string();
    let Some(group) = signature
        .rsplit_once(')')
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .filter(|group| group.starts_with('#'))
    else {
        // A function carrying no attribute at all is printed without an attribute group, so it
        // carries no `noinline`.
        return false;
    };
    let group_line = format!("attributes {} =", group);
    let attributes = ir
        .lines()
        .find(|line| line.starts_with(&group_line))
        .unwrap_or_else(|| panic!("the emitted IR should define `{}`", group_line));
    attributes.contains("noinline")
}

/// How many times `ir` calls the function whose name starts with `name`.
fn count_calls_to(ir: &str, name: &str) -> usize {
    ir.lines()
        .filter(|line| line.contains("call") && line.contains(name))
        .count()
}

/// The initializer of a global read from many places is a function of its own, which is left there.
///
/// Reading a global tests an initialization flag and loads the storage — four instructions on
/// x86-64. An accessor holding the initializer as well is as large as the initializer, and an
/// inlining decided by size leaves it a call: one per read, with the flag test and the load stuck
/// in the loop behind it. An accessor whose initializer is elsewhere is small enough to be placed
/// at every reader without being asked.
///
/// The accessor is the initializer's only caller, so an inliner reaching the initializer would fold
/// it straight back in.
#[test]
pub fn test_the_initializer_of_a_shared_global_sits_outside_the_accessor() {
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
    let ir = module_defining(temp_dir.path(), TABLE_INITIALIZER);

    // The readers the decision rests on.
    assert!(
        count_calls_to(&ir, TABLE_ACCESSOR) > 1,
        "the program should read `table` from more than one place"
    );

    let accessor = sole_body(&ir, TABLE_ACCESSOR);
    assert!(
        accessor.contains(TABLE_INITIALIZER),
        "the accessor should call the initializer, and its body is:\n{}",
        accessor
    );
    assert!(
        stays_out_of_its_callers(&ir, TABLE_INITIALIZER),
        "the initializer of `table` should stay out of the accessor"
    );
}

/// The initializer of a global read from one place is left in the accessor.
///
/// It has one place to be either way, and the place that costs nothing is the one the reader can
/// see: what the initializer knows — the length of an array, the shape of a structure — reaches the
/// code that reads the global, where it takes bounds checks out of loops.
#[test]
pub fn test_the_initializer_of_a_global_read_once_stays_where_its_reader_sees_it() {
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
    let ir = module_defining(temp_dir.path(), READ_ONCE_INITIALIZER);

    // The reader the decision rests on.
    assert_eq!(
        count_calls_to(&ir, READ_ONCE_ACCESSOR),
        1,
        "the program should read `read_once` from one place"
    );

    assert!(
        !stays_out_of_its_callers(&ir, READ_ONCE_INITIALIZER),
        "the initializer of `read_once` should be free to join the accessor"
    );
}

/// A reader of a global sees every write to the global's storage and flag.
///
/// This is what lets a reader lift its reads out of a loop. The reads are of two module-level
/// variables, and the call to the initializer sits between them in the reader's loop: a reader that
/// could not see the writes would have to assume that call performs them, and read the flag and the
/// storage again on every turn.
#[test]
pub fn test_a_reader_of_a_global_sees_every_write_to_it() {
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
    let ir = module_defining(temp_dir.path(), TABLE_INITIALIZER);
    let accessor = sole_body(&ir, TABLE_ACCESSOR);

    for variable in [TABLE_STORAGE, TABLE_FLAG] {
        let writes = |text: &str| {
            text.lines()
                .filter(|line| line.trim_start().starts_with("store") && line.contains(variable))
                .count()
        };
        let in_module = writes(&ir);
        assert!(in_module > 0, "the program should write `{}`", variable);
        assert_eq!(
            writes(&accessor),
            in_module,
            "every write to `{}` should be in the accessor, and {} of the {} are:\n{}",
            variable,
            writes(&accessor),
            in_module,
            accessor
        );
    }
}

/// A global read inside a loop is read without a call.
///
/// A read of `table` goes through its accessor, twice per element: the bounds check reads the
/// length and the element read reads the pointer. The property is read off the emitted LLVM IR: it
/// is about the code the build emits, and a program cannot observe a call it does not make.
///
/// This is the requirement. `test_the_initializer_of_a_shared_global_sits_outside_the_accessor` and
/// `test_a_reader_of_a_global_sees_every_write_to_it` pin the two properties this compiler reaches
/// it by, and another mechanism would keep this test green and turn those red.
#[test]
pub fn test_reading_a_global_in_a_loop_costs_no_call() {
    let temp_dir = build_emitting_llvm_ir(TWO_GLOBALS_SOURCE);
    let dir = temp_dir.path();

    // The generated IR holds the calls the optimized IR is then checked for: a program reaching
    // `table` some other way would satisfy that check for free.
    let generated_ir = emitted_llvm_ir(dir, EmittedIr::BeforeOptimization);
    assert!(
        count_calls_to(&generated_ir, TABLE_ACCESSOR) > 0,
        "the program should read `table` through its accessor"
    );

    let optimized_ir = emitted_llvm_ir(dir, EmittedIr::AfterOptimization);
    let remaining_calls: Vec<_> = optimized_ir
        .lines()
        .filter(|line| line.contains("call") && line.contains(TABLE_ACCESSOR))
        .collect();
    assert!(
        remaining_calls.is_empty(),
        "reading `table` should cost no call, and the optimized IR holds {}:\n{}",
        remaining_calls.len(),
        remaining_calls.join("\n")
    );
}

/// A global read from a compilation unit that does not own it is read without a call.
///
/// `counter` is read from two units: `Main::main` reads it in the unit its own code is compiled in,
/// and the C function `FFI_EXPORT` builds reads it in the unit that carries the exported C
/// functions and the program's entry point, which holds no symbol of the program. The reading unit
/// is given a copy of the accessor — the flag test and the load, with the initializer left where
/// the owning unit computes it — so a read across a unit boundary costs what a read within one
/// does.
#[test]
pub fn test_a_global_read_from_another_unit_costs_no_call() {
    let temp_dir = build_emitting_llvm_ir(EXPORTED_GLOBAL_SOURCE);
    let dir = temp_dir.path();

    // The readers the property is about: a unit reading `counter` without a reader in another unit
    // would satisfy the check below for free.
    let reading_units = emitted_llvm_ir_modules(dir, EmittedIr::BeforeOptimization)
        .iter()
        .filter(|ir| count_calls_to(ir, COUNTER_ACCESSOR) > 0)
        .count();
    assert_eq!(
        reading_units, 2,
        "`counter` should be read from the unit holding `Main::main` and from the unit holding the \
         exported C function"
    );

    let optimized_ir = emitted_llvm_ir(dir, EmittedIr::AfterOptimization);
    let remaining_calls: Vec<_> = optimized_ir
        .lines()
        .filter(|line| line.contains("call") && line.contains(COUNTER_ACCESSOR))
        .collect();
    assert!(
        remaining_calls.is_empty(),
        "reading `counter` should cost no call, and the optimized IR holds {}:\n{}",
        remaining_calls.len(),
        remaining_calls.join("\n")
    );
}

/// A program that names one global string from three places.
///
/// Building the literal allocates a buffer and copies the bytes into it, so a global whose body is
/// a literal costs one allocation where it is named once and three where it is named three times.
const STRING_GLOBAL_SOURCE: &str = r#"
    module Main;

    greeting : String;
    greeting = "hello";

    first : I64;
    first = greeting.get_bytes.get_size;

    second : I64;
    second = greeting.get_bytes.@(0).to_I64;

    third : I64;
    third = greeting.get_bytes.@(1).to_I64;

    main : IO ();
    main = println((first + second + third).to_string);
"#;

/// The literal's construction, as the RC IR dump names it.
const GREETING_BUF: &str = "string_buf(\"hello\")";

/// The global the program names, as the RC IR dump spells it, before the suffix the compiler adds.
const GREETING: &str = "Main::greeting";

#[test]
fn test_a_global_string_is_built_once_however_many_names_it() {
    let dump = build_run_and_read_rc_ir(
        STRING_GLOBAL_SOURCE,
        "max",
        "211",
        "a global string named from three places",
    );

    // The places the property is about: the value reaches a reader as a name of the global or as a
    // copy of the construction, so this counts the readers either way, and falls only where a
    // reader stopped reading.
    let places = dump
        .lines()
        .filter(|line| line.contains(GREETING) && !line.starts_with("global "))
        .count()
        + dump.matches(GREETING_BUF).count();
    assert!(
        places > 1,
        "the program should hold `greeting` in more than one place, and it holds it in {}:\n{}",
        places,
        dump
    );

    let built = dump.matches(GREETING_BUF).count();
    assert_eq!(
        built, 1,
        "the literal of a global is built {} times, once for the global and once more wherever \
         its body was put; the dump is:\n{}",
        built, dump
    );
}

/// A program that names two globals whose bodies are literals from two places each.
///
/// An integer and a floating-point literal evaluate to a value held in a register, so a copy of the
/// literal where the global is named costs what naming it costs.
const SCALAR_GLOBAL_SOURCE: &str = r#"
    module Main;

    answer : I64;
    answer = 42;

    ratio : F64;
    ratio = 1.5;

    doubled : I64;
    doubled = answer * 2;

    raised : I64;
    raised = answer + 1;

    scaled : F64;
    scaled = ratio * 2.0;

    halved : F64;
    halved = ratio / 2.0;

    main : IO ();
    main = println((doubled + raised + (scaled + halved).to_I64).to_string);
"#;

/// The globals of `SCALAR_GLOBAL_SOURCE` whose bodies are literals, each beside the construction of
/// its literal as the RC IR dump names it.
const SCALAR_GLOBALS: [(&str, &str); 2] = [("Main::answer", "int(42)"), ("Main::ratio", "float(1.5)")];

/// A global whose body is a scalar literal is put at every name.
///
/// Its value is held in a register, so a copy of the literal at each name costs what the name
/// costs, and the global it would otherwise stand in puts an initialization flag and a load in
/// front of every read.
#[test]
fn test_a_global_scalar_literal_is_put_at_every_name() {
    let dump = build_run_and_read_rc_ir(
        SCALAR_GLOBAL_SOURCE,
        "max",
        "130",
        "two global scalar literals named from two places each",
    );

    for (global, literal) in SCALAR_GLOBALS {
        let built = dump.matches(literal).count();
        assert!(
            built > 1,
            "`{}` should be at each of the two names of `{}`, and the dump holds {}:\n{}",
            literal,
            global,
            built,
            dump
        );

        let standing: Vec<_> = dump
            .lines()
            .filter(|line| line.starts_with("global ") && line.contains(global))
            .collect();
        assert!(
            standing.is_empty(),
            "`{}` should cost no global of its own, and the dump opens {}:\n{}",
            global,
            standing.join("\n"),
            dump
        );
    }
}

/// A program whose `main` performs three IO actions.
const IO_ACTIONS_SOURCE: &str = r#"
    module Main;

    main : IO ();
    main = (
        println("one");;
        println("two");;
        println("three")
    );
"#;

/// The making of an `IOState`, and the `Std` global that makes one, as the RC IR dump names them.
const IOSTATE_CREATE: &str = "iostate_create";
const IOSTATE_GLOBAL: &str = "Std::IO::IOState::_unsafe_create";

/// An `IOState` is made where it is used.
///
/// It is an unboxed value with no field, so making one allocates nothing and a copy of the making
/// at each name costs nothing. A global standing for it would put an initialization flag and a load
/// in front of the IO actions the program performs.
#[test]
fn test_an_iostate_is_made_where_it_is_used() {
    let dump = build_run_and_read_rc_ir(
        IO_ACTIONS_SOURCE,
        "max",
        "one\ntwo\nthree",
        "a program performing three IO actions",
    );

    // The making the property is about: a program that makes no `IOState` keeps none in a global.
    assert!(
        dump.contains(IOSTATE_CREATE),
        "the program should make an `IOState`, and the dump is:\n{}",
        dump
    );

    let standing: Vec<_> = dump
        .lines()
        .filter(|line| line.starts_with("global ") && line.contains(IOSTATE_GLOBAL))
        .collect();
    assert!(
        standing.is_empty(),
        "an `IOState` should cost no global of its own, and the dump opens {}:\n{}",
        standing.join("\n"),
        dump
    );
}
