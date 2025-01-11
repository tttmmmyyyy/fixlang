use std::mem::take;

use crate::{misc::Set, Program};

pub fn run(prg: &mut Program) {
    // Collect names of entry point values.
    let mut seeds = vec![];
    if let Some(entry_io) = &mut prg.entry_io_value {
        seeds.push(entry_io.get_var().name.clone());
    }
    for export_stmt in &mut prg.export_statements {
        if let Some(entry_io) = &mut export_stmt.value_expr {
            seeds.push(entry_io.get_var().name.clone());
        }
    }

    // Collect names called by the entry point values.
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

    // Discaed all symbols not in `called_syms`.
    let mut new_syms = vec![];
    for (name, sym) in take(&mut prg.symbols) {
        if called_syms.contains(&name) {
            new_syms.push((name, sym));
        }
    }
    prg.symbols = new_syms.into_iter().collect();
}
