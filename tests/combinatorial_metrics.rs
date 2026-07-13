//! Backend scenario — user-driven metric selection with zero vtable.
//!
//! The backend receives pre-built metrics via `Option` slots
//! (the caller — proto deserialization — chose the parameters).
//!
//! Limitation: typestate builders cannot `if add(); if add(); build()`
//! because each `.add()` changes the type and Rust's `if` requires
//! both branches to agree.  The zero-vtable workaround is: `if`-collect
//! into a buffer, then `match` on count to build the correct-arity scorer.

use score_set::*;

// ============================================================================
// backend
// ============================================================================

#[allow(dead_code)]
mod backend {
    use score_set::*;

    #[derive(Clone, Copy, Debug)]
    pub struct Product {
        pub price: f32,
        pub review_count: f32,
        pub avg_rating: f32,
        pub shipping_days: f32,
    }

    #[derive(Clone)]
    pub struct MetricSlot {
        pub metric: Metric32<Product>,
        pub weight: f32,
    }

    pub struct UserRequest {
        pub slot0: Option<MetricSlot>,
        pub slot1: Option<MetricSlot>,
        pub slot2: Option<MetricSlot>,
    }

    /// Check each slot — if present, collect.  Then match on count.
    pub fn score_batch(req: &UserRequest, products: &[Product]) -> Vec<f32> {
        let slots: [Option<&MetricSlot>; 3] =
            [req.slot0.as_ref(), req.slot1.as_ref(), req.slot2.as_ref()];
        let active: Vec<&MetricSlot> = slots.iter().filter_map(|s| *s).collect();
        let n = active.len();

        match n {
            1 => {
                let s = active[0];
                let scorer = ScorerBuilder32::new()
                    .add(s.weight, s.metric.clone())
                    .build()
                    .unwrap();
                products
                    .iter()
                    .map(|p| ScoreSet32::score(&scorer, p))
                    .collect()
            }
            2 => {
                let (s0, s1) = (active[0], active[1]);
                let scorer = ScorerBuilder32::new()
                    .add(s0.weight, s0.metric.clone())
                    .add(s1.weight, s1.metric.clone())
                    .build()
                    .unwrap();
                products
                    .iter()
                    .map(|p| ScoreSet32::score(&scorer, p))
                    .collect()
            }
            3 => {
                let (s0, s1, s2) = (active[0], active[1], active[2]);
                let scorer = ScorerBuilder32::new()
                    .add(s0.weight, s0.metric.clone())
                    .add(s1.weight, s1.metric.clone())
                    .add(s2.weight, s2.metric.clone())
                    .build()
                    .unwrap();
                products
                    .iter()
                    .map(|p| ScoreSet32::score(&scorer, p))
                    .collect()
            }
            _ => Vec::new(),
        }
    }

    pub fn breakdown_batch(req: &UserRequest, product: &Product) -> Vec<Breakdown32> {
        let slots: [Option<&MetricSlot>; 3] =
            [req.slot0.as_ref(), req.slot1.as_ref(), req.slot2.as_ref()];
        let active: Vec<&MetricSlot> = slots.iter().filter_map(|s| *s).collect();
        let n = active.len();

        match n {
            1 => {
                let s = active[0];
                let scorer = ScorerBuilder32::new()
                    .add(s.weight, s.metric.clone())
                    .build()
                    .unwrap();
                ScoreSet32::breakdown(&scorer, product)
            }
            2 => {
                let (s0, s1) = (active[0], active[1]);
                let scorer = ScorerBuilder32::new()
                    .add(s0.weight, s0.metric.clone())
                    .add(s1.weight, s1.metric.clone())
                    .build()
                    .unwrap();
                ScoreSet32::breakdown(&scorer, product)
            }
            3 => {
                let (s0, s1, s2) = (active[0], active[1], active[2]);
                let scorer = ScorerBuilder32::new()
                    .add(s0.weight, s0.metric.clone())
                    .add(s1.weight, s1.metric.clone())
                    .add(s2.weight, s2.metric.clone())
                    .build()
                    .unwrap();
                ScoreSet32::breakdown(&scorer, product)
            }
            _ => Vec::new(),
        }
    }
}

// ============================================================================
// tests — proptest with random bool vector palette
// ============================================================================

#[cfg(test)]
mod tests {
    use super::backend::*;
    use proptest::prelude::*;
    use score_set::*;

    type M = fn(&Product) -> f32;
    fn _price(p: &Product) -> f32 {
        p.price
    }
    fn _afford(p: &Product) -> f32 {
        500.0 - p.price
    }
    fn _reviews(p: &Product) -> f32 {
        p.review_count
    }
    fn _rating(p: &Product) -> f32 {
        p.avg_rating
    }
    fn _speed_inv(p: &Product) -> f32 {
        30.0 - p.shipping_days
    }
    fn _speed(p: &Product) -> f32 {
        p.shipping_days
    }

