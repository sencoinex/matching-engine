use crate::{OrderId, Price, Quantity};

#[derive(Debug, Clone)]
pub struct MarketOrder {
    pub order_id: OrderId,
    pub quantity: Quantity,
}

#[derive(Debug, Clone)]
pub struct LimitOrder {
    pub order_id: OrderId,
    pub price: Price,
    pub quantity: Quantity,
}
