use super::*;
use crate::score_set32;

// ---------------------------------------------------------------------------
// score_set32! macro tests
// ---------------------------------------------------------------------------

#[test]
fn empty_set_rejected_via_zero_weight() {
    let m = metric32("x")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();
    assert!(score_set32! { 0.0 => m }.is_err());
}

#[test]
fn single_metric_score() -> Result<(), &'static str> {
    let m = metric32("test")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    let scorer = score_set32! { 1.0 => m }?;
    let result = scorer.score(&0.5);
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

struct RestaurantCtx {
    cleanliness: f32,
    food_quality: f32,
}

#[test]
fn multiple_metrics_weighted_sum() -> Result<(), &'static str> {
    let m1 = metric32("clean")
        .measure()
        .by(|ctx: &RestaurantCtx| ctx.cleanliness)
        .map01()
        .linear(100.0);

    let m2 = metric32("food")
        .measure()
        .by(|ctx: &RestaurantCtx| ctx.food_quality)
        .map01()
        .identity();

    let scorer = score_set32! { 2.0 => m1, 3.0 => m2 }?;

    let ctx = RestaurantCtx {
        cleanliness: 80.0,
        food_quality: 4.0,
    };
    let total = scorer.score(&ctx);

    // clean: 80/100 = 0.8, weight = 2/5 = 0.4, contribution = 0.32
    // food: 4.0 clamped to 1.0 (identity), weight = 3/5 = 0.6, contribution = 0.6
    // total = 0.32 + 0.6 = 0.92
    assert!((total - 0.92).abs() < 1e-6);
    Ok(())
}

struct DnaCtx {
    gc: f32,
    len: f32,
}

#[test]
fn breakdown_matches_score() -> Result<(), &'static str> {
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

    let scorer = score_set32! { 2.0 => gc.clone(), 1.0 => len.clone() }?;
    let ctx = DnaCtx { gc: 0.6, len: 50.0 };
    let total = scorer.score(&ctx);
    let rows = scorer.breakdown(&ctx);

    let breakdown_sum: f32 = rows.iter().map(|r| r.contribution).sum();
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
    let m = metric32("x")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    assert!(score_set32! { 0.0 => m }.is_err());
}

#[test]
fn negative_weight_rejected() {
    let m = metric32("x")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    assert!(score_set32! { -1.0 => m }.is_err());
}

#[test]
fn nan_weight_rejected() {
    let m = metric32("x")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    assert!(score_set32! { f32::NAN => m }.is_err());
}

#[test]
fn two_metrics_equal_weights() -> Result<(), &'static str> {
    let m1 = metric32("a")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    let m2 = metric32("b")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    let scorer = score_set32! { 1.0 => m1, 1.0 => m2 }?;
    let result = scorer.score(&0.5);
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

#[test]
fn equal_weights_normalization() -> Result<(), &'static str> {
    let m1 = metric32("a")
        .measure()
        .by(|ctx: &f32| *ctx)
        .map01()
        .identity();

    let m2 = metric32("b")
        .measure()
        .by(|ctx: &f32| 1.0 - ctx)
        .map01()
        .identity();

    let scorer = score_set32! { 1.0 => m1, 1.0 => m2 }?;
    let result = scorer.score(&0.0);
    // m1: 0.0 * 0.5 = 0.0, m2: 1.0 * 0.5 = 0.5
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

// ---------------------------------------------------------------------------
// Partial application — the core use case for closure-based by()
// ---------------------------------------------------------------------------

struct DnaScorerConfig {
    gc_threshold: f32,
    len_max: f32,
}

/// Build a DNA scorer from config — closures capture parameters at construction;
/// the scorer only needs `&DnaCtx` to produce a score.
fn build_dna_scorer(
    cfg: &DnaScorerConfig,
) -> Result<Scored32<DnaCtx, impl ScoreSetTrait32<DnaCtx>>, &'static str> {
    let gc = {
        let t = cfg.gc_threshold;
        metric32("gc")
            .measure()
            .by(move |ctx: &DnaCtx| if ctx.gc > t { ctx.gc } else { 0.0 })
            .map01()
            .identity()
    };

    let len = {
        let max = cfg.len_max;
        metric32("len")
            .measure()
            .by(move |ctx: &DnaCtx| ctx.len)
            .map01()
            .linear(max)
    };

    score_set32! { 2.0 => gc, 1.0 => len }
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
