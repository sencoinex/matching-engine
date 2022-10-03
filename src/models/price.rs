use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Eq)]
pub struct Price {
    pub num: u64,
    pub scale: u32,
}

impl PartialEq for Price {
    fn eq(&self, other: &Self) -> bool {
        self.scale == other.scale && self.num == other.num
    }
}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(self.scale, other.scale);
        self.num.cmp(&other.num)
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
