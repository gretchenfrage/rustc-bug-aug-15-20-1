//! Fully ordered, hashable, optionally finite-only f32 and f64.
//! 
//! If the `std` feature is disabled, this becomes `no_std`. 

#![cfg_attr(not(feature = "std"), no_std)]

// TODO: FromStr implementations
// TODO: num_trait implementations for FpRepr and TryMath

extern crate num_traits;

use core::{
    hint::unreachable_unchecked,
    fmt::{Debug, Display},
};
use num_traits::float::FloatCore;
use error::NotRealError;

/// Implementation of arithmatic ops.
mod impl_ops;
/// Implementation of comparison ops, also hashing.
mod impl_cmp;
/// Implementation of formatting.
mod impl_fmt;
/// Implementation of `num_traits::real::Real`.
#[cfg(feature = "std")]
mod impl_real;

/// Wrapper around `Real` that returns results for arithmatic operations.
pub mod try_math;

/// Float-variants errors.
pub mod error;

/// Trait for valid inner float types. 
///
/// Unlike `num_traits::FloatCore`, this is unsafe, because incorrect float
/// behavior may cause undefined behavior in this crate's unsafe code. 
pub unsafe trait Fp: FloatCore + Debug + Display {}

unsafe impl Fp for f32 {}
unsafe impl Fp for f64 {}

/// Helper constructor for `Real<f32>`.
pub fn r32(f: f32) -> Real<f32> {
    Real::new(f)
}

/// Helper constructor for `Real<f64>`.
pub fn r64(f: f64) -> Real<f64> {
    Real::new(f)
}

/// Float which is not inf, -inf, or nan.
/// 
/// Impls `Eq`, `Ord`, and `Hash`, as well as all arithmetic ops. Arithmetic 
/// that creates non-real variants cause runtime panic.
#[derive(Copy, Clone)]
pub struct Real<F: Fp>(F);

impl<F: Fp> Real<F> {
    /// Construct from a float, or error.
    #[inline(always)]
    pub fn try_new(f: F) -> Result<Self, NotRealError<F>> {
        if f.is_infinite() || f.is_nan() {
            Err(NotRealError(f))
        } else {
            Ok(Real(f))
        }
    }

    /// Construct from a float, or panic.
    #[inline(always)]
    pub fn new(f: F) -> Self {
        match Real::try_new(f) {
            Ok(r) => r,
            Err(e) => panic!("{}", e),
        }
    }

    /// Construct from a float, UB if the float is invalid.
    #[inline(always)]
    pub unsafe fn new_unchecked(f: F) -> Self {
        Real(f)
    }

    /// Extract the raw float.
    pub fn to_float(self) -> F {
        self.0
    }

    /// Map the inner float. Panic if the output is non-real.
    pub fn map<M>(self, map: M) -> Self 
    where
        M: FnOnce(F) -> F
    {
        Real::new(map(self.0))
    }
}
/*
use num_traits::{
    Num,
    FloatCore,
    ParseFloatError,
};

impl<F: FloatCore + FloatCore> Num for Real<F> {
    type FromStrRadixErr = ParseFloatError;

    fn from_str_radix(
        str: &str, 
        radix: u32
    ) -> Result<Self, ParseFloatError> {
        F::from_str_radix(str, radix)
            .and_then(|f| {
                Real::try_new(f)
                    .map_err(drop)
                    .map_err(|()| ParseFloatError::Invalid)
            })
    }
}

#[cfg(feature = "stdlib")]
use num_traits::{
    Real,
}

#[cfg(feature = "stdlib")]
impl<F: FloatCore + FloatCore> Real for Real<F> {

}
*/
/// Explicit representation of float real and non-real variants.
///
/// Impls `Eq`, `Ord`, and `Hash`, as well as all arithmetic ops. Considers 
/// NaN equal to NaN, and greater than non-NaN variants.
#[derive(Copy, Clone, Debug)]
pub enum FpRepr<F: Fp> {
    NegInf,
    Real(Real<F>),
    PosInf,
    Nan,
}

impl<F: Fp> FpRepr<F> {
    /// Construct from a float. Always succeeds.
    pub fn new(f: F) -> Self {
        unsafe {
            if f.is_infinite() {
                if f.is_sign_positive() {
                    FpRepr::PosInf
                } else {
                    FpRepr::NegInf
                }
            } else if f.is_nan() {
                FpRepr::Nan
            } else {
                FpRepr::Real(Real::new_unchecked(f))
            }
        }
    }

    /// Extract `Real` variant, or panic.
    pub fn to_real(self) -> Real<F> {
        match self {
            FpRepr::Real(real) => real,
            _ => {
                panic!("to_real on non-real FpRepr: {:?}", self);
            }
        }
    }

    /// Convert to a raw float.
    pub fn to_float(self) -> F {
        match self {
            FpRepr::NegInf => F::neg_infinity(),
            FpRepr::Real(real) => real.to_float(),
            FpRepr::PosInf => F::infinity(),
            FpRepr::Nan => F::nan(),
        }
    }

    /// Map the raw float.
    pub fn map<M>(self, map: M) -> Self 
    where
        M: FnOnce(F) -> F
    {
        FpRepr::new(map(self.to_float()))
    }
}

impl<F: Fp> From<Real<F>> for FpRepr<F> {
    fn from(real: Real<F>) -> Self {
        Self::new(real.to_float())
    }
}

impl<F: Fp> From<FpRepr<F>> for Real<F> {
    fn from(var: FpRepr<F>) -> Self {
        var.to_real()         
    }
}

