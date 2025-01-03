use crate::{Configuration, Program};

use super::{borrowing, uncurry};

pub fn run(prg: &mut Program, config: &Configuration) {
    if config.emit_symbols {
        prg.emit_symbols("0");
    }

    // Perform eta expand optimization.
    // if config.perform_eta_expand_optimization() {
    //     eta_expand::run(prg);
    //     if config.emit_symbols {
    //         prg.emit_symbols("eta_expand");
    //     }
    // }

    // Perform uncurrying optimization.
    if config.perform_uncurry_optimization() {
        uncurry::run(prg);
        if config.emit_symbols {
            prg.emit_symbols("uncurry");
        }
    }

    // Perform borrowing optimization.
    if config.perform_borrowing_optimization() {
        borrowing::borrowing_optimization(prg);
        if config.emit_symbols {
            prg.emit_symbols("borrowing");
        }
    }

    // Use call_graph_inst_syms.
}
