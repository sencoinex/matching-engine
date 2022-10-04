use crate::{OrderId, OrderSide, OrderType, Price, Quantity};
use std::time::SystemTime;

#[derive(Debug)]
pub enum MatchingEngineOutput {
    Accepted {
        id: OrderId,
        order_type: OrderType,
        timestamp: SystemTime,
    },

    Filled {
        id: OrderId,
        side: OrderSide,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },

    PartiallyFilled {
        id: OrderId,
        side: OrderSide,
        order_type: OrderType,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },

    Amended {
        id: OrderId,
        target_id: OrderId,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    },

    Cancelled {
        id: OrderId,
        target_id: OrderId,
        timestamp: SystemTime,
    },
}
