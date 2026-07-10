use super::*;
use crate::score_set64;

// ============================================================================
// A1: Map01 parameter validation — Linear max must be positive
// ============================================================================

#[test]
fn linear_zero_max_rejected() {
    let m = metric64("bad")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .linear(0.0);

    assert!(m.eval(&0.5).is_err());
}

#[test]
fn linear_negative_max_rejected() {
    let m = metric64("bad")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .linear(-10.0);

    assert!(m.eval(&5.0).is_err());
}

// ============================================================================
// A2: Cauchy half_width=0 — produces NaN at center, caught by Value01
// ============================================================================

#[test]
fn cauchy_zero_half_width_rejected() {
    let m = metric64("bad")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .cauchy(5.0, 1.0, 0.0);

    assert!(m.eval(&5.0).is_err());
}

// ============================================================================
// A3: Inf weight rejected
// ============================================================================

#[test]
fn infinite_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(score_set64! { f64::INFINITY => m }.is_err());
}

#[test]
fn neg_infinite_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(score_set64! { f64::NEG_INFINITY => m }.is_err());
}

// ============================================================================
// A4: NaN raw values through Map01 variants
// ============================================================================

#[test]
fn nan_raw_identity_rejected() {
    let m = metric64("nan")
        .measure()
        .by(|_: &f64| f64::NAN)
        .map01()
        .identity();

    assert!(m.eval(&0.0).is_err());
}

#[test]
fn nan_raw_linear_rejected() {
    let m = metric64("nan")
        .measure()
        .by(|_: &f64| f64::NAN)
        .map01()
        .linear(100.0);

    assert!(m.eval(&0.0).is_err());
}

#[test]
fn nan_raw_cauchy_rejected() {
    let m = metric64("nan")
        .measure()
        .by(|_: &f64| f64::NAN)
        .map01()
        .cauchy(5.0, 1.0, 1.0);

    assert!(m.eval(&0.0).is_err());
}

#[test]
fn nan_raw_custom_rejected() {
    let m = metric64("nan")
        .measure()
        .by(|_: &f64| f64::NAN)
        .map01()
        .by(|raw| raw.clamp(0.0, 1.0));

    assert!(m.eval(&0.0).is_err());
}

// ============================================================================
// A5: score() silently skips failed metrics
// ============================================================================

#[test]
fn sum_skips_failed_metric() -> Result<(), &'static str> {
    let good = metric64("good")
        .measure()
        .by(|_: &f64| 1.0)
        .map01()
        .identity();

    let bad = metric64("bad")
        .measure()
        .by(|_: &f64| 0.5)
        .map01()
        .by(|_| 1.5);

    // good: score=1.0, norm-weight=0.5, contribution=0.5
    // bad:  skipped, contribution=0.0 — total=0.5
    let scorer = score_set64! { 1.0 => good, 1.0 => bad }?;
    let total = scorer.score(&0.0);

    assert!((total - 0.5).abs() < 1e-6);
    Ok(())
}

// ============================================================================
// A6: breakdown() defaults failed metric score to 0.0
// ============================================================================

#[test]
fn breakdown_defaults_failed_metric_to_zero() -> Result<(), &'static str> {
    let good = metric64("good")
        .measure()
        .by(|_: &f64| 0.8)
        .map01()
        .identity();

    let bad = metric64("bad")
        .measure()
        .by(|_: &f64| 0.5)
        .map01()
        .by(|_| 1.5);

    let rows = score_set64! { 1.0 => good, 1.0 => bad }?.breakdown(&0.0);

    assert_eq!(rows.len(), 2);
    let bad_row = rows.iter().find(|r| r.name == "bad").unwrap();
    assert!((bad_row.raw - 0.5).abs() < 1e-7);
    assert!((bad_row.score - 0.0).abs() < 1e-7);
    assert!((bad_row.contribution - 0.0).abs() < 1e-7);
    let good_row = rows.iter().find(|r| r.name == "good").unwrap();
    assert!((good_row.score - 0.8).abs() < 1e-6);
    Ok(())
}

