use super::{Asset, AssetPair, OrderId, OrderSide, OrderType, Price, Quantity};
use std::time::SystemTime;

#[derive(Debug)]
pub struct MarketOrder<ID: OrderId, A: Asset> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub quantity: Quantity,
    pub timestamp: SystemTime,
}

impl<ID: OrderId, A: Asset> MarketOrder<ID, A> {
    pub fn sub_quantity(&self, sub: Quantity) -> Self {
        Self {
            id: self.id,
            asset_pair: self.asset_pair.clone(),
            side: self.side,
            quantity: self.quantity - sub,
            timestamp: self.timestamp,
        }
    }
}

#[derive(Debug)]
pub struct LimitOrder<ID: OrderId, A: Asset> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: SystemTime,
}

impl<ID: OrderId, A: Asset> LimitOrder<ID, A> {
    pub fn sub_quantity(&self, sub: Quantity) -> Self {
        Self {
            id: self.id,
            asset_pair: self.asset_pair.clone(),
            side: self.side,
            price: self.price,
            quantity: self.quantity - sub,
            timestamp: self.timestamp,
        }
    }
}

#[derive(Debug)]
pub struct AmendOrder<ID: OrderId, A: Asset> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub target_id: ID,
    pub target_order_type: OrderType,
    pub side: OrderSide,
    pub price: Price,
    pub quantity: Quantity,
    pub timestamp: SystemTime,
}

#[derive(Debug)]
pub struct CancelOrder<ID: OrderId, A: Asset> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub target_id: ID,
    pub target_order_type: OrderType,
    pub side: OrderSide,
}
