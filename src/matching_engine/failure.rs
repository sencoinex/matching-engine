use crate::OrderId;

#[derive(Debug)]
pub enum MatchingEngineFailure<ID: OrderId> {
    OrderNotFound { order_id: ID, target_order_id: ID },
    FailedToEnqueueOrder(ID),
    NoMatch(ID),
}
