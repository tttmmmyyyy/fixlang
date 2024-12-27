use std::path::PathBuf;

use inkwell::passes::{PassManager, PassManagerSubType};

pub fn add_passes<T: PassManagerSubType>(
    passmgr: &PassManager<T>,
    llvm_passes_file: &Option<PathBuf>,
) {
    let lines = match llvm_passes_file {
        None => include_str!("llvm_passes.txt").to_string(),
        Some(file) => std::fs::read_to_string(file).unwrap(),
    };
    for line in lines.lines() {
        // Skip empty lines
        if line.is_empty() {
            continue;
        }
        // Add the pass to the pass manager.
        match line {
            "aggressive_dce_pass" => passmgr.add_aggressive_dce_pass(),
            "aggressive_inst_combiner_pass" => passmgr.add_aggressive_inst_combiner_pass(),
            "alignment_from_assumptions_pass" => passmgr.add_alignment_from_assumptions_pass(),
            "always_inliner_pass" => passmgr.add_always_inliner_pass(),
            "argument_promotion_pass" => passmgr.add_argument_promotion_pass(),
            "basic_alias_analysis_pass" => passmgr.add_basic_alias_analysis_pass(),
            "bit_tracking_dce_pass" => passmgr.add_bit_tracking_dce_pass(),
            "cfg_simplification_pass" => passmgr.add_cfg_simplification_pass(),
            "constant_merge_pass" => passmgr.add_constant_merge_pass(),
            "coroutine_cleanup_pass" => passmgr.add_coroutine_cleanup_pass(),
            "coroutine_early_pass" => passmgr.add_coroutine_early_pass(),
            "coroutine_elide_pass" => passmgr.add_coroutine_elide_pass(),
            "coroutine_split_pass" => passmgr.add_coroutine_split_pass(),
            "correlated_value_propagation_pass" => passmgr.add_correlated_value_propagation_pass(),
            "dead_arg_elimination_pass" => passmgr.add_dead_arg_elimination_pass(),
            "dead_store_elimination_pass" => passmgr.add_dead_store_elimination_pass(),
            "demote_memory_to_register_pass" => passmgr.add_demote_memory_to_register_pass(),
            "early_cse_mem_ssa_pass" => passmgr.add_early_cse_mem_ssa_pass(),
            "early_cse_pass" => passmgr.add_early_cse_pass(),
            "function_attrs_pass" => passmgr.add_function_attrs_pass(),
            "function_inlining_pass" => passmgr.add_function_inlining_pass(),
            "global_dce_pass" => passmgr.add_global_dce_pass(),
            "global_optimizer_pass" => passmgr.add_global_optimizer_pass(),
            "gvn_pass" => passmgr.add_gvn_pass(),
            "ind_var_simplify_pass" => passmgr.add_ind_var_simplify_pass(),
            "instruction_combining_pass" => passmgr.add_instruction_combining_pass(),
            "instruction_simplify_pass" => passmgr.add_instruction_simplify_pass(),
            "ipsccp_pass" => passmgr.add_ipsccp_pass(),
            "jump_threading_pass" => passmgr.add_jump_threading_pass(),
            "licm_pass" => passmgr.add_licm_pass(),
            "loop_deletion_pass" => passmgr.add_loop_deletion_pass(),
            "loop_idiom_pass" => passmgr.add_loop_idiom_pass(),
            "loop_reroll_pass" => passmgr.add_loop_reroll_pass(),
            "loop_rotate_pass" => passmgr.add_loop_rotate_pass(),
            "loop_unroll_and_jam_pass" => passmgr.add_loop_unroll_and_jam_pass(),
            "loop_unroll_pass" => passmgr.add_loop_unroll_pass(),
            "loop_unswitch_pass" => passmgr.add_loop_unswitch_pass(),
            "loop_vectorize_pass" => passmgr.add_loop_vectorize_pass(),
            "lower_expect_intrinsic_pass" => passmgr.add_lower_expect_intrinsic_pass(),
            "lower_switch_pass" => passmgr.add_lower_switch_pass(),
            "memcpy_optimize_pass" => passmgr.add_memcpy_optimize_pass(),
            "merge_functions_pass" => passmgr.add_merge_functions_pass(),
            "merged_load_store_motion_pass" => passmgr.add_merged_load_store_motion_pass(),
            "new_gvn_pass" => passmgr.add_new_gvn_pass(),
            "partially_inline_lib_calls_pass" => passmgr.add_partially_inline_lib_calls_pass(),
            "promote_memory_to_register_pass" => passmgr.add_promote_memory_to_register_pass(),
            "prune_eh_pass" => passmgr.add_prune_eh_pass(),
            "reassociate_pass" => passmgr.add_reassociate_pass(),
            "scalar_repl_aggregates_pass" => passmgr.add_scalar_repl_aggregates_pass(),
            "scalar_repl_aggregates_pass_ssa" => passmgr.add_scalar_repl_aggregates_pass_ssa(),
            "scalarizer_pass" => passmgr.add_scalarizer_pass(),
            "sccp_pass" => passmgr.add_sccp_pass(),
            "scoped_no_alias_aa_pass" => passmgr.add_scoped_no_alias_aa_pass(),
            "simplify_lib_calls_pass" => passmgr.add_simplify_lib_calls_pass(),
            "slp_vectorize_pass" => passmgr.add_slp_vectorize_pass(),
            "strip_dead_prototypes_pass" => passmgr.add_strip_dead_prototypes_pass(),
            "strip_symbol_pass" => passmgr.add_strip_symbol_pass(),
            "tail_call_elimination_pass" => passmgr.add_tail_call_elimination_pass(),
            "type_based_alias_analysis_pass" => passmgr.add_type_based_alias_analysis_pass(),
            "verifier_pass" => passmgr.add_verifier_pass(),
            _ => panic!("Unknown pass: {}", line),
        }
    }
}
