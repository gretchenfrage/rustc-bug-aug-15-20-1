
use super::*;
use core::fmt::{self, Display, Formatter};
#[cfg(feature = "std")]
use std::error::Error;

/// Error type for failing to parse a `Real` from a string. 
#[derive(Copy, Clone, Debug)]
pub enum ParseRealError<E, F: Fp> {
    NotFloat(E),
    NotReal(NotRealError<F>)
}

/// Error type when a float is non-real.
#[derive(Copy, Clone, Debug)]
pub struct NotRealError<F: Fp>(pub F);

impl<F: Fp> Display for NotRealError<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("attempt to construct Real from non-real float: ")?;
        Display::fmt(&self.0, f)?;
        Ok({})
    }
}

/// Error when performing mathematically-undefined operation that outputs
/// a non-real float.
#[derive(Copy, Clone, Debug)]
pub struct MathError<F: Fp> {
    pub output: F,
    pub operation: MathOp<F>,
}

impl<F: Fp> Display for MathError<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("operation on Real with non-real result: ")?;
        Display::fmt(&self.operation, f)?;
        f.write_str("==")?;
        Display::fmt(&self.output, f)?;
        Ok({})
    }
}

/// Displayable form of non-nested mathematical expressions.
#[derive(Copy, Clone, Debug)]
pub enum MathOp<F: Fp> {
    Infix(F, char, F),
    Prefix(char, F),
}

impl<F: Fp> Display for MathOp<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            &MathOp::Infix(ref a, ref b, ref c) => {
                Display::fmt(a, f)?;
                Display::fmt(b, f)?;
                Display::fmt(c, f)?;
            }
            &MathOp::Prefix(ref a, ref b) => {
                Display::fmt(a, f)?;
                Display::fmt(b, f)?;
            }
        };
        Ok({})
    }
}

#[cfg(feature = "std")]
impl<F: Fp> Error for NotRealError<F> {}

#[cfg(feature = "std")]
impl<F: Fp> Error for MathError<F> {}
