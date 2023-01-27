use super::{Asset, AssetPair, OrderId, OrderSide, Price, Quantity, TimeInForce};

#[derive(Debug)]
pub struct MarketOrder<ID: OrderId, A: Asset, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub quantity: Q,
    pub timestamp_ms: u64,
}

impl<ID: OrderId, A: Asset, Q: Quantity> MarketOrder<ID, A, Q> {
    pub fn new(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        quantity: Q,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            id,
            asset_pair,
            side,
            quantity,
            timestamp_ms,
        }
    }

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
    pub time_in_force: TimeInForce,
    pub price: P,
    pub quantity: Q,
    pub timestamp_ms: u64,
}

impl<ID: OrderId, A: Asset, P: Price, Q: Quantity> LimitOrder<ID, A, P, Q> {
    pub fn new(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        time_in_force: TimeInForce,
        price: P,
        quantity: Q,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            id,
            asset_pair,
            side,
            time_in_force,
            price,
            quantity,
            timestamp_ms,
        }
    }

    pub fn sub_quantity(&self, sub: Q) -> Self {
        Self {
            id: self.id,
            asset_pair: self.asset_pair.clone(),
            side: self.side,
            time_in_force: self.time_in_force,
            price: self.price,
            quantity: self.quantity - sub,
            timestamp_ms: self.timestamp_ms,
        }
    }
}

#[derive(Debug)]
pub struct StopOrder<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub stop_price: P,
    pub quantity: Q,
    pub timestamp_ms: u64,
}

impl<ID: OrderId, A: Asset, P: Price, Q: Quantity> StopOrder<ID, A, P, Q> {
    pub fn new(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        stop_price: P,
        quantity: Q,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            id,
            asset_pair,
            side,
            stop_price,
            quantity,
            timestamp_ms,
        }
    }

    pub fn issue_market_order(&self, timestamp_ms: u64) -> MarketOrder<ID, A, Q> {
        MarketOrder::new(
            self.id,
            self.asset_pair.clone(),
            self.side,
            self.quantity,
            timestamp_ms,
        )
    }
}

#[derive(Debug)]
pub struct StopLimitOrder<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub stop_price: P,
    pub time_in_force: TimeInForce,
    pub price: P,
    pub quantity: Q,
    pub timestamp_ms: u64,
}

impl<ID: OrderId, A: Asset, P: Price, Q: Quantity> StopLimitOrder<ID, A, P, Q> {
    pub fn new(
        id: ID,
        asset_pair: AssetPair<A>,
        side: OrderSide,
        stop_price: P,
        time_in_force: TimeInForce,
        price: P,
        quantity: Q,
        timestamp_ms: u64,
    ) -> Self {
        Self {
            id,
            asset_pair,
            side,
            stop_price,
            time_in_force,
            price,
            quantity,
            timestamp_ms,
        }
    }

    pub fn issue_limit_order(&self, timestamp_ms: u64) -> LimitOrder<ID, A, P, Q> {
        LimitOrder::new(
            self.id,
            self.asset_pair.clone(),
            self.side,
            self.time_in_force,
            self.price,
            self.quantity,
            timestamp_ms,
        )
    }
}