// ============================================================================
// A7: Single-element
// ============================================================================

#[test]
fn single_element_normalization() -> Result<(), &'static str> {
    let m = metric64("only")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let scorer = score_set64! { 5.0 => m }?;
    let total = scorer.score(&0.7);

    // Single weight 5.0 → normalized to 1.0, score = 0.7, total = 0.7
    assert!((total - 0.7).abs() < 1e-6);
    Ok(())
}

#[test]
fn single_element_breakdown_weight_is_one() -> Result<(), &'static str> {
    let m = metric64("only")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let rows = score_set64! { 3.0 => m }?.breakdown(&0.4);

    assert_eq!(rows.len(), 1);
    assert!((rows[0].weight - 1.0).abs() < 1e-7);
    assert!((rows[0].raw - 0.4).abs() < 1e-7);
    assert!((rows[0].score - 0.4).abs() < 1e-7);
    assert!((rows[0].contribution - 0.4).abs() < 1e-7);
    Ok(())
}

// ============================================================================
// A8: Boundary values
// ============================================================================

#[test]
fn identity_exact_zero_and_one() {
    let m = metric64("boundary")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!((m.eval(&0.0).unwrap().into_inner() - 0.0).abs() < 1e-7);
    assert!((m.eval(&1.0).unwrap().into_inner() - 1.0).abs() < 1e-7);
}

#[test]
fn linear_exact_boundaries() {
    let m = metric64("boundary")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .linear(50.0);

    assert!((m.eval(&0.0).unwrap().into_inner() - 0.0).abs() < 1e-7);
    assert!((m.eval(&50.0).unwrap().into_inner() - 1.0).abs() < 1e-7);
    assert!((m.eval(&100.0).unwrap().into_inner() - 1.0).abs() < 1e-7);
    assert!((m.eval(&-10.0).unwrap().into_inner() - 0.0).abs() < 1e-7);
}

#[test]
fn sigmoid_extremes_approach_zero_and_one() {
    let m = metric64("sig")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .inc_sigmoid(0.0, 10.0);

    assert!(m.eval(&-100.0).unwrap().into_inner() < 1e-4);
    assert!(m.eval(&1000.0).unwrap().into_inner() > 0.9999);
}

#[test]
fn cauchy_peak_is_one() {
    let m = metric64("cauchy")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .cauchy(5.0, 1.0, 2.0);

    assert!((m.eval(&5.0).unwrap().into_inner() - 1.0).abs() < 1e-7);
}

#[test]
fn dec_sigmoid_reversed_extremes() {
    let m = metric64("decsig")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .dec_sigmoid(0.0, 10.0);

    assert!(m.eval(&-100.0).unwrap().into_inner() > 0.9999);
    assert!(m.eval(&1000.0).unwrap().into_inner() < 1e-4);
}

// ============================================================================
// A9: Extreme but valid weights
// ============================================================================

#[test]
fn extreme_weights_normalize() -> Result<(), &'static str> {
    let m1 = metric64("tiny")
        .measure()
        .by(|_: &f64| 1.0)
        .map01()
        .identity();

    let m2 = metric64("huge")
        .measure()
        .by(|_: &f64| 0.0)
        .map01()
        .identity();

    let scorer = score_set64! { f64::MIN_POSITIVE => m1, f64::MAX => m2 }?;
    let total = scorer.score(&0.0);

    assert!(total.is_finite());
    assert!(total < 1e-6);
    Ok(())
}

// ============================================================================
// A10: Multiple metrics (stress test normalization)
// ============================================================================

#[test]
fn multiple_metrics_normalize() -> Result<(), &'static str> {
    let m = metric64("n")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let scorer = score_set64! {
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
    }?;

    let total = scorer.score(&0.5);
    assert!(total.is_finite());
    assert!((total - 0.5).abs() < 1e-6);
    Ok(())
}

