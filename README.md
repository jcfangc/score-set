# score-set

[![crates.io](https://img.shields.io/crates/v/score-set?label=crates.io)](https://crates.io/crates/score-set)
[![Coverage](https://codecov.io/gh/jcfangc/score-set/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/score-set)

A Rust library for building **weighted scoring operator sets** with three
dispatch strategies — from compile-time fixed to fully dynamic.

## Quick example (Layer 1 — fixed)

```rust
use score_set::*;

let gc = metric("gc")
    .measure().by(|dna: &&str| gc_ratio(dna))
    .map01().by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());

let len = metric("len")
    .measure().by(|len: &usize| *len)
    .map01().by(|raw: &usize, _: &usize| {
        Value01::witness((*raw as f64 / 100.0).min(1.0)).unwrap()
    });

let ms = fixed_score_set! { 2.0 => gc, 3.0 => len }?;

let dna = "ACGTACGT";
let score = ms.score().by(|(gc, len)| {
    gc.contribute(gc.metric().eval(&dna))
        + len.contribute(len.metric().eval(&dna.len()))
});
```

## Three-layer architecture

| Layer | Type | Macro | Dispatch | Use when |
|---|---|---|---|---|
| 1 — fixed | `FixedScoreSet` | `fixed_score_set!` | Compile-time, zero vtable | Metric set known at compile time |
| 2 — finite | `FiniteScoreSet` | `finite_score_set!` | Enum match, zero vtable | Runtime composition, known metric types |
| 3 — dynamic | `DynamicScoreSet` | `dynamic_score_set!` | Vtable per call | Fully heterogeneous, runtime assembly |

All three layers share the same `{ weight => metric, ... }` macro syntax.

### Layer 2 — finite

Declare a metric enum with named keys, then assemble:

```rust
finite_metric! {
    metric     => RestaurantMetric,
    float      => f64,
    subject    => Restaurant,
    dimensions =>
        Clean(Cleanliness),
        Quality(FoodQuality),
        Price(PriceScore),
}

let set = finite_score_set! {
    3.0 => RestaurantMetric::Clean(Cleanliness::new()),
    5.0 => RestaurantMetric::Quality(FoodQuality::new()),
    2.0 => RestaurantMetric::Price(PriceScore::new()),
}?;

let total = set.sum(&restaurant);
let rows  = set.breakdown(&restaurant);  // per-metric detail
```

Or skip the enum declaration entirely — bare metrics are auto-wrapped in an
anonymous zero-vtable enum:

```rust
let set = finite_score_set! { 2.0 => gc, 3.0 => len }?;
let total = set.sum(&input);
```

### Layer 3 — dynamic

Same syntax, metrics are auto-boxed:

```rust
let set = dynamic_score_set! { 2.0 => gc, 3.0 => len }?;
let total = set.sum(&input);
```

## Scoring

| Method | Layer 1 | Layer 2 | Layer 3 |
|---|---|---|---|
| `set.sum(&input)` | — | ✅ | ✅ |
| `set.score().by(closure)` | ✅ | ✅ | ✅ |
| `set.breakdown(&input)` | — | ✅ | ✅ |
| `set.iter()` / `.len()` | — | ✅ | ✅ |
| Builder `.push().build()` | — | ✅ | ✅ |

## Building a metric

```rust
let m = metric("name")            // name it
    .measure().by(|input| raw)    // measure: I → Raw
    .map01().by(|raw, input| v);  // normalise: Raw → [0, 1]
```

## Features

Default arity is 128 members per set. Opt into smaller feature sets:

```toml
score-set = { default-features = false, features = ["level-8"] }
score-set = { features = ["level-16"] }
```

Available levels: `level-8`, `level-16`, `level-32`, `level-64`, `level-128`.

Per-layer control:

```toml
score-set = { features = ["fixed-level-8", "finite-level-8"] }
```

## License

MIT OR Apache-2.0
