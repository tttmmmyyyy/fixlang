#!/usr/bin/env python3
"""Generates programs with gen.py, builds each with the compiler under test and the reference
compiler, runs the first under Valgrind MemCheck and the second natively, and records a finding
where MemCheck reports an error, the program crashes, or its output differs from the reference's.

Usage: run.py FIX_UNDER_TEST SEED_FROM SEED_TO --reference FIX [--out DIR] [--suppressions FILE]
              [--jobs 6] [--stop-on-finding]

The seeds run are SEED_FROM, SEED_FROM + 1, ..., SEED_TO - 1. Each program is kept in
DIR/<seed>/main.fix, and one line per seed is appended to DIR/results.jsonl.
"""
import argparse
import json
import os
import shutil
import sys
from concurrent.futures import ProcessPoolExecutor, as_completed

from common import (DEFAULT_SUPPRESSIONS, MEMCHECK_ERROR_EXIT, SIGILL_EXIT, build,
                    is_inside_repository, memcheck_cmd, resolve_compiler, run)
from gen import Gen

# Each program is run with each of these argument lists; `main` reads their count.
ARG_SETS = [[], ['x'], ['x', 'y', 'z']]


def check(fix, ref, supp, text, d):
    """Checks the program `text` in the directory `d`; returns a result dict."""
    res = {'dir': d}
    ok_t, log_t = build(fix, os.path.join(d, 't'), text, emit_rc_ir=True)
    ok_r, log_r = build(ref, os.path.join(d, 'r'), text)
    if not ok_r:
        res['status'] = 'ref_build_fail'
        res['log'] = log_r
        return res
    if not ok_t:
        res['status'] = 'FINDING_build_fail'
        res['log'] = log_t
        return res
    problems = []
    outs = []
    timed_out = False
    for args in ARG_SETS:
        rc_r, out_r, err_r = run(['./prog'] + args, os.path.join(d, 'r'))
        rc_t, out_t, err_t = run(memcheck_cmd(supp, args), os.path.join(d, 't'))
        outs.append((out_r.strip(), out_t.strip()))
        if 'timeout' in (rc_r, rc_t):
            timed_out = True
            break
        if rc_t == MEMCHECK_ERROR_EXIT:
            problems.append({'args': args, 'kind': 'memcheck', 'stderr': err_t[-4000:]})
        elif rc_t != 0 or rc_r != 0:
            if SIGILL_EXIT in (rc_t, rc_r):
                problems.append({'args': args, 'kind': 'SIGILL_setup', 'rc': [rc_t, rc_r]})
            elif rc_t != rc_r:
                problems.append({'args': args, 'kind': 'exit', 'rc': [rc_t, rc_r], 'stderr': err_t[-2000:]})
        if out_r != out_t:
            problems.append({'args': args, 'kind': 'output', 'ref': out_r[-200:], 'test': out_t[-200:]})
    res['outs'] = outs
    if problems:
        res['status'] = 'FINDING'
        res['problems'] = problems
    elif timed_out:
        res['status'] = 'timeout'
    else:
        res['status'] = 'pass'
        # Keep the source, drop the build products.
        for sub in ('t', 'r'):
            shutil.rmtree(os.path.join(d, sub), ignore_errors=True)
    return res


def one(out, fix, ref, supp, seed):
    d = os.path.join(out, str(seed))
    os.makedirs(d, exist_ok=True)
    text = Gen(seed).program()
    with open(os.path.join(d, 'main.fix'), 'w') as f:
        f.write(text)
    r = check(fix, ref, supp, text, d)
    r['seed'] = seed
    return r


def main():
    ap = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    ap.add_argument('fix', help='the compiler under test')
    ap.add_argument('lo', type=int, help='the first seed')
    ap.add_argument('hi', type=int, help='one past the last seed')
    ap.add_argument('--reference', required=True, help='the reference compiler')
    ap.add_argument('--out', help='the output directory (default: rc-fuzz-out in the current directory)')
    ap.add_argument('--suppressions', default=DEFAULT_SUPPRESSIONS,
                    help="Valgrind's suppression file (default: the repository's valgrind.supp)")
    ap.add_argument('--jobs', type=int, default=6)
    ap.add_argument('--stop-on-finding', action='store_true')
    a = ap.parse_args()
    if a.out is None:
        a.out = os.path.join(os.getcwd(), 'rc-fuzz-out')
        if is_inside_repository(a.out):
            sys.exit('the current directory is inside the repository; name the output directory with --out')
    out = os.path.abspath(a.out)
    fix, ref = resolve_compiler(a.fix), resolve_compiler(a.reference)
    supp = os.path.abspath(a.suppressions)
    os.makedirs(out, exist_ok=True)
    counts = {}
    with ProcessPoolExecutor(a.jobs) as ex, open(os.path.join(out, 'results.jsonl'), 'a') as log:
        futs = [ex.submit(one, out, fix, ref, supp, s) for s in range(a.lo, a.hi)]
        for fu in as_completed(futs):
            r = fu.result()
            log.write(json.dumps(r) + '\n')
            log.flush()
            counts[r['status']] = counts.get(r['status'], 0) + 1
            if r['status'].startswith('FINDING'):
                print('FINDING seed', r['seed'], json.dumps(r.get('problems', r.get('log')))[:300], flush=True)
                if a.stop_on_finding:
                    for f2 in futs:
                        f2.cancel()
                    break
    print('counts', counts, flush=True)


if __name__ == '__main__':
    main()
