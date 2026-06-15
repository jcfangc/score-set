use crate::*;

// ---------------------------------------------------------------------------
// Helper: build a boxed ErasedMetric
// ---------------------------------------------------------------------------

fn make_identity_metric(name: &'static str) -> Box<dyn ErasedMetric<f64, f64>> {
    fn measure(x: &f64) -> f64 {
        *x
    }
    fn map01(raw: &f64, _: &f64) -> Witnessed<f64, Value01> {
        Value01::witness(raw.min(1.0).max(0.0)).unwrap()
    }
    Box::new(metric(name).measure().by(measure).map01().by(map01))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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
    let a: Box<dyn ErasedMetric<f32, f32>> =
        Box::new(metric("a").measure().by(measure).map01().by(map01));
    let b: Box<dyn ErasedMetric<f32, f32>> =
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
    let a: Box<dyn ErasedMetric<f64, f64>> = make_identity_metric("a");
    let b: Box<dyn ErasedMetric<f64, f64>> = make_identity_metric("b");

    let set = DynamicScoreSet::<f64, f64>::new(vec![(1.0, a), (2.0, b)])?;

    // Weights: 1/3, 2/3; at input=0.6: 1/3*0.6 + 2/3*0.6 = 0.6
    let total = set.score(&0.6);
    assert!((total - 0.6).abs() < 1e-10);

    Ok(())
}
