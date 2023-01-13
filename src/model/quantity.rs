use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Copy, Clone, Eq)]
pub struct Quantity {
    pub num: u64,
    pub scale: u32,
}

impl Quantity {
    pub fn new(num: u64, scale: u32) -> Self {
        Self { num, scale }
    }
}

impl PartialEq for Quantity {
    fn eq(&self, other: &Self) -> bool {
        self.scale == other.scale && self.num == other.num
    }
}

impl Ord for Quantity {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.scale, other.scale);
        self.num.cmp(&other.num)
    }
}

impl PartialOrd for Quantity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Quantity {
    type Output = Quantity;
    fn add(self, other: Self) -> Self {
        assert_eq!(self.scale, other.scale);
        Self {
            num: self.num + other.num,
            scale: self.scale,
        }
    }
}

impl AddAssign for Quantity {
    fn add_assign(&mut self, other: Self) {
        assert_eq!(self.scale, other.scale);
        self.num.add_assign(other.num)
    }
}

impl Sub for Quantity {
    type Output = Quantity;
    fn sub(self, other: Self) -> Self {
        assert_eq!(self.scale, other.scale);
        Self {
            num: self.num - other.num,
            scale: self.scale,
        }
    }
}

impl SubAssign for Quantity {
    fn sub_assign(&mut self, other: Self) {
        assert_eq!(self.scale, other.scale);
        self.num.sub_assign(other.num)
    }
}
