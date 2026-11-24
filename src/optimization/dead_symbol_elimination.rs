use crate::ast::program::Program;

/// Drop every symbol the program cannot reach from the values the C world enters it through.
pub fn run(prg: &mut Program) {
    let reached = prg.reached_symbol_names();
    prg.symbols.retain(|name, _| reached.contains(name));
}
