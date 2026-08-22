mod support;

use score_set::{
    Metric32, Metric64,
    traits::{EvalF32, EvalF64},
};
use support::{Context32, Context64, Latency32, Latency64, LowerIsBetter32, LowerIsBetter64};

#[test]
fn f64_metric_composes_measure_map_and_weight() {
    let metric = Metric64::new(Latency64, LowerIsBetter64 { limit: 100.0 }, 0.7);
    let ctx = Context64 {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    // 0.7 * (1 - 40 / 100) = 0.42
    assert!((metric.eval(&ctx) - 0.42).abs() < 1e-12);
}

#[test]
fn f32_metric_composes_measure_map_and_weight() {
    let metric = Metric32::new(Latency32, LowerIsBetter32 { limit: 100.0 }, 0.7);
    let ctx = Context32 {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    // 0.7 * (1 - 40 / 100) = 0.42
    assert!((metric.eval(&ctx) - 0.42).abs() < 1e-6);
}
