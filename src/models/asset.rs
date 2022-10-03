use core::fmt;

pub trait Asset: Eq + Clone + fmt::Debug + fmt::Display {}
impl<T> Asset for T where T: Eq + Clone + fmt::Debug + fmt::Display {}
