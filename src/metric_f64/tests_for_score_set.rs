use super::*;

// ---------------------------------------------------------------------------
// ScorerBuilder64 + Scorer64 + ScoreSet64 tests
// ---------------------------------------------------------------------------

#[test]
fn empty_builder_noop() {
    // ScorerBuilder64<C, ()> can be created but cannot build (no MetricTuple64
    // impl for ()).  Just verify new() works.
    let _builder: ScorerBuilder64<f64, ()> = ScorerBuilder64::new();
}

#[test]
fn empty_set_rejected_via_zero_weight() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();
    assert!(ScorerBuilder64::new().add(0.0, m).build().is_err());
}

#[test]
fn single_metric_score() -> Result<(), &'static str> {
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    let scorer = ScorerBuilder64::new().add(1.0, m).build()?;
    let result = ScoreSet64::score(&scorer, &0.5);
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

// ---------------------------------------------------------------------------
// Context types — the things being measured
// ---------------------------------------------------------------------------

/// A restaurant — the object under evaluation.
///
/// Fields are raw observations, *not* pre-interpreted scores.
struct Restaurant {
    dust_count: f64, // airborne particles (0-200)
    avg_rating: f64, // average customer rating (0-5 stars)
}

#[test]
fn multiple_metrics_weighted_sum() -> Result<(), &'static str> {
    // "cleanliness": less dust → higher score.  Invert raw count.
    let m1 = metric64("cleanliness")
        .measure()
        .by(|r: &Restaurant| 200.0 - r.dust_count)
        .map01()
        .linear(200.0);

    // "popularity": raw rating, linear to 5.0
    let m2 = metric64("popularity")
        .measure()
        .by(|r: &Restaurant| r.avg_rating)
        .map01()
        .linear(5.0);

    let scorer = ScorerBuilder64::new().add(1.0, m1).add(3.0, m2).build()?;

    let r = Restaurant {
        dust_count: 20.0,
        avg_rating: 4.0,
    };
    let total = ScoreSet64::score(&scorer, &r);

    // cleanliness: (200-20)/200 = 0.9, weight = 1/4 = 0.25, contrib = 0.225
    // popularity:   4.0/5.0 = 0.8, weight = 3/4 = 0.75, contrib = 0.6
    // total = 0.225 + 0.6 = 0.825
    assert!((total - 0.825).abs() < 1e-6);
    Ok(())
}

/// A DNA sequence — the object under evaluation.
struct DnaSequence {
    gc_bases: f64,    // number of G + C bases
    total_bases: f64, // sequence length in bases
}

#[test]
fn breakdown_matches_score() -> Result<(), &'static str> {
    // "gc_content": proportion of G+C in the sequence
    let gc_content = metric64("gc_content")
        .measure()
        .by(|dna: &DnaSequence| dna.gc_bases / dna.total_bases)
        .map01()
        .identity();

    // "length_score": longer → higher, up to 10 kb
    let length_score = metric64("length")
        .measure()
        .by(|dna: &DnaSequence| dna.total_bases)
        .map01()
        .linear(10_000.0);

    let scorer = ScorerBuilder64::new()
        .add(2.0, gc_content.clone())
        .add(1.0, length_score.clone())
        .build()?;
    let dna = DnaSequence {
        gc_bases: 2400.0,
        total_bases: 5000.0,
    };
    let total = ScoreSet64::score(&scorer, &dna);
    let rows = ScoreSet64::breakdown(&scorer, &dna);

    let breakdown_sum: f64 = rows.iter().map(|r| r.contribution).sum();
    assert!((total - breakdown_sum).abs() < 1e-5);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "gc_content");
    // raw = 2400/5000 = 0.48
    assert!((rows[0].raw - 0.48).abs() < 1e-7);
    assert!((rows[0].score - 0.48).abs() < 1e-7);
    assert_eq!(rows[1].name, "length");
    // raw = 5000.0
    assert!((rows[1].raw - 5000.0).abs() < 1e-5);
    // score = 5000/10000 = 0.5
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

    assert!(ScorerBuilder64::new().add(0.0, m).build().is_err());
}

#[test]
fn negative_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(ScorerBuilder64::new().add(-1.0, m).build().is_err());
}

#[test]
fn nan_weight_rejected() {
    let m = metric64("x")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert!(ScorerBuilder64::new().add(f64::NAN, m).build().is_err());
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

    let scorer = ScorerBuilder64::new().add(1.0, m1).add(1.0, m2).build()?;
    let result = ScoreSet64::score(&scorer, &0.5);
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

    let scorer = ScorerBuilder64::new().add(1.0, m1).add(1.0, m2).build()?;
    let result = ScoreSet64::score(&scorer, &0.0);
    // m1: 0.0 * 0.5 = 0.0, m2: 1.0 * 0.5 = 0.5
    assert!((result - 0.5).abs() < 1e-6);
    Ok(())
}

// ---------------------------------------------------------------------------
// Partial application — the core use case for closure-based by()
// ---------------------------------------------------------------------------

struct DnaScorerConfig {
    gc_threshold: f64,
    length_max: f64,
}

/// Build a DNA scorer from config — closures capture parameters at construction;
/// the scorer only needs `&DnaSequence` to produce a score.
fn build_dna_scorer(
    cfg: &DnaScorerConfig,
) -> Result<Scorer64<DnaSequence, impl MetricTuple64<DnaSequence>>, &'static str> {
    let gc = {
        let t = cfg.gc_threshold;
        metric64("gc_filtered")
            .measure()
            .by(move |dna: &DnaSequence| {
                let gc = dna.gc_bases / dna.total_bases;
                if gc > t { gc } else { 0.0 }
            })
            .map01()
            .identity()
    };

    let len = {
        let max = cfg.length_max;
        metric64("length")
            .measure()
            .by(move |dna: &DnaSequence| dna.total_bases)
            .map01()
            .linear(max)
    };

    ScorerBuilder64::new().add(2.0, gc).add(1.0, len).build()
}

#[test]
fn partial_application_dna_scorer() -> Result<(), &'static str> {
    let config = DnaScorerConfig {
        gc_threshold: 0.5,
        length_max: 10_000.0,
    };

    // Build once — config is baked into closures
    let scorer = build_dna_scorer(&config)?;

    // Reuse across multiple contexts
    let seq1 = DnaSequence {
        gc_bases: 2400.0,
        total_bases: 3000.0,
    }; // gc_content = 0.8 (above threshold)
    let seq2 = DnaSequence {
        gc_bases: 300.0,
        total_bases: 1000.0,
    }; // gc_content = 0.3 (below threshold)

    let s1 = ScoreSet64::score(&scorer, &seq1);
    let s2 = ScoreSet64::score(&scorer, &seq2);

    // seq1: gc=0.8 exceeds threshold=0.5, seq2: gc=0.3 below threshold
    assert!(s1 > s2);
    assert!(s1 > 0.0);
    assert!(s2 > 0.0);

    // Same sequence, stricter threshold → gc_filtered contributes 0
    let strict = DnaScorerConfig {
        gc_threshold: 0.9,
        length_max: 10_000.0,
    };
    let strict_scorer = build_dna_scorer(&strict)?;
    let s_strict = ScoreSet64::score(&strict_scorer, &seq1);
    // gc = 0.8 < 0.9 → gc_filtered contributes 0
    assert!(s_strict < s1);

    // Breakdown on the original scorer
    let rows = ScoreSet64::breakdown(&scorer, &seq1);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].name, "gc_filtered");

    Ok(())
}
