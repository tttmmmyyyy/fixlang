from math import ceil
import subprocess
import re
import itertools
import sys
import random
from statsmodels.stats.weightstats import ttest_ind
import numpy as np

# Optimizes llvm optimization passes by updating llvm_passes.rs.
SOURCE_FILE = './src/llvm_passes_opt.rs'

LOG_FILE = './passes_optimizer.log'

INITIAL_PASSES = '''
add_early_cse_pass
add_scalar_repl_aggregates_pass_ssa
add_global_optimizer_pass
add_aggressive_dce_pass
add_loop_unroll_and_jam_pass
add_ipsccp_pass
add_function_inlining_pass
add_early_cse_pass
add_loop_deletion_pass
add_constant_merge_pass
add_dead_store_elimination_pass
add_constant_merge_pass
add_function_inlining_pass
add_instruction_combining_pass
add_scalar_repl_aggregates_pass
add_tail_call_elimination_pass
add_promote_memory_to_register_pass
add_jump_threading_pass
add_ipsccp_pass
add_loop_rotate_pass
add_loop_vectorize_pass
add_scalar_repl_aggregates_pass_ssa
add_strip_dead_prototypes_pass
add_always_inliner_pass
add_always_inliner_pass
add_strip_dead_prototypes_pass
add_scoped_no_alias_aa_pass
add_slp_vectorize_pass
add_sccp_pass
add_dead_store_elimination_pass
add_aggressive_dce_pass
add_ipsccp_pass
add_simplify_lib_calls_pass
add_instruction_combining_pass
add_ind_var_simplify_pass
add_aggressive_inst_combiner_pass
add_dead_store_elimination_pass
add_always_inliner_pass
add_early_cse_pass
add_scalar_repl_aggregates_pass_ssa
add_function_attrs_pass
add_simplify_lib_calls_pass
add_sccp_pass
add_loop_vectorize_pass
add_global_optimizer_pass
add_aggressive_dce_pass
add_promote_memory_to_register_pass
add_ipsccp_pass
add_loop_idiom_pass
add_bit_tracking_dce_pass
add_instruction_simplify_pass
add_simplify_lib_calls_pass
add_strip_dead_prototypes_pass
add_type_based_alias_analysis_pass
add_ind_var_simplify_pass
add_strip_dead_prototypes_pass
add_global_optimizer_pass
add_promote_memory_to_register_pass
add_basic_alias_analysis_pass
add_slp_vectorize_pass
add_aggressive_inst_combiner_pass
add_instruction_simplify_pass
add_basic_alias_analysis_pass
add_loop_unroll_pass
add_early_cse_pass
add_instruction_simplify_pass
add_correlated_value_propagation_pass
add_loop_rotate_pass
add_loop_reroll_pass
add_type_based_alias_analysis_pass
add_dead_arg_elimination_pass
add_scalar_repl_aggregates_pass
add_bit_tracking_dce_pass
add_memcpy_optimize_pass
add_memcpy_optimize_pass
add_lower_switch_pass
add_aggressive_dce_pass
add_loop_reroll_pass
add_loop_unroll_and_jam_pass
add_ipsccp_pass
add_merged_load_store_motion_pass
add_loop_vectorize_pass
add_scoped_no_alias_aa_pass
'''
INITIAL_PASSES = INITIAL_PASSES.split('\n')
INITIAL_PASSES = [line.strip() for line in INITIAL_PASSES]
INITIAL_PASSES = [line for line in INITIAL_PASSES if len(line) > 0]

HEADER = '''
// LLVM passes selected by "passes_optimizer.py".

use inkwell::passes::{PassManager, PassManagerSubType};

pub fn add_optimized_optimization_passes<T: PassManagerSubType>(passmgr: &PassManager<T>) {
'''

FOOTER = '''
}
'''

ADD_PASS_FORMAT = 'passmgr.{}();'

ADDED_PASSES_NUM = 10

