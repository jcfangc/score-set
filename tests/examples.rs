//! Design validation: restaurant scoring with three layers.
//!
//! Scenario: rate restaurants on cleanliness (0-100), food quality (0-5 stars),
//! and price (lower = better). The scoring operator set is composed at runtime
//! but the set of *possible* metrics is known at compile time — perfect for
//! Layer 2 (finite enum).

use core::marker::PhantomData;
use score_set::*;

// ===========================================================================
// Domain input type
// ===========================================================================

struct Restaurant {
    cleanliness: f64,  // 0–100
    food_quality: f64, // 0–5
    price: f64,        // $ per meal, lower is better
    wait_minutes: f64, // avg wait time
}

// ===========================================================================
// Concrete metric types — one per scoring dimension.
// Each is a newtype around Metric with named fn-pointer closures.
// ===========================================================================

fn measure_cleanliness(r: &Restaurant) -> f64 {
    r.cleanliness
}
fn map01_linear_100(raw: &f64, _: &Restaurant) -> Witnessed<f64, Value01> {
    Value01::witness((raw / 100.0).min(1.0).max(0.0)).unwrap()
}

struct Cleanliness(
    Metric<
        f64,
        Restaurant,
        f64,
        fn(&Restaurant) -> f64,
        fn(&f64, &Restaurant) -> Witnessed<f64, Value01>,
    >,
);

impl Cleanliness {
    fn new() -> Self {
        Self(
            metric("cleanliness")
                .measure()
                .by(measure_cleanliness as fn(&Restaurant) -> f64)
                .map01()
                .by(map01_linear_100),
        )
    }
    fn eval(&self, r: &Restaurant) -> Witnessed<f64, Value01> {
        self.0.eval(r)
    }
    fn name(&self) -> &str {
        self.0.name()
    }
}

fn measure_quality(r: &Restaurant) -> f64 {
    r.food_quality
}
fn map01_linear_5(raw: &f64, _: &Restaurant) -> Witnessed<f64, Value01> {
    Value01::witness((raw / 5.0).min(1.0).max(0.0)).unwrap()
}

struct FoodQuality(
    Metric<
        f64,
        Restaurant,
        f64,
        fn(&Restaurant) -> f64,
        fn(&f64, &Restaurant) -> Witnessed<f64, Value01>,
    >,
);

impl FoodQuality {
    fn new() -> Self {
        Self(
            metric("food_quality")
                .measure()
                .by(measure_quality as fn(&Restaurant) -> f64)
                .map01()
                .by(map01_linear_5),
        )
    }
    fn eval(&self, r: &Restaurant) -> Witnessed<f64, Value01> {
        self.0.eval(r)
    }
    fn name(&self) -> &str {
        self.0.name()
    }
}

fn measure_price(r: &Restaurant) -> f64 {
    r.price
}
fn map01_invert_50(raw: &f64, _: &Restaurant) -> Witnessed<f64, Value01> {
    // Invert: $0 = best (1.0), $50+ = worst (0.0)
    Value01::witness((1.0 - raw / 50.0).min(1.0).max(0.0)).unwrap()
}

struct PriceScore(
    Metric<
        f64,
        Restaurant,
        f64,
        fn(&Restaurant) -> f64,
        fn(&f64, &Restaurant) -> Witnessed<f64, Value01>,
    >,
);

impl PriceScore {
    fn new() -> Self {
        Self(
            metric("price")
                .measure()
                .by(measure_price as fn(&Restaurant) -> f64)
                .map01()
                .by(map01_invert_50),
        )
    }
    fn eval(&self, r: &Restaurant) -> Witnessed<f64, Value01> {
        self.0.eval(r)
    }
    fn name(&self) -> &str {
        self.0.name()
    }
}

// ===========================================================================
// Layer 2: declare the finite metric enum (concrete form — Restaurant+ f64
// are baked in, no generic noise)
// ===========================================================================

finite_metric! {
    RestaurantMetric(f64, Restaurant) =>
        Clean(Cleanliness),
        Quality(FoodQuality),
        Price(PriceScore),
}

// ===========================================================================
// Layer 2 usage
// ===========================================================================

