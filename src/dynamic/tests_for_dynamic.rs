use crate::*;

// ===========================================================================
// DynMetric trait tests
// ===========================================================================

#[test]
fn dyn_metric_from_metric_builder() -> Result<(), &'static str> {
    let m = metric("test-metric")
        .measure()
        .by(|x: &f64| *x)
        .map01()
        .identity();

    let dyn_metric: Box<dyn DynMetric<f64, f64>> = Box::new(m);

    assert_eq!(dyn_metric.name(), "test-metric");
    let score = dyn_metric.eval(&0.5);
    assert!((*score - 0.5).abs() < 1e-10);

    Ok(())
}

#[test]
fn dyn_metric_clamp_behavior() -> Result<(), &'static str> {
    let m = metric("clampy")
        .measure()
        .by(|x: &f64| *x)
        .map01()
        .identity();

    let dyn_metric: Box<dyn DynMetric<f64, f64>> = Box::new(m);

    assert!((*dyn_metric.eval(&1.5) - 1.0).abs() < 1e-10);
    assert!((*dyn_metric.eval(&-0.5) - 0.0).abs() < 1e-10);

    Ok(())
}

#[test]
fn dyn_metric_nested_box() -> Result<(), &'static str> {
    let m = metric("nested")
        .measure()
        .by(|x: &f64| *x)
        .map01()
        .identity();

    let inner: Box<dyn DynMetric<f64, f64>> = Box::new(m);
    // Box<dyn DynMetric> itself implements DynMetric
    let outer: Box<dyn DynMetric<f64, f64>> = Box::new(inner);

    assert_eq!(outer.name(), "nested");
    assert!((*outer.eval(&0.75) - 0.75).abs() < 1e-10);

    Ok(())
}

#[test]
fn dyn_metric_different_input_types() -> Result<(), &'static str> {
    // Metric over &str input
    let m = metric("gc")
        .measure()
        .by(|dna: &&str| crate::lab::gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());

    let dyn_metric: Box<dyn DynMetric<f64, &str>> = Box::new(m);
    let score = dyn_metric.eval(&"ACGT");
    assert!((*score - 0.5).abs() < 1e-10);

    Ok(())
}

// ===========================================================================
// DynamicScoreSet tests
// ===========================================================================

fn make_identity_metric(name: &'static str) -> Box<dyn DynMetric<f64, f64>> {
    fn measure(x: &f64) -> f64 {
        *x
    }
    fn map01(raw: &f64, _: &f64) -> Witnessed<f64, Value01> {
        Value01::witness(raw.min(1.0).max(0.0)).unwrap()
    }
    Box::new(metric(name).measure().by(measure).map01().by(map01))
}

#[test]
fn dynamic_score_set_new_and_score() -> Result<(), &'static str> {
    let set = DynamicScoreSet::<f64, f64>::new(vec![
        (2.0, make_identity_metric("a")),
        (3.0, make_identity_metric("b")),
    ])?;

    assert_eq!(set.len(), 2);
    assert!(!set.is_empty());

    // Normalized weights: 0.4, 0.6
    // At input=0.5: 0.4*0.5 + 0.6*0.5 = 0.5
    let total = set.score(&0.5);
    assert!((total - 0.5).abs() < 1e-10);

    Ok(())
}

#[test]
fn dynamic_score_set_single_member() -> Result<(), &'static str> {
    let set = DynamicScoreSet::<f64, f64>::new(vec![(5.0, make_identity_metric("only"))])?;

    // Single member: weight = 1.0
    let total = set.score(&0.75);
    assert!((total - 0.75).abs() < 1e-10);

    Ok(())
}

#[test]
fn dynamic_score_set_equal_weights() -> Result<(), &'static str> {
    let set = DynamicScoreSet::<f64, f64>::new(vec![
        (1.0, make_identity_metric("a")),
        (1.0, make_identity_metric("b")),
    ])?;

    // Each weight = 0.5; at input=1.0: 0.5*1.0 + 0.5*1.0 = 1.0
    let total = set.score(&1.0);
    assert!((total - 1.0).abs() < 1e-10);

    Ok(())
}

#[test]
fn dynamic_score_set_empty_rejected() {
    let result = DynamicScoreSet::<f64, f64>::new(vec![]);
    assert!(result.is_err());
}

#[test]
fn dynamic_score_set_zero_weight_rejected() {
    let result = DynamicScoreSet::<f64, f64>::new(vec![(0.0, make_identity_metric("bad"))]);
    assert!(result.is_err());
}

