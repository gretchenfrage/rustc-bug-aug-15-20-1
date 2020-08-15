
use super::*;
use crate::error::ParseRealError;
use num_traits::{
    Num,
    real::Real as RealTrait,
    cast::{NumCast, ToPrimitive},
    identities::{Zero, One},
    float::Float,
};

impl<F: Fp + Float> Num for Real<F> {
    type FromStrRadixErr = ParseRealError<<F as Num>::FromStrRadixErr, F>;

    fn from_str_radix(
        str: &str, 
        radix: u32
    ) -> Result<Self, Self::FromStrRadixErr> {
        F::from_str_radix(str, radix)
            .map_err(ParseRealError::NotFloat)
            .and_then(|f| {
                Real::try_new(f)
                    .map_err(ParseRealError::NotReal)
            })
    }
}

impl<F: Fp + Float> NumCast for Real<F> {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        F::from(n).and_then(|f| Real::try_new(f).ok())
    }
}

macro_rules! ctor_delegate {
    ($(
        fn $name:ident() -> Self;
    )*)=>{$(
        fn $name() -> Self {
            Real::try_new(<F as RealTrait>::$name()).unwrap()
        }
    )*};
}

macro_rules! map_delegate {
    ($(
        fn $name:ident(self $(,$param:ident: Self)*) -> Self;
    )*)=>{$(
        fn $name(self $(,$param: Self)*) -> Self {
            Real::try_new(
                RealTrait::$name(self.0 $(,$param.0)*)
            ).unwrap()
        }
    )*};
}

impl<F: Fp + Float> RealTrait for Real<F> {
    ctor_delegate! {
        fn min_value() -> Self;
        fn min_positive_value() -> Self;
        fn epsilon() -> Self;
        fn max_value() -> Self;
    }

    map_delegate! {
        fn floor(self) -> Self;
        fn ceil(self) -> Self;
        fn round(self) -> Self;
        fn trunc(self) -> Self;
        fn fract(self) -> Self;
        fn abs(self) -> Self;
        fn signum(self) -> Self;
        fn mul_add(self, a: Self, b: Self) -> Self;
        fn recip(self) -> Self;
        fn powf(self, n: Self) -> Self;
        fn sqrt(self) -> Self;
        fn exp(self) -> Self;
        fn exp2(self) -> Self;
        fn ln(self) -> Self;
        fn log(self, base: Self) -> Self;
        fn log2(self) -> Self;
        fn log10(self) -> Self;
        fn to_degrees(self) -> Self;
        fn to_radians(self) -> Self;
        fn max(self, other: Self) -> Self;
        fn min(self, other: Self) -> Self;
        fn abs_sub(self, other: Self) -> Self;
        fn cbrt(self) -> Self;
        fn hypot(self, other: Self) -> Self;
        fn sin(self) -> Self;
        fn cos(self) -> Self;
        fn tan(self) -> Self;
        fn asin(self) -> Self;
        fn acos(self) -> Self;
        fn atan(self) -> Self;
        fn atan2(self, other: Self) -> Self;
        fn exp_m1(self) -> Self;
        fn ln_1p(self) -> Self;
        fn sinh(self) -> Self;
        fn cosh(self) -> Self;
        fn tanh(self) -> Self;
        fn asinh(self) -> Self;
        fn acosh(self) -> Self;
        fn atanh(self) -> Self;
    }

    fn powi(self, n: i32) -> Self {
        Real::try_new(RealTrait::powi(self.0, n)).unwrap()   
    }

    fn is_sign_positive(self) -> bool {
        RealTrait::is_sign_positive(self.0)
    }

    fn is_sign_negative(self) -> bool {
        RealTrait::is_sign_negative(self.0)
    }

    fn sin_cos(self) -> (Self, Self) {
        let (fsin, fcos) = RealTrait::sin_cos(self.0);
        (Real::try_new(fsin).unwrap(), Real::try_new(fcos).unwrap())
    }
}

impl<F: Fp + Float> Zero for Real<F> {
    fn zero() -> Self {
        Real::try_new(F::zero()).unwrap()
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<F: Fp + Float> One for Real<F> {
    fn one() -> Self {
        Real::try_new(F::zero()).unwrap()
    }

    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        self.0.is_one()
    }
}

macro_rules! to_primitive_delegate {
    ($(
        fn $name:ident(&self) -> $ret:ty;
    )*)=>{$(
        fn $name(&self) -> $ret {
            ToPrimitive::$name(&self.0)
        }
    )*};
}

impl<F: Fp + Float> ToPrimitive for Real<F> {
    to_primitive_delegate! {
        fn to_i64(&self) -> Option<i64>;
        fn to_u64(&self) -> Option<u64>;
        fn to_isize(&self) -> Option<isize>;
        fn to_i8(&self) -> Option<i8>;
        fn to_i16(&self) -> Option<i16>;
        fn to_i32(&self) -> Option<i32>;
        fn to_i128(&self) -> Option<i128>;
        fn to_usize(&self) -> Option<usize>;
        fn to_u8(&self) -> Option<u8>;
        fn to_u16(&self) -> Option<u16>;
        fn to_u32(&self) -> Option<u32>;
        fn to_u128(&self) -> Option<u128>;
        fn to_f32(&self) -> Option<f32>;
        fn to_f64(&self) -> Option<f64>;
    }
}