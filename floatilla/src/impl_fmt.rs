
use super::*;
use crate::try_math::TryMath;
use core::fmt::{self, Debug, Display, Formatter};

impl<F: Fp> Display for Real<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<F: Fp> Debug for Real<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<F: Fp> Display for FpRepr<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.to_float(), f)
    }
}

impl<F: Fp> Debug for TryMath<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<F: Fp> Display for TryMath<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}