use super::*;

// ---------------------------------------------------------------------------
// Context types for testing
// ---------------------------------------------------------------------------

struct DnaCtx {
    gc: f64,
    len: f64,
}

struct RestaurantCtx {
    cleanliness: f64,
    food_quality: f64,
}

// ---------------------------------------------------------------------------
// ScoreSet tests
// ---------------------------------------------------------------------------

#[test]
fn empty_set_rejected() {
    assert!(ScoreSet64::<()>::new().sum().is_err());
    assert!(ScoreSet64::<()>::new().breakdown().is_err());
}

#[test]
fn single_metric_sum() -> Result<(), &'static str> {
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let scorer = ScoreSet64::new().push(1.0, m)?.sum()?;
    let result = scorer(&0.5);
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

#[test]
fn multiple_metrics_weighted_sum() -> Result<(), &'static str> {
    let m1 = metric64("clean")
        .measure()
        .by(|ctx: &RestaurantCtx| ctx.cleanliness)
        .map01()
        .linear(100.0);

    let m2 = metric64("food")
        .measure()
        .by(|ctx: &RestaurantCtx| ctx.food_quality)
        .map01()
        .identity(); // raw is already in [0, 5], identity clamps to [0, 1]

    let scorer = ScoreSet64::new().push(2.0, m1)?.push(3.0, m2)?.sum()?;

    let ctx = RestaurantCtx {
        cleanliness: 80.0,
        food_quality: 4.0,
    };
    let total = scorer(&ctx);

    // clean: 80/100 = 0.8, weight = 2/5 = 0.4, contribution = 0.64
    // food: 4.0 clamped to 1.0 (identity), weight = 3/5 = 0.6, contribution = 0.6
    // total = 0.64 + 0.6 = 0.92
    assert!((total - 0.92).abs() < 1e-6);
    Ok(())
}

#[test]
fn breakdown_matches_sum() -> Result<(), &'static str> {
    let gc = metric64("gc")
        .measure()
        .by(|ctx: &DnaCtx| ctx.gc)
        .map01()
        .identity();

    let len = metric64("len")
        .measure()
        .by(|ctx: &DnaCtx| ctx.len)
        .map01()
        .linear(100.0);

    let scorer = ScoreSet64::new()
        .push(2.0, gc.clone())?
        .push(1.0, len.clone())?
        .sum()?;
    let eval = ScoreSet64::new()
        .push(2.0, gc)?
        .push(1.0, len)?
        .breakdown()?;

    let ctx = DnaCtx { gc: 0.6, len: 50.0 };
    let total = scorer(&ctx);
    let rows: Vec<_> = eval.iter(&ctx).collect();

    let breakdown_sum: f64 = rows.iter().map(|r| r.contribution).sum();
    assert!((total - breakdown_sum).abs() < 1e-5);

    // Check names
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "gc");
    assert_eq!(rows[1].name, "len");
    Ok(())
}

#[test]
fn zero_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(ScoreSet64::new().push(0.0, m).is_err());
}

#[test]
fn negative_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(ScoreSet64::new().push(-1.0, m).is_err());
}

#[test]
fn nan_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(ScoreSet64::new().push(f64::NAN, m).is_err());
}

#[test]
fn builder_incremental_construction() -> Result<(), &'static str> {
    let m1 = metric64("a")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let m2 = metric64("b")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let scorer = ScoreSet64::new().push(1.0, m1)?.push(1.0, m2)?.sum()?;

    let result = scorer(&0.5);
    assert!((result - 0.5).abs() < 1e-6); // equal weights, both eval to 0.5
    Ok(())
}

#[test]
fn equal_weights_normalization() -> Result<(), &'static str> {
    let m1 = metric64("a")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let m2 = metric64("b")
        .measure()
        .by(|ctx: &f64| 1.0 - ctx)
        .map01()
        .identity();

    let scorer = ScoreSet64::new().push(1.0, m1)?.push(1.0, m2)?.sum()?;
    let result = scorer(&0.0);
    // m1: 0.0 * 0.5 = 0.0, m2: 1.0 * 0.5 = 0.5
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}
