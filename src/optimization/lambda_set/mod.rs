// Lambda set specialization: which lambdas a value of function type can be, carried in its type.
//
// Every slot of a type where a function value sits (an arrow, and the arrows a struct, a union or an
// array declares) carries a lambda set variable. Inference unifies the variables of two places whose
// values flow into each other, as the type checker unifies their types, and records each lambda in
// the variable of the place it is built at. A component of the call graph is inferred together and
// generalized over the variables of its symbols' types, so that a function taking a function is
// inferred once and copied for each lambda set it is given. Specialization then walks from the
// program's entry, and gives each copy the concrete lambda sets of its parameters.
//
// The design is laid out in `dev-docs/2026-10-24-lambda-set-specialization/design.md`.

mod concrete;
mod dump;
mod flow;
mod infer;
mod slots;
mod solver;
mod specialize;

pub use flow::BuiltinFlow;

use crate::{ast::program::Program, constants::DOT_FIXLANG};
use slots::SlotLayout;
use std::{fs::File, io::Write, path::PathBuf};

/// Infers the lambda sets of `prg` and the copies they ask for, leaving the program as it is. With
/// `dump_step`, writes them to a file under `.fixlang` named after it.
pub fn run(prg: &Program, dump_step: Option<String>) {
    let layout = SlotLayout::new(&prg.type_env);
    let mut analysis = infer::infer(prg, &layout);
    let spec = specialize::specialize(&mut analysis);
    if let Some(step) = dump_step {
        let text = dump::render(&mut analysis, &spec, &layout);
        let path = PathBuf::from(format!("{}/{}.lambda_sets.txt", DOT_FIXLANG, step));
        let mut file = File::create(&path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }
}
