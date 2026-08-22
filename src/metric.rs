use witnessed::Witnessed;

use crate::{
    normalized_eval::{NormalizedEval32, NormalizedEval64},
    traits::V01,
    weighted::{Weighted32, Weighted64},
};

/// Builds a weighted `f64` metric from a measurement and normalization map.
pub fn metric64<M, G>(
    measure: M,
    map: G,
    weight: Witnessed<f64, V01>,
) -> Weighted64<NormalizedEval64<M, G>> {
    Weighted64::new(NormalizedEval64::new(measure, map), weight)
}

/// Builds a weighted `f32` metric from a measurement and normalization map.
pub fn metric32<M, G>(
    measure: M,
    map: G,
    weight: Witnessed<f32, V01>,
) -> Weighted32<NormalizedEval32<M, G>> {
    Weighted32::new(NormalizedEval32::new(measure, map), weight)
}
