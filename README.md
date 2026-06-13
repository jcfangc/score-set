# score-set

[![crates.io](https://img.shields.io/crates/v/score-set?label=crates.io)](https://crates.io/crates/score-set)
[![Coverage](https://codecov.io/gh/jcfangc/score-set/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/score-set)

A Rust library for building **static weighted scoring operator sets**. Declare a set of named metrics with weights, normalize at build time, score at runtime via a user-provided closure that freely injects inputs and context.

## Usage

```rust
use score_set::*;

// 1. Build metrics
let gc = metric("gc-content")
    .measure()
    .by(|dna: &&str| {
        let gc = dna.chars().filter(|c| *c == 'G' || *c == 'C').count();
        if dna.is_empty() { 0.0 } else { gc as f64 / dna.len() as f64 }
    })
    .map01()
    .by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());

let length = metric("length")
    .measure()
    .by(|len: &usize| *len)
    .map01()
    .by(|raw: &usize, _: &usize| {
        Value01::witness((*raw as f64 / 100.0).min(1.0)).unwrap()
    });

// 2. Declare the set — weights are normalized automatically
let ms = score_set! {
    2.0 => gc,
    3.0 => length,
}?;

// 3. Score with runtime data
let dna = "ACGTACGT";
let score = ms.score().by(|(gc, len)| {
    gc.contribute(gc.metric().eval(&dna))
        + len.contribute(len.metric().eval(&dna.len()))
});
// score ≈ 0.248

# Ok::<(), &'static str>(())
```

## Concepts

**Metrics.** A `Metric` is a two-stage scoring operator: `measure` maps input to a raw value, `map01` maps the raw value (with the original input still available for context) to a validated `[0, 1]` score.

```rust
metric("name")
    .measure().by(|input: &T| raw_value)
    .map01().by(|raw: &Raw, input: &T| Value01::witness(raw).unwrap())
```

**Score Sets.** `score_set!` declares a weighted set of metrics. Weights are validated strictly positive and normalized to sum to 1. Validation fails at build time if any weight is ≤ 0.

```rust
let ms = score_set! { 2.0 => gc, 3.0 => len, 5.0 => specificity }?;
```

**Scoring.** `score().by(closure)` gives access to every member. Each member provides `.metric()` (the operator) and `.contribute(value01)` (score × normalized weight). The closure composes contributions arbitrarily — different operators can consume different input shapes, capture external context, or conditionally participate.

Linear combination:

```rust
let score = ms.score().by(|(gc, len, spec)| {
    gc.contribute(gc.metric().eval(&dna))
        + len.contribute(len.metric().eval(&dna.len()))
        + spec.contribute(spec.metric().eval(&(&dna, &ctx)))
});
```

Geometric (product) — all-or-nothing scoring sensitive to any weak metric:

```rust
let score = ms.score().by(|(gc, len, spec)| {
    gc.contribute(gc.metric().eval(&dna))
        * len.contribute(len.metric().eval(&dna.len()))
        * spec.contribute(spec.metric().eval(&(&dna, &ctx)))
});
```

**Controlled values.**

| Function | Returns | Guarantee |
|---|---|---|
| `Value01::witness(v)` | `Witnessed<T, Value01>` | finite, ∈ [0, 1] |
| `NormalizedContainer::witness(vec)` | `Witnessed<Vec<T>, NormalizedContainer>` | all ∈ [0, 1], sum = 1 |

`NormalizedWeight` credentials are extracted from a validated container via `NormalizedWeight::from_normalized_container(value, &container)`, which binary-searches for membership verification.

## Arity

Default supports up to 8 metrics per set. Opt into larger arities via Cargo features:

```toml
score-set = { features = ["level-16"] }   # up to 16
score-set = { features = ["level-128"] }  # up to 128
```

## License

MIT OR Apache-2.0
