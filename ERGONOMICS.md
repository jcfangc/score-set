# Ergonomics Report — score-set v0.2.0

All issues identified in the initial ergonomics audit have been resolved.

## Resolution summary

| # | Issue | Resolution |
|---|---|---|
| P0 | concrete-form syntax for `finite_metric!` | ✅ Named-key `metric => Name, float => f64, subject => Rest, dimensions => ...` |
| P0 | Metric builder `.boxed()` | ✅ `metric.boxed()` returns `Box<dyn Scorable<T, I>>` |
| P1 | DynamicScoreSet incremental builder | ✅ `DynamicScoreSet::builder().push(w, m)?.build()?` |
| P1 | `.sum(&I)` + `.breakdown(&I)` debug/inspect | ✅ `set.sum(&input)` returns total; `set.breakdown(&input)` returns per-metric detail |
| P2 | `finite_score_set!` / `dynamic_score_set!` macro sugar | ✅ All three layers use `{ weight => metric, ... }` syntax |
| P2 | Anonymous zero-vtable enum (`FiniteEnumN`) | ✅ `finite_score_set! { w => bare_metric }` requires no enum declaration |
| P3 | DnaContext aggregation pattern docs | ✅ Captured in `tests/examples.rs` (restaurant scoring) and `tests/layer_comparison.rs` |
| — | Three-layer constructor naming consistency | ✅ `FixedScoreSet::normalize()`, `FiniteScoreSet::normalize()`, `DynamicScoreSet::normalize()` — all `#[doc(hidden)]`, user-facing entry points are macros and builders |
| — | `DynMetric` → `Scorable` | ✅ Renamed for honesty — the match-based dispatch path has zero vtable overhead |
| — | Concrete form auto-`Custom` removed | ✅ No spurious dead_code warning; use generic form with explicit `Custom` when needed |
| — | `FiniteEnum9..128` exposed | ✅ `pub mod finite_enum` + `pub use *` |

## What stayed

- **Derive macro** — not implemented. The `finite_metric!` macro already eliminates enum boilerplate; a derive wouldn't add enough value to justify proc-macro complexity.
- **Layer migration paths** (`into_finite()`, `into_dynamic()`) — not implemented. The three layers serve different design points; migration would encourage the wrong pattern. Choose the right layer upfront.
