use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem;
use core::num::FpCategory;
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};

pub use half::f16;
use num_bigint::{BigInt, Sign};
use num_traits::ToPrimitive;

use crate::{DivisionError, Integer};

#[derive(Debug, Copy, Clone, PartialOrd)]
pub struct Float(f64);

impl Float {
    const I64_UPPER_BOUNDARY: f64 = (1i64 << f64::MANTISSA_DIGITS) as f64;
    const I64_LOWER_BOUNDARY: f64 = (-1i64 << f64::MANTISSA_DIGITS) as f64;

    pub fn new(float: f64) -> Result<Float, FloatError> {
        FloatError::from_category(float.classify())?;
        Ok(Float(float))
    }

    #[inline(always)]
    pub fn raw(&self) -> u64 {
        unsafe { mem::transmute(self.0) }
    }

    pub fn inner(&self) -> f64 {
        self.0
    }

    pub fn abs(&self) -> Float {
        if self.0 < 0.0 {
            Float(self.0 * -1.0)
        } else {
            *self
        }
    }

    pub fn is_zero(&self) -> bool {
        self.0.classify() == FpCategory::Zero
    }

    pub fn to_integer(&self) -> Integer {
        Integer::Small(self.0 as i64)
    }

    /// Returns whether this float is more precise than an integer.
    /// If the float is precise, the other integer this is being compared to will
    /// be converted to a float.
    /// If the float is not precise, it will be converted to the integer.
    pub fn is_precise(&self) -> bool {
        self.0 >= Self::I64_LOWER_BOUNDARY && self.0 <= Self::I64_UPPER_BOUNDARY
    }

    #[inline]
    pub fn is_finite(&self) -> bool {
        self.0.is_finite()
    }
}
impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl From<f64> for Float {
    #[inline(always)]
    fn from(f: f64) -> Self {
        Self(f)
    }
}
impl From<f32> for Float {
    #[inline(always)]
    fn from(f: f32) -> Self {
        Self(f as f64)
    }
}
impl From<f16> for Float {
    #[inline(always)]
    fn from(f: f16) -> Self {
        Self(f.to_f64())
    }
}
impl Into<f64> for Float {
    #[inline(always)]
    fn into(self) -> f64 {
        self.0
    }
}
impl Into<f32> for Float {
    #[inline(always)]
    fn into(self) -> f32 {
        self.0 as f32
    }
}
impl Into<f16> for Float {
    #[inline]
    fn into(self) -> f16 {
        f16::from_f64(self.0)
    }
}
impl Hash for Float {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}
impl Eq for Float {}
impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        if !self.is_finite() || !other.is_finite() {
            return false;
        }
        self.0.eq(&other.0)
    }
}
impl PartialEq<i64> for Float {
    fn eq(&self, y: &i64) -> bool {
        match self.0 {
            x if x.is_infinite() => false,
            x if x >= Self::I64_UPPER_BOUNDARY || x <= Self::I64_LOWER_BOUNDARY => {
                let x: i64 = unsafe { x.to_int_unchecked() };
                x.eq(y)
            }
            x => x.eq(&(*y as f64)),
        }
    }
}
impl PartialEq<BigInt> for Float {
    fn eq(&self, y: &BigInt) -> bool {
        let Some(y) = y.to_i64() else { return false; };
        self.eq(&y)
    }
}
impl PartialEq<Integer> for Float {
    fn eq(&self, y: &Integer) -> bool {
        match y {
            Integer::Small(i) => self.eq(i),
            Integer::Big(i) => self.eq(i),
        }
    }
}
impl PartialOrd<i64> for Float {
    fn partial_cmp(&self, y: &i64) -> Option<Ordering> {
        match self.0 {
            x if x.is_infinite() => {
                if x.is_sign_negative() {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            x if x >= Self::I64_UPPER_BOUNDARY || x <= Self::I64_LOWER_BOUNDARY => {
                let x: i64 = unsafe { x.to_int_unchecked() };
                Some(x.cmp(y))
            }
            x => x.partial_cmp(&(*y as f64)),
        }
    }
}
impl PartialOrd<BigInt> for Float {
    fn partial_cmp(&self, y: &BigInt) -> Option<Ordering> {
        match self.0 {
            x if x.is_infinite() => {
                if x.is_sign_negative() {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Greater)
                }
            }
            _ => {
                let too_large = if y.sign() == Sign::Minus {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
                let Some(y) = y.to_i64() else { return Some(too_large); };
                self.partial_cmp(&y)
            }
        }
    }
}
impl PartialOrd<Integer> for Float {
    fn partial_cmp(&self, other: &Integer) -> Option<Ordering> {
        match other {
            Integer::Small(i) => self.partial_cmp(i),
            Integer::Big(i) => self.partial_cmp(i),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FloatError {
    Nan,
    Infinite,
}
impl FloatError {
    pub fn from_category(category: FpCategory) -> Result<(), Self> {
        match category {
            FpCategory::Nan => Err(FloatError::Nan),
            FpCategory::Infinite => Err(FloatError::Infinite),
            _ => Ok(()),
        }
    }
}

impl fmt::Display for FloatError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FloatError::Nan => write!(f, "NaN"),
            FloatError::Infinite => write!(f, "Inf"),
        }
    }
}

impl Neg for Float {
    type Output = Float;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Float> for Float {
    type Output = Result<Float, FloatError>;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.0 + rhs.0)
    }
}
impl Add<Integer> for Float {
    type Output = Result<Float, FloatError>;

