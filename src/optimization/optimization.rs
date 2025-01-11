use crate::{Configuration, Program};

use super::{dead_symbol_elimination, inline, remove_tyanno, simplify_symbol_names, uncurry};

pub fn run(prg: &mut Program, config: &Configuration) {
    let mut step = 0;

    if config.emit_symbols {
        prg.emit_symbols(&format!("{}", step));
        step += 1;
    }

    // Perform simplification of global names.
    if config.enable_simplify_symbol_names() {
        simplify_symbol_names::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.simplify_symbol_names", step));
            step += 1;
        }
    }

    // Perform type annotation removal optimization.
    if config.enable_remove_tyanno_optimization() {
        remove_tyanno::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.remove_tyanno", step));
            step += 1;
        }
    }

    // Perform inlining optimization.
    if config.enable_inline_optimization() {
        inline::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.inline", step));
            step += 1;
        }
    }

    // Perform uncurrying optimization.
    if config.enable_uncurry_optimization() {
        uncurry::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.uncurry", step));
            step += 1;
        }
    }

    // Perform dead symbol elimination.
    if config.enable_dead_symbol_elimination() {
        dead_symbol_elimination::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.dead_symbol_elimination", step));
            // step += 1;
        }
    }
}
