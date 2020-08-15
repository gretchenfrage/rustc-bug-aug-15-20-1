//! Geometric angle type.

use std::{
    ops::*,
    fmt::{self, Formatter, Debug, Display, Write},
};
use num_traits::{
    real::Real,
    identities::One,
    cast::NumCast,
};

/// Geometric angle type. 
///
/// Abstracted over degrees vs. radians, but internally stored as radians. 
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
#[repr(transparent)]
pub struct Angle<F: Real>(F);

macro_rules! trig_methods {
    ($(
        #[$($attr:tt)*]
        $func:ident;
    )*)=>{
        $(
        #[$($attr)*]
        pub fn $func(&self) -> F {
            (self.0).$func()
        }
        )*
    };
}

impl<F: Real> Angle<F> {
    /// Cast the inner float to a different float type. 
    ///
    /// Panics if the cast fails.
    pub fn cast<T: NumCast + Real>(self) -> Angle<T> {
        Angle(NumCast::from(self.0).unwrap())
    }

    /// Get as degrees. 
    pub fn deg(&self) -> F {
        self.0.to_degrees()
    }

    /// Get as radians. 
    pub fn rad(&self) -> F {
        self.0
    }

    trig_methods! {
        /// Sine function.
        sin;
        /// Cosine function. 
        cos;
        /// Tangent function. 
        tan;
        /// Hyperbolic sine function.
        sinh;
        /// Hyperbolic cosine function. 
        cosh;
        /// Hyperbolic tangent function. 
        tanh;
    }

    /// Cosecant operation (1 / sin).
    pub fn csc(&self) -> F {
        F::one() / self.sin()
    }

    /// Secant operation (1 / cos).
    pub fn sec(&self) -> F {
        F::one() / self.cos()
    }

    /// Cotangent operation (1 / tan). 
    pub fn cot(&self) -> F {
        F::one() / self.tan()
    }

    /// Get the greatest angle between `self` and `other`. 
    pub fn max(&self, other: Angle<F>) -> Angle<F> {
        Angle(self.0.max(other.0))
    }

    /// Get the least angle between `self` and `other`. 
    pub fn min(&self, other: Angle<F>) -> Angle<F> {
        Angle(self.0.min(other.0))
    }

    /// Clamp the value of this angle between a minimum and maximum. 
    pub fn clamp(&self, min: Angle<F>, max: Angle<F>) -> Angle<F> {
        self.max(min).min(max)
    }
}

/// Helper constructor from degrees to `Angle`. 
pub fn deg<F: Real>(degrees: F) -> Angle<F> {
    Angle(degrees.to_radians())
}

/// Helper constructor from radians to `Angle`. 
pub fn rad<F: Real>(radians: F) -> Angle<F> {
    Angle(radians)
}

macro_rules! trig_ctors {
    (
        $(
        #[$($attr:tt)*]
        $func:ident;
        )*
    )=>{
        $(
            #[$($attr)*]
            pub fn $func<F: Real>(n: F) -> Angle<F> {
                Angle(n.$func())
            }
        )*
    };
}

trig_ctors! {
    /// Inverse sine helper constructor. 
    asin;
    /// Inverse hyperbolic sine helper constructor. 
    asinh;
    /// Inverse cosine helper constructor.
    acos;
    /// Inverse hyperbolic cosine helper constructor. 
    acosh;
    /// Inverse tangent helper constructor. 
    atan;
    /// Inverse hyperbolic tangent helper constructor. 
    atanh;
}

/// atan2 helper constructor. 
pub fn atan2<F: Real>(y: F, x: F) -> Angle<F> {
    Angle(y.atan2(x))
}

impl<F: Real> Display for Angle<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.cast::<f32>().deg(), f)?;
        f.write_char('°')?;
        Ok({})
    }
}

impl<F: Real> Debug for Angle<F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<F: Real> Neg for Angle<F> {
    type Output = Angle<F>;

    fn neg(self) -> Angle<F> {
        Angle(-self.0)
    }
}

impl<F: Real> Add<Angle<F>> for Angle<F> {
    type Output = Angle<F>;

    fn add(self, rhs: Angle<F>) -> Angle<F> {
        Angle(self.0 + rhs.0)
    }
}

impl<F: Real> Sub<Angle<F>> for Angle<F> {
    type Output = Angle<F>;

    fn sub(self, rhs: Angle<F>) -> Angle<F> {
        Angle(self.0 - rhs.0)
    }
}

impl<F: Real> Mul<F> for Angle<F> {
    type Output = Angle<F>;

    fn mul(self, rhs: F) -> Angle<F> {
        Angle(self.0 * rhs)
    }
}

impl<F: Real> Div<F> for Angle<F> {
    type Output = Angle<F>;

    fn div(self, rhs: F) -> Angle<F> {
        Angle(self.0 / rhs)
    }
}

impl<F: Real> Div<Angle<F>> for Angle<F> {
    type Output = F;

    fn div(self, rhs: Angle<F>) -> F {
        self.0 / rhs.0
    }
}

impl<F: Real> Rem<Angle<F>> for Angle<F> {
    type Output = Angle<F>;

    fn rem(self, rhs: Angle<F>) -> Angle<F> {
        Angle(self.0 % rhs.0)
    }
}

impl<F: Real> AddAssign<Angle<F>> for Angle<F> {
    fn add_assign(&mut self, rhs: Angle<F>) {
        self.0 = self.0 + rhs.0;
    }
}

impl<F: Real> SubAssign<Angle<F>> for Angle<F> {
    fn sub_assign(&mut self, rhs: Angle<F>) {
        self.0 = self.0 - rhs.0;
    }
}

impl<F: Real> MulAssign<F> for Angle<F> {
    fn mul_assign(&mut self, rhs: F) {
        self.0 = self.0 * rhs;
    }
}

impl<F: Real> DivAssign<F> for Angle<F> {
    fn div_assign(&mut self, rhs: F) {
        self.0 = self.0 / rhs;
    }
}

impl<F: Real> RemAssign<Angle<F>> for Angle<F> {
    fn rem_assign(&mut self, rhs: Angle<F>) {
        self.0 = self.0 % rhs.0;
    }
}
