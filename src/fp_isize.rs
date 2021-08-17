use std::{convert::TryFrom, fmt::{Debug, Display}, ops::{Add, AddAssign, Neg, Sub, SubAssign}};

/// Fixed-Point decimal number representation
///
/// Implemented to support a precision of up to PRECISION numbers after the decimal point
/// Can maximally represent 64 Bit values
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct FpIsize<const PRECISION: u32> {
    inner: isize,
}

const fn precision_factor(precision: u32) -> isize {
    10isize.pow(precision)
}

impl <const PRECISION: u32> FpIsize<PRECISION> {

    /// Creates a new TxAmount based on the inner value
    ///
    /// The caller is responsible to calculate the correct inner value
    pub const fn new(inner: isize) -> Self {
        FpIsize {
            inner
        }
    }

    /// Creates a new TxAmount with a value of 0
    pub const fn zero() -> Self {
        Self::new(0)
    }
}

impl <const PRECISION: u32> Add for FpIsize<PRECISION> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.inner + rhs.inner)
    }
}

impl <const PRECISION: u32> AddAssign for FpIsize<PRECISION> {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner
    }
}

impl <const PRECISION: u32> Sub for FpIsize<PRECISION> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.inner - rhs.inner)
    }
}

impl <const PRECISION: u32> SubAssign for FpIsize<PRECISION> {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner -= rhs.inner
    }
}

impl <const PRECISION: u32> Neg for FpIsize<PRECISION> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.inner)
    }
}

impl <const PRECISION: u32> From<(&str,&str)> for FpIsize<PRECISION> {

    /// Converts 2 string arguments to a new TxAmount
    ///
    /// Expected format of the original string: "integral.fractional"
    fn from((integral, fractional): (&str, &str)) -> Self {
        let precision = u32::try_from(fractional.len()).unwrap();
        let integral = isize::from_str_radix(integral, 10).unwrap();
        let fractional = isize::from_str_radix(fractional, 10).unwrap();

        Self::new(integral * 10isize.pow(PRECISION) + fractional * 10isize.pow(PRECISION - precision))
    }

}

impl <const PRECISION: u32> Debug for FpIsize<PRECISION> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl <const PRECISION: u32> Display for FpIsize<PRECISION> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let integral = self.inner / precision_factor(PRECISION);
        let fractional = (self.inner % precision_factor(PRECISION)).abs();

        write!(f, "{}.{:0precision$}", integral, fractional, precision = PRECISION as usize) 
    }
}