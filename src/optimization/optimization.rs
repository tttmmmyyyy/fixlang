use super::{
    closure_specialization, dead_symbol_elimination, defunctionalize_fix, inline, inline_local,
    optimize_act, remove_tyanno, simplify_symbol_names, skip_eval, uncurry, unwrap_newtype,
};
use crate::{ast::program::Program, configuration::Configuration, tool::stopwatch::StopWatch};

/// Rewrite `prg` in place by running the optimization passes in order, each one gated by the
/// setting in `config` that turns it on. A pass sees the program the passes above it left.
pub fn run(prg: &mut Program, config: &Configuration) {
    let _sw = StopWatch::new("optimization::run", config.show_build_times);

    if config.emit_symbols {
        prg.emit_symbols(&format!("{}", prg.optimization_step));
        prg.optimization_step += 1;
    }

    // Drop the side expressions of `eval`, before any pass looks at them. Type checking and
    // instantiation are done by now, so whether the program compiles is already settled, and every
    // pass below sees the simplified tree.
    run_pass(prg, config, config.skip_eval, "skip_eval", skip_eval::run);

    // Specialize `act_` on the functor it is used at. It runs before the pass that unwraps
    // newtypes, which is what lets it recognize what it specializes: it finds an `act_` by asking
    // `TypeEnv::is_struct_act` about the symbol's name, and it recognizes `Std::Identity` and
    // `Std::Const` by the string of the type. Both are one-field unboxed structs, so unwrapping
    // replaces them and leaves neither question answerable.
    run_pass(
        prg,
        config,
        config.enable_act_optimization(),
        "optimize_act",
        |prg| optimize_act::run(prg, config),
    );

    run_pass(
        prg,
        config,
        config.enable_simplify_symbol_names(),
        "simplify_symbol_names",
        simplify_symbol_names::run,
    );

    run_pass(
        prg,
        config,
        config.enable_remove_tyanno_optimization(),
        "remove_tyanno",
        remove_tyanno::run,
    );

    run_pass(
        prg,
        config,
        config.enable_unwrap_newtype_optimization(),
        "unwrap_newtype",
        unwrap_newtype::run,
    );

    // Defunctionalize `Std::fix` into directly self-recursive global functions. It runs before
    // inlining and closure specialization, which would otherwise rewrite the `fix` argument out of
    // the literal lambda form this pass matches; uncurrying (later) then turns each self-call into a
    // direct call that LLVM folds into a loop.
    run_pass(
        prg,
        config,
        config.enable_defunctionalize_fix(),
        "defunctionalize_fix",
        |prg| defunctionalize_fix::run(prg, config.show_build_times),
    );

    run_pass(
        prg,
        config,
        config.enable_inline_optimization(),
        "inline",
        inline::run,
    );

    run_pass(
        prg,
        config,
        config.enable_inline_local_optimization(),
        "inline_local",
        inline_local::run,
    );

    run_pass(
        prg,
        config,
        config.enable_closure_specialization(),
        "closure_specialization",
        |prg| closure_specialization::run(prg, config.show_build_times),
    );

    run_pass(
        prg,
        config,
        config.enable_uncurry_optimization(),
        "uncurry",
        uncurry::run,
    );

    run_pass(
        prg,
        config,
        config.enable_dead_symbol_elimination(),
        "dead_symbol_elimination",
        dead_symbol_elimination::run,
    );

    if config.emit_symbols {
        let _sw = StopWatch::new("simplify_symbol_names::run", config.show_build_times);
        simplify_symbol_names::run(prg);
        prg.emit_symbols(&format!("{}.final", prg.optimization_step));
        prg.optimization_step += 1;
    }
}

/// Runs the pass named `pass_name` on `prg` if `enabled`, timing it and emitting the symbols it
/// leaves behind as one optimization step.
fn run_pass(
    prg: &mut Program,
    config: &Configuration,
    enabled: bool,
    pass_name: &str,
    pass: impl FnOnce(&mut Program),
) {
    if !enabled {
        return;
    }
    let _sw = StopWatch::new(&format!("{}::run", pass_name), config.show_build_times);
    pass(prg);
    if config.emit_symbols {
        prg.emit_symbols(&format!("{}.{}", prg.optimization_step, pass_name));
        prg.optimization_step += 1;
    }
}
