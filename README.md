# score-set

[![Crates.io](https://img.shields.io/crates/v/score-set.svg)](https://crates.io/crates/score-set)
[![Documentation](https://docs.rs/score-set/badge.svg)](https://docs.rs/score-set)
[![License](https://img.shields.io/crates/l/score-set.svg)](https://crates.io/crates/score-set)
[![CI](https://github.com/jcfangc/score-set/actions/workflows/gate.yml/badge.svg)](https://github.com/jcfangc/score-set/actions/workflows/gate.yml)
[![Coverage](https://codecov.io/gh/jcfangc/score-set/branch/main/graph/badge.svg)](https://codecov.io/gh/jcfangc/score-set)

---

`score-set` provides small, statically composed primitives for building
weighted scoring functions.

The crate supports `no_std`. Static metrics require no allocation; dynamic
score sets use `alloc` for their boxed heterogeneous metric collection.

A metric consists of:

- a `Measure<Ctx>` that extracts a raw value from a context;
- a `Map01F32` or `Map01F64` that normalizes that value;
- a `Weighted32` or `Weighted64` wrapper that applies a witnessed weight.

The normalized result is returned as `Witnessed<f32, V01>` or
`Witnessed<f64, V01>`. This makes the `[0, 1]` boundary an explicit type-level
fact for downstream code.

## Installation

```toml
[dependencies]
score-set = "3.0.0"
```

## Quick start

The measurement output and map input are connected through associated types.
The mapper must accept exactly the value produced by the measurement.

```rust
use score_set::{metric64, weight64, traits::{EvalF64, Map01F64, Measure, V01, prove_v01_f64}};
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

        score.witness().by(prove_v01_f64).expect("score was clamped")
    }
}

let weight = weight64(0.7).unwrap();
let metric = metric64(Latency, LowerIsBetter { limit: 100.0 }, weight);
let score = metric.eval(&Context { latency_ms: 40.0 });

assert!((score - 0.42).abs() < 1e-12);
```

`Map01F32` has the same API and returns `Witnessed<f32, V01>`:

```rust
use score_set::traits::{Map01F32, V01, prove_v01_f32};
use witnessed::{WitnessExt, Witnessed};

struct Identity;

impl Map01F32 for Identity {
    type Input = f32;

    fn map(&self, value: Self::Input) -> Witnessed<f32, V01> {
        let score = value.clamp(0.0, 1.0);

        score.witness().by(prove_v01_f32).expect("score was clamped")
    }
}
```

## Dynamic score sets

`DynScoreSet32<Ctx>` and `DynScoreSet64<Ctx>` store heterogeneous metrics
behind `EvalF32<Ctx>` or `EvalF64<Ctx>` trait objects.

```rust
use score_set::{DynScoreSet64, metric64, weight64};

let weight = weight64(0.7).unwrap();
let score_set = DynScoreSet64::<Context>::builder()
    .append(metric64(Latency, LowerIsBetter { limit: 100.0 }, weight))
    .build();

let score = score_set.eval(&Context { latency_ms: 40.0 });
```

The `metric32` and `metric64` functions are convenience constructors for a
`Weighted32<NormalizedEval32<...>>` or
`Weighted64<NormalizedEval64<...>>` composition.

## Weighting evaluators

`Weighted32` and `Weighted64` can scale any evaluator, including user-defined
static score sets and dynamic score sets. A weight must carry the `V01`
witness, so an unchecked or out-of-range coefficient cannot enter the scoring
tree. The `weight32` and `weight64` constructors validate raw coefficients and
return a `Result`.

```rust
use score_set::{Weighted64, weight64};

let weight = weight64(0.8).unwrap();
let weighted_score_set = Weighted64::new(score_set, weight);
```

The witness guarantees each weight is in `[0, 1]`. It does not guarantee that
several weights sum to one or that a complete score remains normalized.

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

## Design Rationale

`score-set` models a score as a weighted composition of a measurement and a mapping function.

For a context `x`, a single metric is defined as:

```text
metric(x) = w · g(m(x))
```

where:

* `m` is a measurement;
* `g` maps the measurement result into a normalized score;
* `w` is the metric weight.

A complete score set evaluates multiple metrics and sums their contributions:

```text
score(x) = Σ(i=1..n) w_i · g_i(m_i(x))
```

The library needs to support two different use cases:

1. a fixed, predefined metric set used by most applications;
2. a runtime-configurable metric set, typically constructed from protobuf configuration.

These use cases have different implementation requirements and are therefore represented by separate execution paths.

## Static metric composition

When the metric set is known at compile time, metrics can be represented directly through generic composition:

```rust
type LatencyMetric = Weighted64<NormalizedEval64<Latency, Cauchy>>;
type CpuMetric = Weighted64<NormalizedEval64<CpuUsage, Linear>>;
type SimilarityMetric = Weighted64<NormalizedEval64<Similarity, Sigmoid>>;
```

A statically defined score set can contain concrete metric types:

```rust
pub struct DefaultScoreSet {
    latency: LatencyMetric,
    cpu: CpuMetric,
    similarity: SimilarityMetric,
}
```

Its evaluation can be written as a direct expression:

```rust
impl Eval<Context> for DefaultScoreSet {
    #[inline]
    fn eval(&self, ctx: &Context) -> f64 {
        self.latency.eval(ctx)
            + self.cpu.eval(ctx)
            + self.similarity.eval(ctx)
    }
}
```

This representation allows Rust to monomorphize the complete evaluation path. It requires no runtime type selection and permits inlining across the measurement, mapping, and aggregation layers.

This is the preferred representation for predefined score sets.

## Runtime-configurable metric composition

A runtime configuration may select an arbitrary subset of available metrics.

For example, one configuration may select:

```text
LatencyCauchy
CpuLinear
```

while another may select:

```text
LatencyIdentity
SimilaritySigmoid
MemoryLinear
```

The concrete generic types of these score sets are different. A function that constructs a score set from runtime data must nevertheless return one stable Rust type.

This creates a fundamental distinction between compile-time and runtime composition.

A compile-time composition may have a type such as:

```text
Append<
    Append<Zero, LatencyCauchy>,
    CpuLinear
>
```

However, a runtime configuration may produce any of the following:

```text
Zero
Append<Zero, LatencyCauchy>
Append<Zero, CpuLinear>
Append<Append<Zero, LatencyCauchy>, CpuLinear>
```

These are different concrete Rust types.

Return-position `impl Trait` does not unify them. It hides one concrete type selected at compile time; it does not represent several types selected by runtime data.

In general, the following three properties cannot be obtained simultaneously
in ordinary ahead-of-time Rust:

```text
runtime-selected structure
+ one concrete static type
+ no enumeration of all structures
```

A runtime-configurable implementation therefore requires a common representation.

## Alternatives considered

Several representations were considered.

### Fixed complete metric set

All possible metrics can be stored in one fixed structure, with disabled metrics assigned zero weight.

This provides a single static type and avoids dynamic dispatch. However, disabled metrics may still require evaluation, and expensive measurements may be repeated unnecessarily.

This approach is appropriate when:

* the complete metric set is small;
* most metrics are usually enabled;
* individual measurements are inexpensive.

It is less suitable for sparse runtime configurations.

### Enumeration of all metric combinations

Every possible runtime subset can be represented as a separate enum variant.

For `N` independently optional metrics, the number of possible subsets is
`2^N`.

For example:

| Optional metrics | Possible subsets |
| ---------------: | ---------------: |
|                2 |                4 |
|                4 |               16 |
|                8 |              256 |
|               16 |           65,536 |

This representation can provide near-static runtime performance, but its code size and compile-time cost grow exponentially.

It is not suitable as a general-purpose library strategy.

### Cartesian-product enum

If the sets of measurements and mappings are closed, the library can generate one enum variant for each supported pair:

```rust
type LatencyIdentity = Weighted64<NormalizedEval64<Latency, Identity>>;
type LatencyCauchy = Weighted64<NormalizedEval64<Latency, Cauchy>>;
type CpuLinear = Weighted64<NormalizedEval64<CpuUsage, Linear>>;

pub enum MetricOp {
    LatencyIdentity(LatencyIdentity),
    LatencyCauchy(LatencyCauchy),
    CpuLinear(CpuLinear),
}
```

A runtime score set can then be represented as:

```rust
pub struct ScoreSet {
    metrics: Box<[MetricOp]>,
}
```

Each metric evaluation performs one enum dispatch, after which the concrete measurement and mapping types are known.

This avoids trait objects and preserves static dispatch inside each enum branch.
However, the generated representation grows with the Cartesian product
`|M| × |G|`, where `M` is the measurement set and `G` is the mapping set.

Adding a new measurement or mapping expands the generated enum and its conversion logic.

### JIT compilation

A runtime configuration could be translated into an intermediate representation and compiled into a specialized native function.

Conceptually:

```text
protobuf configuration
    -> score plan
    -> JIT intermediate representation
    -> native scoring function
```

This can provide runtime-selected composition without per-metric dispatch.

However, it introduces substantial engineering requirements:

* executable-memory management;
* ABI boundaries between generated code and Rust;
* unsafe function-pointer handling;
* platform-specific testing;
* lifetime management for compiled code and metric state;
* integration of user-defined measurements;
* runtime compilation overhead.

For the expected number and cost of metrics, this complexity is not justified.

### Dynamic dispatch

The simplest runtime representation is a heterogeneous collection of evaluators:

```rust
pub trait Eval<Ctx> {
    fn eval(&self, ctx: &Ctx) -> f64;
}

pub struct DynScoreSet<Ctx> {
    metrics: Box<[
        Box<dyn Eval<Ctx> + Send + Sync>
    ]>,
}
```

Concrete metrics remain generic compositions:

```rust
let metric = Weighted64::new(
    NormalizedEval64::new(measure, map),
    weight,
);
```

Runtime configuration constructs only the enabled metrics:

```rust
let mut metrics = Vec::new();

if let Some(config) = proto.latency_cauchy {
    metrics.push(Box::new(
        metric64(Latency, Cauchy::compile(config)?, latency_weight),
    ));
}

if let Some(config) = proto.cpu_linear {
    metrics.push(Box::new(
        metric64(CpuUsage, Linear::compile(config)?, cpu_weight),
    ));
}
```

The dynamic boundary exists only between the score set and each concrete metric:

```text
DynScoreSet
    -> dyn Eval
    -> Weighted64<NormalizedEval64<M, G>>
```

Inside each composed evaluator, both the measurement type and mapping type
remain concrete and monomorphized.

The runtime cost is one indirect call per enabled metric. In exchange, the representation provides:

* arbitrary runtime metric subsets;
* execution of enabled metrics only;
* a stable return type;
* straightforward ownership and lifetime management;
* no generated Cartesian-product enum;
* no exponential type expansion;
* simple addition of new measurements and mappings.

## Selected design

The library uses two execution paths.

### Default path

The default metric set is represented as a concrete static type.

```text
default configuration
    -> static score set
    -> monomorphized evaluation
```

This path is intended for the common case and provides:

* static dispatch;
* direct aggregation;
* full inlining opportunities;
* no runtime metric-selection overhead.

### Custom path

User-defined runtime configurations are represented by `DynScoreSet`.

```text
custom configuration
    -> concrete Weighted64<NormalizedEval64<M, G>> values
    -> Box<dyn Eval<Ctx>>
    -> dynamic score set
```

This path provides runtime flexibility while keeping the implementation small and maintainable.

The expected workload is dominated by the default configuration. Therefore, the static path optimizes the common case, while the dynamic path handles uncommon custom configurations without imposing additional complexity on the entire library.

## Execution-path selection

The execution mode can be selected during initialization:

```rust
match custom_config {
    None => {
        let score_set = DefaultScoreSet::compile()?;
        run_service(score_set).await
    }

    Some(config) => {
        let score_set = DynScoreSet::compile(config)?;
        run_service(score_set).await
    }
}
```

The service itself remains generic:

```rust
async fn run_service<E>(
    score_set: E,
) -> Result<(), Error>
where
    E: Eval<Context> + Send + Sync + 'static,
{
    // Service implementation.
}
```

The compiler only needs to instantiate the service for the two top-level score-set types:

```text
run_service::<DefaultScoreSet>
run_service::<DynScoreSet>
```

It does not need to instantiate the service for every possible metric subset.

## Design principle

The selected design does not attempt to force compile-time and runtime composition into one representation.

Instead, it uses the representation appropriate to each case:

```text
predefined configuration -> static composition
runtime configuration    -> dynamic composition
```

Dynamic dispatch is limited to the boundary where runtime heterogeneity must be represented. The internal implementation of each concrete metric remains generic and statically typed.

This keeps the common path fully static while allowing the configurable path to remain direct, extensible, and maintainable.

## Application integration guidance

The library does not require applications to use separate static and dynamic execution paths.

It provides the building blocks needed for both forms of composition:

* concrete generic evaluators such as
  `Weighted64<NormalizedEval64<M, G>>`;
* a common `Eval<Ctx>` interface;
* a dynamically composed score set for runtime-defined configurations.

Applications may choose the representation that best matches their workload.

### Static application-defined score sets

When an application has a predefined metric set, it may define a concrete score-set type directly:

```rust
pub struct DefaultScoreSet {
    latency: Weighted64<NormalizedEval64<Latency, Cauchy>>,
    cpu: Weighted64<NormalizedEval64<CpuUsage, Linear>>,
    similarity: Weighted64<NormalizedEval64<Similarity, Sigmoid>>,
}
```

Its evaluation can be expressed through direct aggregation:

```rust
impl Eval<Context> for DefaultScoreSet {
    #[inline]
    fn eval(&self, ctx: &Context) -> f64 {
        self.latency.eval(ctx)
            + self.cpu.eval(ctx)
            + self.similarity.eval(ctx)
    }
}
```

Because the complete structure is known at compile time, Rust may monomorphize and inline the evaluation path.

This type is application-defined. It is not a special execution mode required or managed by the library.

### Runtime-configurable score sets

When a metric set is selected from runtime data, the application may construct a `DynScoreSet`:

```text
runtime configuration
    -> concrete Weighted64<NormalizedEval64<M, G>> values
    -> Box<dyn Eval<Ctx>>
    -> DynScoreSet
```

This representation supports arbitrary runtime-selected metric subsets while preserving concrete generic implementations inside each metric.

The dynamic boundary is limited to the collection of heterogeneous metrics:

```text
DynScoreSet
    -> dyn Eval<Ctx>
    -> Weighted64<NormalizedEval64<M, G>>
```

Inside each concrete composed evaluator, the measurement and mapping types
remain statically known.

### Optional application-level specialization

Applications whose workload is dominated by one predefined configuration may choose to use a static type for that common case and `DynScoreSet` only for runtime overrides.

For example:

```rust
match custom_config {
    None => {
        let score_set = DefaultScoreSet::new()?;
        run_service(score_set).await
    }

    Some(config) => {
        let score_set = DynScoreSet::compile(config)?;
        run_service(score_set).await
    }
}
```

The service can remain generic:

```rust
async fn run_service<E>(
    score_set: E,
) -> Result<(), Error>
where
    E: Eval<Context> + Send + Sync + 'static,
{
    // Service implementation.
}
```

In this architecture, the compiler instantiates the service for the application-defined top-level score-set types:

```text
run_service::<DefaultScoreSet>
run_service::<DynScoreSet>
```

It does not instantiate the service for every possible runtime metric subset.

This split is an application optimization, not a requirement of `score-set`.

## Representation principle

Compile-time and runtime composition have different representation requirements:

```text
compile-time-known composition -> concrete generic type
runtime-selected composition   -> type-erased heterogeneous collection
```

The library supports both representations without requiring applications to expose both.

Applications may use only concrete score sets, only `DynScoreSet`, or a combination of the two.

Dynamic dispatch is introduced only when runtime heterogeneity must be represented. Concrete metric implementations remain generic and statically typed.

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License