#[test]
fn restaurant_scoring_layer2() -> Result<(), &'static str> {
    // Compose a scoring set at runtime.
    // In a real app, this configuration might come from a config file or user
    // preferences — but the *set of possible metrics* is known at compile time.
    let set = FiniteScoreSet::new(vec![
        (3.0, RestaurantMetric::Clean(Cleanliness::new())),
        (5.0, RestaurantMetric::Quality(FoodQuality::new())),
        (2.0, RestaurantMetric::Price(PriceScore::new())),
    ])?;

    let r = Restaurant {
        cleanliness: 85.0,
        food_quality: 4.2,
        price: 18.0,
        wait_minutes: 12.0,
    };

    let total = set.score(&r);
    // cleanliness: 85/100 = 0.85    (weight 3/10 = 0.3)
    // quality:     4.2/5 = 0.84    (weight 5/10 = 0.5)
    // price:       1-18/50 = 0.64  (weight 2/10 = 0.2)
    // total = 0.3*0.85 + 0.5*0.84 + 0.2*0.64 = 0.255 + 0.42 + 0.128 = 0.803
    let expected = 0.3 * 0.85 + 0.5 * 0.84 + 0.2 * 0.64;
    assert!((total - expected).abs() < 1e-10);

    Ok(())
}

#[test]
fn restaurant_scoring_layer2_inspect() -> Result<(), &'static str> {
    let set = FiniteScoreSet::new(vec![
        (1.0, RestaurantMetric::Clean(Cleanliness::new())),
        (1.0, RestaurantMetric::Quality(FoodQuality::new())),
    ])?;

    let r = Restaurant {
        cleanliness: 90.0,
        food_quality: 3.0,
        price: 25.0,
        wait_minutes: 5.0,
    };

    // Can inspect individual metric results before aggregation
    let mut results: Vec<(&str, f64, f64)> = vec![];
    for m in set.iter() {
        let name = m.metric().name();
        let score = *m.metric().eval(&r);
        let weight = m.weight.into_inner();
        results.push((name, weight, score));
    }
    assert_eq!(results[0].0, "cleanliness");
    assert_eq!(results[1].0, "food_quality");
    assert!((results[0].1 - 0.5).abs() < 1e-10); // equal weights
    assert!((results[1].1 - 0.5).abs() < 1e-10);

    Ok(())
}

#[test]
fn restaurant_scoring_layer2_custom_escape_hatch() -> Result<(), &'static str> {
    // Suppose we need a metric that doesn't fit the pre-declared enum.
    // The Custom variant accepts any Box<dyn DynMetric>.
    let wait_metric = metric("wait_time")
        .measure()
        .by(|r: &Restaurant| r.wait_minutes)
        .map01()
        .linear(30.0)
        .boxed(); // P0 improvement!

    let set = FiniteScoreSet::new(vec![
        (1.0, RestaurantMetric::Clean(Cleanliness::new())),
        (1.0, RestaurantMetric::Quality(FoodQuality::new())),
        (1.0, RestaurantMetric::Price(PriceScore::new())),
        (1.0, RestaurantMetric::Custom(wait_metric)),
    ])?;

    let r = Restaurant {
        cleanliness: 100.0,
        food_quality: 5.0,
        price: 0.0,
        wait_minutes: 15.0,
    };
    let total = set.score(&r);
    // All metrics at max (1.0) except wait = 0.5. Equal weights = 0.25 each.
    // total = 0.25*1 + 0.25*1 + 0.25*1 + 0.25*0.5 = 0.875
    assert!((total - 0.875).abs() < 1e-10);

    Ok(())
}

// ===========================================================================
// Layer 3: fully dynamic — same scenario but with Box<dyn> for everything
// ===========================================================================

#[test]
fn restaurant_scoring_layer3() -> Result<(), &'static str> {
    let set = DynamicScoreSet::new(vec![
        (
            3.0,
            metric("cleanliness")
                .measure()
                .by(|r: &Restaurant| r.cleanliness)
                .map01()
                .linear(100.0)
                .boxed(),
        ),
        (
            5.0,
            metric("food_quality")
                .measure()
                .by(|r: &Restaurant| r.food_quality)
                .map01()
                .linear(5.0)
                .boxed(),
        ),
        (
            2.0,
            metric("price")
                .measure()
                .by(|r: &Restaurant| r.price)
                .map01()
                .by(|raw: &f64, _: &Restaurant| {
                    Value01::witness((1.0 - raw / 50.0).min(1.0).max(0.0)).unwrap()
                })
                .boxed(),
        ),
    ])?;

    let r = Restaurant {
        cleanliness: 85.0,
        food_quality: 4.2,
        price: 18.0,
        wait_minutes: 12.0,
    };
    let total = set.score(&r);
    let expected = 0.3 * 0.85 + 0.5 * 0.84 + 0.2 * 0.64;
    assert!((total - expected).abs() < 1e-10);

    Ok(())
}

