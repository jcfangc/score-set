use crate::*;

// ---------------------------------------------------------------------------
// ErasedMetric — basic coverage
// ---------------------------------------------------------------------------

#[test]
fn erased_metric_from_metric_builder() -> Result<(), &'static str> {
    let m = metric("test-metric")
        .measure()
        .by(|x: &f64| *x)
        .map01()
        .identity();

    let erased: Box<dyn ErasedMetric<f64, f64>> = Box::new(m);

    assert_eq!(erased.name(), "test-metric");
    let score = erased.eval(&0.5);
    assert!((*score - 0.5).abs() < 1e-10);

    Ok(())
}

#[test]
fn erased_metric_clamp_behavior() -> Result<(), &'static str> {
    let m = metric("clampy")
        .measure()
        .by(|x: &f64| *x)
        .map01()
        .identity();

    let erased: Box<dyn ErasedMetric<f64, f64>> = Box::new(m);

    assert!((*erased.eval(&1.5) - 1.0).abs() < 1e-10);
    assert!((*erased.eval(&-0.5) - 0.0).abs() < 1e-10);

    Ok(())
}

#[test]
fn erased_metric_nested_box() -> Result<(), &'static str> {
    let m = metric("nested")
        .measure()
        .by(|x: &f64| *x)
        .map01()
        .identity();

    let inner: Box<dyn ErasedMetric<f64, f64>> = Box::new(m);
    // Box<dyn ErasedMetric> itself implements ErasedMetric
    let outer: Box<dyn ErasedMetric<f64, f64>> = Box::new(inner);

    assert_eq!(outer.name(), "nested");
    assert!((*outer.eval(&0.75) - 0.75).abs() < 1e-10);

    Ok(())
}

#[test]
fn erased_metric_different_input_types() -> Result<(), &'static str> {
    // Metric over &str input
    let m = metric("gc")
        .measure()
        .by(|dna: &&str| crate::lab::gc_ratio(dna))
        .map01()
        .by(|raw: &f64, _: &&str| Value01::witness(*raw).unwrap());

    let erased: Box<dyn ErasedMetric<f64, &str>> = Box::new(m);
    let score = erased.eval(&"ACGT");
    assert!((*score - 0.5).abs() < 1e-10);

    Ok(())
}
