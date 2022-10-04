use crate::OrderId;

#[derive(Debug)]
pub enum MatchingEngineFailure {
    OrderNotFound {
        order_id: OrderId,
        target_order_id: OrderId,
    },
    FailedToEnqueueOrder(OrderId),
    NoMatch(OrderId),
}