#[test]
fn dynamic_score_set_negative_weight_rejected() {
    let result = DynamicScoreSet::<f64, f64>::new(vec![(-1.0, make_identity_metric("bad"))]);
    assert!(result.is_err());
}

#[test]
fn dynamic_score_set_f32() -> Result<(), &'static str> {
    fn measure(x: &f32) -> f32 {
        *x
    }
    fn map01(raw: &f32, _: &f32) -> Witnessed<f32, Value01> {
        Value01::witness(raw.min(1.0).max(0.0)).unwrap()
    }
    let a: Box<dyn DynMetric<f32, f32>> =
        Box::new(metric("a").measure().by(measure).map01().by(map01));
    let b: Box<dyn DynMetric<f32, f32>> =
        Box::new(metric("b").measure().by(measure).map01().by(map01));

    let set = DynamicScoreSet::<f32, f32>::new(vec![(1.0, a), (3.0, b)])?;

    // Weights: 0.25, 0.75; at input=0.5: 0.25*0.5 + 0.75*0.5 = 0.5
    let total = set.score(&0.5_f32);
    assert!((total - 0.5_f32).abs() < 1e-6);

    Ok(())
}

#[test]
fn dynamic_score_set_clamped_input() -> Result<(), &'static str> {
    let set = DynamicScoreSet::<f64, f64>::new(vec![(1.0, make_identity_metric("clamp"))])?;

    // Identity metric clamps to [0, 1]
    assert!((set.score(&1.5) - 1.0).abs() < 1e-10);
    assert!((set.score(&-0.5) - 0.0).abs() < 1e-10);

    Ok(())
}

#[test]
fn dynamic_score_set_iter() -> Result<(), &'static str> {
    let set = DynamicScoreSet::<f64, f64>::new(vec![
        (1.0, make_identity_metric("first")),
        (1.0, make_identity_metric("second")),
    ])?;

    let names: Vec<&str> = set.iter().map(|m| m.metric().name()).collect();
    assert_eq!(names, vec!["first", "second"]);

    Ok(())
}

#[test]
fn dynamic_score_set_heterogeneous_metrics() -> Result<(), &'static str> {
    // Mix different metric types — the whole point of DynamicScoreSet.
    let a: Box<dyn DynMetric<f64, f64>> = make_identity_metric("a");
    let b: Box<dyn DynMetric<f64, f64>> = make_identity_metric("b");

    let set = DynamicScoreSet::<f64, f64>::new(vec![(1.0, a), (2.0, b)])?;

    // Weights: 1/3, 2/3; at input=0.6: 1/3*0.6 + 2/3*0.6 = 0.6
    let total = set.score(&0.6);
    assert!((total - 0.6).abs() < 1e-10);

    Ok(())
}

// ===========================================================================
// Real-world example: DNA sequence scoring
// ===========================================================================

#[derive(Debug, Clone)]
struct DnaContext<'a> {
    dna: &'a str,
    len: usize,
}

impl<'a> DnaContext<'a> {
    fn new(dna: &'a str) -> Self {
        Self {
            dna,
            len: dna.len(),
        }
    }
}

#[test]
fn dynamic_score_set_dna_example() -> Result<(), &'static str> {
    let gc: Box<dyn DynMetric<f64, DnaContext>> = Box::new(
        metric("gc_ratio")
            .measure()
            .by(|ctx: &DnaContext| crate::lab::gc_ratio(ctx.dna))
            .map01()
            .by(|raw: &f64, _: &DnaContext| Value01::witness(raw.min(1.0).max(0.0)).unwrap()),
    );

    let len: Box<dyn DynMetric<f64, DnaContext>> = Box::new(
        metric("seq_len")
            .measure()
            .by(|ctx: &DnaContext| ctx.len as f64)
            .map01()
            .linear(100.0),
    );

    let set = DynamicScoreSet::<f64, DnaContext>::new(vec![(2.0, gc), (1.0, len)])?;

    let ctx = DnaContext::new("ACGTACGTACGT");
    let total = set.score(&ctx);
    assert!(total >= 0.0 && total <= 1.0);

    // Inspect individual contributions
    for m in set.iter() {
        let raw_weight: f64 = m.weight.into_inner();
        let score = m.metric().eval(&ctx);
        let _contribution = raw_weight * (*score);
    }

    Ok(())
}
