use super::*;

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

struct DnaSequence {
    gc_bases: f64,
    total_bases: f64,
}

// ---------------------------------------------------------------------------
// Metric builder pipeline tests
// ---------------------------------------------------------------------------

#[test]
fn pipeline_identity() {
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
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
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
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
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .inc_sigmoid(0.0, 10.0);

    let lo = m.eval(&0.0).unwrap().into_inner();
    let hi = m.eval(&10.0).unwrap().into_inner();
    let mid = m.eval(&5.0).unwrap().into_inner();

    assert!(lo < 0.1);
    assert!(hi > 0.9);
    assert!((mid - 0.5).abs() < 0.1);
}

#[test]
fn pipeline_dec_sigmoid() {
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .dec_sigmoid(0.0, 10.0);

    let lo = m.eval(&0.0).unwrap().into_inner();
    let hi = m.eval(&10.0).unwrap().into_inner();
    let mid = m.eval(&5.0).unwrap().into_inner();

    assert!(lo > 0.9);
    assert!(hi < 0.1);
    assert!((mid - 0.5).abs() < 0.1);
}

#[test]
fn pipeline_cauchy() {
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .cauchy(5.0, 1.0, 1.0);

    let peak = m.eval(&5.0).unwrap().into_inner();
    let wing = m.eval(&10.0).unwrap().into_inner();

    assert!((peak - 1.0).abs() < 1e-6);
    assert!(wing < peak);
    assert!(wing > 0.0);

    // Asymmetric
    let m2 = metric64("asym")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .cauchy(5.0, 1.0, 3.0);

    let left = m2.eval(&4.0).unwrap().into_inner();
    let right = m2.eval(&6.0).unwrap().into_inner();
    assert!(left < right);
    assert!(left > 0.0);
    assert!(right > 0.0);
}

#[test]
fn pipeline_custom_map01() {
    let m = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .by(|raw| (raw / 50.0).min(1.0));

    let score = m.eval(&25.0).unwrap();
    assert!((score.into_inner() - 0.5).abs() < 1e-6);
}

#[test]
fn custom_map01_validates_range() {
    let m = metric64("bad")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .by(|_| 1.5);

    assert!(m.eval(&0.0).is_err());
}

#[test]
fn metric_name_preserved() {
    let m = metric64("cleanliness")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .identity();

    assert_eq!(m.name, "cleanliness");
}

#[test]
fn metric_clone() {
    let m1 = metric64("test")
        .measure()
        .by(|ctx: &f64| *ctx)
        .map01()
        .linear(50.0);

    let m2 = m1.clone();
    let s = m2.eval(&25.0).unwrap().into_inner();
    assert!((s - 0.5).abs() < 1e-6);
}

#[test]
fn metrics_with_different_ctx_fields() -> Result<(), &'static str> {
    let gc_content = metric64("gc_content")
        .measure()
        .by(|dna: &DnaSequence| dna.gc_bases / dna.total_bases)
        .map01()
        .identity();

    let length = metric64("length")
        .measure()
        .by(|dna: &DnaSequence| dna.total_bases)
        .map01()
        .linear(10_000.0);

    let scorer = ScorerBuilder64::new()
        .add(1.0, gc_content)
        .add(1.0, length)
        .build()?;

    let dna = DnaSequence {
        gc_bases: 2400.0,
        total_bases: 5000.0,
    };
    let total = ScoreSet64::score(&scorer, &dna);

    // gc_content: 0.48 * 0.5 = 0.24
    // length:     0.5  * 0.5 = 0.25
    // total = 0.49
    assert!((total - 0.49).abs() < 1e-5);
    Ok(())
}

#[test]
fn readme_quick_example_compiles() -> Result<(), &'static str> {
    struct Restaurant {
        dust_count: f64, // raw: 0-200 particles
        avg_rating: f64, // raw: 0-5 stars
    }

    let clean = metric64("cleanliness")
        .measure()
        .by(|r: &Restaurant| 200.0 - r.dust_count)
        .map01()
        .linear(200.0);

    let rating = metric64("rating")
        .measure()
        .by(|r: &Restaurant| r.avg_rating)
        .map01()
        .linear(5.0);

    let scorer = ScorerBuilder64::new()
        .add(2.0, clean.clone())
        .add(1.0, rating.clone())
        .build()?;

    let r = Restaurant {
        dust_count: 20.0,
        avg_rating: 4.0,
    };
    let total: f64 = ScoreSet64::score(&scorer, &r);
    // cleanliness: (200-20)/200=0.9, weight=2/3≈0.667, contrib=0.6
    // rating:      4.0/5.0=0.8,     weight=1/3≈0.333, contrib≈0.267
    // total ≈ 0.867
    assert!((total - 0.866667).abs() < 1e-5);

    let rows = ScoreSet64::breakdown(&scorer, &r);

    assert_eq!(rows.len(), 2);
    Ok(())
}
