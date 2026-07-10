# score-set

[![crates.io](https://img.shields.io/crates/v/score-set?label=crates.io)](https://crates.io/crates/score-set)
[![Coverage](https://codecov.io/gh/jcfangc/score-set/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/score-set)

A `#![no_std]` + `alloc` Rust library for building **weighted scoring
operators** — define metrics via a builder pipeline, combine them with weights
into a zero-vtable flat scorer, and evaluate against any context.

Three concepts:

- **`Metric32<C, F>`** — a single named metric: measure closure + Map01 normalization
- **`score_set32!`** — macro that packs heterogeneous metrics into a flat tuple scorer
- **`Scored32<C, T>`** — the concrete scorer with `.score(&C)` and `.breakdown(&C)`

## Quick example

```rust
use score_set::*;

struct Restaurant {
    cleanliness: f32,
    food_quality: f32,
}

// Define metrics — by() accepts closures (including capturing ones)
let clean = metric32("cleanliness")
    .measure()
    .by(|r: &Restaurant| r.cleanliness)
    .map01()
    .linear(100.0);

let food = metric32("food")
    .measure()
    .by(|r: &Restaurant| r.food_quality)
    .map01()
    .identity();

// Combine into a flat scorer (zero vtable, static dispatch)
let scorer = score_set32! { 2.0 => clean, 1.0 => food }?;

let r = Restaurant { cleanliness: 80.0, food_quality: 4.0 };
let total: f32 = scorer.score(&r);          // ~0.87

// Per-metric breakdown
let rows = scorer.breakdown(&r);
for row in rows {
    // row.name, row.raw, row.score, row.weight, row.contribution
}
# Ok::<(), &'static str>(())
```

### Partial application (capturing closures)

```rust
let threshold: f32 = 0.6;
let quality = metric32("quality")
    .measure()
    .by(move |ctx: &MyCtx| if ctx.value > threshold { ctx.value } else { 0.0 })
    .map01()
    .identity();

// threshold is baked into the closure — scorer only needs &MyCtx
let scorer = score_set32! { 1.0 => quality }?;
let result = scorer.score(&ctx);
```

## Precision

| Feature flag | Available types |
|---|---|
| *(default)* | `Metric32`, `Scored32`, `Breakdown32`, `metric32()`, `score_set32!`, … |
| `f64` | `Metric64`, `Scored64`, `Breakdown64`, `metric64()`, `score_set64!`, … |
| `both` | all of the above |

```toml
[dependencies]
score-set = "0.6"                                   # f32 only
score-set = { version = "0.6", features = ["f64"] }  # f64 only
score-set = { version = "0.6", features = ["both"] } # both
```

## Building a metric

```
metric32("name")             // MetricNamingStage32
    .measure()               // MeasureStage32
    .by(|ctx| raw)           // MeasuredStage32<C, F>  — impl Fn(&C) -> f32
    .map01()                 // Map01Stage32<C, F>
    .<shape>(...)            // Metric32<C, F>
```

`by()` accepts any `impl Fn(&C) -> f32` — function pointers, non-capturing
closures, and capturing closures all work.

### Map01 shapes

| Shape | Constructor | Formula |
|---|---|---|
| Identity | `.identity()` | `raw.clamp(0, 1)` |
| Linear | `.linear(max)` | `raw / max`, clamped |
| Increasing sigmoid | `.inc_sigmoid(low, high)` | `1/(1 + e⁻ᵏ⁽ˣ⁻ˣ⁰⁾)`, k auto-calibrated |
| Decreasing sigmoid | `.dec_sigmoid(low, high)` | `1/(1 + eᵏ⁽ˣ⁻ˣ⁰⁾)`, k auto-calibrated |
| Asymmetric Cauchy | `.cauchy(center, half_left, half_right)` | `1/(1 + ((x−c)/h)²)`, h per-side |
| Custom | `.by(fn(f32) -> f32)` | user-provided fn pointer |

All variants (except Custom) guarantee output in `[0, 1]` by construction.

## API

### `score_set32!` — heterogeneous scorer

```rust
score_set32! { 2.0 => gc_metric, 1.0 => len_metric }?
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//          weight => metric pairs, comma-separated, up to 64 metrics

// Returns Result<Scored32<C, T>, &'static str>
```

Weights must be finite and strictly positive (> 0). The macro normalizes them
to sum to 1 at construction time.

### `Scored32<C, T>` — the concrete scorer

```rust
impl<C, T: ScoreSetTrait32<C>> Scored32<C, T> {
    /// Weighted sum over all metrics.
    pub fn score(&self, ctx: &C) -> f32;

    /// Per-metric breakdown rows.
    pub fn breakdown(&self, ctx: &C) -> Vec<Breakdown32>;
}
```

`T` is a flat tuple of `Metric32<C, F>` types — each `F` can be a different
closure type. Trait dispatch is fully static (zero vtable).

### `Metric32<C, F>` — single metric

```rust
pub struct Metric32<C, F = fn(&C) -> f32> {
    pub name: &'static str,
    // measure: F          (private)
    // map01: Map0132       (private)
}

impl<C, F: Fn(&C) -> f32> Metric32<C, F> {
    pub fn eval(&self, ctx: &C) -> Result<Witnessed<f32, Value01>, &'static str>;
    pub fn make_breakdown(&self, weight: f32, ctx: &C) -> Breakdown32;
}
```

### `Breakdown32`

```rust
pub struct Breakdown32 {
    pub name: &'static str,
    pub raw: f32,           // original measured value
    pub score: f32,         // normalized, in [0, 1]
    pub weight: f32,        // normalized weight (sum = 1)
    pub contribution: f32,  // score × weight
}
```

## Arity limits

By default, `score_set32!` supports up to 8 metrics. Enable higher arities via
feature flags:

```toml
score-set = { version = "0.6", features = ["level-16"] }   # up to 16
score-set = { version = "0.6", features = ["level-32"] }   # up to 32
score-set = { version = "0.6", features = ["level-64"] }   # up to 64
```

Generated files (`gen_score_set32.rs` / `gen_score_set64.rs`) are maintained by
`cargo run -p xtask -- gen --max <N>`.

## `no_std`

This crate is `#![no_std]` with `extern crate alloc`. It only needs `Vec`
from the allocator and `libm` for `exp` and `log`. Works on bare-metal targets.

## License

MIT OR Apache-2.0
