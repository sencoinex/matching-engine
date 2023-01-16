use crate::{OrderId, OrderSide, OrderType, Price, Quantity};

#[derive(Debug)]
pub enum MatchingEngineOutput<ID: OrderId, P: Price, Q: Quantity> {
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
}
