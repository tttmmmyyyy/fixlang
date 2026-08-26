//! What dividing a program into compilation units gives it: a build after an edit that regenerates
//! only what the edit reaches, and units that keep their code to themselves wherever the rest of
//! the program does not name it.

use crate::misc::Set;
use crate::tests::test_util::{
    emitted_llvm_ir_modules, fix_build_source_command, fix_command_at_opt_level, EmittedIr,
};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// A program that exports a C function, so that a unit has a C name to publish beside the entry
/// point.
const EXPORTING_SOURCE: &str = r#"
    module Main;

    triple : I64 -> I64;
    triple = |x| x * 3;
    FFI_EXPORT[triple, fixtest_triple];

    main : IO ();
    main = println(triple(14).to_string);
"#;

/// Build `source` at `-O max` with `--emit-llvm` in a directory of its own, and return that
/// directory.
fn build_at_max_emitting_llvm_ir(source: &str) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let build = fix_build_source_command(temp_dir.path(), source, "max")
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

/// The name an LLVM `define` or `declare` line gives its function.
fn function_name(line: &str) -> Option<String> {
    let after_at = line.split_once('@')?.1;
    let name = match after_at.strip_prefix('"') {
        Some(quoted) => quoted.split_once('"')?.0,
        None => after_at.split('(').next()?,
    };
    Some(name.to_string())
}

/// The names of the functions `ir` defines under a linkage that publishes them to the linker.
///
/// A definition LLVM keeps to its module is printed with `internal` or `private` between
/// `define` and the return type; one without either is reachable from outside the module.
fn published_function_names(ir: &str) -> Set<String> {
    ir.lines()
        .filter(|line| line.starts_with("define "))
        .filter(|line| {
            let head = line
                .split('@')
                .next()
                .expect("a line has a part before its first `@`");
            !head.contains(" internal ") && !head.contains(" private ")
        })
        .filter_map(function_name)
        .collect()
}

/// The names of the functions `ir` declares without defining: the ones it calls in another module.
fn declared_function_names(ir: &str) -> Set<String> {
    ir.lines()
        .filter(|line| line.starts_with("declare "))
        .filter_map(function_name)
        .collect()
}

/// The name an LLVM global-variable line gives its variable.
fn global_variable_name(line: &str) -> Option<String> {
    let after_at = line.strip_prefix('@')?;
    let name = match after_at.strip_prefix('"') {
        Some(quoted) => quoted.split_once('"')?.0,
        None => after_at.split(' ').next()?,
    };
    Some(name.to_string())
}

/// The names of the global variables `ir` defines storage for, and the ones it declares as storage
/// another module defines.
///
/// A global variable is printed as its name, `=`, the linkage where it carries one, and either the
/// value it is initialized to or the word `external`.
fn defined_and_declared_global_variables(ir: &str) -> (Set<String>, Set<String>) {
    let mut defined = Set::default();
    let mut declared = Set::default();
    for line in ir.lines().filter(|line| line.starts_with('@')) {
        let Some((_, definition)) = line.split_once(" = ") else {
            continue;
        };
        let Some(name) = global_variable_name(line) else {
            continue;
        };
        if definition.starts_with("external ") {
            declared.insert(name);
        } else {
            defined.insert(name);
        }
    }
    (defined, declared)
}

/// A project of two modules: `Worker`, which holds a table and a function reading one element of
/// it, and `Main`, which calls that function. The function is small enough for the unit holding
/// `Main` to take a copy of, and that copy is then the only code the program reads the table from.
fn write_one_reader_global_project(dir: &Path) {
    fs::write(
        dir.join("fixproj.toml"),
        r#"[general]
name = "one-reader-global"
version = "0.1.0"
[build]
files = ["main.fix", "worker.fix"]
"#,
    )
    .expect("Failed to write the project file");
    fs::write(
        dir.join("worker.fix"),
        r#"module Worker;

table : Array I64;
table = Array::from_map(64, |i| i * i + 1);

lookup : I64 -> I64;
lookup = |i| table.@(i % 64);
"#,
    )
    .expect("Failed to write the worker module");
    fs::write(
        dir.join("main.fix"),
        r#"module Main;

import Worker::{lookup};

main : IO ();
main = println(lookup(1000).to_string);
"#,
    )
    .expect("Failed to write the main module");
}

