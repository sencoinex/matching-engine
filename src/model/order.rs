use super::{Asset, AssetPair, OrderId, OrderSide, OrderType, Price, Quantity};

#[derive(Debug)]
pub struct MarketOrder<ID: OrderId, A: Asset, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub quantity: Q,
    pub timestamp_ms: u64,
}

impl<ID: OrderId, A: Asset, Q: Quantity> MarketOrder<ID, A, Q> {
    pub fn sub_quantity(&self, sub: Q) -> Self {
        Self {
            id: self.id,
            asset_pair: self.asset_pair.clone(),
            side: self.side,
            quantity: self.quantity - sub,
            timestamp_ms: self.timestamp_ms,
        }
    }
}

#[derive(Debug)]
pub struct LimitOrder<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub price: P,
    pub quantity: Q,
    pub timestamp_ms: u64,
}

impl<ID: OrderId, A: Asset, P: Price, Q: Quantity> LimitOrder<ID, A, P, Q> {
    pub fn sub_quantity(&self, sub: Q) -> Self {
        Self {
            id: self.id,
            asset_pair: self.asset_pair.clone(),
            side: self.side,
            price: self.price,
            quantity: self.quantity - sub,
            timestamp_ms: self.timestamp_ms,
        }
    }
}

#[derive(Debug)]
pub struct AmendOrder<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub target_id: ID,
    pub target_order_type: OrderType,
    pub side: OrderSide,
    pub price: P,
    pub quantity: Q,
    pub timestamp_ms: u64,
}

#[derive(Debug)]
pub struct CancelOrder<ID: OrderId, A: Asset> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub target_id: ID,
    pub target_order_type: OrderType,
    pub side: OrderSide,
}
