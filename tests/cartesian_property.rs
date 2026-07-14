//! Property tests for the finite scoring space generator pattern.
//!
//! This test validates the architecture:
//!
//! 1. **Declaration**: `score_space! { measures { X, Y, OffsetZ } maps { Identity, Positive, Clamp } }`
//! 2. **Generation** (what the macro produces):
//!    - Config structs per Measure×Map pair (the "proto messages")
//!    - `ScoreConfig` — aggregates all as `optional` fields
//!    - `ScoreSet` — all |M|×|P| metrics stored by value + normalized weights
//! 3. **Compile phase** (once per config change):
//!    `ScoreSet::build(&config)` — params captured, weights normalized
//! 4. **Hot path** (millions of calls):
//!    `score_set.score(&ctx)` — branchless arithmetic, zero dispatch

#[path = "cartesian_property/dev_declaration.rs"]
mod dev_declaration;
#[path = "cartesian_property/lib_internals.rs"]
mod lib_internals;
#[path = "cartesian_property/proto_caller.rs"]
mod proto_caller;

use dev_declaration::*;
use proptest::prelude::*;
use proto_caller::UserConfig;

// ============================================================================
// Domain
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct Sample {
    x: f32,
    y: f32,
    z: f32,
}

// ============================================================================
// Proptest strategies
// ============================================================================

fn arb_f32() -> impl Strategy<Value = f32> {
    (-1_000_000i32..=1_000_000i32).prop_map(|i| i as f32 * 0.001)
}

fn arb_sample() -> impl Strategy<Value = Sample> {
    (arb_f32(), arb_f32(), arb_f32()).prop_map(|(x, y, z)| Sample { x, y, z })
}

fn arb_weight() -> impl Strategy<Value = f32> {
    (0.01f32..100.0f32).prop_map(|v| v)
}

/// Generate a random [`ScoreConfig`] — simulates proto deserialization.
///
/// Each of the 9 optional metric messages may or may not be set.
fn arb_score_config() -> impl Strategy<Value = ScoreConfig> {
    // Each metric: (present_bool, weight, measure_params, map_params)
    let metric = |present: bool,
                  weight: f32,
                  extra: f32,
                  extra2: f32|
     -> (
        Option<XIdentityMsg>,
        Option<XPositiveMsg>,
        Option<XClampMsg>,
        Option<YIdentityMsg>,
        Option<YPositiveMsg>,
        Option<YClampMsg>,
        Option<OffsetZIdentityMsg>,
        Option<OffsetZPositiveMsg>,
        Option<OffsetZClampMsg>,
    ) {
        let xi = present.then_some(XIdentityMsg { weight });
        let xp = present.then_some(XPositiveMsg { weight });
        let xc = present.then_some(XClampMsg {
            weight,
            min: extra.min(extra2),
            max: extra.max(extra2),
        });
        let yi = present.then_some(YIdentityMsg { weight });
        let yp = present.then_some(YPositiveMsg { weight });
        let yc = present.then_some(YClampMsg {
            weight,
            min: extra.min(extra2),
            max: extra.max(extra2),
        });
        let ozi = present.then_some(OffsetZIdentityMsg {
            weight,
            offset: extra,
        });
        let ozp = present.then_some(OffsetZPositiveMsg {
            weight,
            offset: extra,
        });
        let ozc = present.then_some(OffsetZClampMsg {
            weight,
            offset: extra,
            min: extra.min(extra2),
            max: extra.max(extra2),
        });
        (xi, xp, xc, yi, yp, yc, ozi, ozp, ozc)
    };

    (any::<bool>(), arb_weight(), arb_f32(), arb_f32())
        .prop_map(move |(present, w, a, b)| metric(present, w, a, b))
        .prop_map(
            |(
                x_identity,
                x_positive,
                x_clamp,
                y_identity,
                y_positive,
                y_clamp,
                offset_z_identity,
                offset_z_positive,
                offset_z_clamp,
            )| {
                ScoreConfig {
                    x_identity,
                    x_positive,
                    x_clamp,
                    y_identity,
                    y_positive,
                    y_clamp,
                    offset_z_identity,
                    offset_z_positive,
                    offset_z_clamp,
                }
            },
        )
        .prop_filter("at least one metric active", |c| {
            // At least one field must be Some
            c.x_identity.is_some()
                || c.x_positive.is_some()
                || c.x_clamp.is_some()
                || c.y_identity.is_some()
                || c.y_positive.is_some()
                || c.y_clamp.is_some()
                || c.offset_z_identity.is_some()
                || c.offset_z_positive.is_some()
                || c.offset_z_clamp.is_some()
        })
}

/// Build a reference config from a ScoreConfig for the oracle.
fn config_to_slots(config: &ScoreConfig) -> Vec<(MetricSlot, f32)> {
    let mut slots = Vec::new();
    if let Some(m) = &config.x_identity {
        slots.push((MetricSlot::XIdentity, m.weight));
    }
    if let Some(m) = &config.x_positive {
        slots.push((MetricSlot::XPositive, m.weight));
    }
    if let Some(m) = &config.x_clamp {
        slots.push((
            MetricSlot::XClamp {
                min: m.min,
                max: m.max,
            },
            m.weight,
        ));
    }
    if let Some(m) = &config.y_identity {
        slots.push((MetricSlot::YIdentity, m.weight));
    }
    if let Some(m) = &config.y_positive {
        slots.push((MetricSlot::YPositive, m.weight));
    }
    if let Some(m) = &config.y_clamp {
        slots.push((
            MetricSlot::YClamp {
                min: m.min,
                max: m.max,
            },
            m.weight,
        ));
    }
    if let Some(m) = &config.offset_z_identity {
        slots.push((MetricSlot::OffsetZIdentity { offset: m.offset }, m.weight));
    }
    if let Some(m) = &config.offset_z_positive {
        slots.push((MetricSlot::OffsetZPositive { offset: m.offset }, m.weight));
    }
    if let Some(m) = &config.offset_z_clamp {
        slots.push((
            MetricSlot::OffsetZClamp {
                offset: m.offset,
                min: m.min,
                max: m.max,
            },
            m.weight,
        ));
    }
    slots
}

