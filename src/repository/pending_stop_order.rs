use crate::{Asset, OrderId, Price, Quantity, StopLimitOrder, StopOrder};

pub enum PendingStopOrder<ID: OrderId, A: Asset, P: Price, Q: Quantity> {
    StopOrder(StopOrder<ID, A, P, Q>),
    StopLimitOrder(StopLimitOrder<ID, A, P, Q>),
}

pub trait PendingStopOrderRepository: Send {
    type Err;
    type Asset: Asset;
    type OrderId: OrderId;
    type Price: Price;
    type Quantity: Quantity;
    type Transaction;

    fn create(
        &self,
        tx: &mut Self::Transaction,
        order: &PendingStopOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<(), Self::Err>;

    fn update(
        &self,
        tx: &mut Self::Transaction,
        order: &PendingStopOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<(), Self::Err>;

    fn delete(
        &self,
        tx: &mut Self::Transaction,
        order: &PendingStopOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<bool, Self::Err>;

    fn delete_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> Result<bool, Self::Err>;

    fn get_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> Result<
        Option<PendingStopOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>>,
        Self::Err,
    >;

    fn get_list_by_market_price(
        &self,
        tx: &mut Self::Transaction,
        market_price: &Self::Price,
        limit: i64,
    ) -> Result<
        Vec<PendingStopOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>>,
        Self::Err,
    >;
}
