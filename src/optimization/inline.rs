/*
Inlining optimization.
*/

use crate::{InstantiatedSymbol, Program};

pub fn run(prg: &mut Program) {
    for (_name, sym) in &mut prg.instantiated_symbols {
        run_on_symbol(sym);
    }
}

fn run_on_symbol(_sym: &mut InstantiatedSymbol) {}
