use core::fmt::{Debug, Display};
use std::ops::Sub;

pub trait Quantity:
    PartialOrd + Ord + PartialEq + Eq + Sub<Output = Self> + Clone + Copy + Debug + Display
{
}

impl<T> Quantity for T where
    T: PartialOrd + Ord + PartialEq + Eq + Sub<Output = Self> + Clone + Copy + Debug + Display
{
}
