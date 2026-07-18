# Where optimizations live: AST vs RC IR

A general architecture note (emerged from the BCE / RC-IR-simplifier design, but applies to
any future optimization). Status: guidance, not a change.

## Two tiers

Fix's back end has two IRs, and each is the natural home for a different kind of work:

1. **Typed `Expr` AST** (monomorphic, post-instantiation). Rich type/instance/HKT structure;
   not A-normal form; nested expressions with shadowing; closures still nested. Home of
   `src/optimization/*`.
2. **RC IR `RcProgram`**, specifically the form **produced by `lower_program`, before
   `insert_rc`**: A-normal form, SSA (each variable bound once, globally unique names),
   control flow reduced to `Match`, lambda-lifted, monomorphic — and no `Retain`/`Release`
   yet. This is effectively Fix's mid-level optimization IR (the GHC-Core / LLVM-IR tier).

## Guidance

**General dataflow / term-rewriting optimizations belong on the pre-`insert_rc` RC IR.**
Inlining, copy propagation, dead-code elimination, CSE, case-of-known-constructor, case-of-
case, scalar-replacement-of-aggregates, value-range analysis, and the like are all
dramatically easier and safer there, because ANF/SSA + explicit control flow + lambda-lifting
+ monomorphism is exactly the substrate these algorithms assume. Concrete evidence: the AST
`let_elimination` pass carries a shadowing-aware `FreeOccurrenceProbe` and capture-avoiding
substitution — accidental complexity that ANF removes entirely. (Do this work **before**
`insert_rc`: rewriting after RC insertion means maintaining `Retain`/`Release` as you go,
which is the hard, error-prone case. Simplify first, then let `insert_rc` compute optimal RC
over the result.)

**Type- and representation-directed transformations stay on the AST.** Typeclass/functor
specialization (`optimize_act`), newtype unwrapping (`unwrap_newtype`), higher-kinded-tyvar
elimination (`remove_hktvs`), closure conversion (`decapturing`), and uncurrying
(`uncurry`) need the type environment, instances, and HKT structure that lowering consumes —
and several must run before lowering anyway. They cannot move to the RC IR.

## Consequences

- **Not "everything to RC IR."** The split is the standard compiler shape: a high-level phase
  for type/representation work, a normalized mid-level IR for the bulk of optimization.
- **Some passes may exist at both tiers.** Inlining is the clearest case: AST-level inlining
  feeds `decapturing`'s higher-order specialization (it must run there), while a small
  RC-level "inline a funptr called once" pass drives the RC simplifier's fixpoint. Different
  jobs, both legitimate.
- **Invest in the RC-IR term-rewriting framework.** Today the RC IR has only RC-specific
  passes (`borrow_ify`/`cancel`/`specialize`) and the `provenance` analysis — no general
  simplifier. Building one (see `simplifier-design.md`) creates reusable infrastructure that
  makes every future general optimization cheap to add there, whereas extending the AST
  passes keeps paying the non-ANF complexity tax. **Default new general optimizations to the
  pre-`insert_rc` RC IR.**