/// Build the project in `dir` at `-O max`, emitting the LLVM IR of every compilation unit.
fn build_at_max_emitting_llvm_ir_in(dir: &Path) {
    let build = fix_command_at_opt_level("build", "max")
        .arg("--emit-llvm")
        .current_dir(dir)
        .output()
        .expect("Failed to execute fix build");
    assert!(
        build.status.success(),
        "the build should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );
}

/// A global one compilation unit reads is that unit's own, storage and all.
///
/// Storage published to the linker is storage LLVM has to assume a store anywhere in the unit
/// writes: the test of the initialization flag and the load of the storage stay inside every loop
/// that reads the global, and so do the bounds checks the lifted load would have taken out. So a
/// global the program reads from one unit is held by that unit, and none of the names it is built
/// from is published.
#[test]
fn test_a_global_one_unit_reads_is_that_units_own() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    write_one_reader_global_project(temp_dir.path());
    build_at_max_emitting_llvm_ir_in(temp_dir.path());
    let modules = emitted_llvm_ir_modules(temp_dir.path(), EmittedIr::BeforeOptimization);

    let storage_of_the_table = |name: &String| name.starts_with("GlobalVar#Worker::table");
    let mut units_holding_the_storage = 0;
    for module in &modules {
        let (defined, declared) = defined_and_declared_global_variables(module);
        if defined.iter().any(storage_of_the_table) {
            units_holding_the_storage += 1;
        }
        for name in declared.iter().filter(|name| storage_of_the_table(name)) {
            panic!(
                "`{}` is declared, so a unit that does not hold it reads it",
                name
            );
        }
    }
    assert_eq!(
        units_holding_the_storage, 1,
        "one unit should hold the storage of `Worker::table`, and {} do",
        units_holding_the_storage
    );
}

/// A compilation unit publishes a symbol of the program only where another unit names it.
///
/// A function LLVM must assume something outside the module calls is one it can neither delete
/// after inlining it into every call it sees nor specialize to the arguments those calls pass. A
/// unit that published every symbol it holds would lose both for the whole program, so what the
/// rest of the program does not name stays inside the unit that holds it, and a unit optimized on
/// its own reaches what one compiled in a single piece does.
///
/// The C world enters the program through names of its own — the entry point, the function an
/// `FFI_EXPORT` statement builds, the runtime's — and those carry no `#`, which is what separates
/// them here from the symbols the compiler mangles.
#[test]
fn test_a_unit_publishes_a_symbol_only_where_another_unit_names_it() {
    let temp_dir = build_at_max_emitting_llvm_ir(EXPORTING_SOURCE);
    let modules = emitted_llvm_ir_modules(temp_dir.path(), EmittedIr::BeforeOptimization);
    let published: Vec<Set<String>> = modules
        .iter()
        .map(|ir| published_function_names(ir))
        .collect();
    let declared: Vec<Set<String>> = modules
        .iter()
        .map(|ir| declared_function_names(ir))
        .collect();

    // The units of this program do publish symbols to one another, so the rule below has something
    // to hold of.
    assert!(
        published.iter().flatten().any(|name| name.contains('#')),
        "the units of this program should publish symbols to one another"
    );

    for (index, names) in published.iter().enumerate() {
        for name in names.iter().filter(|name| name.contains('#')) {
            assert!(
                declared
                    .iter()
                    .enumerate()
                    .any(|(other, declared_names)| other != index && declared_names.contains(name)),
                "no other unit names `{}`, which unit {} publishes",
                name,
                index
            );
        }
    }

    // The C world's own names are published, whatever the rule above does with the symbols.
    let published_anywhere: Set<String> = published.iter().flatten().cloned().collect();
    for name in ["main", "fixtest_triple"] {
        assert!(
            published_anywhere.contains(name),
            "the program should publish `{}` for the C world to enter through",
            name
        );
    }
}