# All passes
# Exclude:
#  add_scalar_repl_aggregates_pass_with_threshold (because requires parameter),
#  add_internalize_pass (because requires parameter, and breaks program),
#  add_gvn_pass (segfaults),
#  add_new_gvn_pass (breaks program)
#  add_licm_pass (breaks program)
#  add_early_cse_mem_ssa_pass (breaks program (Random module))
#  add_merge_functions_pass (breaks program)
PASSES = '''
add_function_inlining_pass
add_early_cse_pass
add_function_inlining_pass
add_scalar_repl_aggregates_pass_ssa
add_sccp_pass
add_instruction_simplify_pass
add_loop_reroll_pass
add_ipsccp_pass
add_function_inlining_pass
add_loop_unroll_pass
add_correlated_value_propagation_pass
add_strip_dead_prototypes_pass
add_loop_unroll_and_jam_pass
add_scalar_repl_aggregates_pass
add_ind_var_simplify_pass
add_cfg_simplification_pass
add_scalar_repl_aggregates_pass
add_partially_inline_lib_calls_pass
add_instruction_combining_pass
add_reassociate_pass
add_loop_reroll_pass
add_partially_inline_lib_calls_pass
add_aggressive_inst_combiner_pass
add_loop_deletion_pass
add_loop_unroll_pass
add_aggressive_inst_combiner_pass
add_instruction_combining_pass
add_loop_rotate_pass
add_merged_load_store_motion_pass
add_loop_deletion_pass
add_loop_unroll_pass
add_function_inlining_pass
add_loop_idiom_pass
add_slp_vectorize_pass
add_ipsccp_pass
add_early_cse_pass
add_early_cse_pass
add_ind_var_simplify_pass
'''


def get_all_passes():
    passes = []
    for p in PASSES.split('\n'):
        if len(p.strip()) > 0:
            passes.append(p)
    return passes


def write_source_file(passes):
    with open(SOURCE_FILE, 'w') as f:
        f.write(HEADER)
        f.write('\n')
        for p in passes:
            f.write(ADD_PASS_FORMAT.format(p))
            f.write('\n')
        f.write(FOOTER)
        f.write('\n')


def run_benchmark(timeout=10):
    work_dir = "./benchmark/speedtest"
    cp = subprocess.run(['cargo', 'run', '--', 'build'],
                        capture_output=True, text=True, cwd=work_dir)
    if cp.returncode != 0:
        print('build failed.')
        print('stdout:')
        print(cp.stdout)
        print('stderr:')
        print(cp.stderr)
        sys.exit(1)

    try:
        cp = subprocess.run(['python3', './cachegrind-benchmarking/cachegrind.py', './a.out'],
                            capture_output=True, text=True, timeout=timeout, cwd=work_dir)
        if cp.returncode != 0:
            print('run failed.')
            print('stdout:')
            print(cp.stdout)
            print('stderr:')
            print(cp.stderr)
            return None
        else:
            # Split the output by comma and take the second element
            int(cp.output.split(',')[1])

    except subprocess.TimeoutExpired:
        return None


def print_passes(passes):
    print('  ' + ', '.join(passes), flush=True)


def add_log(phase, time, passes):
    passes = [ADD_PASS_FORMAT.format(p) for p in passes]
    with open(LOG_FILE, 'a') as f:
        f.write(f'{phase},{time},"{",".join(passes)}"\n')


def optimize():
    all_passes = get_all_passes()
    optimum_passes = INITIAL_PASSES.copy()

    # Clear log file
    with open(LOG_FILE, 'w') as f:
        f.write('')

    print('Initial passes:')
    print_passes(optimum_passes)

    write_source_file(optimum_passes)
    optimum_time = run_benchmark()
    if optimum_time is None:
        print('Initial benchmark failed.')
        sys.exit(1)

    phase = 0

    while True:
        # add passes
        print(f'Phase {phase}:')
        phase += 1
        added_passes_count = random.randint(1, ADDED_PASSES_NUM)
        added_passes = []
        for _ in range(added_passes_count):
            idx = random.randint(0, len(all_passes)-1)
            added_passes.append(all_passes[idx])
        print('Try adding passes:')
        print_passes(added_passes)
        passes = optimum_passes.copy()
        for p in added_passes:
            passes.append(p)
        write_source_file(passes)
        time = run_benchmark()
        if time is not None and time < optimum_time:
            optimum_passes = passes
            optimum_time = time
            print('New optimum passes found!')
            add_log(phase, time, passes)
        else:
            print('No improvement found.')

        # minimize passes
        print(f'Phase {phase}:')
        phase += 1
        passes = []
        removed_passes = []
        for p in optimum_passes:
            if random.randint(0, int(ceil(len(optimum_passes) / ADDED_PASSES_NUM))) != 0:
                passes.append(p)
            else:
                removed_passes.append(p)
        print('Try removing passes:')
        print_passes(removed_passes)
        write_source_file(passes)
        time = run_benchmark()
        if time <= optimum_time:
            optimum_passes = passes
            optimum_time = time
            print('Minimize success!')
            add_log(phase, time, passes)
        else:
            print('Minimize failed.')

        # print current optimum
        print('Current optimum passes:')
        print_passes(optimum_passes)
        print('Current optimum time: {}'.format(optimum_time))


if __name__ == '__main__':
    optimize()
