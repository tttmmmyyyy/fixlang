from math import ceil
import select
import subprocess
import re
import itertools
import sys
import random
from statsmodels.stats.weightstats import ttest_ind
import numpy as np

LLVM_PASSES_MAIN_FILE = './src/llvm_passes.txt'
LLVM_PASSES_TMP_FILE = './llvm_passes_tmp.txt'

LOG_FILE = './passes_optimizer.log'

ADDED_PASSES_NUM = 10

# All passes
# Can be found in: "opt --print-passes"
# See also: https://gist.github.com/gingerBill/d889ae03d429653a4a9081ad6dc2a6c3
# Exclude:
#  scalar_repl_aggregates_pass_with_threshold (because requires parameter),
#  internalize_pass (because requires parameter, and breaks program),
#  strip_symbol_pass (because added by default),
#  verifier_pass (because added by default),
PASSES = '''
aa-eval
basic-aa
basiccg
da
domfrontier
domtree
globals-aa
instcount
iv-users
lazy-value-info
loops
memdep
postdomtree
regions
scalar-evolution
scev-aa
stack-safety
adce
always-inline
argpromotion
block-placement
break-crit-edges
codegenprepare
constmerge
dce
deadargelim
dse
function-attrs
globaldce
globalopt
gvn
indvars
inline
instcombine
aggressive-instcombine
internalize
ipsccp
jump-threading
lcssa
licm
loop-deletion
loop-extract
loop-reduce
loop-rotate
loop-simplify
loop-unroll
loop-unroll-and-jam
lower-global-dtors
lower-atomic
lower-invoke
lower-switch
mem2reg
memcpyopt
mergefunc
mergereturn
partial-inliner
reassociate
rel-lookup-table-converter
reg2mem
sroa
sccp
simplifycfg
sink
simple-loop-unswitch
strip
strip-dead-prototypes
tailcallelim
'''


def get_all_passes():
    passes = []
    for p in PASSES.split('\n'):
        if len(p.strip()) > 0:
            passes.append(p)
    return passes


def write_llvm_passes_file(passes, file_type):
    if file_type == 'main':
        path = LLVM_PASSES_MAIN_FILE
    elif file_type == 'tmp':
        path = LLVM_PASSES_TMP_FILE
    else:
        # Print error message
        print('Invalid file type: {}'.format(file_type))
        exit(1)

    print('Writing passes to {}'.format(path))
    with open(path, 'w') as f:
        for p in passes:
            f.write(p)
            f.write('\n')


def install_fix():
    subprocess.run(['cargo', 'install', '--locked',
                   '--path', '.'], capture_output=True)


def run_benchmark(timeout=10):
    work_dir = "./benchmark/speedtest"
    cp = subprocess.run(['fix', 'build', '-O', 'experimental', '--llvm-passes-file', '../../' + LLVM_PASSES_TMP_FILE],
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
    with open(LLVM_PASSES_MAIN_FILE, 'r') as f:
        initial_passes = f.readlines()
        initial_passes = [line.strip() for line in initial_passes]

    optimum_passes = initial_passes

    # Clear log file
    with open(LOG_FILE, 'w') as f:
        f.write('')

    print('Initial passes:')
    print_passes(optimum_passes)

    write_llvm_passes_file(optimum_passes, 'tmp')
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
        write_llvm_passes_file(passes, 'tmp')
        time = run_benchmark()
        if time is not None and time < optimum_time:
            optimum_passes = passes
            optimum_time = time
            print('New optimum passes found!')
            add_log(phase, time, passes)
            # Write the optimum passes to the main file
            write_llvm_passes_file(passes, 'main')
        else:
            print('No improvement found.')

        # minimize passes

        # Remove one pass randomly
        remove_idx = random.randint(0, len(optimum_passes) - 1)
        removed_passes = [optimum_passes[remove_idx]]
        passes = [optimum_passes[i]
                  for i in range(len(optimum_passes)) if i != remove_idx]

        # If no passes are removed, skip the minimize phase.
        if removed_passes == []:
            continue

        print(f'\nPhase {phase}:')
        phase += 1
        print('Try removing passes:')
        print_passes(removed_passes)
        write_llvm_passes_file(passes, 'tmp')
        time = run_benchmark()
        if time is not None and time <= optimum_time:
            optimum_passes = passes
            optimum_time = time
            print('Minimize success!')
            add_log(phase, time, passes)
            # Write the optimum passes to the main file
            write_llvm_passes_file(passes, 'main')
        else:
            print('Minimize failed.')

        # print current optimum
        print('Current optimum passes:')
        print_passes(optimum_passes)
        print('Current optimum time: {}'.format(optimum_time))


if __name__ == '__main__':
    optimize()
