use score_set::{
    DynScoreSet32, DynScoreSet64, Metric32, Metric64,
    traits::{EvalF32, EvalF64, Map01F32, Map01F64, Measure, V01},
};
use witnessed::{WitnessExt, Witnessed};

mod f64_tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Context {
        latency_ms: f64,
        cpu_usage: f64,
    }

    struct Latency;

    impl Measure<Context> for Latency {
        type Output = f64;

        fn measure(&self, ctx: &Context) -> Self::Output {
            ctx.latency_ms
        }
    }

    struct CpuUsage;

    impl Measure<Context> for CpuUsage {
        type Output = f64;

        fn measure(&self, ctx: &Context) -> Self::Output {
            ctx.cpu_usage
        }
    }

    struct LowerIsBetter {
        limit: f64,
    }

    impl Map01F64 for LowerIsBetter {
        type Input = f64;

        fn map(&self, value: Self::Input) -> Witnessed<f64, V01> {
            let value = (1.0 - value / self.limit).clamp(0.0, 1.0);
            // Safety: the value is clamped to the `[0, 1]` range above.
            unsafe { value.witness().by_unchecked::<V01>() }
        }
    }

    struct Identity;

    impl Map01F64 for Identity {
        type Input = f64;

        fn map(&self, value: Self::Input) -> Witnessed<f64, V01> {
            let value = value.clamp(0.0, 1.0);
            // Safety: the value is clamped to the `[0, 1]` range above.
            unsafe { value.witness().by_unchecked::<V01>() }
        }
    }

    #[test]
    fn metric_composes_measure_map_and_weight() {
        let metric = Metric64::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7);

        let ctx = Context {
            latency_ms: 40.0,
            cpu_usage: 0.25,
        };

        // 0.7 * (1 - 40 / 100) = 0.42
        assert!((metric.eval(&ctx) - 0.42).abs() < 1e-12);
    }

    #[test]
    fn dyn_score_set_sums_heterogeneous_metrics() {
        let score_set = DynScoreSet64::<Context>::builder()
            .append(Metric64::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7))
            .append(Metric64::new(CpuUsage, Identity, 0.3))
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
        let score_set = DynScoreSet64::<Context>::builder().build();

        let ctx = Context {
            latency_ms: 40.0,
            cpu_usage: 0.25,
        };

        assert_eq!(score_set.eval(&ctx), 0.0);
    }
}

mod f32_tests {
    use super::*;

    #[derive(Clone, Copy)]
    struct Context {
        latency_ms: f32,
        cpu_usage: f32,
    }

    struct Latency;

    impl Measure<Context> for Latency {
        type Output = f32;

        fn measure(&self, ctx: &Context) -> Self::Output {
            ctx.latency_ms
        }
    }

    struct CpuUsage;

    impl Measure<Context> for CpuUsage {
        type Output = f32;

        fn measure(&self, ctx: &Context) -> Self::Output {
            ctx.cpu_usage
        }
    }

    struct LowerIsBetter {
        limit: f32,
    }

    impl Map01F32 for LowerIsBetter {
        type Input = f32;

        fn map(&self, value: Self::Input) -> Witnessed<f32, V01> {
            let value = (1.0 - value / self.limit).clamp(0.0, 1.0);
            // Safety: the value is clamped to the `[0, 1]` range above.
            unsafe { value.witness().by_unchecked::<V01>() }
        }
    }

    struct Identity;

    impl Map01F32 for Identity {
        type Input = f32;

        fn map(&self, value: Self::Input) -> Witnessed<f32, V01> {
            let value = value.clamp(0.0, 1.0);
            // Safety: the value is clamped to the `[0, 1]` range above.
            unsafe { value.witness().by_unchecked::<V01>() }
        }
    }

    #[test]
    fn metric_composes_measure_map_and_weight() {
        let metric = Metric32::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7);

        let ctx = Context {
            latency_ms: 40.0,
            cpu_usage: 0.25,
        };

        // 0.7 * (1 - 40 / 100) = 0.42
        assert!((metric.eval(&ctx) - 0.42).abs() < 1e-6);
    }

    #[test]
    fn dyn_score_set_sums_heterogeneous_metrics() {
        let score_set = DynScoreSet32::<Context>::builder()
            .append(Metric32::new(Latency, LowerIsBetter { limit: 100.0 }, 0.7))
            .append(Metric32::new(CpuUsage, Identity, 0.3))
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
                (actual - expected).abs() < 1e-6,
                "expected {expected}, got {actual}",
            );
        }
    }

    #[test]
    fn empty_dyn_score_set_evaluates_to_zero() {
        let score_set = DynScoreSet32::<Context>::builder().build();

        let ctx = Context {
            latency_ms: 40.0,
            cpu_usage: 0.25,
        };

        assert_eq!(score_set.eval(&ctx), 0.0);
    }
}
