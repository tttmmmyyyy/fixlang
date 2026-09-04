use std::mem::take;

use crate::{ast::program::Program, misc::Set};

/// Drop every symbol the program cannot reach from the values the C world enters it through.
// PROOF: P26 (dev-docs/proof/rc_ir/borrow-cancel)
pub fn run(prg: &mut Program) {
    let mut seeds = prg.root_value_names();

    // Collect names called by the root values.
    let mut called_syms = seeds.clone().into_iter().collect::<Set<_>>();
    while seeds.len() > 0 {
        let mut new_seeds = vec![];
        for seed in seeds {
            let sym = prg.symbols.get(&seed).unwrap();
            for sym in sym.expr.as_ref().unwrap().free_vars() {
                if !called_syms.contains(&sym) {
                    called_syms.insert(sym.clone());
                    new_seeds.push(sym.clone());
                }
            }
        }
        seeds = new_seeds;
    }

    // Discard all symbols not in `called_syms`.
    let mut new_syms = vec![];
    for (name, sym) in take(&mut prg.symbols) {
        if called_syms.contains(&name) {
            new_syms.push((name, sym));
        }
    }
    prg.symbols = new_syms.into_iter().collect();
}
