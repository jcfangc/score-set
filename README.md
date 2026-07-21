# score-set

`score-set` provides small, statically composed primitives for building
weighted scoring functions.

A metric consists of:

- a `Measure<Ctx>` that extracts a raw value from a context;
- a `Map01F32` or `Map01F64` that normalizes that value;
- a weight applied to the normalized score.

The normalized result is returned as `Witnessed<f32, V01>` or
`Witnessed<f64, V01>`. This makes the `[0, 1]` boundary an explicit type-level
fact for downstream code.

## Installation

```toml
[dependencies]
score-set = "2.0.0"
```

## Quick start

The measurement output and map input are connected through associated types.
The mapper must accept exactly the value produced by the measurement.

```rust
use score_set::{Metric64, traits::{EvalF64, Map01F64, Measure, V01}};
use witnessed::{WitnessExt, Witnessed};

struct Context {
    latency_ms: f64,
}

struct Latency;

impl Measure<Context> for Latency {
    type Output = f64;

    fn measure(&self, ctx: &Context) -> Self::Output {
        ctx.latency_ms
    }
}

struct LowerIsBetter {
    limit: f64,
}

impl Map01F64 for LowerIsBetter {
    type Input = f64;

    fn map(&self, value: Self::Input) -> Witnessed<f64, V01> {
        let score = (1.0 - value / self.limit).clamp(0.0, 1.0);

        // Safety: `score` was clamped to `[0, 1]` above.
        unsafe { score.witness().by_unchecked::<V01>() }
    }
}

let metric = Metric64::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7);
let score = metric.eval(&Context { latency_ms: 40.0 });

assert!((score - 0.42).abs() < 1e-12);
```

`Map01F32` has the same API and returns `Witnessed<f32, V01>`:

```rust
use score_set::traits::{Map01F32, V01};
use witnessed::{WitnessExt, Witnessed};

struct Identity;

impl Map01F32 for Identity {
    type Input = f32;

    fn map(&self, value: Self::Input) -> Witnessed<f32, V01> {
        let score = value.clamp(0.0, 1.0);

        // Safety: `score` was clamped to `[0, 1]` above.
        unsafe { score.witness().by_unchecked::<V01>() }
    }
}
```

## Dynamic score sets

`DynScoreSet32<Ctx>` and `DynScoreSet64<Ctx>` store heterogeneous metrics
behind `EvalF32<Ctx>` or `EvalF64<Ctx>` trait objects.

```rust
use score_set::{DynScoreSet64, Metric64};

let score_set = DynScoreSet64::<Context>::builder()
    .append(Metric64::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7))
    .build();

let score = score_set.eval(&Context { latency_ms: 40.0 });
```

Use the concrete `Metric32`/`Metric64` types when the metric composition is
known at compile time. Use a dynamic score set when the enabled metrics are
selected at runtime.

## Witnesses

`Measure` returns an ordinary associated `Output`. The witness is produced by
the normalization map:

```rust
pub trait Measure<Ctx: ?Sized> {
    type Output;

    fn measure(&self, ctx: &Ctx) -> Self::Output;
}

pub trait Map01F64 {
    type Input;

    fn map(&self, value: Self::Input) -> Witnessed<f64, V01>;
}
```

`V01` is a marker type representing a value known to be in the normalized
`[0, 1]` range. `Witnessed<T, V01>` is a transparent wrapper and has no
runtime witness field.

When a result is derived from already-witnessed values and rechecking is
unnecessary, `by_unchecked` may be used at an explicitly audited unsafe
boundary. The caller must document why the invariant is preserved.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License
