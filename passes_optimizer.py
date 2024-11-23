from math import ceil
import select
import subprocess
import re
import itertools
import sys
import random
from statsmodels.stats.weightstats import ttest_ind
import numpy as np

SOURCE_FILE = './src/llvm_passes_optimized.txt'

LOG_FILE = './passes_optimizer.log'

ADDED_PASSES_NUM = 10

# All passes
# Exclude:
#  scalar_repl_aggregates_pass_with_threshold (because requires parameter),
#  internalize_pass (because requires parameter, and breaks program),
#  strip_symbol_pass (because added by default),
#  verifier_pass (because added by default),
#  gvn_pass (segfaults),
#  new_gvn_pass (breaks program)
#  licm_pass (breaks program)
#  early_cse_mem_ssa_pass (breaks program (Random module))
#  merge_functions_pass (breaks program)
PASSES = '''
aggressive_dce_pass
aggressive_inst_combiner_pass
alignment_from_assumptions_pass
always_inliner_pass
argument_promotion_pass
basic_alias_analysis_pass
bit_tracking_dce_pass
cfg_simplification_pass
constant_merge_pass
coroutine_cleanup_pass
coroutine_early_pass
coroutine_elide_pass
coroutine_split_pass
correlated_value_propagation_pass
dead_arg_elimination_pass
dead_store_elimination_pass
demote_memory_to_register_pass
early_cse_pass
function_attrs_pass
function_inlining_pass
global_dce_pass
global_optimizer_pass
ind_var_simplify_pass
instruction_combining_pass
instruction_simplify_pass
ipsccp_pass
jump_threading_pass
loop_deletion_pass
loop_idiom_pass
loop_reroll_pass
loop_rotate_pass
loop_unroll_and_jam_pass
loop_unroll_pass
loop_unswitch_pass
loop_vectorize_pass
lower_expect_intrinsic_pass
lower_switch_pass
memcpy_optimize_pass
merged_load_store_motion_pass
partially_inline_lib_calls_pass
promote_memory_to_register_pass
prune_eh_pass
reassociate_pass
scalar_repl_aggregates_pass
scalar_repl_aggregates_pass_ssa
scalarizer_pass
sccp_pass
scoped_no_alias_aa_pass
simplify_lib_calls_pass
slp_vectorize_pass
strip_dead_prototypes_pass
tail_call_elimination_pass
type_based_alias_analysis_pass
'''


def get_all_passes():
    passes = []
    for p in PASSES.split('\n'):
        if len(p.strip()) > 0:
            passes.append(p)
    return passes


def write_source_file(passes):
    with open(SOURCE_FILE, 'w') as f:
        for p in passes:
            f.write(p)
            f.write('\n')


def install_fix():
    subprocess.run(['cargo', 'install', '--locked',
                   '--path', '.'], capture_output=True)


def run_benchmark(timeout=10):
    work_dir = "./benchmark/speedtest"
    cp = subprocess.run(['fix', 'build', '--llvm-passes-file', '../../' + SOURCE_FILE],
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
            return int(cp.stdout.strip().split(',')[1])

    except subprocess.TimeoutExpired:
        return None


def print_passes(passes):
    print('  ' + ', '.join(passes), flush=True)


def add_log(phase, time, passes):
    with open(LOG_FILE, 'a') as f:
        f.write(f'{phase},{time},"{",".join(passes)}"\n')


def optimize():
    install_fix()

    all_passes = get_all_passes()

    # Read the initial passes from the source file
    with open(SOURCE_FILE, 'r') as f:
        initial_passes = f.readlines()
        initial_passes = [line.strip() for line in initial_passes]

    optimum_passes = initial_passes

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
        # If any character with newline is pressed, break the loop
        readable, _, _ = select.select([sys.stdin], [], [], 0.1)
        if readable:
            user_input = sys.stdin.readline().strip()
            if len(user_input) > 0:
                break

        # add passes
        print(f'\nPhase {phase}:')
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
        print(f'\nPhase {phase}:')
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
        if time is not None and time <= optimum_time:
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

    # Write the final optimum passes to the source file
    write_source_file(optimum_passes)


if __name__ == '__main__':
    optimize()
