use crate::constants::C_ENTRY_POINT_NAME;
use crate::fixstd::runtime::{RUNTIME_GET_ARGC, RUNTIME_GET_ARGV};
use crate::misc::Set;
use crate::tests::test_util::{first_local_value, generated_llvm_ir_modules, llvm_function_bodies};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use tempfile::TempDir;

/// A program that reaches every place the code generator makes a value of its own.
///
/// A global whose type occupies no storage gives an initializer that hands back a value of no bits.
/// A union of variants of three widths gives payload buffers read both wider and narrower than the
/// value put in them, and one of those variants holds a byte no field owns. A boxed value gives the
/// helpers that retain, release and traverse one. A fold gives a value a loop carries, which is
/// defined below the `phi` that reads it. `undefined` gives an arm that ends the program rather
/// than handing a value back. An `FFI_CALL` gives a value whose bits the declared signature does
/// not settle; the C function it names is written again in `C_FUNCTION_NAME`, which is what the
/// emitted call is looked up under. And an `FFI_EXPORT` gives a function whose callers are C code
/// this compiler never sees; the name it offers that function under is written again in
/// `EXPORTED_C_FUNCTION_NAME`, which is what the emitted definition is looked up under.
const BOUNDARY_SOURCE: &str = r#"
    module Main;

    nothing : ();
    nothing = ();

    type Padded = unbox struct { tag : U8, count : I64 };
    type Choice = unbox union { small : U8, padded : Padded, wide : (I64, I64, I64) };
    type Node = box struct { label : String, next : Option Node };

    read : Choice -> I64;
    read = |c| (
        if c.is_small { c.as_small.to_I64 };
        if c.is_padded { c.as_padded.@count };
        let w = c.as_wide;
        w.@0 + w.@1 + w.@2
    );

    depth : Option Node -> I64;
    depth = |node| if node.is_none { 0 } else { 1 + depth(node.as_some.@next) };

    total : I64 -> I64;
    total = |n| Iterator::range(0, n + 3).fold(0, |acc, i| acc + i);

    positive : I64 -> I64;
    positive = |x| if x > 0 { x } else { undefined("a negative count") };

    offered : I64 -> I64;
    offered = |x| x + 1;

    FFI_EXPORT[offered, c_offered];

    main : IO ();
    main = (
        eval nothing;
        let n = (*get_args).@size;
        let c = if n == 1 { Choice::small(7_U8) } else { Choice::padded(Padded { tag : 1_U8, count : n }) };
        let chain = Node { label : "root", next : Option::some(Node { label : "leaf", next : Option::none() }) };
        let seven = FFI_CALL[CInt abs(CInt), (0 - 7).to_CInt];
        let three = *FFI_CALL_IO[CInt abs(CInt), (0 - 3).to_CInt];
        println $ (read(c) + read(Choice::wide((1, 2, 3))) + depth(Option::some(chain)) + total(n) + positive(n) + seven.to_I64 + three.to_I64).to_string
    );
"#;

/// The C function `BOUNDARY_SOURCE` calls, whose result the code generator fixes the bits of.
const C_FUNCTION_NAME: &str = "abs";

/// The modules the compiler writes for `BOUNDARY_SOURCE`, built once and shared by every test that
/// reads them. The build is what these tests spend their time on.
///
/// These tests read the code as the generator wrote it, built at `-O none`: an optimized module
/// also holds what LLVM itself introduced, and LLVM may name `undef` where the generator named
/// `poison`.
fn boundary_modules() -> &'static [String] {
    static MODULES: OnceLock<Vec<String>> = OnceLock::new();
    MODULES.get_or_init(|| generated_llvm_ir_modules(BOUNDARY_SOURCE, "none", &[]))
}

/// The name `BOUNDARY_SOURCE` offers its exported function to C under.
const EXPORTED_C_FUNCTION_NAME: &str = "c_offered";

/// The functions of an emitted module that carry no statement about their boundary values.
///
/// A function whose boundary is a C signature stands outside the statement, since the calls across
/// it come from code this compiler never sees: the two runtime functions a Fix program reaches
/// through their C signatures, the entry point whose arguments the C runtime supplies, and a
/// function `FFI_EXPORT` offers. `Generator::add_generated_function` states it on every other
/// function the compiler emits a body for.
const FUNCTIONS_OUTSIDE_THE_STATEMENT: [&str; 4] = [
    RUNTIME_GET_ARGC,
    RUNTIME_GET_ARGV,
    C_ENTRY_POINT_NAME,
    EXPORTED_C_FUNCTION_NAME,
];

