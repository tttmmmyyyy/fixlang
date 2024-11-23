// LLVM passes selected by hand.

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
