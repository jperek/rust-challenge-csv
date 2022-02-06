use std::{fmt, ops::{Add, Sub}};

type ClientId = u16;
type TransactionId = u32;

type UnderlyingAmountType = u64;

#[derive(Debug, PartialEq, Eq)]
pub struct Amount {
    value: UnderlyingAmountType,
}

const DECIMAL_PLACES: u32 = 4;
const AMOUNT_ONE: UnderlyingAmountType = (10 as UnderlyingAmountType).pow(DECIMAL_PLACES);

impl Amount {
    pub fn new(value: UnderlyingAmountType) -> Self {
        Self {
            value,
        }
    }

    pub fn trunc(&self) -> UnderlyingAmountType {
        self.trunc_fract().0
    }

    pub fn fract(&self) -> UnderlyingAmountType {
        self.trunc_fract().1
    }

    fn trunc_fract(&self) -> (UnderlyingAmountType, UnderlyingAmountType) {
        let trunc = self.value / AMOUNT_ONE;
        let xx = trunc * AMOUNT_ONE;
        let fract = self.value - xx;
        debug_assert!(fract < AMOUNT_ONE);
        (trunc, fract)
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.value + other.value)
    }
}

impl Sub for Amount {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        if self.value < other.value {
            panic!("tried to subtract {} from {}", self, other);
        }
        Self::new(self.value - other.value)
    }
}

fn count_remove_trailing_zeroes(mut value: UnderlyingAmountType) -> (usize, UnderlyingAmountType) {
    let mut count = 0;
    if value > 0 {
        while value % 10 == 0 {
            value /= 10;
            count += 1;
        }
    }
   (count, value)
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (trunc, fract) = self.trunc_fract();
        if fract == 0 {
            write!(f, "{}", trunc)
        } else {
            let (count, fract) = count_remove_trailing_zeroes(fract);
            let width = DECIMAL_PLACES as usize - count;
            write!(f, "{}.{:0>width$}", trunc, fract, width = width)
        }
    } 
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn formatting() {
        assert_eq!(format!("{}", Amount::new(0)), "0");
        assert_eq!(format!("{}", Amount::new(10000)), "1");
        assert_eq!(format!("{}", Amount::new(11000)), "1.1");
        assert_eq!(format!("{}", Amount::new(10100)), "1.01");
        assert_eq!(format!("{}", Amount::new(10010)), "1.001");
        assert_eq!(format!("{}", Amount::new(10001)), "1.0001");
        assert_eq!(format!("{}", Amount::new(11100)), "1.11");
        assert_eq!(format!("{}", Amount::new(10110)), "1.011");
        assert_eq!(format!("{}", Amount::new(10011)), "1.0011");
        assert_eq!(format!("{}", Amount::new(11110)), "1.111");
        assert_eq!(format!("{}", Amount::new(10111)), "1.0111");
        assert_eq!(format!("{}", Amount::new(11111)), "1.1111");
        assert_eq!(format!("{}", Amount::new(9999990000)), "999999");
        assert_eq!(format!("{}", Amount::new(9999990100)), "999999.01");
        assert_eq!(format!("{}", Amount::new(UnderlyingAmountType::MAX)), "1844674407370955.1615");
    }

    #[test]
    fn adding() {
        assert_eq!(Amount::new(11111) + Amount::new(0), Amount::new(11111));
        assert_eq!(Amount::new(0) + Amount::new(11111), Amount::new(11111));
        assert_eq!(Amount::new(11111) + Amount::new(11111), Amount::new(22222));
    }

    #[test]
    fn subtracting() {
        assert_eq!(Amount::new(11111) - Amount::new(0), Amount::new(11111));
        assert_eq!(Amount::new(22222) - Amount::new(11111), Amount::new(11111));
        assert_eq!(Amount::new(10001) - Amount::new(10000), Amount::new(1));
        assert_eq!(Amount::new(10001) - Amount::new(10001), Amount::new(0));
    }

    #[test]
    #[should_panic]
    fn subtracting_overflow() {
        let _ = Amount::new(10000) - Amount::new(10001);
    }

    #[test]
    fn counting_and_removing_trailing_zeroes() {
        assert_eq!(count_remove_trailing_zeroes(0), (0, 0));
        assert_eq!(count_remove_trailing_zeroes(1), (0, 1));
        assert_eq!(count_remove_trailing_zeroes(9), (0, 9));
        assert_eq!(count_remove_trailing_zeroes(10), (1, 1));
        assert_eq!(count_remove_trailing_zeroes(90), (1, 9));
        assert_eq!(count_remove_trailing_zeroes(100), (2, 1));
        assert_eq!(count_remove_trailing_zeroes(5000), (3, 5));
        assert_eq!(count_remove_trailing_zeroes(900090), (1, 90009)); 
        assert_eq!(count_remove_trailing_zeroes(50000000000), (10, 5));
    }
}