/// One `define` line of an emitted module, cut into the parts that carry the statement.
struct DefinedFunction<'a> {
    /// The name the compiler gave the function, without the `@` and any quotes around it.
    name: &'a str,
    /// The text before the name: the linkage, the return type, and the attributes on the result.
    before_name: &'a str,
    /// The parameters, each with the attributes on it.
    parameters: Vec<&'a str>,
}

/// The function `line` defines, where `line` opens a definition.
fn defined_function(line: &str) -> Option<DefinedFunction<'_>> {
    let signature = line.strip_prefix("define ")?;
    let at = signature.find('@')?;
    let (before_name, from_name) = signature.split_at(at);
    let from_name = &from_name[1..];
    let (name, after_name) = match from_name.strip_prefix('"') {
        Some(quoted) => {
            let end = quoted
                .find('"')
                .unwrap_or_else(|| panic!("a quoted function name is closed: {}", line));
            (&quoted[..end], &quoted[end + 1..])
        }
        None => {
            let end = from_name.find('(').unwrap_or_else(|| {
                panic!("a function name is followed by its parameters: {}", line)
            });
            (&from_name[..end], &from_name[end..])
        }
    };
    let parameters = after_name
        .strip_prefix('(')
        .unwrap_or_else(|| panic!("a function name is followed by its parameters: {}", line))
        .rsplit_once(')')
        .unwrap_or_else(|| panic!("a parameter list is closed: {}", line))
        .0;
    Some(DefinedFunction {
        name,
        before_name,
        parameters: split_at_top_level_commas(parameters),
    })
}

