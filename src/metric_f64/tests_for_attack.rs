use super::*;

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

    // raw == center, raw >= center → half_right=0 → 0/0 = NaN → Value01 rejects
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

    assert!(ScoreSet64::new().push(f64::INFINITY, m).is_err());
}

#[test]
fn neg_infinite_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(ScoreSet64::new().push(f64::NEG_INFINITY, m).is_err());
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
// A5: sum() silently skips failed metrics
// ============================================================================

#[test]
fn sum_skips_failed_metric() -> Result<(), &'static str> {
    // Good metric: always returns 1.0
    let good = metric64("good")
        .measure()
        .by(|_: &f64| 1.0)
        .map01()
        .identity();

    // Bad metric: custom map01 returns >1, always fails eval
    let bad = metric64("bad")
        .measure()
        .by(|_: &f64| 0.5)
        .map01()
        .by(|_| 1.5);

    // Equal weights, only good contributes:
    // good: score=1.0, norm-weight=0.5, contribution=0.5
    // bad:  skipped, contribution=0.0 — total=0.5
    let total = ScoreSet64::new().push(1.0, good)?.push(1.0, bad)?.sum()?(&0.0);

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

    // Bad metric that always fails
    let bad = metric64("bad")
        .measure()
        .by(|_: &f64| 0.5)
        .map01()
        .by(|_| 1.5);

    let rows: Vec<_> = ScoreSet64::new()
        .push(1.0, good)?
        .push(1.0, bad)?
        .breakdown(&0.0)?
        .into_iter()
        .collect();

    assert_eq!(rows.len(), 2);
    // Find the failed metric
    let bad_row = rows.iter().find(|r| r.name == "bad").unwrap();
    assert!((bad_row.raw - 0.5).abs() < 1e-7);
    assert!((bad_row.score - 0.0).abs() < 1e-7);
    assert!((bad_row.contribution - 0.0).abs() < 1e-7);
    // Good metric should have normalized weight 0.5
    let good_row = rows.iter().find(|r| r.name == "good").unwrap();
    assert!((good_row.score - 0.8).abs() < 1e-6);
    Ok(())
}

// ============================================================================
// A7: Single-element ScoreSet
// ============================================================================

#[test]
fn single_element_normalization() -> Result<(), &'static str> {
    let m = metric64("only")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let total = ScoreSet64::new().push(5.0, m)?.sum()?(&0.7);

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

    let rows: Vec<_> = ScoreSet64::new()
        .push(3.0, m)?
        .breakdown(&0.4)?
        .into_iter()
        .collect();

    assert_eq!(rows.len(), 1);
    assert!((rows[0].weight - 1.0).abs() < 1e-7);
    assert!((rows[0].raw - 0.4).abs() < 1e-7);
    assert!((rows[0].score - 0.4).abs() < 1e-7);
    assert!((rows[0].contribution - 0.4).abs() < 1e-7);
    Ok(())
}

// ============================================================================
// A8: Boundary values — raw exactly 0.0 and 1.0
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

    // raw=0 → 0/50 = 0.0
    assert!((m.eval(&0.0).unwrap().into_inner() - 0.0).abs() < 1e-7);
    // raw=50 → 50/50 = 1.0
    assert!((m.eval(&50.0).unwrap().into_inner() - 1.0).abs() < 1e-7);
    // raw=100 → 100/50 = 2.0, clamped to 1.0
    assert!((m.eval(&100.0).unwrap().into_inner() - 1.0).abs() < 1e-7);
    // raw=-10 → -10/50 = -0.2, clamped to 0.0
    assert!((m.eval(&-10.0).unwrap().into_inner() - 0.0).abs() < 1e-7);
}

#[test]
fn sigmoid_extremes_approach_zero_and_one() {
    let m = metric64("sig")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .inc_sigmoid(0.0, 10.0);

    // Far below low → ≈0
    assert!(m.eval(&-100.0).unwrap().into_inner() < 1e-4);
    // Far above high → ≈1
    assert!(m.eval(&1000.0).unwrap().into_inner() > 0.9999);
}

#[test]
fn cauchy_peak_is_one() {
    let m = metric64("cauchy")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .cauchy(5.0, 1.0, 2.0);

    // At center → 1.0
    assert!((m.eval(&5.0).unwrap().into_inner() - 1.0).abs() < 1e-7);
}

#[test]
fn dec_sigmoid_reversed_extremes() {
    let m = metric64("decsig")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .dec_sigmoid(0.0, 10.0);

    // Far below low → ≈1 (decreasing)
    assert!(m.eval(&-100.0).unwrap().into_inner() > 0.9999);
    // Far above high → ≈0
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

    // f64::MIN_POSITIVE and f64::MAX are both finite and > 0
    let total = ScoreSet64::new()
        .push(f64::MIN_POSITIVE, m1)?
        .push(f64::MAX, m2)?
        .sum()?(&0.0);

    // m1 contributes ~0 (tiny normalized weight), m2 contributes 0 (score=0)
    // total ≈ 1.0 * tiny_weight ≈ 1.0 (since MIN_POSITIVE / (MIN_POSITIVE + MAX) ≈ 0)
    // Actually: min_pos / (min_pos + max) ≈ 0 for f64. So weight1 ≈ 0, weight2 ≈ 1.
    // contribution = 1.0*tiny_prob + 0.0*large_prob ≈ tiny_prob.
    assert!(total.is_finite());
    // With f64::MAX, the sum is dominated by MAX so tiny weight is ~0
    assert!(total < 1e-6);
    Ok(())
}

// ============================================================================
// A10: Multiple metrics (stress test normalization tolerance)
// ============================================================================

#[test]
fn multiple_metrics_normalize() -> Result<(), &'static str> {
    let mut builder = ScoreSet64::new();
    for _ in 0..8 {
        let m = metric64("n")
            .measure()
            .by(|ctx: &f64| *ctx)
            .map01()
            .identity();
        builder = builder.push(1.0, m)?;
    }

    let total = builder.sum()?(&0.5);
    assert!(total.is_finite());
    // All 8 metrics: weight=0.125 (exact in binary), score=0.5, total=0.5
    assert!((total - 0.5).abs() < 1e-6);
    Ok(())
}

#[test]
fn multiple_metrics_breakdown_consistent() -> Result<(), &'static str> {
    let mut builder = ScoreSet64::new();
    for _ in 0..4 {
        let m = metric64("n")
            .measure()
            .by(|ctx: &f64| *ctx)
            .map01()
            .identity();
        builder = builder.push(1.0, m)?;
    }

    let rows: Vec<_> = builder.breakdown(&0.3)?.into_iter().collect();
    assert_eq!(rows.len(), 4);
    // Each weight = 0.25, raw = score = 0.3 (identity map01)
    for r in &rows {
        assert!((r.weight - 0.25).abs() < 1e-7);
        assert!((r.raw - 0.3).abs() < 1e-7);
        assert!((r.score - 0.3).abs() < 1e-7);
    }
    let total: f64 = rows.iter().map(|r| r.contribution).sum();
    assert!((total - 0.3).abs() < 1e-6);
    Ok(())
}
