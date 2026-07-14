//! Proto caller — the simplest possible user.
//!
//! Receives a [`ScoreConfig`] (deserialized from protobuf), calls
//! [`ScoreSet::build`] once, then scores millions of samples.
//!
//! # What this module knows
//!
//! - [`ScoreConfig`] and its per-metric message types
//! - [`ScoreSet`] (opaque) and its `.build()` + `.score()` methods
//!
//! # What this module does NOT know
//!
//! - `Measure`/`Map01` traits
//! - Concrete measure types: `X`, `Y`, `OffsetZ`
//! - Concrete map types: `Identity`, `Positive`, `Clamp`
//! - `Metric` struct

use super::dev_declaration::{ScoreConfig, ScoreSet};

/// Configuration from proto deserialization.
#[derive(Clone, Debug, Default)]
pub(crate) struct UserConfig {
    pub score_config: ScoreConfig,
}

/// Score a batch of samples.
///
/// 1. Compile: `ScoreSet::build(&config.score_config)` — once.
/// 2. Score: `score_set.score(sample)` — per sample, branchless arithmetic.
pub(crate) fn score_batch(config: &UserConfig, samples: &[super::Sample]) -> Vec<f32> {
    let score_set = ScoreSet::build(&config.score_config);
    samples.iter().map(|s| score_set.score(s)).collect()
}