    fn add(self, rhs: Integer) -> Self::Output {
        self + rhs.to_efloat()?
    }
}
impl Add<&Integer> for Float {
    type Output = Result<Float, FloatError>;

    fn add(self, rhs: &Integer) -> Self::Output {
        self + rhs.to_efloat()?
    }
}
impl Add<Float> for Integer {
    type Output = Result<Float, FloatError>;

    fn add(self, rhs: Float) -> Self::Output {
        self.to_efloat()? + rhs
    }
}
impl Add<Float> for &Integer {
    type Output = Result<Float, FloatError>;

    fn add(self, rhs: Float) -> Self::Output {
        self.to_efloat()? + rhs
    }
}
impl Sub<Float> for Float {
    type Output = Result<Float, FloatError>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.0 - rhs.0)
    }
}
impl Sub<Integer> for Float {
    type Output = Result<Float, FloatError>;
    fn sub(self, rhs: Integer) -> Self::Output {
        self - rhs.to_efloat()?
    }
}
impl Sub<&Integer> for Float {
    type Output = Result<Float, FloatError>;
    fn sub(self, rhs: &Integer) -> Self::Output {
        self - rhs.to_efloat()?
    }
}
impl Sub<Float> for Integer {
    type Output = Result<Float, FloatError>;
    fn sub(self, rhs: Float) -> Self::Output {
        self.to_efloat()? - rhs
    }
}
impl Sub<Float> for &Integer {
    type Output = Result<Float, FloatError>;
    fn sub(self, rhs: Float) -> Self::Output {
        self.to_efloat()? - rhs
    }
}
impl Mul<Float> for Float {
    type Output = Result<Float, FloatError>;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.0 * rhs.0)
    }
}
impl Mul<Integer> for Float {
    type Output = Result<Float, FloatError>;
    fn mul(self, rhs: Integer) -> Self::Output {
        self * rhs.to_efloat()?
    }
}
impl Mul<&Integer> for Float {
    type Output = Result<Float, FloatError>;
    fn mul(self, rhs: &Integer) -> Self::Output {
        self * rhs.to_efloat()?
    }
}
impl Mul<Float> for Integer {
    type Output = Result<Float, FloatError>;
    fn mul(self, rhs: Float) -> Self::Output {
        self.to_efloat()? * rhs
    }
}
impl Mul<Float> for &Integer {
    type Output = Result<Float, FloatError>;
    fn mul(self, rhs: Float) -> Self::Output {
        self.to_efloat()? * rhs
    }
}
impl Div<Float> for Float {
    type Output = Result<Float, DivisionError>;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            Err(DivisionError)
        } else {
            Self::new(self.0 / rhs.0).map_err(|_| DivisionError)
        }
    }
}
impl Div<Integer> for Float {
    type Output = Result<Float, DivisionError>;

    fn div(self, rhs: Integer) -> Self::Output {
        self / rhs.to_efloat().map_err(|_| DivisionError)?
    }
}
impl Div<&Integer> for Float {
    type Output = Result<Float, DivisionError>;

    fn div(self, rhs: &Integer) -> Self::Output {
        self / rhs.to_efloat().map_err(|_| DivisionError)?
    }
}
impl Div<Float> for Integer {
    type Output = Result<Float, DivisionError>;

    fn div(self, rhs: Float) -> Self::Output {
        self.to_efloat().map_err(|_| DivisionError)? / rhs
    }
}
impl Div<Float> for &Integer {
    type Output = Result<Float, DivisionError>;

    fn div(self, rhs: Float) -> Self::Output {
        self.to_efloat().map_err(|_| DivisionError)? / rhs
    }
}
impl Rem<Float> for Float {
    type Output = Result<Float, DivisionError>;

    fn rem(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            Err(DivisionError)
        } else {
            Self::new(self.0 % rhs.0).map_err(|_| DivisionError)
        }
    }
}
impl Rem<Integer> for Float {
    type Output = Result<Float, DivisionError>;

    fn rem(self, rhs: Integer) -> Self::Output {
        self % rhs.to_efloat().map_err(|_| DivisionError)?
    }
}
impl Rem<&Integer> for Float {
    type Output = Result<Float, DivisionError>;

    fn rem(self, rhs: &Integer) -> Self::Output {
        self % rhs.to_efloat().map_err(|_| DivisionError)?
    }
}
impl Rem<Float> for Integer {
    type Output = Result<Float, DivisionError>;

    fn rem(self, rhs: Float) -> Self::Output {
        self.to_efloat().map_err(|_| DivisionError)? % rhs
    }
}
impl Rem<Float> for &Integer {
    type Output = Result<Float, DivisionError>;

    fn rem(self, rhs: Float) -> Self::Output {
        self.to_efloat().map_err(|_| DivisionError)? % rhs
    }
}
