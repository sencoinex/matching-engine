use crate::{OrderId, OrderSide, OrderType, Price, Quantity};
use std::time::SystemTime;

#[derive(Debug)]
pub enum MatchingEngineOutput<ID: OrderId> {
    Accepted {
        id: ID,
        order_type: OrderType,
        timestamp: SystemTime,
    },

    Filled {
        id: ID,
        side: OrderSide,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },

    PartiallyFilled {
        id: ID,
        side: OrderSide,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },

    Amended {
        id: ID,
        target_id: ID,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },

    Cancelled {
        id: ID,
        target_id: ID,
        timestamp: SystemTime,
    },
}