#[test]
fn multiple_metrics_breakdown_consistent() -> Result<(), &'static str> {
    let m = metric64("n")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let rows = score_set64! {
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
        1.0 => m.clone(),
    }?
    .breakdown(&0.3);

    assert_eq!(rows.len(), 4);
    for r in &rows {
        assert!((r.weight - 0.25).abs() < 1e-7);
        assert!((r.raw - 0.3).abs() < 1e-7);
        assert!((r.score - 0.3).abs() < 1e-7);
    }
    let total: f64 = rows.iter().map(|r| r.contribution).sum();
    assert!((total - 0.3).abs() < 1e-6);
    Ok(())
}

// ============================================================================
// A11: Heterogeneous capturing closures — the core attack on static dispatch
// ============================================================================

#[test]
fn heterogeneous_capturing_closures_deterministic() -> Result<(), &'static str> {
    // Three metrics with three distinct F types sharing the same scorer:
    // A) captures Vec<f64> (owned, non-Copy) — proves Fn bound is satisfied
    // B) captures f64 threshold (Copy) — different F type from A
    // C) non-capturing — a third distinct F type

    let owned_data = alloc::vec![1.0_f64, 2.0, 3.0];

    let with_vec = {
        let data = owned_data; // move Vec into closure
        metric64("with-vec")
            .measure()
            .by(move |ctx: &f64| {
                let _ = &data; // borrow captured Vec through &self → proves Fn
                *ctx
            })
            .map01()
            .identity()
    };

    let with_threshold = {
        let t: f64 = 0.5;
        metric64("threshold")
            .measure()
            .by(move |ctx: &f64| if *ctx > t { 1.0 } else { 0.0 })
            .map01()
            .identity()
    };

    let non_capturing = metric64("plain")
        .measure()
        .by(|ctx: &f64| 1.0 - ctx)
        .map01()
        .identity();

    let scorer = score_set64! {
        1.0 => with_vec,
        2.0 => with_threshold,
        1.0 => non_capturing,
    }?;

    // Deterministic: same ctx → same result every time
    let r1 = scorer.score(&0.8);
    let r2 = scorer.score(&0.8);
    assert!((r1 - r2).abs() < 1e-9);

    // Different ctx → different result
    let lo = scorer.score(&0.0);
    let hi = scorer.score(&1.0);
    assert!(hi > lo);

    // Breakdown matches score
    let rows = scorer.breakdown(&0.8);
    let breakdown_sum: f64 = rows.iter().map(|r| r.contribution).sum();
    assert!((r1 - breakdown_sum).abs() < 1e-6);

    assert_eq!(rows.len(), 3);
    // Normalized weights: 1+2+1 = 4 → [0.25, 0.5, 0.25]
    assert!((rows[0].weight - 0.25).abs() < 1e-7);
    assert!((rows[1].weight - 0.5).abs() < 1e-7);
    assert!((rows[2].weight - 0.25).abs() < 1e-7);

    Ok(())
}

#[test]
fn capturing_closure_repeated_eval_proves_fn_bound() -> Result<(), &'static str> {
    // Captures an owned Vec<f64> — non-Copy.
    // Repeated eval via shared ref proves `F: Fn` (not just FnOnce).
    let captured = alloc::vec![10.0_f64, 20.0];
    let m = metric64("captured")
        .measure()
        .by(move |ctx: &f64| {
            let _ = &captured; // borrow through &self → requires Fn
            *ctx * 2.0
        })
        .map01()
        .linear(2.0);

    // 3× eval via shared reference
    let s1 = m.eval(&0.5)?.into_inner();
    let s2 = m.eval(&0.5)?.into_inner();
    let s3 = m.eval(&0.5)?.into_inner();
    assert!((s1 - 0.5).abs() < 1e-6);
    assert!((s2 - 0.5).abs() < 1e-6);
    assert!((s3 - 0.5).abs() < 1e-6);

    // Also through score() — trait-dispatched
    let scorer = score_set64! { 3.0 => m }?;
    let r1 = scorer.score(&0.5);
    let r2 = scorer.score(&0.5);
    assert!((r1 - 0.5).abs() < 1e-6);
    assert!((r2 - 0.5).abs() < 1e-6);

    Ok(())
}
