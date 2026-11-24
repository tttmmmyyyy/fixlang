#!/usr/bin/env python3
"""Generates a small deterministic Fix program from a seed. The program reads values out of boxed
containers, updates the containers (in place or after a copy), and uses the values read; it
prints one checksum of the values alive at the end of `main`.

Usage: gen.py SEED > main.fix
"""
import os
import random
import sys

# FUZZ_DENSE=1 weights the statements toward reads followed by updates, branches and calls.
DENSE = os.environ.get('FUZZ_DENSE') == '1'

# Type codes and their Fix spellings.
FIX_TY = {
    'I': 'I64',
    'A': 'Array I64',
    'AA': 'Array (Array I64)',
    'S': 'S',
    'T': 'T',
    'U': 'U',
    'P': '(Array I64, S)',
    'AS': 'Array S',
}
BOXED = ['A', 'AA', 'S', 'T', 'U', 'P', 'AS']  # types holding a reference-counted object
ALL = ['I'] + BOXED
CK = {'I': '', 'A': 'sum_a', 'AA': 'sum_aa', 'S': 'sum_s', 'T': 'sum_t', 'U': 'sum_u',
      'P': 'sum_p', 'AS': 'sum_as'}

PRELUDE = r'''module Main;

type S = box struct { arr : Array I64, n : I64 };
type T = box struct { inner : S, arrs : Array (Array I64), k : I64, tag : Array I64 };
type U = struct { ua : Array I64, us : S };

// The helpers are recursive, so the inliner keeps the calls to them.
fresh : I64 -> Array I64;
fresh = |n| if n <= 1 { [n + 1] } else { fresh(n - 1).push_back(n % 97) };

sum_a_go : I64 -> I64 -> Array I64 -> I64;
sum_a_go = |i, acc, a| if i >= a.@size { acc } else { sum_a_go(i + 1, (acc * 31 + a.@(i)) % 1000003, a) };

sum_a : Array I64 -> I64;
sum_a = |a| sum_a_go(0, 7, a);

sum_aa_go : I64 -> I64 -> Array (Array I64) -> I64;
sum_aa_go = |i, acc, aa| if i >= aa.@size { acc } else { sum_aa_go(i + 1, (acc * 37 + sum_a(aa.@(i))) % 1000003, aa) };

sum_aa : Array (Array I64) -> I64;
sum_aa = |aa| sum_aa_go(0, 11, aa);

sum_s : S -> I64;
sum_s = |s| (sum_a(s.@arr) * 7 + s.@n) % 1000003;

sum_t : T -> I64;
sum_t = |t| (sum_s(t.@inner) * 13 + sum_aa(t.@arrs) * 5 + t.@k * 3 + sum_a(t.@tag)) % 1000003;

sum_u : U -> I64;
sum_u = |u| (sum_a(u.@ua) * 17 + sum_s(u.@us)) % 1000003;

sum_p : (Array I64, S) -> I64;
sum_p = |p| (sum_a(p.@0) * 19 + sum_s(p.@1)) % 1000003;

sum_as_go : I64 -> I64 -> Array S -> I64;
sum_as_go = |i, acc, xs| if i >= xs.@size { acc } else { sum_as_go(i + 1, (acc * 41 + sum_s(xs.@(i))) % 1000003, xs) };

sum_as : Array S -> I64;
sum_as = |xs| sum_as_go(0, 3, xs);

// Consumes the array and updates it, in place where it is unique.
grow_a : I64 -> Array I64 -> Array I64;
grow_a = |k, a| if k <= 0 { a } else { grow_a(k - 1, a.push_back(k % 5).set(0, (a.@(0) + k) % 1009)) };

// Consumes the struct and updates its array.
bump_s : I64 -> S -> S;
bump_s = |k, s| if k <= 0 { s } else { bump_s(k - 1, s.mod_arr(|a| a.set(0, (a.@(0) + 1) % 1009)).set_n((s.@n + k) % 1009)) };

// Consumes the struct: takes it apart and consumes its array.
eat_s : S -> I64;
eat_s = |s| (
    let S { arr : a, n : m } = s;
    (sum_a(grow_a(m % 2 + 1, a)) + m) % 1000003
);

bump_t : I64 -> T -> T;
bump_t = |k, t| if k <= 0 { t } else { bump_t(k - 1, t.mod_inner(bump_s(1)).mod_arrs(|aa| aa.mod(0, grow_a(1))).set_k((t.@k + 1) % 1009)) };

bump_aa : I64 -> Array (Array I64) -> Array (Array I64);
bump_aa = |k, aa| if k <= 0 { aa } else { bump_aa(k - 1, aa.mod(k % aa.@size, grow_a(1))) };

// Returns an element, consuming the array of arrays.
pick_a : I64 -> Array (Array I64) -> Array I64;
pick_a = |i, aa| if i <= 0 { aa.@(0) } else { pick_a(i - 1, aa) };

// Returns the struct's array, consuming the struct.
take_arr : I64 -> S -> Array I64;
take_arr = |i, s| if i <= 0 { s.@arr } else { take_arr(i - 1, s) };

// Reads the array and the struct, and returns a value depending on both.
peek2 : I64 -> Array I64 -> S -> I64;
peek2 = |i, a, s| if i <= 0 { (a.@(0) * 3 + s.@arr.@(0) + a.@size) % 1009 } else { peek2(i - 1, a, s) };
'''


