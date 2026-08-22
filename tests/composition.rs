mod support;

use score_set::{
    DynScoreSet32, DynScoreSet64, Weighted32, Weighted64, metric32, metric64,
    traits::{EvalF32, EvalF64},
    weight32, weight64,
};
use support::{
    Context32, Context64, CpuUsage32, CpuUsage64, Identity32, Identity64, Latency64,
    LowerIsBetter64,
};

struct StaticScoreSet64<A, B> {
    first: A,
    second: B,
}

impl<Ctx, A, B> EvalF64<Ctx> for StaticScoreSet64<A, B>
where
    Ctx: ?Sized,
    A: EvalF64<Ctx>,
    B: EvalF64<Ctx>,
{
    fn eval(&self, ctx: &Ctx) -> f64 {
        self.first.eval(ctx) + self.second.eval(ctx)
    }
}

#[test]
fn static_and_dynamic_f64_score_sets_compose_recursively() {
    let inner_static = StaticScoreSet64 {
        first: metric64(
            Latency64,
            LowerIsBetter64 { limit: 100.0 },
            weight64(0.4).unwrap(),
        ),
        second: metric64(CpuUsage64, Identity64, weight64(0.2).unwrap()),
    };

    let middle_dynamic = DynScoreSet64::<Context64>::builder()
        .append(inner_static)
        .append(metric64(CpuUsage64, Identity64, weight64(0.1).unwrap()))
        .build();

    let weighted_dynamic = Weighted64::new(middle_dynamic, weight64(0.8).unwrap());
    let erased_dynamic: Box<dyn EvalF64<Context64>> = Box::new(weighted_dynamic);
    let outer_static = StaticScoreSet64 {
        first: erased_dynamic,
        second: metric64(
            Latency64,
            LowerIsBetter64 { limit: 100.0 },
            weight64(0.3).unwrap(),
        ),
    };

    let erased_static: Box<dyn EvalF64<Context64>> = Box::new(outer_static);
    let outer_dynamic = DynScoreSet64::<Context64>::builder()
        .append(erased_static)
        .build();

    let ctx = Context64 {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    // inner static: 0.4 * 0.6 + 0.2 * 0.25 = 0.29
    // middle dynamic: 0.29 + 0.1 * 0.25 = 0.315
    // weighted middle dynamic: 0.8 * 0.315 = 0.252
    // outer static/dynamic: 0.252 + 0.3 * 0.6 = 0.432
    assert!((outer_dynamic.eval(&ctx) - 0.432).abs() < 1e-12);
}

#[test]
fn erased_f32_evaluator_can_be_repacked() {
    let inner_dynamic = DynScoreSet32::<Context32>::builder()
        .append(metric32(CpuUsage32, Identity32, weight32(0.5).unwrap()))
        .build();
    let erased: Box<dyn EvalF32<Context32>> =
        Box::new(Weighted32::new(inner_dynamic, weight32(0.5).unwrap()));
    let outer_dynamic = DynScoreSet32::<Context32>::builder().append(erased).build();
    let ctx = Context32 {
        latency_ms: 40.0,
        cpu_usage: 0.25,
    };

    assert!((outer_dynamic.eval(&ctx) - 0.0625).abs() < 1e-6);
}
