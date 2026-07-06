use super::*;

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
    let m = metric32("test")
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
    let m = metric32("test")
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
    let m = metric32("test")
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
    let m = metric32("test")
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
    let m = metric32("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .cauchy(5.0, 1.0, 1.0);

    let peak = m.eval(&5.0).unwrap().into_inner();
    let wing = m.eval(&10.0).unwrap().into_inner();

    assert!((peak - 1.0).abs() < 1e-6); // max at center
    assert!(wing < peak); // decays away from center
    assert!(wing > 0.0);

    // Asymmetric: different decay rates on each side
    let m2 = metric32("asym")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .cauchy(5.0, 1.0, 3.0);

    let left = m2.eval(&4.0).unwrap().into_inner();
    let right = m2.eval(&6.0).unwrap().into_inner();
    assert!(left < right); // tighter on left → lower at distance 1
    assert!(left > 0.0);
    assert!(right > 0.0);
}

#[test]
fn pipeline_custom_map01() {
    let m = metric32("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .by(|raw| (raw / 50.0).min(1.0));

    let score = m.eval(&25.0).unwrap();
    assert!((score.into_inner() - 0.5).abs() < 1e-6);
}

#[test]
fn custom_map01_validates_range() {
    let m = metric32("bad")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .by(|_| 1.5); // returns >1

    assert!(m.eval(&0.0).is_err());
}

#[test]
fn metric_name_preserved() {
    let m = metric32("cleanliness")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    assert_eq!(m.name, "cleanliness");
}

#[test]
fn metric_clone() {
    let m1 = metric32("test")
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
    let gc = metric32("gc")
        .measure()
        .by(|ctx: &DnaCtx| ctx.gc)
        .map01()
        .identity();

    let len = metric32("len")
        .measure()
        .by(|ctx: &DnaCtx| ctx.len)
        .map01()
        .linear(100.0);

    let scorer = ScoreSet32::new().push(1.0, gc)?.push(1.0, len)?.sum()?;

    let ctx = DnaCtx { gc: 0.6, len: 50.0 };
    let total = scorer(&ctx);

    // gc: 0.6 * 0.5 = 0.3, len: 0.5 * 0.5 = 0.25, total = 0.55
    assert!((total - 0.55).abs() < 1e-5);
    Ok(())
}

#[test]
fn readme_quick_example_compiles() -> Result<(), &'static str> {
    struct Restaurant {
        cleanliness: f32,
        food_quality: f32,
    }

    let clean = metric32("cleanliness")
        .measure()
        .by(|r: &Restaurant| r.cleanliness)
        .map01()
        .linear(100.0);

    let food = metric32("food")
        .measure()
        .by(|r: &Restaurant| r.food_quality)
        .map01()
        .identity();

    let score = ScoreSet32::new()
        .push(2.0, clean.clone())?
        .push(1.0, food.clone())?
        .sum()?;

    let r = Restaurant {
        cleanliness: 80.0,
        food_quality: 4.0,
    };
    let total: f32 = score(&r);
    // clean: 0.8×2/3 ≈ 0.533, food: 1.0×1/3 ≈ 0.333, total ≈ 0.867
    assert!((total - 0.866667).abs() < 1e-5);

    let rows: Vec<Breakdown32> = ScoreSet32::new()
        .push(2.0, clean)?
        .push(1.0, food)?
        .breakdown(&r)?
        .into_iter()
        .collect();

    assert_eq!(rows.len(), 2);
    Ok(())
}