/// A project of two modules: `Worker`, whose globals are compiled from `Std` alone, and `Main`,
/// which calls into it. `argument` is what `Main` passes, and changing it is the edit.
fn write_two_module_project(dir: &Path, argument: i64) {
    fs::write(
        dir.join("fixproj.toml"),
        r#"[general]
name = "compilation-units"
version = "0.1.0"
[build]
files = ["main.fix", "worker.fix"]
"#,
    )
    .expect("Failed to write the project file");
    fs::write(
        dir.join("worker.fix"),
        r#"module Worker;

scaled : I64 -> Array I64;
scaled = |n| Array::from_map(n, |i| (n - i) * 7 + i * i);

sorted : I64 -> Array I64;
sorted = |n| scaled(n).sort_by(|(a, b)| a < b);

digits : I64 -> Array I64;
digits = |n| loop(([], n), |(acc, n)|
    if n == 0 { break $ acc } else { continue $ (acc.push_back(n % 10), n / 10) }
);

digit_sum : I64 -> I64;
digit_sum = |n| digits(n).to_iter.fold(0, Add::add);

spread : I64 -> I64;
spread = |n| (
    let arr = sorted(n);
    arr.@(arr.@size - 1) - arr.@(0)
);

folded : I64 -> I64;
folded = |n| Iterator::range(0, n).map(|i| digit_sum(i * 31 + 7)).fold(0, Add::add);

report : I64 -> String;
report = |n| (spread(n) + folded(n)).to_string + ":" + sorted(n).@size.to_string;
"#,
    )
    .expect("Failed to write the worker module");
    fs::write(
        dir.join("main.fix"),
        format!(
            r#"module Main;

import Worker;

main : IO ();
main = println(Worker::report({}));
"#,
            argument
        ),
    )
    .expect("Failed to write the main module");
}

/// Build the project in `dir` at `-O max`, with `cu_size` entries to a compilation unit, and
/// return how many compilation units the build generated and how many it took from the cache.
///
/// The two counts are read off what the build reports under `--verbose`, which is where it says
/// what it did with each unit.
fn build_at_max_in(dir: &Path, cu_size: &str) -> (usize, usize) {
    let build = fix_command_at_opt_level("build", "max")
        .args(["--cu-size", cu_size, "--verbose"])
        .current_dir(dir)
        .output()
        .expect("Failed to execute fix build");
    assert!(
        build.status.success(),
        "the build should succeed.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr),
    );
    let reported = String::from_utf8_lossy(&build.stdout).to_string()
        + &String::from_utf8_lossy(&build.stderr);
    let count = |phrase: &str| reported.matches(phrase).count();
    let (generated, cached) = (
        count("Generating code for"),
        count("Skipping generation of code for"),
    );
    assert!(
        generated + cached > 0,
        "the build should report what it did with each compilation unit:\n{}",
        reported
    );
    (generated, cached)
}

/// A build after an edit generates only the compilation units the edit reaches.
///
/// This is what dividing the program is for. A unit is cached under a digest of the code it
/// generates, so an edit to `Main` leaves the units holding `Worker`'s globals — whose code `Std`
/// alone decides — where they are, and the second build takes their code from the first. A program
/// compiled in one piece is one unit, and every build of it generates all of it.
#[test]
fn test_an_edit_regenerates_only_the_units_it_reaches() {
    const CU_SIZE: &str = "2";
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();

    write_two_module_project(dir, 120);
    let (unit_count, _) = build_at_max_in(dir, CU_SIZE);
    assert!(
        unit_count >= 10,
        "{} symbols to a unit should divide this program into several units, and it made {}",
        CU_SIZE,
        unit_count
    );

    write_two_module_project(dir, 122);
    let (generated, cached) = build_at_max_in(dir, CU_SIZE);
    assert!(
        cached * 3 >= generated + cached,
        "the edit reaches `Main` alone, so the second build should take at least a third of its \
         {} units from the first, and it generated {} of them again",
        generated + cached,
        generated
    );
}

