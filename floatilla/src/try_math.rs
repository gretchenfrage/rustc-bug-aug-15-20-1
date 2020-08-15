
use super::*;
use crate::error::MathError;

pub type Result<F> = core::result::Result<TryMath<F>, MathError<F>>;

/// Wrapper around `Real` that returns results for arithmatic operations.
#[derive(Copy, Clone)]
pub struct TryMath<F: Fp>(pub Real<F>);

impl<F: Fp> TryMath<F> {
    pub fn to_real(self) -> Real<F> {
        self.0
    }

    pub fn to_float(self) -> F {
        self.0.to_float()
    }
}

