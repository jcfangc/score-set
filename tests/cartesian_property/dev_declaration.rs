//! Developer declaration — macro-expanded domain-specific code.
//!
//! This module represents what a `score_space!` macro generates when given:
//!
//! ```ignore
//! score_space! {
//!     context = Sample;
//!
//!     measures {
//!         X,
//!         Y,
//!         OffsetZ { offset: f32 },
//!     }
//!
//!     maps {
//!         Identity,
//!         Positive,
//!         Clamp { min: f32, max: f32 },
//!     }
//! }
//! ```
//!
//! # What gets generated
//!
//! For each Measure×Map pair (3×3 = 9):
//! - A **config struct** (proto-message equivalent): carries weight + params
//! - A **concrete metric type** (alias for `Metric<M, P, Sample>`)
//!
//! Plus:
//! - **`ScoreConfig`** — aggregates all 9 optional config messages
//! - **`ScoreSet`** — the compiled scorer with all 9 metrics + weights
//! - **`ScoreSet::build(config)`** — constructor (compile phase)
//! - **`ScoreSet::score(&self, ctx)`** — branchless arithmetic (hot path)
//!
//! # Architecture
//!
//! ```text
//! protobuf wire bytes
//!       │ (proto deserialize — once per config change)
//!       ▼
//! ScoreConfig { gc_linear: Option<GcLinearMsg>, ... }
//!       │ (ScoreSet::build — once per config change)
//!       ▼
//! ScoreSet { gc_linear: Metric<Gc, Linear>, weights: [f32; 9], ... }
//!       │ (score — millions of times)
//!       ▼
//! branchless arithmetic: Σ metric.score(ctx) * weight
//! ```
//!
//! `dyn`/`Box`/`match` only exist in `build()`. Hot path is pure arithmetic.

use super::Sample;
use super::lib_internals::*;

// ============================================================================
// 1. Measure implementations (developer-defined)
// ============================================================================

pub(crate) struct X;
impl Measure<Sample> for X {
    #[inline]
    fn eval(&self, s: &Sample) -> f32 {
        s.x
    }
}

pub(crate) struct Y;
impl Measure<Sample> for Y {
    #[inline]
    fn eval(&self, s: &Sample) -> f32 {
        s.y
    }
}

/// Parameterized measure: `sample.z + offset`.
pub(crate) struct OffsetZ {
    pub offset: f32,
}
impl Measure<Sample> for OffsetZ {
    #[inline]
    fn eval(&self, s: &Sample) -> f32 {
        s.z + self.offset
    }
}

// ============================================================================
// 2. Generated config structs — one per Measure×Map combination
// ============================================================================
//
// Each struct is the "proto message" for that specific (measure, map) pair.
// It carries: weight + measure params + map params.