// ===========================================================================
// Design analysis: why Layer 2 exists (it's not just a worse Layer 3)
// ===========================================================================

// Layer 3 (DynamicScoreSet) is fully dynamic — every .eval() call goes
// through two levels of indirection:
//   1. vtable lookup on Box<dyn DynMetric>
//   2. vtable lookup on Box<dyn Fn> (inside the Metric's closures)
//
// Layer 2 (FiniteScoreSet) with finite_metric! eliminates BOTH:
//   - The enum's match compiles to a jump table (zero indirection)
//   - Variant types use fn pointers (zero indirection)
//
// In hot loops (e.g., scoring millions of items), the difference is real.
// But Layer 2 requires the set of possible metric *types* to be known at
// compile time. Layer 3 can load arbitrary metrics at runtime.

// ===========================================================================
// Comparison: manual enum (what users would write without finite_metric!)
// ===========================================================================

// Without finite_metric!, a user who wants zero-vtable dispatch must write:
//
//   enum RestaurantMetricManual {
//       Clean(Cleanliness),
//       Quality(FoodQuality),
//       Price(PriceScore),
//       Custom(Box<dyn DynMetric<f64, Restaurant>>),
//   }
//
//   impl DynMetric<f64, Restaurant> for RestaurantMetricManual {
//       fn eval(&self, input: &Restaurant) -> Witnessed<f64, Value01> {
//           match self {
//               Self::Clean(m) => m.eval(input),
//               Self::Quality(m) => m.eval(input),
//               Self::Price(m) => m.eval(input),
//               Self::Custom(c) => c.eval(input),
//           }
//       }
//       fn name(&self) -> &str {
//           match self {
//               Self::Clean(m) => m.name(),
//               Self::Quality(m) => m.name(),
//               Self::Price(m) => m.name(),
//               Self::Custom(c) => c.name(),
//           }
//       }
//   }
//
// finite_metric! collapses this to 4 lines (Custom auto-generated):
//
//   finite_metric! {
//       RestaurantMetric(f64, Restaurant) =>
//           Clean(Cleanliness),
//           Quality(FoodQuality),
//           Price(PriceScore),
//   }
//
// And when you add a 4th metric, it's 1 line vs 4 (variant + 3 match arms).

// ===========================================================================
// Demonstration: generic form (when metric types ARE generic)
// ===========================================================================

/// A generic metric that always returns a constant.
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
    fn eval(&self, _: &I) -> Witnessed<T, Value01> {
        Value01::witness(self.value.min(T::one()).max(T::zero())).unwrap()
    }
    fn name(&self) -> &str {
        self.name
    }
}

// Generic form: enum is parameterized over T and I.
// Useful when the same metric types work for multiple input types.
finite_metric! {
    pub GenericKind<T, I> =>
        Yes(ConstMetric<T, I>),
        No(ConstMetric<T, I>),
        Custom(Box<dyn DynMetric<T, I>>),
}

#[test]
fn generic_form_works_across_types() -> Result<(), &'static str> {
    // Same enum, different T/I combinations:

    // f64 with &str input
    let yes_f64: GenericKind<f64, &str> = GenericKind::Yes(ConstMetric::new("yes", 1.0_f64));
    assert_eq!(yes_f64.name(), "yes");
    assert!((*yes_f64.eval(&"ignored") - 1.0).abs() < 1e-10);

    // f32 with i32 input
    let no_f32: GenericKind<f32, i32> = GenericKind::No(ConstMetric::new("no", 0.0_f32));
    assert_eq!(no_f32.name(), "no");
    assert!((*no_f32.eval(&42) - 0.0).abs() < 1e-6);

    Ok(())
}
