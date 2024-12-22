use crate::{Configuration, Program};

use super::{
    borrowing_optimization::borrowing_optimization, uncurry_optimization::uncurry_optimization,
};

pub fn optimize(prg: &mut Program, config: &Configuration) {
    if config.emit_symbols {
        prg.emit_symbols("0");
    }

    // Perform uncurrying optimization.
    if config.perform_uncurry_optimization() {
        uncurry_optimization(prg);
        if config.emit_symbols {
            prg.emit_symbols("uncurry");
        }
    }

    // Perform borrowing optimization.
    if config.perform_borrowing_optimization() {
        borrowing_optimization(prg);
        if config.emit_symbols {
            prg.emit_symbols("borrowing");
        }
    }

    // Use call_graph_inst_syms.
}
