use crate::*;

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

struct DnaCtx {
    gc: f32,
    len: f32,
}

// ---------------------------------------------------------------------------
// Metric builder pipeline tests
// ---------------------------------------------------------------------------

#[test]
fn pipeline_identity() {
    let m = metric("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    let score = m.eval(&0.5).unwrap();
    assert!((score.into_inner() - 0.5).abs() < 1e-6);

    // Clamping
    let score = m.eval(&1.5).unwrap();
    assert!((score.into_inner() - 1.0).abs() < 1e-6);

    let score = m.eval(&-0.5).unwrap();
    assert!((score.into_inner() - 0.0).abs() < 1e-6);
}

#[test]
fn pipeline_linear() {
    let m = metric("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .linear(100.0);

    let score = m.eval(&50.0).unwrap();
    assert!((score.into_inner() - 0.5).abs() < 1e-6);

    let score = m.eval(&100.0).unwrap();
    assert!((score.into_inner() - 1.0).abs() < 1e-6);

    let score = m.eval(&0.0).unwrap();
    assert!((score.into_inner() - 0.0).abs() < 1e-6);

    // Clamping above max
    let score = m.eval(&150.0).unwrap();
    assert!((score.into_inner() - 1.0).abs() < 1e-6);
}

#[test]
fn pipeline_inc_sigmoid() {
    let m = metric("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .inc_sigmoid(0.0, 10.0);

    let lo = m.eval(&0.0).unwrap().into_inner();
    let hi = m.eval(&10.0).unwrap().into_inner();
    let mid = m.eval(&5.0).unwrap().into_inner();

    assert!(lo < 0.1); // near 0 at low end
    assert!(hi > 0.9); // near 1 at high end
    assert!((mid - 0.5).abs() < 0.1); // near 0.5 at midpoint
}

#[test]
fn pipeline_dec_sigmoid() {
    let m = metric("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .dec_sigmoid(0.0, 10.0);

    let lo = m.eval(&0.0).unwrap().into_inner();
    let hi = m.eval(&10.0).unwrap().into_inner();
    let mid = m.eval(&5.0).unwrap().into_inner();

    assert!(lo > 0.9); // near 1 at low end (decreasing)
    assert!(hi < 0.1); // near 0 at high end
    assert!((mid - 0.5).abs() < 0.1); // still near 0.5 at midpoint
}

#[test]
fn pipeline_cauchy() {
    let m = metric("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .cauchy(5.0, 1.0);

    let peak = m.eval(&5.0).unwrap().into_inner();
    let wing = m.eval(&10.0).unwrap().into_inner();

    assert!((peak - 1.0).abs() < 1e-6); // max at center
    assert!(wing < peak); // decays away from center
    assert!(wing > 0.0);
}

#[test]
fn pipeline_custom_map01() {
    let m = metric("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .by(|raw| (raw / 50.0).min(1.0));

    let score = m.eval(&25.0).unwrap();
    assert!((score.into_inner() - 0.5).abs() < 1e-6);
}

#[test]
fn custom_map01_validates_range() {
    let m = metric("bad")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .by(|_| 1.5); // returns >1

    assert!(m.eval(&0.0).is_err());
}

#[test]
fn metric_name_preserved() {
    let m = metric("cleanliness")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    assert_eq!(m.name, "cleanliness");
}

#[test]
fn metric_clone() {
    let m1 = metric("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .linear(50.0);

    let m2 = m1.clone();
    let s = m2.eval(&25.0).unwrap().into_inner();
    assert!((s - 0.5).abs() < 1e-6);
}

#[test]
fn metrics_with_different_ctx_fields() -> Result<(), &'static str> {
    let gc = metric("gc")
        .measure()
        .by(|ctx: &DnaCtx| ctx.gc)
        .map01()
        .identity();

    let len = metric("len")
        .measure()
        .by(|ctx: &DnaCtx| ctx.len)
        .map01()
        .linear(100.0);

    let scorer = ScoreSet::new().push(1.0, gc)?.push(1.0, len)?.sum()?;

    let ctx = DnaCtx { gc: 0.6, len: 50.0 };
    let total = scorer(&ctx);

    // gc: 0.6 * 0.5 = 0.3, len: 0.5 * 0.5 = 0.25, total = 0.55
    assert!((total - 0.55).abs() < 1e-5);
    Ok(())
}