/// `--cu-size inf` puts the whole program in one unit beside the main unit.
///
/// The main unit holds no entry of the program: it builds the C entry point and the exported C
/// functions. So a program asked for one unit generates two, and the program it generates runs.
#[test]
fn test_cu_size_inf_puts_the_whole_program_in_one_unit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();

    write_two_module_project(dir, 120);
    let (generated, cached) = build_at_max_in(dir, "inf");
    assert_eq!(
        (generated, cached),
        (2, 0),
        "the whole program is one unit, and the main unit is the other"
    );
    assert!(
        !printed_by_the_program(dir).is_empty(),
        "the program built as one unit should run and print what it computes"
    );
}

/// The `cu_size` field of the project file decides the size of a compilation unit, and `--cu-size`
/// on the command line decides it over the field.
///
/// The field is what a project says about itself, and the option is what one build of it asks for,
/// so the option is the one that answers where both are given.
#[test]
fn test_the_project_file_sets_the_unit_size_and_the_option_overrides_it() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();
    write_two_module_project(dir, 120);
    let project_file = dir.join("fixproj.toml");
    let text = fs::read_to_string(&project_file).expect("Failed to read the project file");
    fs::write(&project_file, text.replace("[build]\n", "[build]\ncu_size = 2\n"))
        .expect("Failed to write the project file");

    // What the project file asks for, with no `--cu-size` on the command line.
    let build = fix_command_at_opt_level("build", "max")
        .arg("--verbose")
        .current_dir(dir)
        .output()
        .expect("Failed to execute fix build");
    let reported = String::from_utf8_lossy(&build.stdout).to_string()
        + &String::from_utf8_lossy(&build.stderr);
    assert!(
        build.status.success(),
        "the build reading `cu_size` from the project file failed:\n{}",
        reported
    );
    let from_the_field = reported.matches("Generating code for").count();
    assert!(
        from_the_field >= 10,
        "the project file asks for 2 entries to a unit, which divides this program into several \
         units, and the build made {}:\n{}",
        from_the_field,
        reported
    );

    // What the option asks for, over what the project file says.
    write_two_module_project(dir, 120);
    let text = fs::read_to_string(&project_file).expect("Failed to read the project file");
    fs::write(&project_file, text.replace("[build]\n", "[build]\ncu_size = 2\n"))
        .expect("Failed to write the project file");
    let (generated, cached) = build_at_max_in(dir, "inf");
    assert_eq!(
        generated + cached,
        2,
        "`--cu-size inf` decides over the `cu_size = 2` the project file gives, so the build is the \
         whole program and the main unit; it made {} and took {} from the builds before it",
        generated,
        cached
    );
}

/// An edit that writes no code regenerates no compilation unit.
///
/// A comment is compiled into nothing, and the code every unit of the program generates is what it
/// was, so every unit keeps the object file the build before the comment compiled it into. What
/// moves is where the code after the comment is written, which a build asked for no debug
/// information puts nowhere.
#[test]
fn test_a_comment_added_to_a_module_regenerates_no_unit() {
    const CU_SIZE: &str = "2";
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir = temp_dir.path();

    write_two_module_project(dir, 120);
    let (unit_count, _) = build_at_max_in(dir, CU_SIZE);

    let worker = dir.join("worker.fix");
    let commented = format!(
        "// a comment the compiler generates no code from\n{}",
        fs::read_to_string(&worker).expect("Failed to read the worker module")
    );
    fs::write(&worker, commented).expect("Failed to write the worker module");

    let (generated, _) = build_at_max_in(dir, CU_SIZE);
    assert_eq!(
        generated, 0,
        "the comment writes no code, so the build after it should take all {} units from the \
         first, and it generated {} of them again",
        unit_count, generated
    );
}

