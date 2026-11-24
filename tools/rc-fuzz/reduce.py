#!/usr/bin/env python3
"""Line-based reducer for a failing Fix program.

Usage: reduce.py FIX_UNDER_TEST IN.fix OUT.fix --reference FIX [--kind memcheck|output]
                 [--args x ...] [--suppressions FILE] [--jobs 6]

A candidate is interesting when the compiler under test builds it and
- memcheck: Valgrind MemCheck reports an error in the program it built, run with the args; or
- output: its output, run natively, differs from that of the reference compiler's program;
and, in both cases, the reference compiler's program runs clean under MemCheck.

OUT.fix holds the smallest interesting program found so far. The candidates are built in the
directory OUT.fix.work, which is removed at the end.
"""
import argparse
import os
import shutil
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

from common import (DEFAULT_SUPPRESSIONS, MEMCHECK_ERROR_EXIT, SIGILL_EXIT, build, memcheck_cmd,
                    resolve_compiler, run)


def interesting(fix, ref, supp, work, text, kind, args):
    d = tempfile.mkdtemp(dir=work)
    try:
        t, r = os.path.join(d, 't'), os.path.join(d, 'r')
        if not build(fix, t, text)[0]:
            return False
        vg = memcheck_cmd(supp, args)
        if kind == 'memcheck':
            rc, _, _ = run(vg, t)
            if rc != MEMCHECK_ERROR_EXIT:
                return False
        if not build(ref, r, text)[0]:
            return False
        rc_r, out_r, _ = run(vg, r)
        if rc_r != 0:
            return False
        if kind == 'output':
            rc_t, out_t, _ = run(['./prog'] + args, t)
            if rc_t == SIGILL_EXIT or out_t == out_r:
                return False
        return True
    finally:
        shutil.rmtree(d, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('fix', help='the compiler under test')
    ap.add_argument('inp', help='the failing program')
    ap.add_argument('out', help='where to write the reduced program')
    ap.add_argument('--reference', required=True, help='the reference compiler')
    ap.add_argument('--kind', choices=['memcheck', 'output'], default='memcheck')
    ap.add_argument('--args', nargs='*', default=[], help='the arguments to run the program with')
    ap.add_argument('--suppressions', default=DEFAULT_SUPPRESSIONS,
                    help="Valgrind's suppression file (default: the repository's valgrind.supp)")
    ap.add_argument('--jobs', type=int, default=6)
    a = ap.parse_args()
    fix, ref = resolve_compiler(a.fix), resolve_compiler(a.reference)
    supp = os.path.abspath(a.suppressions)
    work = os.path.abspath(a.out) + '.work'
    os.makedirs(work, exist_ok=True)
    try:
        reduce(a, fix, ref, supp, work)
    finally:
        shutil.rmtree(work, ignore_errors=True)


def reduce(a, fix, ref, supp, work):
    with open(a.inp) as f:
        lines = f.read().split('\n')
    test = lambda ls: interesting(fix, ref, supp, work, '\n'.join(ls), a.kind, a.args)
    if not test(lines):
        sys.exit('the input is not interesting')
    ex = ThreadPoolExecutor(a.jobs)
    # First drop whole top-level definitions (blocks separated by blank lines), one at a time.
    k = 0
    while True:
        text = '\n'.join(lines)
        blocks = text.split('\n\n')
        if k >= len(blocks):
            break
        cand = blocks[:k] + blocks[k + 1:]
        if test('\n\n'.join(cand).split('\n')):
            lines = '\n\n'.join(cand).split('\n')
            print('blocks', len(cand), flush=True)
        else:
            k += 1
    # Then drop chunks of lines, halving the chunk size while no chunk can go.
    chunk = max(1, len(lines) // 4)
    while True:
        progress = False
        i = 0
        while i < len(lines):
            # Try the next `jobs` chunks at once; keep the first that stays interesting.
            starts = list(range(i, min(len(lines), i + chunk * a.jobs), chunk))
            cands = [lines[:s] + lines[s + chunk:] for s in starts]
            oks = list(ex.map(test, cands))
            hit = next((k for k, ok in enumerate(oks) if ok), None)
            if hit is None:
                i = starts[-1] + chunk
            else:
                lines = cands[hit]
                i = starts[hit]
                progress = True
                with open(a.out, 'w') as f:
                    f.write('\n'.join(lines))
                print('lines', len(lines), 'chunk', chunk, flush=True)
        if chunk == 1 and not progress:
            break
        if not progress:
            chunk = max(1, chunk // 2)
    ex.shutdown()
    with open(a.out, 'w') as f:
        f.write('\n'.join(lines))
    print('done', len(lines))


if __name__ == '__main__':
    main()
