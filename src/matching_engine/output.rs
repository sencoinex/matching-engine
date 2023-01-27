use crate::{OrderId, OrderSide, OrderType, Price, Quantity};

#[derive(Debug)]
pub struct MatchingEngineOutput<ID: OrderId, P: Price, Q: Quantity> {
    pub market_price: Option<P>,
    pub events: Vec<Result<MatchingEngineEvent<ID, P, Q>, MatchingEngineFailure<ID>>>,
}

impl<ID: OrderId, P: Price, Q: Quantity> MatchingEngineOutput<ID, P, Q> {
    pub fn new(market_price: Option<P>) -> Self {
        Self {
            market_price,
            events: vec![],
        }
    }

    pub fn set_market_price(&mut self, price: P) {
        self.market_price = Some(price);
    }

    pub fn push(
        &mut self,
        event: Result<MatchingEngineEvent<ID, P, Q>, MatchingEngineFailure<ID>>,
    ) {
        self.events.push(event)
    }
}

#[derive(Debug)]
pub enum MatchingEngineEvent<ID: OrderId, P: Price, Q: Quantity> {
    Accepted {
        id: ID,
        order_type: OrderType,
        timestamp_ms: u64,
    },

    Filled {
        id: ID,
        side: OrderSide,
        order_type: OrderType,
        price: P,
        quantity: Q,
        timestamp_ms: u64,
    },

    PartiallyFilled {
        id: ID,
        side: OrderSide,
        order_type: OrderType,
        price: P,
        quantity: Q,
        timestamp_ms: u64,
    },

    Amended {
        id: ID,
        target_id: ID,
        price: P,
        quantity: Q,
        timestamp_ms: u64,
    },

    Cancelled {
        id: ID,
        target_id: ID,
        timestamp_ms: u64,
    },

    StopOrderIssueMarketOrder {
        id: ID,
        timestamp_ms: u64,
    },

    StopLimitOrderIssueLimitOrder {
        id: ID,
        timestamp_ms: u64,
    },
}

#[derive(Debug)]
pub enum MatchingEngineFailure<ID: OrderId> {
    OrderNotFound { order_id: ID, target_order_id: ID },
    NoMatch(ID),
    MissingMarketPriceForStopOrder(ID),
}
