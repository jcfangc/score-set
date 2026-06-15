use crate::*;
use core::marker::PhantomData;

// ===========================================================================
// Test helper: a concrete, nameable metric type generic over T and I
// ===========================================================================

/// A metric that always returns a constant value (clamped to [0, 1]).
struct ConstMetric<T: Float, I> {
    name: &'static str,
    value: T,
    _phantom: PhantomData<I>,
}

impl<T: Float, I> ConstMetric<T, I> {
    fn new(name: &'static str, value: T) -> Self {
        Self {
            name,
            value,
            _phantom: PhantomData,
        }
    }

    fn eval(&self, _input: &I) -> Witnessed<T, Value01> {
        Value01::witness(self.value.min(T::one()).max(T::zero())).unwrap()
    }

    fn name(&self) -> &str {
        self.name
    }
}

// ---------------------------------------------------------------------------
// Declare test enums
// ---------------------------------------------------------------------------

// Generic form: enum is generic over T, I
finite_metric! {
    TestKind<T, I> =>
        AlwaysZero(ConstMetric<T, I>),
        AlwaysOne(ConstMetric<T, I>),
        Half(ConstMetric<T, I>),
        Custom(Box<dyn DynMetric<T, I>>),
}

// Concrete form: enum locks T and I to specific types
finite_metric! {
    ConcreteKind for f64, &'static str =>
        Zero(ConstMetric<f64, &'static str>),
        One(ConstMetric<f64, &'static str>),
        Custom(Box<dyn DynMetric<f64, &'static str>>),
}

// ===========================================================================
// finite_metric! macro tests
// ===========================================================================

#[test]
fn finite_metric_eval_static_dispatch() {
    let zero = TestKind::AlwaysZero(ConstMetric::new("zero", 0.0_f64));
    let one = TestKind::AlwaysOne(ConstMetric::new("one", 1.0_f64));

    assert_eq!(zero.name(), "zero");
    assert_eq!(one.name(), "one");

    let score_zero = zero.eval(&"unused input");
    let score_one = one.eval(&"unused input");

    assert!((*score_zero - 0.0).abs() < 1e-10);
    assert!((*score_one - 1.0).abs() < 1e-10);
}

#[test]
fn finite_metric_custom_variant() -> Result<(), &'static str> {
    // Build a real Metric and box it
    fn measure(x: &f64) -> f64 {
        *x
    }
    fn map01(raw: &f64, _: &f64) -> Witnessed<f64, Value01> {
        Value01::witness(raw.min(1.0).max(0.0)).unwrap()
    }
    let m = metric("linear").measure().by(measure).map01().by(map01);
    let dyn_metric: Box<dyn DynMetric<f64, f64>> = Box::new(m);

    let custom = TestKind::Custom(dyn_metric);
    assert_eq!(custom.name(), "linear");
    let score = custom.eval(&0.75);
    assert!((*score - 0.75).abs() < 1e-10);

    Ok(())
}

#[test]
fn finite_metric_is_dyn_metric() -> Result<(), &'static str> {
    // TestKind implements DynMetric directly (via finite_metric!)
    let zero = TestKind::AlwaysZero(ConstMetric::new("zero", 0.0_f64));

    let dyn_ref: &dyn DynMetric<f64, &str> = &zero;
    assert_eq!(dyn_ref.name(), "zero");
    assert!((*dyn_ref.eval(&"anything") - 0.0).abs() < 1e-10);

    Ok(())
}

#[test]
fn finite_metric_boxed_as_dyn() -> Result<(), &'static str> {
    let one = TestKind::AlwaysOne(ConstMetric::new("one", 1.0_f64));
    let boxed: Box<dyn DynMetric<f64, &str>> = Box::new(one);

    assert_eq!(boxed.name(), "one");
    assert!((*boxed.eval(&"x") - 1.0).abs() < 1e-10);

    Ok(())
}

// ===========================================================================
// FiniteScoreSet tests
// ===========================================================================

#[test]
fn finite_score_set_new_and_score() -> Result<(), &'static str> {
    let set = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![
        (2.0, TestKind::AlwaysZero(ConstMetric::new("zero", 0.0))),
        (3.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
    ])?;

    assert_eq!(set.len(), 2);
    assert!(!set.is_empty());

    // Normalized weights: 2/(2+3)=0.4, 3/(2+3)=0.6
    // Score = 0.4 * 0 + 0.6 * 1 = 0.6
    let total = set.score(&"anything");
    assert!((total - 0.6).abs() < 1e-10);

    Ok(())
}

#[test]
fn finite_score_set_single_member() -> Result<(), &'static str> {
    let set = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![(
        5.0,
        TestKind::Half(ConstMetric::new("half", 0.5)),
    )])?;

    // Single member: weight = 1.0, score = 1.0 * 0.5 = 0.5
    let total = set.score(&"x");
    assert!((total - 0.5).abs() < 1e-10);

    Ok(())
}

