use super::*;
use crate::score_set64;

// ---------------------------------------------------------------------------
// score_set64! macro tests
// ---------------------------------------------------------------------------

#[test]
fn empty_set_rejected_via_zero_weight() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();
    assert!(score_set64! { 0.0 => m }.is_err());
}

#[test]
fn single_metric_score() -> Result<(), &'static str> {
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let scorer = score_set64! { 1.0 => m }?;
    let result = scorer.score(&0.5);
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

struct RestaurantCtx {
    cleanliness: f64,
    food_quality: f64,
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
        .identity();

    let scorer = score_set64! { 2.0 => m1, 3.0 => m2 }?;

    let ctx = RestaurantCtx {
        cleanliness: 80.0,
        food_quality: 4.0,
    };
    let total = scorer.score(&ctx);

    // clean: 80/100 = 0.8, weight = 2/5 = 0.4, contribution = 0.64
    // food: 4.0 clamped to 1.0 (identity), weight = 3/5 = 0.6, contribution = 0.6
    // total = 0.64 + 0.6 = 0.92
    assert!((total - 0.92).abs() < 1e-6);
    Ok(())
}

struct DnaCtx {
    gc: f64,
    len: f64,
}

#[test]
fn breakdown_matches_score() -> Result<(), &'static str> {
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

    let scorer = score_set64! { 2.0 => gc.clone(), 1.0 => len.clone() }?;
    let ctx = DnaCtx { gc: 0.6, len: 50.0 };
    let total = scorer.score(&ctx);
    let rows = scorer.breakdown(&ctx);

    let breakdown_sum: f64 = rows.iter().map(|r| r.contribution).sum();
    assert!((total - breakdown_sum).abs() < 1e-5);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "gc");
    assert!((rows[0].raw - 0.6).abs() < 1e-7);
    assert!((rows[0].score - 0.6).abs() < 1e-7);
    assert_eq!(rows[1].name, "len");
    assert!((rows[1].raw - 50.0).abs() < 1e-5);
    assert!((rows[1].score - 0.5).abs() < 1e-7);
    Ok(())
}

#[test]
fn zero_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(score_set64! { 0.0 => m }.is_err());
}

#[test]
fn negative_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(score_set64! { -1.0 => m }.is_err());
}

#[test]
fn nan_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(score_set64! { f64::NAN => m }.is_err());
}

#[test]
fn two_metrics_equal_weights() -> Result<(), &'static str> {
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

    let scorer = score_set64! { 1.0 => m1, 1.0 => m2 }?;
    let result = scorer.score(&0.5);
    assert!((result - 0.5).abs() < 1e-6);
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

    let scorer = score_set64! { 1.0 => m1, 1.0 => m2 }?;
    let result = scorer.score(&0.0);
    // m1: 0.0 * 0.5 = 0.0, m2: 1.0 * 0.5 = 0.5
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

// ---------------------------------------------------------------------------
// Partial application — the core use case for closure-based by()
// ---------------------------------------------------------------------------

struct DnaScorerConfig {
    gc_threshold: f64,
    len_max: f64,
}

/// Build a DNA scorer from config — closures capture parameters at construction;
/// the scorer only needs `&DnaCtx` to produce a score.
fn build_dna_scorer(
    cfg: &DnaScorerConfig,
) -> Result<Scored64<DnaCtx, impl ScoreSetTrait64<DnaCtx>>, &'static str> {
    let gc = {
        let t = cfg.gc_threshold;
        metric64("gc")
            .measure()
            .by(move |ctx: &DnaCtx| if ctx.gc > t { ctx.gc } else { 0.0 })
            .map01()
            .identity()
    };

    let len = {
        let max = cfg.len_max;
        metric64("len")
            .measure()
            .by(move |ctx: &DnaCtx| ctx.len)
            .map01()
            .linear(max)
    };

    score_set64! { 2.0 => gc, 1.0 => len }
}

#[test]
fn partial_application_dna_scorer() -> Result<(), &'static str> {
    let config = DnaScorerConfig {
        gc_threshold: 0.5,
        len_max: 100.0,
    };

    // Build once — config is baked into closures
    let scorer = build_dna_scorer(&config)?;

    // Reuse across multiple contexts
    let ctx1 = DnaCtx { gc: 0.8, len: 80.0 };
    let ctx2 = DnaCtx { gc: 0.3, len: 50.0 };

    let s1 = scorer.score(&ctx1);
    let s2 = scorer.score(&ctx2);

    // ctx1: gc exceeds threshold, ctx2: gc below threshold
    assert!(s1 > s2);
    assert!(s1 > 0.0);
    assert!(s2 > 0.0);

    // Same ctx, different config → different scorer
    let strict = DnaScorerConfig {
        gc_threshold: 0.9,
        len_max: 50.0,
    };
    let strict_scorer = build_dna_scorer(&strict)?;
    let s_strict = strict_scorer.score(&ctx1);
    // gc_threshold=0.9, ctx1.gc=0.8 is below → gc contributes 0
    assert!(s_strict < s1);

    // Breakdown on the original scorer
    let rows = scorer.breakdown(&ctx1);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "gc");

    Ok(())
}
