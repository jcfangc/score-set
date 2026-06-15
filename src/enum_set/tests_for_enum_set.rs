use crate::*;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// Test helper types
// ---------------------------------------------------------------------------

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

declare_metric_enum! {
    TestKind<T, I> =>
        AlwaysZero(ConstMetric<T, I>),
        AlwaysOne(ConstMetric<T, I>),
        Half(ConstMetric<T, I>),
        Custom(Box<dyn ErasedMetric<T, I>>),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn enum_score_set_new_and_score() -> Result<(), &'static str> {
    let set = EnumScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![
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
fn enum_score_set_single_member() -> Result<(), &'static str> {
    let set = EnumScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![(
        5.0,
        TestKind::Half(ConstMetric::new("half", 0.5)),
    )])?;

    // Single member: weight = 1.0, score = 1.0 * 0.5 = 0.5
    let total = set.score(&"x");
    assert!((total - 0.5).abs() < 1e-10);

    Ok(())
}

#[test]
fn enum_score_set_equal_weights() -> Result<(), &'static str> {
    let set = EnumScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![
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
fn enum_score_set_empty_rejected() {
    let result = EnumScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![]);
    assert!(result.is_err());
}

#[test]
fn enum_score_set_zero_weight_rejected() {
    let result = EnumScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![(
        0.0,
        TestKind::AlwaysOne(ConstMetric::new("one", 1.0)),
    )]);
    assert!(result.is_err());
}

#[test]
fn enum_score_set_negative_weight_rejected() {
    let result = EnumScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![(
        -1.0,
        TestKind::AlwaysOne(ConstMetric::new("one", 1.0)),
    )]);
    assert!(result.is_err());
}

#[test]
fn enum_score_set_custom_variant() -> Result<(), &'static str> {
    fn measure(x: &f64) -> f64 {
        *x
    }
    fn map01(raw: &f64, _: &f64) -> Witnessed<f64, Value01> {
        Value01::witness(raw.min(1.0).max(0.0)).unwrap()
    }
    let m = metric("linear").measure().by(measure).map01().by(map01);
    let erased: Box<dyn ErasedMetric<f64, f64>> = Box::new(m);

    let set = EnumScoreSet::<f64, f64, TestKind<f64, f64>>::new(vec![
        (4.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
        (1.0, TestKind::Custom(erased)),
    ])?;

    // Weights: 4/(4+1)=0.8, 1/(4+1)=0.2
    // Score: 0.8*1 + 0.2*0.75 = 0.8 + 0.15 = 0.95
    let total = set.score(&0.75);
    assert!((total - 0.95).abs() < 1e-10);

    Ok(())
}

#[test]
fn enum_score_set_with_f32() -> Result<(), &'static str> {
    let set = EnumScoreSet::<f32, &str, TestKind<f32, &str>>::new(vec![
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
fn enum_score_set_iter() -> Result<(), &'static str> {
    let set = EnumScoreSet::<f64, &str, TestKind<f64, &str>>::new(vec![
        (1.0, TestKind::AlwaysZero(ConstMetric::new("zero", 0.0))),
        (1.0, TestKind::AlwaysOne(ConstMetric::new("one", 1.0))),
    ])?;

    let names: Vec<&str> = set.iter().map(|m| m.metric().name()).collect();
    assert_eq!(names, vec!["zero", "one"]);

    Ok(())
}