/// `text` cut at the commas between its parts.
///
/// A type is written with commas inside it — `{ ptr, i64 }`, `[2 x i64]`, `<4 x i64>` — so the cut
/// is made only where no bracket is open.
fn split_at_top_level_commas(text: &str) -> Vec<&str> {
    let mut depth = 0;
    let mut parts = Vec::new();
    let mut start = 0;
    for (at, character) in text.char_indices() {
        match character {
            '{' | '[' | '<' | '(' => depth += 1,
            '}' | ']' | '>' | ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(text[start..at].trim());
                start = at + character.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(text[start..].trim());
    parts.into_iter().filter(|part| !part.is_empty()).collect()
}

/// Whether `text` carries `noundef` as a word of its own.
fn states_the_value_is_written(text: &str) -> bool {
    text.split_whitespace().any(|word| word == "noundef")
}

/// Every value a generated function takes and returns has all of its bits written, and the
/// generated code says so.
///
/// LLVM assumes neither of these on its own: without the statement it reads every argument as one
/// that may be undefined, and it inserts a `freeze` before branching on one.
/// `Generator::add_generated_function` is the one constructor that puts the statement on, so a
/// function declared through `Module::add_function` instead arrives here bare.
#[test]
pub fn test_every_generated_function_states_its_boundary_values_are_written() {
    let mut functions_read = 0;
    let mut bare_boundary_values = Vec::new();
    let mut outside_the_statement: Set<&str> = Set::default();
    for module in boundary_modules() {
        for line in module.lines() {
            let Some(function) = defined_function(line) else {
                continue;
            };
            if FUNCTIONS_OUTSIDE_THE_STATEMENT.contains(&function.name) {
                outside_the_statement.insert(function.name);
                continue;
            }
            functions_read += 1;
            let returns_a_value = function.before_name.split_whitespace().last() != Some("void");
            if returns_a_value && !states_the_value_is_written(function.before_name) {
                bare_boundary_values.push(format!("the result of `{}`", function.name));
            }
            for parameter in &function.parameters {
                if *parameter != "..." && !states_the_value_is_written(parameter) {
                    bare_boundary_values.push(format!(
                        "the parameter `{}` of `{}`",
                        parameter, function.name
                    ));
                }
            }
        }
    }
    assert!(
        bare_boundary_values.is_empty(),
        "every value crossing a generated function's boundary should be stated to have all of its \
         bits written, but {} do not:\n{}",
        bare_boundary_values.len(),
        bare_boundary_values.join("\n"),
    );
    assert!(
        functions_read > 0,
        "the program should be compiled into functions, so that this test has boundaries to read",
    );
    for name in FUNCTIONS_OUTSIDE_THE_STATEMENT {
        assert!(
            outside_the_statement.contains(name),
            "`{}` is named as standing outside the statement, so a build of an executable should \
             define it",
            name,
        );
    }
}

/// The code generator names `poison` where it says a value is never read, and never `undef`.
///
/// A poison is one value for all of its readers, so LLVM may merge two reads of it or duplicate
/// one. An `undef` may give a different value at each use. The code that emits the constant chooses
/// between the two, since LLVM may weaken a poison into an `undef` and never the reverse.
#[test]
pub fn test_the_code_generator_names_poison_rather_than_undef() {
    let mut lines_naming_undef = Vec::new();
    for module in boundary_modules() {
        for line in module.lines() {
            if names_undef(line) {
                lines_naming_undef.push(line.trim().to_string());
            }
        }
    }
    assert!(
        lines_naming_undef.is_empty(),
        "the code generator should name `poison` where a value is never read, but {} lines name \
         `undef`:\n{}",
        lines_naming_undef.len(),
        lines_naming_undef.join("\n"),
    );
}

/// Whether `line` names the `undef` constant.
///
/// `noundef` is a word of its own, so a line stating that a boundary value is written does not name
/// it.
fn names_undef(line: &str) -> bool {
    line.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|word| word == "undef")
}

/// The result of a C function is given one answer for every read of it.
///
/// What a C function leaves behind is outside what Fix's types cover — C leaves a result undefined
/// where the caller ignores it — so the value carries bits LLVM holds to be undefined, and a read
/// of such a bit may answer differently each time. `Generator::build_freeze` chooses one answer and
/// holds it, so the value can cross a boundary that states it is written.
#[test]
pub fn test_the_result_of_a_c_function_is_given_one_answer_for_every_read() {
    let mut calls_read = 0;
    let mut unfrozen_calls = Vec::new();
    for module in boundary_modules() {
        for body in llvm_function_bodies(module, "") {
            let mut frozen: Set<String> = Set::default();
            for line in body.lines() {
                if let Some(operand) = frozen_operand(line.trim()) {
                    frozen.insert(operand.to_string());
                }
            }
            for line in body.lines() {
                let line = line.trim();
                let Some(result) = result_of_a_call_to(line, C_FUNCTION_NAME) else {
                    continue;
                };
                calls_read += 1;
                if !frozen.contains(result) {
                    unfrozen_calls.push(line.to_string());
                }
            }
        }
    }
    assert!(
        unfrozen_calls.is_empty(),
        "the result of a C function should be frozen before Fix code reads it, but {} are not:\n{}",
        unfrozen_calls.len(),
        unfrozen_calls.join("\n"),
    );
    assert!(
        calls_read > 0,
        "the program calls `{}` through `FFI_CALL`, so that this test has a C result to read",
        C_FUNCTION_NAME,
    );
}

/// The value a call to the C function `callee` binds, where `line` is such a call.
fn result_of_a_call_to<'a>(line: &'a str, callee: &str) -> Option<&'a str> {
    let (result, expression) = line.split_once(" = ")?;
    let opcode = expression.split_whitespace().next()?;
    if !["call", "tail", "musttail", "notail"].contains(&opcode) {
        return None;
    }
    expression
        .contains(&format!("@{}(", callee))
        .then_some(result.trim())
}

/// The value `line` freezes, where `line` is a `freeze`.
fn frozen_operand(line: &str) -> Option<&str> {
    let (_, expression) = line.split_once(" = ")?;
    first_local_value(expression.strip_prefix("freeze ")?)
}

/// The path `check_no_undefined_bits.py` is at.
fn undefined_bits_check_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("check_no_undefined_bits.py")
}

/// Runs `check_no_undefined_bits.py` with `arguments`, and hands back what it printed and the
/// status it exited with.
fn run_the_undefined_bits_check<S: AsRef<OsStr>>(arguments: impl IntoIterator<Item = S>) -> Output {
    let script = undefined_bits_check_script();
    Command::new("python3")
        .arg(&script)
        .args(arguments)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run `python3 {}`: {}", script.display(), e))
}

/// The modules of `boundary_modules` written to files, for a tool that reads LLVM IR by path.
///
/// The directory holds them, so the caller keeps it alive for as long as the paths are read.
fn boundary_module_files() -> (TempDir, Vec<PathBuf>) {
    let dir = TempDir::new().expect("Failed to create temp directory");
    let paths = boundary_modules()
        .iter()
        .enumerate()
        .map(|(index, module)| {
            let path = dir.path().join(format!("module-{}.ll", index));
            fs::write(&path, module).expect("Failed to write an emitted LLVM IR file");
            path
        })
        .collect();
    (dir, paths)
}

