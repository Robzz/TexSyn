use num_traits::Float;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::ops::{Add, AddAssign};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct OrderedFloat<F> where F: Float {
    val: F
}

impl<F> OrderedFloat<F> where F: Float {
    pub fn as_float(&self) -> F { self.val }
}

impl<F> TryFrom<F> for OrderedFloat<F> where F: Float {
    type Err = ();
    fn try_from(val: F) -> Result<OrderedFloat<F>, Self::Err> {
        if val.is_nan() { Err(()) }
        else { Ok(OrderedFloat { val: val }) }
    }
}

impl<F> Eq for OrderedFloat<F> where F: Float { }

impl<F> Ord for OrderedFloat<F> where F: Float {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.val.is_nan(), other.val.is_nan()) {
            (false, false) => self.partial_cmp(other).unwrap(),
            _ => panic!("OrderedFloat is NaN")
        }
    }
}

impl<F> Add for OrderedFloat<F> where F: Float {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        OrderedFloat { val: self.val + other.val }
    }
}

impl<F> AddAssign for OrderedFloat<F> where F: Float {
    fn add_assign(&mut self, other: Self) {
        self.val = self.val + other.val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn test_ordered_float_tryfrom() {
        let f = 72.;
        let of: OrderedFloat<_> = f.try_into().unwrap();
        assert!(of.val == f);
    }
}
