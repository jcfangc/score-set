// ---------------------------------------------------------------------------
// declare_metric_enum! — generate a zero-vtable metric enum (Layer 2)
// ---------------------------------------------------------------------------

/// Declare a finite enum of metric types with static-dispatch `eval`.
///
/// This macro generates an enum whose variants each wrap a concrete metric
/// type. The generated [`ErasedMetric`](crate::ErasedMetric) implementation
/// uses `match` + static dispatch — zero vtable overhead for all variants
/// except the optional `Custom` escape hatch.
///
/// # Syntax
///
/// ```ignore
/// declare_metric_enum! {
///     pub MetricKind<T, I> =>
///         Gc(GcMetric),
///         Tm(TmMetric),
///         Extinction(ExtinctionMetric),
///         Custom(Box<dyn ErasedMetric<T, I>>),
/// }
/// ```
///
/// # Generated items
///
/// - An enum `MetricKind<T: Float, I>` with the listed variants.
/// - An [`ErasedMetric`](crate::ErasedMetric) implementation with
///   static-dispatch `eval` and `name` methods.
///
/// # Requirements on variant types
///
/// Each variant's inner type must provide:
/// - `fn eval(&self, input: &I) -> Witnessed<T, Value01>`
/// - `fn name(&self) -> &str`
///
/// Both [`Metric`](crate::Metric) and `Box<dyn ErasedMetric<T, I>>` satisfy
/// this contract.
///
/// # Example
///
/// ```ignore
/// use score_set::*;
/// use core::marker::PhantomData;
///
/// // Define a concrete metric type
/// struct GcRatio<T: Float, I> { _phantom: PhantomData<(T, I)> }
/// impl<T: Float, I> GcRatio<T, I> {
///     fn eval(&self, _: &I) -> Witnessed<T, Value01> { ... }
///     fn name(&self) -> &str { "gc" }
/// }
///
/// // Declare the enum
/// declare_metric_enum! {
///     pub MyMetric<T, I> =>
///         Gc(GcRatio<T, I>),
///         Custom(Box<dyn ErasedMetric<T, I>>),
/// }
/// ```
#[macro_export]
macro_rules! declare_metric_enum {
    (
        $(#[$attr:meta])*
        $vis:vis $name:ident<$T:ident, $I:ident> =>
        $($variant:ident($ty:ty)),+ $(,)?
    ) => {
        $(#[$attr])*
        #[allow(clippy::pub_enum_variant_fields)]
        $vis enum $name<$T: $crate::Float, $I> {
            $($variant($ty)),+
        }

        impl<$T: $crate::Float, $I> $crate::ErasedMetric<$T, $I> for $name<$T, $I> {
            #[inline]
            fn eval(&self, input: &$I) -> $crate::Witnessed<$T, $crate::Value01> {
                match self {
                    $(Self::$variant(m) => m.eval(input)),+
                }
            }

            #[inline]
            fn name(&self) -> &str {
                match self {
                    $(Self::$variant(m) => m.name()),+
                }
            }
        }
    };
}

#[cfg(test)]
mod tests_for_metric_enum;
