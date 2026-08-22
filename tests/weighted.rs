use score_set::{Weighted64, traits::EvalF64, weight32, weight64};

struct Constant64(f64);

impl EvalF64<()> for Constant64 {
    fn eval(&self, _ctx: &()) -> f64 {
        self.0
    }
}

#[test]
fn weight_constructors_accept_v01_values() {
    assert_eq!(*weight32(0.25).unwrap(), 0.25);
    assert_eq!(*weight64(1.0).unwrap(), 1.0);
}

#[test]
fn weight_constructors_reject_invalid_values() {
    assert!(weight32(-0.1).is_err());
    assert!(weight64(1.1).is_err());
    assert!(weight64(f64::NAN).is_err());
}

#[test]
fn weighted_scales_any_evaluator() {
    let weighted = Weighted64::new(Constant64(0.8), weight64(0.25).unwrap());

    assert!((weighted.eval(&()) - 0.2).abs() < 1e-12);
}
