use score_set::traits::{prove_v01_f32, prove_v01_f64};
use witnessed::{WitnessExt, Witnessed};

#[test]
fn proof_functions_accept_boundaries_for_both_float_types() {
    assert!(0.0_f32.witness().by(prove_v01_f32).is_ok());
    assert!(1.0_f64.witness().by(prove_v01_f64).is_ok());
}

#[test]
fn proof_functions_reject_out_of_range_and_nan_values() {
    assert!((-0.1_f32).witness().by(prove_v01_f32).is_err());
    assert!(1.1_f64.witness().by(prove_v01_f64).is_err());
    assert!(f32::NAN.witness().by(prove_v01_f32).is_err());
}

#[test]
fn proof_functions_produce_the_expected_witness_types() {
    let _: Witnessed<f32, score_set::traits::V01> = 0.5_f32.witness().by(prove_v01_f32).unwrap();
    let _: Witnessed<f64, score_set::traits::V01> = 0.5_f64.witness().by(prove_v01_f64).unwrap();
}