class Gen:
    def __init__(self, seed):
        self.r = random.Random(seed)
        self.counter = 0
        self.funcs = []  # (name, has_depth, param types, result types)
        self.out_funcs = []

    def fresh_name(self):
        self.counter += 1
        return 'v%d' % self.counter

    def chance(self, p):
        return self.r.random() < p

    def choice(self, xs):
        return self.r.choice(xs)

    # ---- small expressions -----------------------------------------------------------------

    def vars_of(self, env, ty):
        return [v for v, t in env.items() if t == ty]

    def pick_var(self, env, ty):
        vs = self.vars_of(env, ty)
        return self.choice(vs) if vs else None

    def int_atom(self, env):
        vs = self.vars_of(env, 'I')
        if vs and self.chance(0.7):
            return self.choice(vs)
        return str(self.r.randint(0, 5))

    def index(self, env, arr_expr):
        if self.chance(0.4):
            return '0'
        return '%s %% %s.@size' % (self.int_atom(env), arr_expr)

    def cond(self, env):
        ivs = self.vars_of(env, 'I')
        k = self.r.randint(0, 3)
        if ivs and self.chance(0.8):
            v = self.choice(ivs)
            m = self.r.randint(2, 3)
            return '%s %% %d == %d' % (v, m, self.r.randint(0, m - 1))
        avs = self.vars_of(env, 'A')
        if avs:
            a = self.choice(avs)
            return '%s.@size > %d' % (a, k + 1)
        return 'true'

    def consume(self, env, v, p=0.6):
        """Consume `v` (drop it from scope) with probability p."""
        if v in env and self.chance(p):
            del env[v]

    def make_value(self, ty, env, depth):
        """An expression of type `ty` built from scratch or from variables in `env`. Returns the
        expression; may drop consumed variables from env."""
        r = self.r
        if ty == 'I':
            opts = ['lit', 'arith', 'size', 'elem', 'sum', 'n', 'eat', 'peek']
            o = self.choice(opts)
            if o == 'lit':
                return str(r.randint(0, 9))
            if o == 'arith':
                return '(%s + %s * %d) %% 1009' % (self.int_atom(env), self.int_atom(env), r.randint(1, 4))
            if o == 'size':
                a = self.pick_var(env, 'A')
                if a:
                    return '%s.@size' % a
            if o == 'elem':
                a = self.pick_var(env, 'A')
                if a:
                    return '%s.@(%s)' % (a, self.index(env, a))
                aa = self.pick_var(env, 'AA')
                if aa:
                    return '%s.@(%s).@(0)' % (aa, self.index(env, aa))
            if o == 'n':
                s = self.pick_var(env, 'S')
                if s:
                    return '%s.@n' % s
                t = self.pick_var(env, 'T')
                if t:
                    return '%s.@k' % t
            if o == 'sum':
                tys = [t for t in BOXED if self.vars_of(env, t)]
                if tys:
                    t = self.choice(tys)
                    return '%s(%s)' % (CK[t], self.pick_var(env, t))
            if o == 'eat':
                s = self.pick_var(env, 'S')
                if s:
                    self.consume(env, s, 0.7)
                    return 'eat_s(%s)' % s
            if o == 'peek':
                a = self.pick_var(env, 'A')
                s = self.pick_var(env, 'S')
                if a and s:
                    return 'peek2(%s %% 2, %s, %s)' % (self.int_atom(env), a, s)
            return str(r.randint(0, 9))
        if ty == 'A':
            o = self.choice(['lit', 'fresh', 'from_map', 'read_s', 'read_aa', 'read_p', 'read_u',
                             'read_t', 'upd', 'upd', 'grow', 'pick', 'take', 'var'])
            if o == 'lit':
                return '[%s, %d]' % (self.int_atom(env), r.randint(0, 9))
            if o == 'fresh':
                return 'fresh(%s %% 3 + 1)' % self.int_atom(env)
            if o == 'from_map':
                return 'Array::from_map(%s %% 3 + 1, |j| (j + %s) %% 100)' % (self.int_atom(env), self.int_atom(env))
            if o == 'read_s':
                s = self.pick_var(env, 'S')
                if s:
                    return '%s.@arr' % s
            if o == 'read_aa':
                aa = self.pick_var(env, 'AA')
                if aa:
                    return '%s.@(%s)' % (aa, self.index(env, aa))
            if o == 'read_p':
                p = self.pick_var(env, 'P')
                if p:
                    return '%s.@0' % p
            if o == 'read_u':
                u = self.pick_var(env, 'U')
                if u:
                    return '%s.@ua' % u
            if o == 'read_t':
                t = self.pick_var(env, 'T')
                if t:
                    return self.choice(['%s.@tag', '%s.@inner.@arr', '%s.@arrs.@(0)']) % t
            if o == 'upd':
                a = self.pick_var(env, 'A')
                if a:
                    e = self.choice([
                        '%s.set(%s, %s)' % (a, self.index(env, a), self.int_atom(env)),
                        '%s.push_back(%s)' % (a, self.int_atom(env)),
                        '%s.mod(%s, |e| (e + %s) %% 1009)' % (a, self.index(env, a), self.int_atom(env)),
                    ])
                    self.consume(env, a)
                    return e
            if o == 'grow':
                a = self.pick_var(env, 'A')
                if a:
                    self.consume(env, a)
                    return 'grow_a(%s %% 3, %s)' % (self.int_atom(env), a)
            if o == 'pick':
                aa = self.pick_var(env, 'AA')
                if aa:
                    self.consume(env, aa)
                    return 'pick_a(%s %% 2, %s)' % (self.int_atom(env), aa)
            if o == 'take':
                s = self.pick_var(env, 'S')
                if s:
                    self.consume(env, s)
                    return 'take_arr(%s %% 2, %s)' % (self.int_atom(env), s)
            if o == 'var':
                a = self.pick_var(env, 'A')
                if a:
                    return a
            return '[%d]' % r.randint(0, 9)
        if ty == 'AA':
            o = self.choice(['lit', 'from_map', 'read_t', 'upd', 'upd', 'bump', 'var'])
            if o == 'lit':
                return '[%s, %s]' % (self.make_value('A', env, depth + 1), self.make_value('A', env, depth + 1))
            if o == 'from_map':
                return 'Array::from_map(%s %% 2 + 1, |j| fresh(j + 1))' % self.int_atom(env)
            if o == 'read_t':
                t = self.pick_var(env, 'T')
                if t:
                    return '%s.@arrs' % t
            if o == 'upd':
                aa = self.pick_var(env, 'AA')
                if aa:
                    a = self.pick_var(env, 'A')
                    opts = ['%s.mod(%s, |e| e.push_back(%s))' % (aa, self.index(env, aa), self.int_atom(env)),
                            '%s[0][0].iset(%s)' % (aa, self.int_atom(env)),
                            '%s.mod(%s, |e| e.set(0, (e.@(0) + 1) %% 1009))' % (aa, self.index(env, aa))]
                    if a:
                        opts += ['%s.set(%s, %s)' % (aa, self.index(env, aa), a),
                                 '%s.push_back(%s)' % (aa, a),
                                 '%s.mod(%s, |e| e.set(0, %s.@(0)))' % (aa, self.index(env, aa), a)]
                    e = self.choice(opts)
                    if a:
                        self.consume(env, a, 0.3)
                    self.consume(env, aa)
                    return e
            if o == 'bump':
                aa = self.pick_var(env, 'AA')
                if aa:
                    self.consume(env, aa)
                    return 'bump_aa(%s %% 3, %s)' % (self.int_atom(env), aa)
            if o == 'var':
                aa = self.pick_var(env, 'AA')
                if aa:
                    return aa
            return '[[%d], fresh(2)]' % r.randint(0, 9)
        if ty == 'S':
            o = self.choice(['lit', 'read_t', 'read_p', 'read_u', 'read_as', 'upd', 'upd', 'upd', 'bump', 'var'])
            if o == 'lit':
                return 'S { arr : %s, n : %s }' % (self.make_value('A', env, depth + 1), self.int_atom(env))
            if o == 'read_t':
                t = self.pick_var(env, 'T')
                if t:
                    return '%s.@inner' % t
            if o == 'read_p':
                p = self.pick_var(env, 'P')
                if p:
                    return '%s.@1' % p
            if o == 'read_u':
                u = self.pick_var(env, 'U')
                if u:
                    return '%s.@us' % u
            if o == 'read_as':
                xs = self.pick_var(env, 'AS')
                if xs:
                    return '%s.@(%s)' % (xs, self.index(env, xs))
            if o == 'upd':
                s = self.pick_var(env, 'S')
                if s:
                    a = self.pick_var(env, 'A')
                    opts = ['%s.set_n(%s)' % (s, self.int_atom(env)),
                            '%s.mod_arr(|x| x.push_back(%s))' % (s, self.int_atom(env)),
                            '%s.mod_arr(|x| x.set(0, %s))' % (s, self.int_atom(env)),
                            '%s[^arr][0].iset(%s)' % (s, self.int_atom(env))]
                    if a:
                        opts += ['%s.set_arr(%s)' % (s, a),
                                 '%s.mod_arr(|x| x.set(0, %s.@(0)))' % (s, a)]
                    e = self.choice(opts)
                    if a and ('set_arr(%s)' % a) in e:
                        self.consume(env, a, 0.5)
                    self.consume(env, s)
                    return e
            if o == 'bump':
                s = self.pick_var(env, 'S')
                if s:
                    self.consume(env, s)
                    return 'bump_s(%s %% 3, %s)' % (self.int_atom(env), s)
            if o == 'var':
                s = self.pick_var(env, 'S')
                if s:
                    return s
            return 'S { arr : [%d, 1], n : %d }' % (r.randint(0, 9), r.randint(0, 9))
        if ty == 'T':
            o = self.choice(['lit', 'upd', 'upd', 'upd', 'bump', 'var'])
            if o == 'lit':
                return 'T { inner : %s, arrs : %s, k : %s, tag : %s }' % (
                    self.make_value('S', env, depth + 1), self.make_value('AA', env, depth + 1),
                    self.int_atom(env), self.make_value('A', env, depth + 1))
            if o == 'upd':
                t = self.pick_var(env, 'T')
                if t:
                    s = self.pick_var(env, 'S')
                    a = self.pick_var(env, 'A')
                    opts = ['%s.set_k(%s)' % (t, self.int_atom(env)),
                            '%s.mod_inner(|s| s.set_n(%s))' % (t, self.int_atom(env)),
                            '%s.mod_inner(|s| s.mod_arr(|x| x.push_back(1)))' % t,
                            '%s[^inner][^arr][0].iset(%s)' % (t, self.int_atom(env)),
                            '%s[^arrs][0][0].imod(|e| (e + 1) %% 1009)' % t,
                            '%s.mod_tag(|x| x.push_back(%s))' % (t, self.int_atom(env))]
                    if s:
                        opts += ['%s.set_inner(%s)' % (t, s)]
                    if a:
                        opts += ['%s.set_tag(%s)' % (t, a), '%s.mod_arrs(|aa| aa.push_back(%s))' % (t, a)]
                    e = self.choice(opts)
                    if s and ('(%s)' % s) in e:
                        self.consume(env, s, 0.5)
                    if a and ('(%s)' % a) in e:
                        self.consume(env, a, 0.5)
                    self.consume(env, t)
                    return e
            if o == 'bump':
                t = self.pick_var(env, 'T')
                if t:
                    self.consume(env, t)
                    return 'bump_t(%s %% 2, %s)' % (self.int_atom(env), t)
            if o == 'var':
                t = self.pick_var(env, 'T')
                if t:
                    return t
            return 'T { inner : S { arr : [1], n : 2 }, arrs : [[3]], k : %d, tag : [4, 5] }' % r.randint(0, 9)
        if ty == 'U':
            o = self.choice(['lit', 'upd', 'var'])
            if o == 'lit':
                return 'U { ua : %s, us : %s }' % (self.make_value('A', env, depth + 1), self.make_value('S', env, depth + 1))
            if o == 'upd':
                u = self.pick_var(env, 'U')
                if u:
                    e = self.choice(['%s.mod_ua(|x| x.push_back(%s))' % (u, self.int_atom(env)),
                                     '%s.mod_us(|s| s.set_n(%s))' % (u, self.int_atom(env)),
                                     '%s[^us][^arr][0].iset(%s)' % (u, self.int_atom(env))])
                    self.consume(env, u)
                    return e
            if o == 'var':
                u = self.pick_var(env, 'U')
                if u:
                    return u
            return 'U { ua : [1, 2], us : S { arr : [%d], n : 3 } }' % r.randint(0, 9)
        if ty == 'P':
            o = self.choice(['lit', 'var', 'upd'])
            if o == 'lit':
                return '(%s, %s)' % (self.make_value('A', env, depth + 1), self.make_value('S', env, depth + 1))
            if o == 'upd':
                p = self.pick_var(env, 'P')
                if p:
                    self.consume(env, p)
                    return '%s.mod_1(|s| s.set_n(%s))' % (p, self.int_atom(env))
            if o == 'var':
                p = self.pick_var(env, 'P')
                if p:
                    return p
            return '([1], S { arr : [%d], n : 1 })' % r.randint(0, 9)
        if ty == 'AS':
            o = self.choice(['lit', 'upd', 'upd', 'var'])
            if o == 'lit':
                return '[%s, %s]' % (self.make_value('S', env, depth + 1), self.make_value('S', env, depth + 1))
            if o == 'upd':
                xs = self.pick_var(env, 'AS')
                if xs:
                    s = self.pick_var(env, 'S')
                    opts = ['%s.mod(%s, |s| s.set_n(%s))' % (xs, self.index(env, xs), self.int_atom(env)),
                            '%s.mod(%s, bump_s(1))' % (xs, self.index(env, xs)),
                            '%s[0][^arr][0].iset(%s)' % (xs, self.int_atom(env))]
                    if s:
                        opts += ['%s.set(%s, %s)' % (xs, self.index(env, xs), s), '%s.push_back(%s)' % (xs, s)]
                    e = self.choice(opts)
                    if s and ('%s)' % s) in e:
                        self.consume(env, s, 0.5)
                    self.consume(env, xs)
                    return e
            if o == 'var':
                xs = self.pick_var(env, 'AS')
                if xs:
                    return xs
            return '[S { arr : [%d], n : 1 }]' % r.randint(0, 9)
        raise ValueError(ty)

    # ---- statements ------------------------------------------------------------------------

    def bind(self, env, ty, expr, lines, ind, prefer_shadow=None):
        """Bind `expr` of type `ty` to a variable: a fresh one, or shadowing one of that type."""
        same = self.vars_of(env, ty)
        if same and (prefer_shadow if prefer_shadow is not None else self.chance(0.35)):
            name = self.choice(same)
        else:
            name = self.fresh_name()
        lines.append('%slet %s = %s;' % (ind, name, expr))
        env.pop(name, None)
        env[name] = ty
        return name

    def statement(self, env, lines, ind, depth):
        r = self.r
        if DENSE:
            kinds = ['value'] * 3 + ['pattern'] * 2 + ['read_upd_use'] * 6 + ['call'] * 3
            if depth < 2:
                kinds += ['if'] * 5 + ['match'] * 2 + ['loop'] + ['fold']
        else:
            kinds = ['value'] * 6 + ['pattern'] * 2 + ['read_upd_use'] * 3 + ['call'] * 2
            if depth < 2:
                kinds += ['if'] * 3 + ['match'] + ['loop'] + ['fold']
        k = self.choice(kinds)
        if k == 'value':
            ty = self.choice(ALL)
            e = self.make_value(ty, env, depth)
            self.bind(env, ty, e, lines, ind)
        elif k == 'pattern':
            self.pattern(env, lines, ind)
        elif k == 'read_upd_use':
            self.read_upd_use(env, lines, ind, depth)
        elif k == 'call':
            self.call(env, lines, ind)
        elif k == 'if':
            self.if_stmt(env, lines, ind, depth)
        elif k == 'match':
            self.match_stmt(env, lines, ind, depth)
        elif k == 'loop':
            self.loop_stmt(env, lines, ind, depth)
        elif k == 'fold':
            self.fold_stmt(env, lines, ind, depth)

    def pattern(self, env, lines, ind):
        cands = [t for t in ['S', 'T', 'U', 'P'] if self.vars_of(env, t)]
        if not cands:
            self.bind(env, 'S', self.make_value('S', env, 0), lines, ind)
            return
        ty = self.choice(cands)
        v = self.pick_var(env, ty)
        fields = {'S': [('arr', 'A'), ('n', 'I')],
                  'T': [('inner', 'S'), ('arrs', 'AA'), ('k', 'I'), ('tag', 'A')],
                  'U': [('ua', 'A'), ('us', 'S')],
                  'P': [(None, 'A'), (None, 'S')]}[ty]
        binds = []
        for f, fty in fields:
            if self.chance(0.75):
                nm = self.fresh_name()
                binds.append((f, nm, fty))
            else:
                binds.append((f, '_', fty))
        if all(b[1] == '_' for b in binds):
            nm = self.fresh_name()
            binds[0] = (binds[0][0], nm, binds[0][2])
        if ty == 'P':
            pat = '(%s)' % ', '.join(b[1] for b in binds)
        else:
            named = [b for b in binds if b[1] != '_']
            pat = '%s { %s%s }' % (ty, ', '.join('%s : %s' % (b[0], b[1]) for b in named),
                                   ', _' if len(named) < len(binds) else '')
        lines.append('%slet %s = %s;' % (ind, pat, v))
        for f, nm, fty in binds:
            if nm != '_':
                env[nm] = fty
        self.consume(env, v, 0.3)

    def read_upd_use(self, env, lines, ind, depth):
        """Reads a value out of a container, updates the container (shadowing it, so that the
        update can run in place, or keeping both), then uses the value read."""
        r = self.r
        shapes = []
        if self.vars_of(env, 'S'):
            shapes.append('S')
        if self.vars_of(env, 'AA'):
            shapes.append('AA')
        if self.vars_of(env, 'T'):
            shapes.append('T')
        if self.vars_of(env, 'AS'):
            shapes.append('AS')
        if self.vars_of(env, 'U'):
            shapes.append('U')
        if not shapes:
            self.bind(env, 'S', self.make_value('S', env, depth), lines, ind)
            return
        sh = self.choice(shapes)
        c = self.pick_var(env, sh)
        x = self.fresh_name()
        if sh == 'S':
            if self.chance(0.5):
                lines.append('%slet %s = %s.@arr;' % (ind, x, c))
            else:
                lines.append('%slet S { arr : %s, _ } = %s;' % (ind, x, c))
            xty = 'A'
            upd = self.choice(['%s.set_arr(%s)' % (c, self.make_value('A', dict(env), depth)),
                               '%s.mod_arr(|y| y.set(0, %s))' % (c, self.int_atom(env)),
                               '%s.mod_arr(|y| y.push_back(%s))' % (c, self.int_atom(env)),
                               'bump_s(%s %% 2 + 1, %s)' % (self.int_atom(env), c),
                               '%s.set_n(%s)' % (c, self.int_atom(env))])
        elif sh == 'AA':
            lines.append('%slet %s = %s.@(%s);' % (ind, x, c, self.index(env, c)))
            xty = 'A'
            upd = self.choice(['%s.set(0, %s)' % (c, self.make_value('A', dict(env), depth)),
                               '%s.mod(0, |y| y.set(0, %s))' % (c, self.int_atom(env)),
                               'bump_aa(%s %% 2 + 1, %s)' % (self.int_atom(env), c),
                               '%s.push_back(fresh(2))' % c])
        elif sh == 'T':
            f = self.choice(['inner', 'arrs', 'tag'])
            if self.chance(0.5):
                lines.append('%slet %s = %s.@%s;' % (ind, x, c, f))
            else:
                lines.append('%slet T { %s : %s, _ } = %s;' % (ind, f, x, c))
            xty = {'inner': 'S', 'arrs': 'AA', 'tag': 'A'}[f]
            upd = self.choice(['%s.set_%s(%s)' % (c, f, self.make_value(xty, dict(env), depth)),
                               'bump_t(%s %% 2 + 1, %s)' % (self.int_atom(env), c),
                               '%s[^inner][^arr][0].iset(%s)' % (c, self.int_atom(env)),
                               '%s.mod_tag(|y| y.set(0, %s))' % (c, self.int_atom(env))])
        elif sh == 'AS':
            lines.append('%slet %s = %s.@(%s);' % (ind, x, c, self.index(env, c)))
            xty = 'S'
            upd = self.choice(['%s.set(0, %s)' % (c, self.make_value('S', dict(env), depth)),
                               '%s.mod(0, |s| s.mod_arr(|y| y.set(0, %s)))' % (c, self.int_atom(env)),
                               '%s.mod(0, bump_s(1))' % c])
        else:  # U (unboxed struct holding a boxed struct)
            if self.chance(0.5):
                lines.append('%slet %s = %s.@us;' % (ind, x, c))
            else:
                lines.append('%slet U { us : %s, _ } = %s;' % (ind, x, c))
            xty = 'S'
            upd = self.choice(['%s.set_us(%s)' % (c, self.make_value('S', dict(env), depth)),
                               '%s.mod_us(bump_s(1))' % c,
                               '%s[^us][^arr][0].iset(%s)' % (c, self.int_atom(env))])
        env[x] = xty
        # Update the container: shadow it (unique, in place) or keep both.
        if self.chance(0.7):
            lines.append('%slet %s = %s;' % (ind, c, upd))
        else:
            nm = self.fresh_name()
            lines.append('%slet %s = %s;' % (ind, nm, upd))
            env[nm] = sh
            if self.chance(0.5):
                del env[c]
        # Use the value read, sometimes by a statement in between.
        if self.chance(0.5):
            self.statement(env, lines, ind, depth + 1)
        if x in env:
            y = self.fresh_name()
            lines.append('%slet %s = %s;' % (ind, y, self.choice(
                ['%s(%s)' % (CK[xty], x)] + (['%s.@(0)' % x, '%s.@size' % x] if xty == 'A' else [])
                + (['%s.@arr.@(0)' % x] if xty == 'S' else []))))
            env[y] = 'I'

    def call(self, env, lines, ind):
        if not self.funcs:
            self.bind(env, 'I', self.make_value('I', env, 0), lines, ind)
            return
        name, has_d, ptys, rtys = self.choice(self.funcs)
        args = []
        if has_d:
            args.append('%s %% 2' % self.int_atom(env))
        consumed = []
        for pt in ptys:
            v = self.pick_var(env, pt)
            if v is None or self.chance(0.2):
                args.append(self.make_value(pt, env, 2))
            else:
                args.append(v)
                consumed.append(v)
        for v in consumed:
            self.consume(env, v, 0.5)
        names = [self.fresh_name() for _ in rtys]
        lines.append('%slet (%s) = %s(%s);' % (ind, ', '.join(names), name, ', '.join(args)))
        for nm, t in zip(names, rtys):
            env[nm] = t

    def result_expr(self, ty, env, depth):
        """An expression of type `ty` ending a block: pass a variable through, rebuild, or call."""
        vs = self.vars_of(env, ty)
        if vs and self.chance(0.65):
            # Prefer the variables bound last, so that what the block computed is used.
            return vs[-1] if self.chance(0.6) else self.choice(vs)
        return self.make_value(ty, env, depth + 1)

    def block(self, env, ind, depth, rtys, nstmts=None):
        """Lines of a block in scope `env` (copied) ending in a tuple (or a value) of `rtys`."""
        env = dict(env)
        lines = []
        n = nstmts if nstmts is not None else self.r.randint(0, 3)
        for _ in range(n):
            self.statement(env, lines, ind, depth)
        res = [self.result_expr(t, env, depth) for t in rtys]
        tail = res[0] if len(res) == 1 else '(%s)' % ', '.join(res)
        return lines, tail

    def pick_result_types(self, env):
        boxed_in_env = [t for t in BOXED if self.vars_of(env, t)]
        t0 = self.choice(boxed_in_env) if boxed_in_env and self.chance(0.8) else self.choice(BOXED)
        t1 = self.choice(ALL)
        return [t0, t1] if self.chance(0.75) else [t0]

    def emit_block(self, lines, ind, blines, tail):
        lines.extend(blines)
        lines.append('%s%s' % (ind, tail))

    def bind_results(self, env, rtys, lines, ind, rhs_lines_fn):
        """Binds the result of a compound expression whose code `rhs_lines_fn` writes."""
        if len(rtys) == 1:
            nm = self.fresh_name() if self.chance(0.6) or not self.vars_of(env, rtys[0]) else self.choice(self.vars_of(env, rtys[0]))
            lines.append('%slet %s = ' % (ind, nm) + rhs_lines_fn())
            env[nm] = rtys[0]
            return
        names = [self.fresh_name() for _ in rtys]
        if self.chance(0.5):
            r = self.fresh_name()
            lines.append('%slet %s = ' % (ind, r) + rhs_lines_fn())
            lines.append('%slet (%s) = %s;' % (ind, ', '.join(names), r))
        else:
            lines.append('%slet (%s) = ' % (ind, ', '.join(names)) + rhs_lines_fn())
        for nm, t in zip(names, rtys):
            env[nm] = t

    def if_stmt(self, env, lines, ind, depth):
        rtys = self.pick_result_types(env)
        c = self.cond(env)
        i2 = ind + '    '
        b1 = self.block(env, i2, depth + 1, rtys)
        b2 = self.block(env, i2, depth + 1, rtys, nstmts=self.r.randint(1, 4))
        if self.chance(0.5):
            b1, b2 = b2, b1

        def rhs():
            out = ['if %s {' % c]
            sub = []
            self.emit_block(sub, i2, *b1)
            out += sub
            out.append('%s} else {' % ind)
            sub = []
            self.emit_block(sub, i2, *b2)
            out += sub
            out.append('%s};' % ind)
            return '\n'.join(out)
        self.drop_used_in_branches(env)
        self.bind_results(env, rtys, lines, ind, rhs)

    def drop_used_in_branches(self, env):
        # Variables used in a branch remain valid afterwards; drop some to end their lives.
        for v in list(env):
            if env[v] != 'I' and self.chance(0.25):
                del env[v]

    def match_stmt(self, env, lines, ind, depth):
        boxed_in_env = [t for t in BOXED if self.vars_of(env, t)]
        if not boxed_in_env:
            self.if_stmt(env, lines, ind, depth)
            return
        oty = self.choice(boxed_in_env)
        ov = self.pick_var(env, oty)
        c = self.cond(env)
        rtys = self.pick_result_types(env)
        i2 = ind + '    '
        pv = self.fresh_name()
        env1 = dict(env)
        env1[pv] = oty
        if self.chance(0.5):
            del env1[ov]
        i3 = i2 + '    '
        b1 = self.block(env1, i3, depth + 1, rtys, nstmts=self.r.randint(1, 3))
        b2 = self.block(env, i3, depth + 1, rtys)
        some_src = ov if self.chance(0.6) else self.make_value(oty, dict(env), depth + 1)

        def rhs():
            out = ['match (if %s { some(%s) } else { none() }) {' % (c, some_src)]
            out.append('%ssome(%s) => (' % (i2, pv))
            sub = []
            self.emit_block(sub, i3, *b1)
            out += sub
            out.append('%s),' % i2)
            out.append('%snone() => (' % i2)
            sub = []
            self.emit_block(sub, i3, *b2)
            out += sub
            out.append('%s)' % i2)
            out.append('%s};' % ind)
            return '\n'.join(out)
        self.drop_used_in_branches(env)
        self.bind_results(env, rtys, lines, ind, rhs)

    def loop_stmt(self, env, lines, ind, depth):
        boxed_in_env = [t for t in BOXED if self.vars_of(env, t)]
        if not boxed_in_env:
            self.bind(env, 'A', self.make_value('A', env, depth), lines, ind)
            return
        ty0 = self.choice(boxed_in_env)
        s0 = self.pick_var(env, ty0)
        ty1 = self.choice(ALL)
        s1 = self.pick_var(env, ty1) or self.make_value(ty1, dict(env), depth + 1)
        bound = '%s %% 3 + 1' % self.int_atom(env)
        li, x0, x1 = self.fresh_name(), self.fresh_name(), self.fresh_name()
        benv = dict(env)
        if self.chance(0.5):
            benv.pop(s0, None)
        benv[li] = 'I'
        benv[x0] = ty0
        benv[x1] = ty1
        i2 = ind + '    '
        i3 = i2 + '    '
        blines, tail = self.block(benv, i3, depth + 1, [ty0, ty1], nstmts=self.r.randint(1, 3))
        self.consume(env, s0, 0.6)
        if s1 in env:
            self.consume(env, s1, 0.5)
        names = [self.fresh_name(), self.fresh_name()]
        out = ['%slet (%s) = loop((0, %s, %s), |(%s, %s, %s)|' % (ind, ', '.join(names), s0, s1, li, x0, x1)]
        out.append('%sif %s >= %s { break $ (%s, %s) } else {' % (i2, li, bound, x0, x1))
        out += blines
        out.append('%slet (%s, %s) = %s;' % (i3, x0, x1, tail))
        out.append('%scontinue $ (%s + 1, %s, %s)' % (i3, li, x0, x1))
        out.append('%s});' % i2)
        lines.extend(out)
        env[names[0]] = ty0
        env[names[1]] = ty1

    def fold_stmt(self, env, lines, ind, depth):
        boxed_in_env = [t for t in BOXED if self.vars_of(env, t)]
        if not boxed_in_env:
            self.bind(env, 'A', self.make_value('A', env, depth), lines, ind)
            return
        ty0 = self.choice(boxed_in_env)
        s0 = self.pick_var(env, ty0)
        j, x0 = self.fresh_name(), self.fresh_name()
        benv = dict(env)
        if self.chance(0.5):
            benv.pop(s0, None)
        benv[j] = 'I'
        benv[x0] = ty0
        i2 = ind + '    '
        blines, tail = self.block(benv, i2, depth + 1, [ty0], nstmts=self.r.randint(1, 3))
        self.consume(env, s0, 0.6)
        nm = self.fresh_name()
        out = ['%slet %s = Iterator::range(0, %s %% 3 + 1).fold(%s, |%s, %s| (' % (ind, nm, self.int_atom(env), s0, j, x0)]
        out += blines
        out.append('%s%s' % (i2, tail))
        out.append('%s));' % ind)
        lines.extend(out)
        env[nm] = ty0

    # ---- functions ---------------------------------------------------------------------------

    def gen_function(self, idx):
        name = 'g%d' % idx
        nparams = self.r.randint(1, 3)
        ptys = [self.choice(BOXED) for _ in range(nparams)]
        if self.chance(0.5):
            ptys.append('I')
        rtys = [self.choice(ptys if self.chance(0.7) else BOXED), self.choice(ALL)]
        has_d = self.chance(0.6)
        params = [self.fresh_name() for _ in ptys]
        env = dict(zip(params, ptys))
        ind = '    '
        lines = []
        for _ in range(self.r.randint(2, 6)):
            self.statement(env, lines, ind, 0)
        res = [self.result_expr(t, env, 0) for t in rtys]
        lines.append('%s(%s)' % (ind, ', '.join(res)))
        sig = ' -> '.join((['I64'] if has_d else []) + [FIX_TY[t] for t in ptys] +
                          ['(%s)' % ', '.join(FIX_TY[t] for t in rtys)])
        lam = ', '.join((['d'] if has_d else []) + params)
        out = ['%s : %s;' % (name, sig)]
        if has_d:
            out.append('%s = |%s| (' % (name, lam))
            out.append('    if d > 0 { %s(d - 1, %s) };' % (name, ', '.join(params)))
        else:
            out.append('%s = |%s| (' % (name, lam))
        out += lines
        out.append(');')
        self.funcs.append((name, has_d, ptys, rtys))
        return '\n'.join(out)

    def gen_main(self):
        env = {'n': 'I'}
        ind = '    '
        lines = ['%slet n = (*get_args).@size;' % ind]
        # Seed containers.
        for ty in self.r.sample(BOXED, self.r.randint(2, 4)):
            self.bind(env, ty, self.make_value(ty, env, 0), lines, ind, prefer_shadow=False)
        for _ in range(self.r.randint(4, 10)):
            self.statement(env, lines, ind, 0)
        live = list(env.items())
        self.r.shuffle(live)
        keep = [x for x in live if self.chance(0.6)] or live[:1]
        acc = 'c'
        lines.append('%slet c = 1;' % ind)
        for v, t in keep:
            e = v if t == 'I' else '%s(%s)' % (CK[t], v)
            lines.append('%slet c = (c * 131 + %s) %% 1000003;' % (ind, e))
        lines.append('%sprintln(c.to_string)' % ind)
        return 'main : IO ();\nmain = (\n' + '\n'.join(lines) + '\n);'

    def program(self):
        parts = [PRELUDE]
        for i in range(self.r.randint(0, 3)):
            parts.append(self.gen_function(i))
        parts.append(self.gen_main())
        return '\n\n'.join(parts) + '\n'


if __name__ == '__main__':
    print(Gen(int(sys.argv[1])).program(), end='')
