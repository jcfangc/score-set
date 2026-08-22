mod support;

use score_set::{
    DynScoreSet32, DynScoreSet64, Metric32, Metric64,
    traits::{EvalF32, EvalF64},
};
use support::{
    Context32, Context64, CpuUsage32, CpuUsage64, Identity32, Identity64, Latency32, Latency64,
    LowerIsBetter32, LowerIsBetter64,
};

#[test]
fn f64_dyn_score_set_sums_heterogeneous_metrics() {
    let score_set = DynScoreSet64::<Context64>::builder()
        .append(Metric64::new(
            Latency64,
            LowerIsBetter64 { limit: 100.0 },
            0.7,
        ))
        .append(Metric64::new(CpuUsage64, Identity64, 0.3))
        .build();

    let cases = [
        (
            Context64 {
                latency_ms: 40.0,
                cpu_usage: 0.25,
            },
            0.495,
        ),
        (
            Context64 {
                latency_ms: 0.0,
                cpu_usage: 1.0,
            },
            1.0,
        ),
        (
            Context64 {
                latency_ms: 200.0,
                cpu_usage: -1.0,
            },
            0.0,
        ),
    ];

    for (ctx, expected) in cases {
        let actual = score_set.eval(&ctx);
        assert!(
            (actual - expected).abs() < 1e-12,
            "expected {expected}, got {actual}",
        );
    }
}

#[test]
fn f32_dyn_score_set_sums_heterogeneous_metrics() {
    let score_set = DynScoreSet32::<Context32>::builder()
        .append(Metric32::new(
            Latency32,
            LowerIsBetter32 { limit: 100.0 },
            0.7,
        ))
        .append(Metric32::new(CpuUsage32, Identity32, 0.3))
        .build();

    let cases = [
        (
            Context32 {
                latency_ms: 40.0,
                cpu_usage: 0.25,
            },
            0.495,
        ),
        (
            Context32 {
                latency_ms: 0.0,
                cpu_usage: 1.0,
            },
            1.0,
        ),
        (
            Context32 {
                latency_ms: 200.0,
                cpu_usage: -1.0,
            },
            0.0,
        ),
    ];

    for (ctx, expected) in cases {
        let actual = score_set.eval(&ctx);
        assert!(
            (actual - expected).abs() < 1e-6,
            "expected {expected}, got {actual}",
        );
    }
}

#[test]
fn empty_f64_dyn_score_set_evaluates_to_zero() {
    let score_set = DynScoreSet64::<Context64>::builder().build();
    let ctx = Context64 {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    assert_eq!(score_set.eval(&ctx), 0.0);
}

#[test]
fn empty_f32_dyn_score_set_evaluates_to_zero() {
    let score_set = DynScoreSet32::<Context32>::builder().build();
    let ctx = Context32 {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    assert_eq!(score_set.eval(&ctx), 0.0);
}
