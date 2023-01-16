use core::fmt::{Debug, Display};

pub trait Price: PartialOrd + Ord + PartialEq + Eq + Clone + Copy + Debug + Display {}

impl<T> Price for T where T: PartialOrd + Ord + PartialEq + Eq + Clone + Copy + Debug + Display {}