#[derive(Clone, Copy, Debug)]
pub(crate) struct XIdentityMsg {
    pub weight: f32,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct XPositiveMsg {
    pub weight: f32,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct XClampMsg {
    pub weight: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct YIdentityMsg {
    pub weight: f32,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct YPositiveMsg {
    pub weight: f32,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct YClampMsg {
    pub weight: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OffsetZIdentityMsg {
    pub weight: f32,
    pub offset: f32,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct OffsetZPositiveMsg {
    pub weight: f32,
    pub offset: f32,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct OffsetZClampMsg {
    pub weight: f32,
    pub offset: f32,
    pub min: f32,
    pub max: f32,
}

// ============================================================================
// 3. ScoreConfig — aggregated proto message (generated)
// ============================================================================

/// Top-level config message. Each `optional` field = one supported metric.
///
/// This IS the protobuf schema, rendered as Rust structs. The caller
/// (proto_caller) populates whichever metrics they want active.
#[derive(Clone, Debug, Default)]
pub(crate) struct ScoreConfig {
    pub x_identity: Option<XIdentityMsg>,
    pub x_positive: Option<XPositiveMsg>,
    pub x_clamp: Option<XClampMsg>,
    pub y_identity: Option<YIdentityMsg>,
    pub y_positive: Option<YPositiveMsg>,
    pub y_clamp: Option<YClampMsg>,
    pub offset_z_identity: Option<OffsetZIdentityMsg>,
    pub offset_z_positive: Option<OffsetZPositiveMsg>,
    pub offset_z_clamp: Option<OffsetZClampMsg>,
}

// ============================================================================
// 4. ScoreSet — compiled scorer (generated)
// ============================================================================

/// The fully-compiled, branchless scoring ensemble.
///
/// All |M|×|P| metrics are stored by value. Inactive slots get `weight = 0.0`.
/// `score()` is a single arithmetic expression — zero branches.
pub(crate) struct ScoreSet {
    // Metrics
    x_identity: Metric<X, Identity, Sample>,
    x_positive: Metric<X, Positive, Sample>,
    x_clamp: Metric<X, Clamp, Sample>,
    y_identity: Metric<Y, Identity, Sample>,
    y_positive: Metric<Y, Positive, Sample>,
    y_clamp: Metric<Y, Clamp, Sample>,
    offset_z_identity: Metric<OffsetZ, Identity, Sample>,
    offset_z_positive: Metric<OffsetZ, Positive, Sample>,
    offset_z_clamp: Metric<OffsetZ, Clamp, Sample>,

    // Normalized weights (0.0 = inactive)
    w_x_identity: f32,
    w_x_positive: f32,
    w_x_clamp: f32,
    w_y_identity: f32,
    w_y_positive: f32,
    w_y_clamp: f32,
    w_offset_z_identity: f32,
    w_offset_z_positive: f32,
    w_offset_z_clamp: f32,
}

impl ScoreSet {
    /// Build a [`ScoreSet`] from a [`ScoreConfig`].
    ///
    /// This is the **compile phase** — runs once per config change.
    /// `match`/`Option`/branching are allowed here.
    #[inline]
    pub(crate) fn build(config: &ScoreConfig) -> Self {
        // Accumulate raw weights
        let mut w_x_identity = 0.0f32;
        let mut w_x_positive = 0.0f32;
        let mut w_x_clamp = 0.0f32;
        let mut w_y_identity = 0.0f32;
        let mut w_y_positive = 0.0f32;
        let mut w_y_clamp = 0.0f32;
        let mut w_offset_z_identity = 0.0f32;
        let mut w_offset_z_positive = 0.0f32;
        let mut w_offset_z_clamp = 0.0f32;

        // Construct metrics from config (params captured by value)
        if let Some(m) = &config.x_identity {
            w_x_identity = m.weight;
        }
        if let Some(m) = &config.x_positive {
            w_x_positive = m.weight;
        }
        let x_clamp = match &config.x_clamp {
            Some(m) => {
                w_x_clamp = m.weight;
                Metric::new(
                    X,
                    Clamp {
                        min: m.min,
                        max: m.max,
                    },
                )
            }
            None => Metric::new(X, Clamp { min: 0.0, max: 1.0 }),
        };
        if let Some(m) = &config.y_identity {
            w_y_identity = m.weight;
        }
        if let Some(m) = &config.y_positive {
            w_y_positive = m.weight;
        }
        let y_clamp = match &config.y_clamp {
            Some(m) => {
                w_y_clamp = m.weight;
                Metric::new(
                    Y,
                    Clamp {
                        min: m.min,
                        max: m.max,
                    },
                )
            }
            None => Metric::new(Y, Clamp { min: 0.0, max: 1.0 }),
        };
        let offset_z_identity = match &config.offset_z_identity {
            Some(m) => {
                w_offset_z_identity = m.weight;
                Metric::new(OffsetZ { offset: m.offset }, Identity)
            }
            None => Metric::new(OffsetZ { offset: 0.0 }, Identity),
        };
        let offset_z_positive = match &config.offset_z_positive {
            Some(m) => {
                w_offset_z_positive = m.weight;
                Metric::new(OffsetZ { offset: m.offset }, Positive)
            }
            None => Metric::new(OffsetZ { offset: 0.0 }, Positive),
        };
        let offset_z_clamp = match &config.offset_z_clamp {
            Some(m) => {
                w_offset_z_clamp = m.weight;
                Metric::new(
                    OffsetZ { offset: m.offset },
                    Clamp {
                        min: m.min,
                        max: m.max,
                    },
                )
            }
            None => Metric::new(OffsetZ { offset: 0.0 }, Clamp { min: 0.0, max: 1.0 }),
        };

        let total = w_x_identity
            + w_x_positive
            + w_x_clamp
            + w_y_identity
            + w_y_positive
            + w_y_clamp
            + w_offset_z_identity
            + w_offset_z_positive
            + w_offset_z_clamp;

        assert!(total > 0.0, "at least one metric must be active");

        ScoreSet {
            x_identity: Metric::new(X, Identity),
            x_positive: Metric::new(X, Positive),
            x_clamp,
            y_identity: Metric::new(Y, Identity),
            y_positive: Metric::new(Y, Positive),
            y_clamp,
            offset_z_identity,
            offset_z_positive,
            offset_z_clamp,

            w_x_identity: w_x_identity / total,
            w_x_positive: w_x_positive / total,
            w_x_clamp: w_x_clamp / total,
            w_y_identity: w_y_identity / total,
            w_y_positive: w_y_positive / total,
            w_y_clamp: w_y_clamp / total,
            w_offset_z_identity: w_offset_z_identity / total,
            w_offset_z_positive: w_offset_z_positive / total,
            w_offset_z_clamp: w_offset_z_clamp / total,
        }
    }

    /// Score via **branchless arithmetic**.
    ///
    /// This is the **hot path** — runs millions of times. No `match`, no
    /// `enum`, no `if`, no `Box`, no `dyn`. With full inlining, the compiler
    /// sees a single floating-point expression:
    ///
    /// ```text
    /// s.x * w_x_identity
    /// + s.x.max(0) * w_x_positive
    /// + s.x.clamp(min,max) * w_x_clamp
    /// + s.y * w_y_identity
    /// + ...
    /// + (s.z+offset).clamp(min,max) * w_offset_z_clamp
    /// ```
    #[inline]
    pub(crate) fn score(&self, ctx: &Sample) -> f32 {
        self.x_identity.score(ctx) * self.w_x_identity
            + self.x_positive.score(ctx) * self.w_x_positive
            + self.x_clamp.score(ctx) * self.w_x_clamp
            + self.y_identity.score(ctx) * self.w_y_identity
            + self.y_positive.score(ctx) * self.w_y_positive
            + self.y_clamp.score(ctx) * self.w_y_clamp
            + self.offset_z_identity.score(ctx) * self.w_offset_z_identity
            + self.offset_z_positive.score(ctx) * self.w_offset_z_positive
            + self.offset_z_clamp.score(ctx) * self.w_offset_z_clamp
    }
}

// ============================================================================
// 5. MetricSlot — kept for reference oracle only
// ============================================================================

/// Proto-level identifier for the reference oracle.
///
/// NOT used in the hot path — only in [`reference_score`] for testing.
#[derive(Clone, Copy, Debug)]
pub(crate) enum MetricSlot {
    XIdentity,
    XPositive,
    XClamp { min: f32, max: f32 },
    YIdentity,
    YPositive,
    YClamp { min: f32, max: f32 },
    OffsetZIdentity { offset: f32 },
    OffsetZPositive { offset: f32 },
    OffsetZClamp { offset: f32, min: f32, max: f32 },
}

// ============================================================================
// 6. Reference interpreter — oracle for property tests
// ============================================================================

fn reference_measure(slot: MetricSlot, ctx: &Sample) -> f32 {
    match slot {
        MetricSlot::XIdentity | MetricSlot::XPositive | MetricSlot::XClamp { .. } => ctx.x,
        MetricSlot::YIdentity | MetricSlot::YPositive | MetricSlot::YClamp { .. } => ctx.y,
        MetricSlot::OffsetZIdentity { offset }
        | MetricSlot::OffsetZPositive { offset }
        | MetricSlot::OffsetZClamp { offset, .. } => ctx.z + offset,
    }
}

fn reference_map(slot: MetricSlot, raw: f32) -> f32 {
    match slot {
        MetricSlot::XIdentity | MetricSlot::YIdentity | MetricSlot::OffsetZIdentity { .. } => raw,
        MetricSlot::XPositive | MetricSlot::YPositive | MetricSlot::OffsetZPositive { .. } => {
            raw.max(0.0)
        }
        MetricSlot::XClamp { min, max } | MetricSlot::YClamp { min, max } => raw.clamp(min, max),
        MetricSlot::OffsetZClamp { min, max, .. } => raw.clamp(min, max),
    }
}

pub(crate) fn reference_score(slots: &[(MetricSlot, f32)], ctx: &Sample) -> f32 {
    let total: f32 = slots.iter().map(|(_, w)| w).sum();
    if total == 0.0 {
        return 0.0;
    }
    slots
        .iter()
        .map(|&(slot, w)| {
            let raw = reference_measure(slot, ctx);
            let score = reference_map(slot, raw);
            score * w / total
        })
        .sum()
}
