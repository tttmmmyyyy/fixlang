use crate::{optimization::remove_hktvs, stopwatch::StopWatch, Configuration, Program};

use super::{
    dead_symbol_elimination, decapturing, inline, remove_tyanno, simplify_symbol_names, uncurry,
    unwrap_newtype,
};

pub fn run(prg: &mut Program, config: &Configuration) {
    let _sw = StopWatch::new("optimization::run", config.show_build_times);

    if config.emit_symbols {
        prg.emit_symbols(&format!("{}", prg.optimization_step));
        prg.optimization_step += 1;
    }

    // Perform simplification of global names.
    if config.enable_simplify_symbol_names() {
        let _sw = StopWatch::new("simplify_symbol_names::run", config.show_build_times);
        simplify_symbol_names::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.simplify_symbol_names", prg.optimization_step));
            prg.optimization_step += 1;
        }
    }

    // Perform type annotation removal optimization.
    if config.enable_remove_tyanno_optimization() {
        let _sw = StopWatch::new("remove_tyanno::run", config.show_build_times);
        remove_tyanno::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.remove_tyanno", prg.optimization_step));
            prg.optimization_step += 1;
        }
    }

    // Perform remove-hktvs transformation.
    if config.enable_remove_hktvs_transformation() {
        let _sw = StopWatch::new("remove_hktvs::run", config.show_build_times);
        remove_hktvs::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.remove_hktvs", prg.optimization_step));
            prg.optimization_step += 1;
        }
    }

    // Perform unwrap_newtype optimization.
    if config.enable_unwrap_newtype_optimization() {
        let _sw = StopWatch::new("unwrap_newtype::run", config.show_build_times);
        unwrap_newtype::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.unwrap_newtype", prg.optimization_step));
            prg.optimization_step += 1;
        }
    }

    // Perform inlining optimization.
    if config.enable_inline_optimization() {
        let _sw = StopWatch::new("inline::run", config.show_build_times);
        inline::run(prg, config.show_build_times);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.inline", prg.optimization_step));
            prg.optimization_step += 1;
        }
    }

    // Perform decapturing optimization
    if config.enable_decapturing_optimization() {
        let _sw = StopWatch::new("decapturing::run", config.show_build_times);
        decapturing::run(prg, config.show_build_times);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.decapturing", prg.optimization_step));
            prg.optimization_step += 1;
        }
    }

    // Perform uncurrying optimization.
    if config.enable_uncurry_optimization() {
        let _sw = StopWatch::new("uncurry::run", config.show_build_times);
        uncurry::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!("{}.uncurry", prg.optimization_step));
            prg.optimization_step += 1;
        }
    }

    // Perform dead symbol elimination.
    if config.enable_dead_symbol_elimination() {
        let _sw = StopWatch::new("dead_symbol_elimination::run", config.show_build_times);
        dead_symbol_elimination::run(prg);
        if config.emit_symbols {
            prg.emit_symbols(&format!(
                "{}.dead_symbol_elimination",
                prg.optimization_step
            ));
            prg.optimization_step += 1;
        }
    }

    if config.emit_symbols {
        let _sw = StopWatch::new("simplify_symbol_names::run", config.show_build_times);
        simplify_symbol_names::run(prg);
        prg.emit_symbols(&format!("{}.final", prg.optimization_step));
        prg.optimization_step += 1;
    }
}
