use crate::{Configuration, Program};

use super::{borrowing, contract_app, eta_expand, uncurry};

pub fn run(prg: &mut Program, config: &Configuration) {
    let mut step = 0;

    if config.emit_symbols {
        prg.emit_symbols(&format!("{}", step));
    }

    // Perform eta expand optimization.
    if config.perform_eta_expand_optimization() {
        eta_expand::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.eta_expand", step));
            step += 1;
        }

        // If we perform eta expand optimization, we need to perform contract application optimization.
        contract_app::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.eta_expand_contract_app", step));
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
