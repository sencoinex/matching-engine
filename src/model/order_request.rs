use super::{Asset, AssetPair, OrderId, OrderSide, OrderType, Price, Quantity, TimeInForce};

#[derive(Debug)]
pub enum OrderRequest<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    Market(MarketOrderRequest<ID, A, Q>),
    Limit(LimitOrderRequest<ID, A, P, Q>),

    /// If the market reaches the stop price, this order becomes a market order.
    /// * Buy:
    ///   * stop_price > market price ... regarded as stop loss order
    ///   * stop_price < market_price ... regarded as take profit order
    /// * Sell:
    ///   * stop_price < market_price ... regarded as stop loss order
    ///   * stop_price > market_price ... regarded as take profit order
    Stop(StopOrderRequest<ID, A, P, Q>),

    /// If the market reaches the stop price, this order becomes a limit order.
    StopLimit(StopLimitOrderRequest<ID, A, P, Q>),

    Amend(AmendOrderRequest<ID, A, P, Q>),
    Cancel(CancelOrderRequest<ID, A>),
}

#[derive(Debug)]
pub struct MarketOrderRequest<ID: OrderId, A: Asset, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub quantity: Q,
}

#[derive(Debug)]
pub struct LimitOrderRequest<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    pub time_in_force: TimeInForce,
    pub price: P,
    pub quantity: Q,
}

#[derive(Debug)]
pub struct StopOrderRequest<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    /// trigger price i.e. If the market reaches this stop price, this order becomes a market order.
    pub stop_price: P,
    pub quantity: Q,
}

#[derive(Debug)]
pub struct StopLimitOrderRequest<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub side: OrderSide,
    /// trigger price i.e. If the market reaches this stop price, this order becomes a limit order.
    pub stop_price: P,
    pub time_in_force: TimeInForce,
    pub price: P,
    pub quantity: Q,
}

#[derive(Debug)]
pub struct AmendOrderRequest<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub target_id: ID,
    pub target_order_type: OrderType,
    pub side: OrderSide,
    pub price: P,
    pub quantity: Q,
}

#[derive(Debug)]
pub struct CancelOrderRequest<ID: OrderId, A: Asset> {
    pub id: ID,
    pub asset_pair: AssetPair<A>,
    pub target_id: ID,
    pub target_order_type: OrderType,
    pub side: OrderSide,
}
