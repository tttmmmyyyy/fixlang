from math import ceil
import select
import subprocess
import re
import itertools
import sys
import random
from statsmodels.stats.weightstats import ttest_ind
import numpy as np
import math

LLVM_PASSES_MAIN_FILE = 'src/llvm_passes.txt'
LLVM_PASSES_TMP_FILE = 'llvm_passes_tmp.txt'

LOG_FILE = 'passes_optimizer.log'

ADDED_PASSES_NUM = 10

# All passes
# Can be found in: "opt --print-passes"
# See also: https://gist.github.com/gingerBill/d889ae03d429653a4a9081ad6dc2a6c3
# Exclude:
# attributor, attributor-cgscc: may breaks the program
PASSES = '''
aa-eval
adce
add-discriminators
aggressive-instcombine
alignment-from-assumptions
always-inline
annotation-remarks
annotation2metadata
argpromotion
assume-builder
assume-simplify
bdce
bounds-checking
break-crit-edges
called-value-propagation
callsite-splitting
canon-freeze
canonicalize-aliases
check-debugify
chr
consthoist
constmerge
constraint-elimination
coro-cleanup
coro-early
coro-elide
coro-split
correlated-propagation
count-visits
cross-dso-cfi
dce
deadargelim
declare-to-assign
dfa-jump-threading
div-rem-pairs
dse
early-cse
ee-instrument
elim-avail-extern
extract-blocks
fix-irreducible
flattencfg
float2int
forceattrs
function-attrs
globaldce
globalopt
globalsplit
guard-widening
gvn
gvn-hoist
gvn-sink
hardware-loops
hotcoldsplit
indvars
infer-address-spaces
inferattrs
inject-tli-mappings
inline
inliner-wrapper
inliner-wrapper-no-mandatory-first
instcombine
instcount
instnamer
instrorderfile
instrprof
instsimplify
internalize
irce
iroutliner
jump-threading
kcfi
lcssa
libcalls-shrinkwrap
licm
lint
lnicm
load-store-vectorizer
loop-bound-split
loop-data-prefetch
loop-deletion
loop-distribute
loop-extract
loop-flatten
loop-fusion
loop-idiom
loop-instsimplify
loop-interchange
loop-load-elim
loop-predication
loop-reduce
loop-reroll
loop-rotate
loop-simplify
loop-simplifycfg
loop-sink
loop-unroll
loop-unroll-and-jam
loop-unroll-full
loop-vectorize
loop-versioning
loop-versioning-licm
lower-constant-intrinsics
lower-expect
lower-global-dtors
lower-guard-intrinsic
lower-ifunc
lower-matrix-intrinsics
lower-widenable-condition
loweratomic
lowerinvoke
lowerswitch
lowertypetests
make-guards-explicit
mem2reg
memcpyopt
mergefunc
mergeicmps
mergereturn
metarenamer
mldst-motion
module-inline
move-auto-init
name-anon-globals
nary-reassociate
newgvn
no-op-cgscc
no-op-function
no-op-loop
no-op-loopnest
no-op-module
pa-eval
partial-inliner
partially-inline-libcalls
place-safepoints
poison-checking
pseudo-probe
pseudo-probe-update
reassociate
recompute-globalsaa
redundant-dbg-inst-elim
reg2mem
rel-lookup-table-converter
rewrite-statepoints-for-gc
rewrite-symbols
rpo-function-attrs
sancov-module
sanmd-module
scalarize-masked-mem-intrin
scalarizer
scc-oz-module-inliner
sccp
separate-const-offset-from-gep
simple-loop-unswitch
simplifycfg
sink
slp-vectorizer
slsr
speculative-execution
sroa
strip-dead-prototypes
structurizecfg
synthetic-counts-propagation
tailcallelim
tlshoist
transform-warning
typepromotion
unify-loop-exits
vector-combine
wholeprogramdevirt
'''


class BenchmarkResult:
    def __init__(self, scores):
        self.scores = scores

    def __str__(self):
        sum_scores = sum(self.scores)
        return f'BenchmarkResult(sum_scores={sum_scores})'

    def __repr__(self):
        return self.__str__()

    def __lt__(self, other):
        """
        True iff prod(self.scores / other.scores) < 1
        """
        if len(self.scores) != len(other.scores):
            raise ValueError(
                "self.scores and other.scores must have the same length.")

        ratios = [s / o for s, o in zip(self.scores, other.scores)]

        product_of_ratios = math.prod(ratios)

        return product_of_ratios < 1


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


def run_benchmark(timeout=60):
    work_dir = "./benchmark/speedtest"
    llvm_passes_file_path = '../../' + LLVM_PASSES_TMP_FILE

    try:
        cp = subprocess.run(['./a.out', '--no-save', '--llvm-passes-file', llvm_passes_file_path],
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
            costs = cp.stdout.strip().split(',')
            print('Benchmark result:', costs)
            # remove the first element (it is not a number)
            return BenchmarkResult([float(x) for x in costs[1:]])

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
        if time is not None and not (time > optimum_time):
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
        print('Current optimum cost: {}'.format(optimum_time))


if __name__ == '__main__':
    optimize()
