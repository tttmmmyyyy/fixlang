use crate::{Configuration, Program};

use super::{
    borrowing_optimization::borrowing_optimization, uncurry_optimization::uncurry_optimization,
};

pub fn optimize(prg: &mut Program, config: &Configuration) {
    if config.output_symbols {
        prg.output_symbols("0");
    }

    // Perform uncurrying optimization.
    if config.perform_uncurry_optimization() {
        uncurry_optimization(prg);
        if config.output_symbols {
            prg.output_symbols("uncurry");
        }
    }

    // Perform borrowing optimization.
    if config.perform_borrowing_optimization() {
        borrowing_optimization(prg);
        if config.output_symbols {
            prg.output_symbols("borrowing");
        }
    }
}
