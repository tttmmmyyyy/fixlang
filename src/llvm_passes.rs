use std::path::PathBuf;

use inkwell::passes::{PassManager, PassManagerSubType};

pub fn add_strip_passes<T: PassManagerSubType>(passmgr: &PassManager<T>) {
    passmgr.add_strip_symbol_pass();
}

pub fn add_basic_optimization_passes<T: PassManagerSubType>(passmgr: &PassManager<T>) {
    passmgr.add_always_inliner_pass();
    passmgr.add_function_inlining_pass();
    passmgr.add_strip_dead_prototypes_pass();
    passmgr.add_dead_store_elimination_pass();
    passmgr.add_global_dce_pass();
    passmgr.add_global_optimizer_pass();
    passmgr.add_scalar_repl_aggregates_pass_ssa();
    passmgr.add_promote_memory_to_register_pass();
    passmgr.add_cfg_simplification_pass();
    passmgr.add_instruction_combining_pass();
    passmgr.add_memcpy_optimize_pass();
    passmgr.add_loop_vectorize_pass();
    passmgr.add_slp_vectorize_pass();
}

pub fn add_optimized_optimization_passes<T: PassManagerSubType>(
    passmgr: &PassManager<T>,
    llvm_passes_file: &Option<PathBuf>,
) {
    let lines = match llvm_passes_file {
        None => include_str!("llvm_passes_optimized.txt").to_string(),
        Some(file) => std::fs::read_to_string(file).unwrap(),
    };
    for line in lines.lines() {
        // Skip empty lines
        if line.is_empty() {
            continue;
        }
        // Add the pass to the pass manager.
        match line {
            "function_inlining_pass" => passmgr.add_function_inlining_pass(),
            "early_cse_pass" => passmgr.add_early_cse_pass(),
            "scalar_repl_aggregates_pass_ssa" => passmgr.add_scalar_repl_aggregates_pass_ssa(),
            "sccp_pass" => passmgr.add_sccp_pass(),
            "instruction_simplify_pass" => passmgr.add_instruction_simplify_pass(),
            "loop_reroll_pass" => passmgr.add_loop_reroll_pass(),
            "ipsccp_pass" => passmgr.add_ipsccp_pass(),
            "correlated_value_propagation_pass" => passmgr.add_correlated_value_propagation_pass(),
            "strip_dead_prototypes_pass" => passmgr.add_strip_dead_prototypes_pass(),
            "loop_unroll_and_jam_pass" => passmgr.add_loop_unroll_and_jam_pass(),
            "scalar_repl_aggregates_pass" => passmgr.add_scalar_repl_aggregates_pass(),
            "ind_var_simplify_pass" => passmgr.add_ind_var_simplify_pass(),
            "cfg_simplification_pass" => passmgr.add_cfg_simplification_pass(),
            "partially_inline_lib_calls_pass" => passmgr.add_partially_inline_lib_calls_pass(),
            "instruction_combining_pass" => passmgr.add_instruction_combining_pass(),
            "reassociate_pass" => passmgr.add_reassociate_pass(),
            "aggressive_inst_combiner_pass" => passmgr.add_aggressive_inst_combiner_pass(),
            "loop_deletion_pass" => passmgr.add_loop_deletion_pass(),
            "loop_unroll_pass" => passmgr.add_loop_unroll_pass(),
            "loop_rotate_pass" => passmgr.add_loop_rotate_pass(),
            "merged_load_store_motion_pass" => passmgr.add_merged_load_store_motion_pass(),
            "slp_vectorize_pass" => passmgr.add_slp_vectorize_pass(),
            _ => panic!("Unknown pass: {}", line),
        }
    }
}