    #[derive(Clone, Copy, Debug)]
    enum Map01Desc {
        Linear { max: f32 },
        IncSigmoid { low: f32, high: f32 },
        DecSigmoid { low: f32, high: f32 },
        Identity,
    }

    fn build_metric(measure: M, name: &'static str, map01: Map01Desc) -> Metric32<Product> {
        match map01 {
            Map01Desc::Linear { max } => metric32(name).measure().by(measure).map01().linear(max),
            Map01Desc::IncSigmoid { low, high } => metric32(name)
                .measure()
                .by(measure)
                .map01()
                .inc_sigmoid(low, high),
            Map01Desc::DecSigmoid { low, high } => metric32(name)
                .measure()
                .by(measure)
                .map01()
                .dec_sigmoid(low, high),
            Map01Desc::Identity => metric32(name).measure().by(measure).map01().identity(),
        }
    }

    struct PaletteEntry {
        measure: M,
        name: &'static str,
        map01: Map01Desc,
    }

    fn palette() -> [PaletteEntry; 8] {
        use Map01Desc::*;
        [
            PaletteEntry {
                measure: _price as M,
                name: "price_lin",
                map01: Linear { max: 500.0 },
            },
            PaletteEntry {
                measure: _price as M,
                name: "price_sig",
                map01: IncSigmoid {
                    low: 10.0,
                    high: 300.0,
                },
            },
            PaletteEntry {
                measure: _afford as M,
                name: "afford",
                map01: Linear { max: 500.0 },
            },
            PaletteEntry {
                measure: _reviews as M,
                name: "reviews_sig",
                map01: IncSigmoid {
                    low: 10.0,
                    high: 200.0,
                },
            },
            PaletteEntry {
                measure: _reviews as M,
                name: "reviews_raw",
                map01: Identity,
            },
            PaletteEntry {
                measure: _rating as M,
                name: "rating_lin",
                map01: Linear { max: 5.0 },
            },
            PaletteEntry {
                measure: _speed as M,
                name: "speed_dec",
                map01: DecSigmoid {
                    low: 1.0,
                    high: 14.0,
                },
            },
            PaletteEntry {
                measure: _speed_inv as M,
                name: "speed_lin",
                map01: Linear { max: 30.0 },
            },
        ]
    }

    fn arb_product() -> impl Strategy<Value = Product> {
        (
            -100.0_f32..600.0_f32,
            0.0_f32..10_000.0_f32,
            1.0_f32..5.0_f32,
            1.0_f32..30.0_f32,
        )
            .prop_map(|(price, review_count, avg_rating, shipping_days)| Product {
                price,
                review_count,
                avg_rating,
                shipping_days,
            })
    }

    fn arb_selection() -> impl Strategy<Value = Vec<bool>> {
        prop::collection::vec(any::<bool>(), palette().len())
    }

    proptest! {
        #[test]
        fn any_subset_scores_in_range(
            selection in arb_selection(),
            products in prop::collection::vec(arb_product(), 1..5),
        ) {
            let pal = palette();
            let slots: Vec<MetricSlot> = selection.iter().enumerate()
                .filter(|(_, on)| **on)
                .take(3)
                .map(|(i, _)| MetricSlot {
                    metric: build_metric(pal[i].measure, pal[i].name, pal[i].map01),
                    weight: 1.0,
                })
                .collect();
            if slots.is_empty() { return Ok(()); }

            let req = UserRequest {
                slot0: slots.first().cloned(),
                slot1: slots.get(1).cloned(),
                slot2: slots.get(2).cloned(),
            };
            let scores = score_batch(&req, &products);
            prop_assert_eq!(scores.len(), products.len());
            for s in &scores {
                prop_assert!(s.is_finite());
                prop_assert!(*s >= 0.0);
                prop_assert!(*s <= 1.0);
            }
        }

        #[test]
        fn any_subset_breakdown_consistent(
            selection in arb_selection(),
            product in arb_product(),
        ) {
            let pal = palette();
            let slots: Vec<MetricSlot> = selection.iter().enumerate()
                .filter(|(_, on)| **on)
                .take(3)
                .map(|(i, _)| MetricSlot {
                    metric: build_metric(pal[i].measure, pal[i].name, pal[i].map01),
                    weight: 1.0,
                })
                .collect();
            if slots.is_empty() { return Ok(()); }

            let n = slots.len();
            let req = UserRequest {
                slot0: slots.first().cloned(),
                slot1: slots.get(1).cloned(),
                slot2: slots.get(2).cloned(),
            };
            let scores = score_batch(&req, &[product]);
            let s = scores[0];
            let rows = breakdown_batch(&req, &product);
            prop_assert_eq!(rows.len(), n);

            let row_sum: f32 = rows.iter().map(|r| r.contribution).sum();
            prop_assert!((s - row_sum).abs() < 1e-4);
            let weight_sum: f32 = rows.iter().map(|r| r.weight).sum();
            prop_assert!((weight_sum - 1.0).abs() < 1e-4);
        }
    }
}
