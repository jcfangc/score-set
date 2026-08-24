use score_set::traits::{PartialEvalF32, PartialEvalF64};

struct TwoStage;

impl PartialEvalF64<(f64, f64)> for TwoStage {
    fn partial(&self, ctx: &(f64, f64)) -> f64 {
        ctx.0
    }

    fn residual(&self, ctx: &(f64, f64)) -> f64 {
        ctx.1
    }
}

impl PartialEvalF32<(f32, f32)> for TwoStage {
    fn partial(&self, ctx: &(f32, f32)) -> f32 {
        ctx.0
    }

    fn residual(&self, ctx: &(f32, f32)) -> f32 {
        ctx.1
    }
}

#[test]
fn evaluates_f64_in_two_stages_through_a_trait_object() {
    let evaluator: Box<dyn PartialEvalF64<(f64, f64)>> = Box::new(TwoStage);
    let ctx = (1.0, 4.0);

    assert_eq!(evaluator.partial(&ctx), 1.0);
    assert_eq!(evaluator.residual(&ctx), 4.0);
}

#[test]
fn evaluates_f32_in_two_stages_through_a_trait_object() {
    let evaluator: Box<dyn PartialEvalF32<(f32, f32)>> = Box::new(TwoStage);
    let ctx = (1.0, 4.0);

    assert_eq!(evaluator.partial(&ctx), 1.0);
    assert_eq!(evaluator.residual(&ctx), 4.0);
}

struct Sum<A, B> {
    first: A,
    second: B,
}

impl<Ctx: ?Sized, A, B> PartialEvalF64<Ctx> for Sum<A, B>
where
    A: PartialEvalF64<Ctx>,
    B: PartialEvalF64<Ctx>,
{
    fn partial(&self, ctx: &Ctx) -> f64 {
        self.first.partial(ctx) + self.second.partial(ctx)
    }

    fn residual(&self, ctx: &Ctx) -> f64 {
        self.first.residual(ctx) + self.second.residual(ctx)
    }
}

#[test]
fn partial_evaluators_compose_recursively() {
    let evaluator = Sum {
        first: TwoStage,
        second: Sum {
            first: TwoStage,
            second: TwoStage,
        },
    };
    let ctx = (1.0, 4.0);

    assert_eq!(evaluator.partial(&ctx), 3.0);
    assert_eq!(evaluator.residual(&ctx), 12.0);
}
