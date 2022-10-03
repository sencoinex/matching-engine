#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OrderId(u64);

impl OrderId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl AsRef<u64> for OrderId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Into<u64> for OrderId {
    fn into(self) -> u64 {
        self.0
    }
}

impl From<u64> for OrderId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
