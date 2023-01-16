use super::order::{AmendOrder, CancelOrder, LimitOrder, MarketOrder};
use super::{Asset, AssetPair, OrderId, OrderSide, OrderType, Price, Quantity};

#[derive(Debug)]
pub enum OrderRequest<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    Market(MarketOrder<ID, A, Q>),
    Limit(LimitOrder<ID, A, P, Q>),
    Amend(AmendOrder<ID, A, P, Q>),
    Cancel(CancelOrder<ID, A>),
}

impl<ID: OrderId, A: Asset, P: Price, Q: Quantity> OrderRequest<ID, A, P, Q> {
    pub fn new_market(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        quantity: Q,
        timestamp_ms: u64,
    ) -> Self {
        Self::Market(MarketOrder {
            id,
            asset_pair,
            side,
            quantity,
            timestamp_ms,
        })
    }

    pub fn new_limit(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        price: P,
        quantity: Q,
        timestamp_ms: u64,
    ) -> Self {
        Self::Limit(LimitOrder {
            id,
            asset_pair,
            side,
            price,
            quantity,
            timestamp_ms,
        })
    }

    pub fn new_amend(
        id: ID,
        asset_pair: AssetPair<A>,
        target_id: ID,
        target_order_type: OrderType,
        side: OrderSide,
        price: P,
        quantity: Q,
        timestamp_ms: u64,
    ) -> Self {
        Self::Amend(AmendOrder {
            id,
            asset_pair,
            target_id,
            target_order_type,
            side,
            price,
            quantity,
            timestamp_ms,
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
