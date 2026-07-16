use score_set::{
    DynScoreSet, Metric,
    traits::{Eval, Map01, Measure},
};

#[derive(Clone, Copy)]
struct Context {
    latency_ms: f64,
    cpu_usage: f64,
}

struct Latency;

impl Measure<Context> for Latency {
    fn measure(&self, ctx: &Context) -> f64 {
        ctx.latency_ms
    }
}

struct CpuUsage;

impl Measure<Context> for CpuUsage {
    fn measure(&self, ctx: &Context) -> f64 {
        ctx.cpu_usage
    }
}

struct LowerIsBetter {
    limit: f64,
}

impl Map01 for LowerIsBetter {
    fn map(&self, value: f64) -> f64 {
        (1.0 - value / self.limit).clamp(0.0, 1.0)
    }
}

struct Identity;

impl Map01 for Identity {
    fn map(&self, value: f64) -> f64 {
        value.clamp(0.0, 1.0)
    }
}

#[test]
fn metric_composes_measure_map_and_weight() {
    let metric = Metric::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7);

    let ctx = Context {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    // 0.7 * (1 - 40 / 100) = 0.42
    assert!((metric.eval(&ctx) - 0.42).abs() < 1e-12);
}

#[test]
fn dyn_score_set_sums_heterogeneous_metrics() {
    let score_set = DynScoreSet::<Context>::builder()
        .append(Metric::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7))
        .append(Metric::new(CpuUsage, Identity, 0.3))
        .build();

    let cases = [
        (
            Context {
                latency_ms: 40.0,
                cpu_usage: 0.25,
            },
            0.495,
        ),
        (
            Context {
                latency_ms: 0.0,
                cpu_usage: 1.0,
            },
            1.0,
        ),
        (
            Context {
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
fn empty_dyn_score_set_evaluates_to_zero() {
    let score_set = DynScoreSet::<Context>::builder().build();

    let ctx = Context {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    assert_eq!(score_set.eval(&ctx), 0.0);
}
