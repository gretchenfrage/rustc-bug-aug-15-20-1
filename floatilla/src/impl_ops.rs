
use super::*;
use crate::{
    error::{NotRealError, MathError, MathOp},
    try_math::TryMath,
};
use core::ops::*;

macro_rules! binary_op {
    ($trait:ident, $method:ident, $assign_trait:ident, $assign_method:ident, $operator_char:expr)=>{
        impl<F: Fp> $trait for Real<F> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self {
                match Real::try_new($trait::$method(self.to_float(), rhs.to_float())) {
                    Ok(r) => r,
                    Err(NotRealError(f)) => {
                        panic!(
                            "operation on Real with non-real result: {}{}{}=={}",
                            self.to_float(),
                            $operator_char,
                            rhs.to_float(),
                            f,
                        );
                    }
                }
            }
        }

        impl<F: Fp> $trait for FpRepr<F> {
            type Output = Self;

            fn $method(self, rhs: Self) -> Self {
                FpRepr::new($trait::$method(self.to_float(), rhs.to_float()))
            }
        }

        impl<F: Fp> $trait for TryMath<F> {
            type Output = Result<Self, MathError<F>>;

            fn $method(self, rhs: Self) -> Result<Self, MathError<F>> {
                Real::try_new($trait::$method(self.to_float(), rhs.to_float()))
                    .map(TryMath)
                    .map_err(|NotRealError(f)| MathError {
                        output: f,
                        operation: MathOp::Infix(
                            self.to_float(), 
                            $operator_char, 
                            rhs.to_float(),
                        )
                    })
            }
        }

        impl<F: Fp> $assign_trait for Real<F> {
            #[inline(always)]
            fn $assign_method(&mut self, rhs: Self) {
                *self = $trait::$method(*self, rhs);
            }
        }

        impl<F: Fp> $assign_trait for FpRepr<F> {
            #[inline(always)]
            fn $assign_method(&mut self, rhs: Self) {
                *self = $trait::$method(*self, rhs);
            }
        }
    };
}

binary_op!(Add, add, AddAssign, add_assign, '+');
binary_op!(Sub, sub, SubAssign, sub_assign, '-');
binary_op!(Mul, mul, MulAssign, mul_assign, '*');
binary_op!(Div, div, DivAssign, div_assign, '/');
binary_op!(Rem, rem, RemAssign, rem_assign, '%');

macro_rules! unary_op {
    ($trait:ident, $method:ident, $operator_char:expr)=>{
        impl<F: Fp> $trait for Real<F> {
            type Output = Self;

            fn $method(self) -> Self {
                match Real::try_new($trait::$method(self.to_float())) {
                    Ok(r) => r,
                    Err(NotRealError(f)) => {
                        panic!(
                            "operation on Real with non-real result: {}{}=={}",
                            $operator_char,
                            self.to_float(),
                            f,
                        );
                    }
                }
            }
        }

        impl<F: Fp> $trait for TryMath<F> {
            type Output = Result<Self, MathError<F>>;

            fn $method(self) -> Result<Self, MathError<F>> {
                Real::try_new($trait::$method(self.to_float()))
                    .map(TryMath)
                    .map_err(|NotRealError(f)| MathError {
                        output: f,
                        operation: MathOp::Prefix(
                            $operator_char, 
                            self.to_float(), 
                        )
                    })
            }
        }

        impl<F: Fp> $trait for FpRepr<F> {
            type Output = Self;

            fn $method(self,) -> Self {
                FpRepr::new($trait::$method(self.to_float()))
            }
        }
    };
}

unary_op!(Neg, neg, '-');
