pub trait MeasureF64<Ctx: ?Sized>: Send + Sync {
    fn measure(&self, ctx: &Ctx) -> f64;
}

pub trait Map01F64: Send + Sync {
    fn map(&self, value: f64) -> f64;
}

pub trait EvalF64<Ctx: ?Sized>: Send + Sync {
    fn eval(&self, ctx: &Ctx) -> f64;
}

pub trait MeasureF32<Ctx: ?Sized>: Send + Sync {
    fn measure(&self, ctx: &Ctx) -> f32;
}

pub trait Map01F32: Send + Sync {
    fn map(&self, value: f32) -> f32;
}

pub trait EvalF32<Ctx: ?Sized>: Send + Sync {
    fn eval(&self, ctx: &Ctx) -> f32;
}