/// A project of three modules: `Types`, which declares the types; `Lib`, which reads the second
/// field of one of them; and `Main`, which builds a value and prints what `Lib` reads out of it.
///
/// `Lib` names `Inner` nowhere, and `T` holds an `Inner` ahead of the field `Lib` reads, so a field
/// added to `Inner` moves that field without moving any part of `Lib`'s code.
///
/// # Arguments
/// * `inner_carries_two_fields` — whether `Inner` holds a second field, which is the edit.
fn write_nested_type_project(dir: &Path, inner_carries_two_fields: bool) {
    fs::write(
        dir.join("fixproj.toml"),
        r#"[general]
name = "nested-type"
version = "0.1.0"
[build]
files = ["main.fix", "lib.fix", "types.fix"]
"#,
    )
    .expect("Failed to write the project file");

    let (inner_fields, inner_literal) = if inner_carries_two_fields {
        ("{x : I64, y : I64}", "Inner{x : 100, y : 200}")
    } else {
        ("{x : I64}", "Inner{x : 100}")
    };
    fs::write(
        dir.join("types.fix"),
        format!(
            "module Types;\n\ntype Inner = unbox struct {};\n\ntype T = box struct {{a : Inner, b : I64}};\n",
            inner_fields
        ),
    )
    .expect("Failed to write the types module");

    // A body long enough that the division leaves the call to it in place instead of copying it
    // into the unit that calls it, so that the reader of the field and its caller are compiled into
    // object files of their own. Each step carries the field's value forward, so the program prints
    // what the field held.
    const STEPS: usize = 120;
    let mut lib = String::from(
        "module Lib;\n\nimport Types;\n\nget_b : T -> I64;\nget_b = |t| (\n    let v0 = t.@b;\n",
    );
    for step in 1..=STEPS {
        lib += &format!("    let v{} = v{} * 3 + {};\n", step, step - 1, step);
    }
    lib += &format!("    v{} % 1000000007\n);\n", STEPS);
    fs::write(dir.join("lib.fix"), lib).expect("Failed to write the lib module");

    fs::write(
        dir.join("main.fix"),
        format!(
            "module Main;\n\nimport Types;\nimport Lib;\n\nmain : IO ();\nmain = println(Lib::get_b(T{{a : {}, b : 7}}).to_string);\n",
            inner_literal
        ),
    )
    .expect("Failed to write the main module");
}

/// Runs the program the build in `dir` produced and returns what it printed.
fn printed_by_the_program(dir: &Path) -> String {
    let run = Command::new(dir.join("a.out"))
        .current_dir(dir)
        .output()
        .expect("Failed to run the program the build produced");
    assert!(
        run.status.success(),
        "the program should run to completion.\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    String::from_utf8_lossy(&run.stdout).trim().to_string()
}

/// A build after a type is widened reads the fields of it where they now sit.
///
/// A type reaches the RC IR as a type expression and a field of it as the index of that field, so
/// widening `Inner` leaves every part of `Lib::get_b`, which reads the field of `T` that the
/// `Inner` sits ahead of. What it moves is where that field sits, and the code reading it, so the
/// unit holding `get_b` is generated again and the program answers as one built from nothing does.
#[test]
fn test_a_build_after_a_type_widens_reads_the_fields_where_they_now_sit() {
    const CU_SIZE: &str = "1";

    let edited = TempDir::new().expect("Failed to create temp directory");
    write_nested_type_project(edited.path(), false);
    build_at_max_in(edited.path(), CU_SIZE);

    write_nested_type_project(edited.path(), true);
    let (_, cached) = build_at_max_in(edited.path(), CU_SIZE);
    assert!(
        cached > 0,
        "the build after the edit should take units from the one before it, or what it links is \
         what it generated"
    );
    let after_the_edit = printed_by_the_program(edited.path());

    let from_nothing = TempDir::new().expect("Failed to create temp directory");
    write_nested_type_project(from_nothing.path(), true);
    build_at_max_in(from_nothing.path(), CU_SIZE);

    assert_eq!(
        after_the_edit,
        printed_by_the_program(from_nothing.path()),
        "the build after the edit should print what a build of the edited sources from nothing does"
    );
}
