#![allow(dead_code)]

use score_set::traits::{Map01F32, Map01F64, Measure, V01, prove_v01_f32, prove_v01_f64};
use witnessed::{WitnessExt, Witnessed};

#[derive(Clone, Copy)]
pub struct Context64 {
    pub latency_ms: f64,
    pub cpu_usage: f64,
}

pub struct Latency64;

impl Measure<Context64> for Latency64 {
    type Output = f64;

    fn measure(&self, ctx: &Context64) -> Self::Output {
        ctx.latency_ms
    }
}

pub struct CpuUsage64;

impl Measure<Context64> for CpuUsage64 {
    type Output = f64;

    fn measure(&self, ctx: &Context64) -> Self::Output {
        ctx.cpu_usage
    }
}

pub struct LowerIsBetter64 {
    pub limit: f64,
}

impl Map01F64 for LowerIsBetter64 {
    type Input = f64;

    fn map(&self, value: Self::Input) -> Witnessed<f64, V01> {
        let value = (1.0 - value / self.limit).clamp(0.0, 1.0);
        value
            .witness()
            .by(prove_v01_f64)
            .expect("value was clamped")
    }
}

pub struct Identity64;

impl Map01F64 for Identity64 {
    type Input = f64;

    fn map(&self, value: Self::Input) -> Witnessed<f64, V01> {
        let value = value.clamp(0.0, 1.0);
        value
            .witness()
            .by(prove_v01_f64)
            .expect("value was clamped")
    }
}

#[derive(Clone, Copy)]
pub struct Context32 {
    pub latency_ms: f32,
    pub cpu_usage: f32,
}

pub struct Latency32;

impl Measure<Context32> for Latency32 {
    type Output = f32;

    fn measure(&self, ctx: &Context32) -> Self::Output {
        ctx.latency_ms
    }
}

pub struct CpuUsage32;

impl Measure<Context32> for CpuUsage32 {
    type Output = f32;

    fn measure(&self, ctx: &Context32) -> Self::Output {
        ctx.cpu_usage
    }
}

pub struct LowerIsBetter32 {
    pub limit: f32,
}

impl Map01F32 for LowerIsBetter32 {
    type Input = f32;

    fn map(&self, value: Self::Input) -> Witnessed<f32, V01> {
        let value = (1.0 - value / self.limit).clamp(0.0, 1.0);
        value
            .witness()
            .by(prove_v01_f32)
            .expect("value was clamped")
    }
}

pub struct Identity32;

impl Map01F32 for Identity32 {
    type Input = f32;

    fn map(&self, value: Self::Input) -> Witnessed<f32, V01> {
        let value = value.clamp(0.0, 1.0);
        value
            .witness()
            .by(prove_v01_f32)
            .expect("value was clamped")
    }
}
