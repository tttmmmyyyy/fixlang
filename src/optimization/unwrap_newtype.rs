use crate::ast::program::Program;

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.symbols {
        // run_on_symbol(sym);
    }
}
