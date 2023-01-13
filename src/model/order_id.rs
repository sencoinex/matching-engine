use core::fmt::{Debug, Display};
use core::hash::Hash;

pub trait OrderId:
    PartialOrd + Ord + PartialEq + Eq + Hash + Clone + Copy + Debug + Display
{
}

impl<T> OrderId for T where
    T: PartialOrd + Ord + PartialEq + Eq + Hash + Clone + Copy + Debug + Display
{
}