/// No value carrying a bit nothing wrote reaches a call argument or a `ret`.
///
/// This is the guarantee behind the statement every generated function makes about its boundary
/// values: LLVM reads a violation of `noundef` as undefined behavior, so the statement is worth
/// what the guarantee is worth. `check_no_undefined_bits.py` follows each `poison` and `undef` the
/// emitted code names through the instructions that carry it, and reports the ones that arrive.
#[test]
pub fn test_no_value_with_undefined_bits_reaches_a_function_boundary() {
    let (_dir, paths) = boundary_module_files();
    let report = run_the_undefined_bits_check(&paths);
    assert!(
        report.status.success(),
        "no value with undefined bits should reach a function boundary:\n{}{}",
        String::from_utf8_lossy(&report.stdout),
        String::from_utf8_lossy(&report.stderr),
    );
}

/// The walk `check_no_undefined_bits.py` makes gives the answers it is known to give.
///
/// A silent run over the emitted modules means something only once the walk is seen to report what
/// it should: `--self-test` runs it over a module written into the script whose answers are known,
/// and fails where the walk goes past one of those answers or reports a function with nothing to
/// report.
#[test]
pub fn test_the_undefined_bits_check_gives_the_answers_it_is_known_to_give() {
    let report = run_the_undefined_bits_check(["--self-test"]);
    assert!(
        report.status.success(),
        "`{} --self-test` should report the functions it is known to report:\n{}{}",
        undefined_bits_check_script().display(),
        String::from_utf8_lossy(&report.stdout),
        String::from_utf8_lossy(&report.stderr),
    );
}

/// A variant exactly as wide as the payload buffer it goes into still has its unowned byte written.
///
/// A struct whose second field is aligned past the end of its first holds a byte no field owns, and
/// a store of such a value leaves that byte as it was. Where the union carries that one variant,
/// the payload buffer is the variant's own width, so nothing is left over at the end of the slot
/// either: the byte inside the value is the only one the store leaves behind. `Generator::bit_cast`
/// writes a zero over the whole slot first, which is what puts a value in it.
#[test]
pub fn test_the_slot_a_padded_value_bit_casts_through_is_written_whole_first() {
    const SOURCE: &str = r#"
        module Main;

        type Padded = unbox struct { tag : U8, count : I64 };
        type OnlyPadded = unbox union { padded : Padded };

        main : IO ();
        main = (
            let n = (*get_args).@size;
            let u = OnlyPadded::padded(Padded { tag : 1_U8, count : n });
            println $ u.as_padded.@count.to_string
        );
    "#;
    // The embedded type of `Padded`, as the value stored into the bit-cast slot is written.
    const PADDED: &str = "{ { i8 }, { i64 } }";

    let mut stores_read = 0;
    let mut unfilled_slots = Vec::new();
    for module in generated_llvm_ir_modules(SOURCE, "none", &[]) {
        for body in llvm_function_bodies(&module, "") {
            let mut filled: Set<String> = Set::default();
            for line in body.lines() {
                let line = line.trim();
                if let Some(slot) = slot_a_store_of_zero_bytes_fills(line) {
                    filled.insert(slot.to_string());
                }
                let Some(slot) = slot_a_store_of_the_type_writes(line, PADDED) else {
                    continue;
                };
                stores_read += 1;
                if !filled.contains(slot) {
                    unfilled_slots.push(line.to_string());
                }
            }
        }
    }
    assert!(
        unfilled_slots.is_empty(),
        "the slot a padded value is stored into should be written whole first, but {} are not:\n{}",
        unfilled_slots.len(),
        unfilled_slots.join("\n"),
    );
    assert!(
        stores_read > 0,
        "the program stores a `Padded` through a bit-cast slot, so that this test has one to read",
    );
}

/// The slot `line` writes a zero of so many bytes over, where `line` is such a store.
fn slot_a_store_of_zero_bytes_fills(line: &str) -> Option<&str> {
    let arguments = line.strip_prefix("store [")?;
    let (_, arguments) = arguments.split_once("x i8] zeroinitializer, ptr ")?;
    first_local_value(arguments)
}

/// The slot `line` stores a value of `ty` into, where `line` is such a store.
fn slot_a_store_of_the_type_writes<'a>(line: &'a str, ty: &str) -> Option<&'a str> {
    let arguments = line.strip_prefix(&format!("store {} ", ty))?;
    let (_, arguments) = arguments.split_once(", ptr ")?;
    first_local_value(arguments)
}
