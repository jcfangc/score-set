pub trait Measure<Ctx: ?Sized>: Send + Sync {
    fn measure(&self, ctx: &Ctx) -> f64;
}

pub trait Map01: Send + Sync {
    fn map(&self, value: f64) -> f64;
}

pub trait Eval<Ctx: ?Sized>: Send + Sync {
    fn eval(&self, ctx: &Ctx) -> f64;
}
