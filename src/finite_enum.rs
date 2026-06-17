//! Anonymous zero-vtable enums for [`finite_score_set!`](crate::finite_score_set!).
//!
//! Each `FiniteEnum{N}` wraps N heterogeneous metric types and dispatches
//! `eval`/`name` via `match` — zero vtable overhead.

/// Generate a finite enum + `DynMetric` impl for a given arity.
macro_rules! finite_enum_inner {
    ($name:ident; $($idx:tt $variant:ident),+ $(,)?) => {
        #[allow(dead_code)]
        pub enum $name<$($variant),+> {
            $($variant($variant)),+
        }

        impl<T: $crate::Float, I, $($variant: $crate::DynMetric<T, I>),+>
            $crate::DynMetric<T, I> for $name<$($variant),+>
        {
            #[inline]
            fn eval(&self, input: &I) -> $crate::Witnessed<T, $crate::Value01> {
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

// ---- arity instances ----

finite_enum_inner!(FiniteEnum1; 0 M0);
finite_enum_inner!(FiniteEnum2; 0 M0, 1 M1);
finite_enum_inner!(FiniteEnum3; 0 M0, 1 M1, 2 M2);
finite_enum_inner!(FiniteEnum4; 0 M0, 1 M1, 2 M2, 3 M3);
finite_enum_inner!(FiniteEnum5; 0 M0, 1 M1, 2 M2, 3 M3, 4 M4);
finite_enum_inner!(FiniteEnum6; 0 M0, 1 M1, 2 M2, 3 M3, 4 M4, 5 M5);
finite_enum_inner!(FiniteEnum7; 0 M0, 1 M1, 2 M2, 3 M3, 4 M4, 5 M5, 6 M6);
finite_enum_inner!(FiniteEnum8; 0 M0, 1 M1, 2 M2, 3 M3, 4 M4, 5 M5, 6 M6, 7 M7);