#[test]
fn finite_score_set_equal_weights() -> Result<(), &'static str> {
    let set = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![
        (1.0, TestKind::AlwaysZero(ConstMetric::new("zero", 0.0))),
        (1.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
        (1.0, TestKind::Half(ConstMetric::new("half", 0.5))),
    ])?;

    // Each weight = 1/3; total = 1/3*0 + 1/3*1 + 1/3*0.5 = 0.5
    let total = set.score(&"x");
    assert!((total - 0.5).abs() < 1e-10);

    Ok(())
}

#[test]
fn finite_score_set_empty_rejected() {
    let result = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![]);
    assert!(result.is_err());
}

#[test]
fn finite_score_set_zero_weight_rejected() {
    let result = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![(
        0.0,
        TestKind::AlwaysOne(ConstMetric::new("one", 1.0)),
    )]);
    assert!(result.is_err());
}

#[test]
fn finite_score_set_negative_weight_rejected() {
    let result = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![(
        -1.0,
        TestKind::AlwaysOne(ConstMetric::new("one", 1.0)),
    )]);
    assert!(result.is_err());
}

#[test]
fn finite_score_set_custom_variant() -> Result<(), &'static str> {
    fn measure(x: &f64) -> f64 {
        *x
    }
    fn map01(raw: &f64, _: &f64) -> Witnessed<f64, Value01> {
        Value01::witness(raw.min(1.0).max(0.0)).unwrap()
    }
    let m = metric("linear").measure().by(measure).map01().by(map01);
    let dyn_metric: Box<dyn DynMetric<f64, f64>> = Box::new(m);

    let set = FiniteScoreSet::<f64, f64, TestKind<f64, f64>>::new(vec![
        (4.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
        (1.0, TestKind::Custom(dyn_metric)),
    ])?;

    // Weights: 4/(4+1)=0.8, 1/(4+1)=0.2
    // Score: 0.8*1 + 0.2*0.75 = 0.8 + 0.15 = 0.95
    let total = set.score(&0.75);
    assert!((total - 0.95).abs() < 1e-10);

    Ok(())
}

#[test]
fn finite_score_set_with_f32() -> Result<(), &'static str> {
    let set = FiniteScoreSet::<f32, &str, TestKind<f32, &str>>::new(vec![
        (
            2.0_f32,
            TestKind::AlwaysOne(ConstMetric::new("one", 1.0_f32)),
        ),
        (
            3.0_f32,
            TestKind::AlwaysZero(ConstMetric::new("zero", 0.0_f32)),
        ),
    ])?;

    // Weights: 0.4, 0.6; Score = 0.4*1 + 0.6*0 = 0.4
    let total = set.score(&"x");
    assert!((total - 0.4_f32).abs() < 1e-6);

    Ok(())
}

#[test]
fn finite_score_set_iter() -> Result<(), &'static str> {
    let set = FiniteScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![
        (1.0, TestKind::AlwaysZero(ConstMetric::new("zero", 0.0))),
        (1.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
    ])?;

    let names: Vec<&str> = set.iter().map(|m| m.metric().name()).collect();
    assert_eq!(names, vec!["zero", "one"]);

    Ok(())
}

// ===========================================================================
// Concrete-form enum tests (finite_metric! { Name for T, I => ... })
// ===========================================================================

#[test]
fn concrete_form_eval() {
    let zero = ConcreteKind::Zero(ConstMetric::new("zero", 0.0_f64));
    let one = ConcreteKind::One(ConstMetric::new("one", 1.0_f64));

    assert_eq!(zero.name(), "zero");
    assert_eq!(one.name(), "one");
    assert!((*zero.eval(&"ignored") - 0.0).abs() < 1e-10);
    assert!((*one.eval(&"ignored") - 1.0).abs() < 1e-10);
}

#[test]
fn concrete_form_as_dyn_metric() {
    let zero = ConcreteKind::Zero(ConstMetric::new("zero", 0.0_f64));
    let dyn_ref: &dyn DynMetric<f64, &str> = &zero;
    assert_eq!(dyn_ref.name(), "zero");
}

#[test]
fn concrete_form_in_finite_score_set() -> Result<(), &'static str> {
    let set = FiniteScoreSet::<f64, &str, ConcreteKind>::new(vec![
        (2.0, ConcreteKind::Zero(ConstMetric::new("zero", 0.0))),
        (3.0, ConcreteKind::One(ConstMetric::new("one", 1.0))),
    ])?;

    // Weights: 0.4, 0.6; Score = 0.4*0 + 0.6*1 = 0.6
    let total = set.score(&"x");
    assert!((total - 0.6).abs() < 1e-10);

    Ok(())
}
