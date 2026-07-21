# Speedtest: the session's effect, `e0943d00` -> `f8ffebe5`

Metric: cachegrind `Ir`, which is deterministic and independent of machine load. Each row was produced
by the `fix` built from that revision, in its own git worktree, over the whole
`benchmark/speedtest/cases/` suite at `-O experimental`.

`log.csv` had not been written since `e1f309d7` (2026-07-17), 85 commits earlier, so the branch row
also refreshes it; the isolated comparison below is against a row measured for `e0943d00`, the commit
this session started from.

## Result

| case | `e0943d00` | `f8ffebe5` | delta |
| --- | ---: | ---: | ---: |
| `bounds_check_indexable` | 105,633,227 | 79,834,194 | **-24.42%** |
| `fannkuch` | 3,784,782,470 | 3,115,057,384 | **-17.70%** |
| `cp_lib_segtree` | 98,164,994 | 93,764,625 | -4.48% |
| `cp_lib_bipartite` | 234,228,768 | 227,362,303 | -2.93% |
| `cp_lib_lsegtree` | 1,195,055,900 | 1,224,286,586 | **+2.45%** |

The other 32 cases move by at most 0.02%; every case carries a constant +46 instructions of startup
difference.

The two double-digit wins are what `f4d9d30f` was for: reading a scalar field of a loop-state struct
no longer retains the struct, so its `Array` leaf keeps its provenance and the loop's uniqueness
checks fold statically.

## The one regression

Measured per commit, the regression is entirely `f4d9d30f`; `26dca513` is neutral on it.

| revision | `cp_lib_lsegtree` Ir |
| --- | ---: |
| `e0943d00` | 1,195,055,894 |
| `f4d9d30f` (the borrow) | 1,224,286,543 |
| `f8ffebe5` (tip) | 1,224,286,533 |

So the conservatism `26dca513` adds to reference-count cancellation — a release of a value whose
object is path-dependent keeps pending retains of the objects it may be — costs nothing measurable.

The RC IR of the case is *better* at `f4d9d30f` by every count:

| | `e0943d00` | `f4d9d30f` |
| --- | ---: | ---: |
| `retain` nodes | 61 | 53 |
| `release` nodes | 54 | 55 |
| ops with their uniqueness check dropped | 29 | 40 |
| `array_set[unique]` | 8 | 11 |
| functions | 101 | 109 |
| of which uniqueness-specialized | 21 | 29 |

Eight more specialized versions exist because more call sites are now provably unique. The extra code
changes LLVM's inlining decisions, and the net instruction count rises. This is downstream of the RC
IR, not reference-count over-insertion — the same effect the P1 flip recorded on three cases.
