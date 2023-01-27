#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    // LimitMaker,
}
