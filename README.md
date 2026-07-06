# score-set

[![crates.io](https://img.shields.io/crates/v/score-set?label=crates.io)](https://crates.io/crates/score-set)
[![Coverage](https://codecov.io/gh/jcfangc/score-set/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/score-set)

A `#![no_std]` + `alloc` Rust library for building **weighted scoring
operators** — combine named metrics with weights, normalize, and produce a
scoring function or breakdown.

Two public types are all you need:
- **`ScoreSet<N>`** (builder): collect metrics → normalise weights → produce result
- **`Metric<N>`** (single metric): `fn(&C) -> f32` (or `f64`) measure + Map01 normalization

Where `N` is `32` (f32) or `64` (f64).

## Quick example

```rust
use score_set::*;

struct Restaurant {
    cleanliness: f32,
    food_quality: f32,
}

// Define metrics via a builder pipeline
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

// Combine with weights → weighted-sum closure
let score = ScoreSet32::new()
    .push(2.0, clean)?
    .push(1.0, food)?
    .sum()?;

let r = Restaurant { cleanliness: 80.0, food_quality: 4.0 };
let total: f32 = score(&r);          // ~0.87

// Or get per-metric breakdown rows
let rows: Vec<Breakdown32> = ScoreSet32::new()
    .push(2.0, clean)?
    .push(1.0, food)?
    .breakdown(&r)?
    .into_iter()
    .collect();

for row in rows {
    // row.name, row.score, row.weight, row.contribution
}
# Ok::<(), &'static str>(())
```

## Precision

| Feature flag | Available types |
|---|---|
| *(default)* | `ScoreSet32`, `Metric32`, `metric32()`, … |
| `f64` | `ScoreSet64`, `Metric64`, `metric64()`, … |
| `both` | all of the above |

```toml
[dependencies]
score-set = "0.4"                         # f32 only
score-set = { version = "0.4", features = ["f64"] }    # f64 only
score-set = { version = "0.4", features = ["both"] }   # both
```

## Building a metric

```
metric32("name")           // MetricNamingStage32
    .measure()             // MeasureStage32
    .by(|ctx| raw)         // MeasuredStage32<C>  — fn pointer, no closures
    .map01()               // Map01Stage32<C>
    .<shape>(...)          // Metric32<C>
```

### Map01 shapes

| Shape | Constructor | Formula |
|---|---|---|
| Identity | `.identity()` | `raw.clamp(0, 1)` |
| Linear | `.linear(max)` | `raw / max`, clamped |
| Increasing sigmoid | `.inc_sigmoid(low, high)` | `1/(1 + e⁻ᵏ⁽ˣ⁻ˣ⁰⁾)`, k auto-calibrated |
| Decreasing sigmoid | `.dec_sigmoid(low, high)` | `1/(1 + eᵏ⁽ˣ⁻ˣ⁰⁾)`, k auto-calibrated |
| Asymmetric Cauchy | `.cauchy(center, half_left, half_right)` | `1/(1 + ((x−c)/h)²)`, h per-side |
| Custom | `.by(fn(f32) -> f32)` | user-provided |

All variants (except Custom) guarantee output in `[0, 1]` by construction.
Custom is validated at evaluation time via `Value01` witness.

## API

### `ScoreSet<N>` — builder

```rust
ScoreSet32::new()                          // empty builder
    .push(weight, metric)?                 // add a metric (weight must be > 0, finite)
    .sum()?                                // → impl Fn(&C) -> f32
    .breakdown(&ctx)?                      // → impl IntoIterator<Item = Breakdown32>
```

`sum()` returns a closure — call it with any number of contexts.  
`breakdown()` evaluates all metrics against one context, returns owned rows.

### `Metric<N>` — single metric

```rust
pub struct Metric32<C> {
    pub name: &'static str,
    // measure: fn(&C) -> f32     (private)
    // map01: Map0132              (private)
}

impl Metric32<C> {
    pub fn eval(&self, ctx: &C) -> Result<Witnessed<f32, Value01>, &'static str>;
}
```

### `Breakdown<N>`

```rust
pub struct Breakdown32 {
    pub name: &'static str,
    pub score: f32,        // normalized, in [0, 1]
    pub weight: f32,        // normalized weight (sum = 1)
    pub contribution: f32,  // score × weight
}
```

## `no_std`

This crate is `#![no_std]` with `extern crate alloc`. It only needs `Vec`
and `String` from the allocator, and `libm` for `exp` and `log`. Works on bare-metal
targets.

## License

MIT OR Apache-2.0