// ============================================================================
// Tests
// ============================================================================

proptest! {
    /// The branchless ScoreSet must match the reference oracle.
    #[test]
    fn score_set_equals_reference(
        config in arb_score_config(),
        sample in arb_sample(),
    ) {
        let score_set = ScoreSet::build(&config);
        let result = score_set.score(&sample);

        let slots = config_to_slots(&config);
        let reference = reference_score(&slots, &sample);

        let tol = 1e-5f32.max(reference.abs() * f32::EPSILON * 128.0);
        prop_assert!(
            (result - reference).abs() < tol,
            "mismatch: result={:.6}, reference={:.6}, diff={:.6e}, tol={:.6e}",
            result, reference, result - reference, tol,
        );
    }

    /// Single-metric configs: each variant scores correctly.
    #[test]
    fn single_metric_scores_correctly(
        config in arb_score_config(),
        sample in arb_sample(),
    ) {
        let score_set = ScoreSet::build(&config);
        let result = score_set.score(&sample);
        let slots = config_to_slots(&config);
        let reference = reference_score(&slots, &sample);

        let tol = 1e-5f32.max(reference.abs() * f32::EPSILON * 128.0);
        prop_assert!(
            (result - reference).abs() < tol,
            "mismatch: result={:.6}, reference={:.6}",
            result, reference,
        );
    }
}

/// Proto caller works without importing internal types.
#[test]
fn proto_caller_works_without_internals() {
    let config = UserConfig {
        score_config: ScoreConfig {
            x_identity: Some(XIdentityMsg { weight: 1.0 }),
            y_positive: Some(YPositiveMsg { weight: 2.0 }),
            ..Default::default()
        },
    };
    let sample = Sample {
        x: 0.5,
        y: -0.3,
        z: 1.2,
    };
    let scores = proto_caller::score_batch(&config, &[sample]);
    assert_eq!(scores.len(), 1);
    assert!(scores[0].is_finite());
}

/// ScoreSet is compact — params stored inline, no heap per-metric.
#[test]
fn score_set_is_compact() {
    use core::mem::size_of;
    let size = size_of::<ScoreSet>();
    // 9 metrics (each = measure+map+PhantomData, mostly ZSTs or small f32 fields)
    // + 9 f32 weights. Well under 256 bytes.
    assert!(
        size <= 256,
        "ScoreSet should be compact (<= 256 bytes), got {} bytes",
        size
    );
}

/// Manual enumeration of all 9 variant configs.
#[test]
fn smoke_test_all_9_variants() {
    let sample = Sample {
        x: 1.0,
        y: -2.0,
        z: 3.0,
    };

    let cases = [
        (
            ScoreConfig {
                x_identity: Some(XIdentityMsg { weight: 1.0 }),
                ..Default::default()
            },
            1.0,
        ),
        (
            ScoreConfig {
                x_positive: Some(XPositiveMsg { weight: 1.0 }),
                ..Default::default()
            },
            1.0,
        ),
        (
            ScoreConfig {
                x_clamp: Some(XClampMsg {
                    weight: 1.0,
                    min: 0.0,
                    max: 0.5,
                }),
                ..Default::default()
            },
            0.5,
        ),
        (
            ScoreConfig {
                y_identity: Some(YIdentityMsg { weight: 1.0 }),
                ..Default::default()
            },
            -2.0,
        ),
        (
            ScoreConfig {
                y_positive: Some(YPositiveMsg { weight: 1.0 }),
                ..Default::default()
            },
            0.0,
        ),
        (
            ScoreConfig {
                y_clamp: Some(YClampMsg {
                    weight: 1.0,
                    min: -1.0,
                    max: 1.0,
                }),
                ..Default::default()
            },
            -1.0,
        ),
        (
            ScoreConfig {
                offset_z_identity: Some(OffsetZIdentityMsg {
                    weight: 1.0,
                    offset: 5.0,
                }),
                ..Default::default()
            },
            8.0,
        ),
        (
            ScoreConfig {
                offset_z_positive: Some(OffsetZPositiveMsg {
                    weight: 1.0,
                    offset: 5.0,
                }),
                ..Default::default()
            },
            8.0,
        ),
        (
            ScoreConfig {
                offset_z_clamp: Some(OffsetZClampMsg {
                    weight: 1.0,
                    offset: 5.0,
                    min: 0.0,
                    max: 6.0,
                }),
                ..Default::default()
            },
            6.0,
        ),
    ];

    for (config, expected) in &cases {
        let score_set = ScoreSet::build(config);
        let result = score_set.score(&sample);
        assert!(
            (result - expected).abs() < 1e-6,
            "result={}, expected={}",
            result,
            expected,
        );
    }
}
