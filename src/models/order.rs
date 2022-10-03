use super::{Asset, AssetPair, OrderId, OrderSide, Price, Quantity};
use crate::OrderType;
use std::time::SystemTime;

#[derive(Debug)]
pub enum Order<A: Asset> {
    Market {
        id: OrderId,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        quantity: Quantity,
        timestamp: SystemTime,
    },
    Limit {
        id: OrderId,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },
    Amend {
        id: OrderId,
        target_id: OrderId,
        target_order_type: OrderType,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },
    Cancel {
        id: OrderId,
        target_id: OrderId,
        target_order_type: OrderType,
        side: OrderSide,
    },
}

impl<A: Asset> Order<A> {
    pub fn new_market(
        id: OrderId,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Self {
        Self::Market {
            id,
            asset_pair,
            side,
            quantity,
            timestamp,
        }
    }

    pub fn new_limit(
        id: OrderId,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Self {
        Self::Limit {
            id,
            asset_pair,
            side,
            price,
            quantity,
            timestamp,
        }
    }

    pub fn new_amend(
        id: OrderId,
        target_id: OrderId,
        target_order_type: OrderType,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Self {
        Self::Amend {
            id,
            target_id,
            target_order_type,
            side,
            price,
            quantity,
            timestamp,
        }
    }

    pub fn new_cancel(
        id: OrderId,
        target_id: OrderId,
        target_order_type: OrderType,
        side: OrderSide,
    ) -> Self {
        Self::Cancel {
            id,
            target_id,
            target_order_type,
            side,
        }
    }
}
