"""Building and running a generated program, shared by run.py and reduce.py."""
import os
import shutil
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.dirname(os.path.dirname(HERE))
DEFAULT_SUPPRESSIONS = os.path.join(REPO_ROOT, 'valgrind.supp')

# Valgrind does not implement AVX-512 or GFNI, so a program using them dies of SIGILL under it.
BUILD_FLAGS = ['-O', 'max', '--disable-cpu-feature', 'avx512.*', '--disable-cpu-feature', 'gfni']
BUILD_TIMEOUT = 600
RUN_TIMEOUT = 120

# The exit code MemCheck gives the program's process when it reports an error.
MEMCHECK_ERROR_EXIT = 99
SIGILL_EXIT = 132


def build(fix, d, text, emit_rc_ir=False):
    """Writes `text` to `d/main.fix` and builds it with `fix` into `d/prog`. Returns whether the
    build succeeded, and the tail of the compiler's output."""
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, 'main.fix'), 'w') as f:
        f.write(text)
    shutil.rmtree(os.path.join(d, '.fixlang'), ignore_errors=True)
    cmd = [fix, 'build', '-f', 'main.fix', '-o', 'prog'] + BUILD_FLAGS
    if emit_rc_ir:
        cmd += ['--emit-rc-ir', 'all']
    try:
        p = subprocess.run(cmd, cwd=d, capture_output=True, text=True, timeout=BUILD_TIMEOUT)
    except subprocess.TimeoutExpired:
        return False, 'build timeout'
    ok = p.returncode == 0 and os.path.exists(os.path.join(d, 'prog'))
    return ok, (p.stdout + p.stderr)[-3000:]


def run(cmd, cwd):
    """Runs `cmd` in `cwd`. Returns the exit code (or 'timeout'), stdout and stderr."""
    try:
        p = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=RUN_TIMEOUT)
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return 'timeout', '', ''


def memcheck_cmd(suppressions, args):
    """The command running `./prog` with `args` under MemCheck."""
    return ['valgrind', '-q', '--error-exitcode=%d' % MEMCHECK_ERROR_EXIT,
            '--suppressions=' + suppressions, './prog'] + args


def resolve_compiler(name):
    """The absolute path of the compiler `name`, a path or a command found on PATH."""
    found = shutil.which(name)
    if found is None:
        sys.exit('compiler not found: %s' % name)
    return os.path.abspath(found)


def is_inside_repository(path):
    """Whether `path` lies in this repository's working tree."""
    path = os.path.realpath(path)
    root = os.path.realpath(REPO_ROOT)
    return os.path.commonpath([path, root]) == root
