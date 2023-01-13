use super::order::{AmendOrder, CancelOrder, LimitOrder, MarketOrder};
use super::{Asset, AssetPair, OrderId, OrderSide, OrderType, Price, Quantity};
use std::time::SystemTime;

#[derive(Debug)]
pub enum OrderRequest<ID: OrderId, A: Asset> {
    Market(MarketOrder<ID, A>),
    Limit(LimitOrder<ID, A>),
    Amend(AmendOrder<ID, A>),
    Cancel(CancelOrder<ID, A>),
}

impl<ID: OrderId, A: Asset> OrderRequest<ID, A> {
    pub fn new_market(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Self {
        Self::Market(MarketOrder {
            id,
            asset_pair,
            side,
            quantity,
            timestamp,
        })
    }

    pub fn new_limit(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Self {
        Self::Limit(LimitOrder {
            id,
            asset_pair,
            side,
            price,
            quantity,
            timestamp,
        })
    }

    pub fn new_amend(
        id: ID,
        asset_pair: AssetPair<A>,
        target_id: ID,
        target_order_type: OrderType,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) -> Self {
        Self::Amend(AmendOrder {
            id,
            asset_pair,
            target_id,
            target_order_type,
            side,
            price,
            quantity,
            timestamp,
        })
    }

    pub fn new_cancel(
        id: ID,
        asset_pair: AssetPair<A>,
        target_id: ID,
        target_order_type: OrderType,
        side: OrderSide,
    ) -> Self {
        Self::Cancel(CancelOrder {
            id,
            asset_pair,
            target_id,
            target_order_type,
            side,
        })
    }
}
