// LLVM passes selected by "passes_optimizer.py".

use inkwell::passes::{PassManager, PassManagerSubType};

pub fn add_optimized_optimization_passes<T: PassManagerSubType>(passmgr: &PassManager<T>) {
    passmgr.add_function_inlining_pass();
    passmgr.add_early_cse_pass();
    passmgr.add_function_inlining_pass();
    passmgr.add_scalar_repl_aggregates_pass_ssa();
    passmgr.add_sccp_pass();
    passmgr.add_instruction_simplify_pass();
    passmgr.add_loop_reroll_pass();
    passmgr.add_ipsccp_pass();
    passmgr.add_function_inlining_pass();
    passmgr.add_loop_unroll_pass();
    passmgr.add_correlated_value_propagation_pass();
    passmgr.add_strip_dead_prototypes_pass();
    passmgr.add_loop_unroll_and_jam_pass();
    passmgr.add_scalar_repl_aggregates_pass();
    passmgr.add_ind_var_simplify_pass();
    passmgr.add_cfg_simplification_pass();
    passmgr.add_scalar_repl_aggregates_pass();
    passmgr.add_partially_inline_lib_calls_pass();
    passmgr.add_instruction_combining_pass();
    passmgr.add_reassociate_pass();
    passmgr.add_loop_reroll_pass();
    passmgr.add_partially_inline_lib_calls_pass();
    passmgr.add_aggressive_inst_combiner_pass();
    passmgr.add_loop_deletion_pass();
    passmgr.add_loop_unroll_pass();
    passmgr.add_aggressive_inst_combiner_pass();
    passmgr.add_instruction_combining_pass();
    passmgr.add_loop_rotate_pass();
    passmgr.add_merged_load_store_motion_pass();
    passmgr.add_loop_deletion_pass();
    passmgr.add_loop_unroll_pass();
    passmgr.add_function_inlining_pass();
    passmgr.add_loop_idiom_pass();
    passmgr.add_slp_vectorize_pass();
    passmgr.add_ipsccp_pass();
    passmgr.add_early_cse_pass();
    passmgr.add_early_cse_pass();
    passmgr.add_ind_var_simplify_pass();
}
