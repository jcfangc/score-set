use crate::*;
use core::marker::PhantomData;

// ---------------------------------------------------------------------------
// Test helper: a concrete, nameable metric type generic over T and I
// ---------------------------------------------------------------------------

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
// Declare a test enum
// ---------------------------------------------------------------------------

declare_metric_enum! {
    TestKind<T, I> =>
        AlwaysZero(ConstMetric<T, I>),
        AlwaysOne(ConstMetric<T, I>),
        Custom(Box<dyn ErasedMetric<T, I>>),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn metric_enum_eval_static_dispatch() {
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
fn metric_enum_custom_variant() -> Result<(), &'static str> {
    // Build a real Metric and box it
    fn measure(x: &f64) -> f64 {
        *x
    }
    fn map01(raw: &f64, _: &f64) -> Witnessed<f64, Value01> {
        Value01::witness(raw.min(1.0).max(0.0)).unwrap()
    }
    let m = metric("linear").measure().by(measure).map01().by(map01);
    let erased: Box<dyn ErasedMetric<f64, f64>> = Box::new(m);

    let custom = TestKind::Custom(erased);
    assert_eq!(custom.name(), "linear");
    let score = custom.eval(&0.75);
    assert!((*score - 0.75).abs() < 1e-10);

    Ok(())
}

#[test]
fn metric_enum_is_erased_metric() -> Result<(), &'static str> {
    // TestKind implements ErasedMetric directly (via declare_metric_enum!)
    let zero = TestKind::AlwaysZero(ConstMetric::new("zero", 0.0_f64));

    let erased: &dyn ErasedMetric<f64, &str> = &zero;
    assert_eq!(erased.name(), "zero");
    assert!((*erased.eval(&"anything") - 0.0).abs() < 1e-10);

    Ok(())
}

#[test]
fn metric_enum_boxed_as_erased() -> Result<(), &'static str> {
    let one = TestKind::AlwaysOne(ConstMetric::new("one", 1.0_f64));
    let boxed: Box<dyn ErasedMetric<f64, &str>> = Box::new(one);

    assert_eq!(boxed.name(), "one");
    assert!((*boxed.eval(&"x") - 1.0).abs() < 1e-10);

    Ok(())
}
