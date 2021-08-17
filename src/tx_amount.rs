use std::{convert::TryFrom, fmt::{Debug, Display}, ops::{Add, AddAssign, Neg, Sub, SubAssign}};

const PRECISION: u32 = 4;
const PRECISION_FACTOR: isize = 10isize.pow(PRECISION);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TxAmount {
    inner: isize,
}

impl TxAmount {
    pub const fn new(inner: isize) -> Self {
        TxAmount {
            inner
        }
    }

    pub const fn zero() -> Self {
        Self::new(0)
    }
}

impl Add for TxAmount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.inner + rhs.inner)
    }
}

impl AddAssign for TxAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner
    }
}

impl Sub for TxAmount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.inner - rhs.inner)
    }
}

impl SubAssign for TxAmount {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner -= rhs.inner
    }
}

impl Neg for TxAmount {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.inner)
    }
}

impl From<(&str,&str)> for TxAmount {

    fn from((tx_amount1, tx_amount2): (&str, &str)) -> Self {
        let precision = u32::try_from(tx_amount2.len()).unwrap();
        let tx_amount1 = isize::from_str_radix(tx_amount1, 10).unwrap();
        let tx_amount2 = isize::from_str_radix(tx_amount2, 10).unwrap();

        TxAmount::new(tx_amount1 * 10isize.pow(4) + tx_amount2 * 10isize.pow(4 - precision))
    }

}

impl Debug for TxAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Display for TxAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let integral = self.inner / PRECISION_FACTOR;
        let fractional = (self.inner % PRECISION_FACTOR).abs();

        write!(f, "{}.{:04}", integral, fractional) 
    }
}