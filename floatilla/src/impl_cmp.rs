
use super::*;
use core::{
    cmp::*,
    hash::{Hash, Hasher},
};

impl<F: Fp> Eq for Real<F> {}

impl<F: Fp> PartialEq for Real<F> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.0, &other.0)
    }
}

impl<F: Fp> Ord for Real<F> {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe {
            match PartialOrd::partial_cmp(&self.0, &other.0) {
                Some(ord) => ord,
                None => unreachable_unchecked(),
            }
        }
    }
}

impl<F: Fp> PartialOrd for Real<F> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<F: Fp> Hash for Real<F> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let (mantissa, exponent, sign) = self.0.integer_decode();
        let sign = sign == 1 || self.0.is_zero();
        Hash::hash(&sign, h);
        Hash::hash(&mantissa, h);   
        Hash::hash(&exponent, h);
    }
}

impl<F: Fp> Eq for FpRepr<F> {}

impl<F: Fp> PartialEq for FpRepr<F> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&FpRepr::NegInf, &FpRepr::NegInf) => true,
            (&FpRepr::Real(a), &FpRepr::Real(b)) => a == b,
            (&FpRepr::PosInf, &FpRepr::PosInf) => true,
            (&FpRepr::Nan, &FpRepr::Nan) => true,
            _ => false,
        }
    }
}

impl<F: Fp> Ord for FpRepr<F> {
    fn cmp(&self, other: &Self) -> Ordering {
        if let (
            &FpRepr::Real(ref a), 
            &FpRepr::Real(ref b),
        ) = (self, other) {
            Ord::cmp(a, b)
        } else {
            let a: u8 = match self {
                &FpRepr::NegInf => 0,
                &FpRepr::Real(_) => 1,
                &FpRepr::PosInf => 2,
                &FpRepr::Nan => 3,
            };
            let b: u8 = match other {
                &FpRepr::NegInf => 0,
                &FpRepr::Real(_) => 1,
                &FpRepr::PosInf => 2,
                &FpRepr::Nan => 3,
            };
            Ord::cmp(&a, &b)
        }
    }
}

impl<F: Fp> PartialOrd for FpRepr<F> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

impl<F: Fp> Hash for FpRepr<F> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        match self {
            &FpRepr::NegInf => h.write_u8(0),
            &FpRepr::Real(ref r) => {
                h.write_u8(1);
                Hash::hash(r, h);
            },
            &FpRepr::PosInf => h.write_u8(2),
            &FpRepr::Nan => h.write_u8(3),
        };
    }
}
