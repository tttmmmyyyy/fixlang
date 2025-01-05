use crate::{Configuration, Program};

use super::{borrowing, inline, remove_tyanno, simplify_global_names, uncurry};

pub fn run(prg: &mut Program, config: &Configuration) {
    let mut step = 0;

    if config.emit_symbols {
        prg.emit_symbols(&format!("{}", step));
        step += 1;
    }

    // Perform simplification of global names.
    if config.perform_simplify_global_names() {
        simplify_global_names::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.simplify_global_names", step));
            step += 1;
        }
    }

    // Perform type annotation removal optimization.
    if config.perform_remove_tyanno_optimization() {
        remove_tyanno::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.remove_tyanno", step));
            step += 1;
        }
    }

    // Perform inlining optimization.
    if config.perform_inline_optimization() {
        inline::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.inline", step));
            step += 1;
        }
    }

    // Perform uncurrying optimization.
    if config.perform_uncurry_optimization() {
        uncurry::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.uncurry", step));
            step += 1;
        }
    }

    // Perform borrowing optimization.
    if config.perform_borrowing_optimization() {
        borrowing::borrowing_optimization(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.borrowing", step));
            // step += 1;
        }
    }

    // Use call_graph_inst_syms.
}
