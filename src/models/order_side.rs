#[derive(Debug, Copy, Clone)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn opposite(&self) -> Self {
        match *self {
            Self::Bid => Self::Ask,
            Self::Ask => Self::Bid,
        }
    }
}
