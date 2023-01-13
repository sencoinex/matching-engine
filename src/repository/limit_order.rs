use crate::{Asset, LimitOrder, OrderId};

pub trait LimitOrderRepositoryLike: Send {
    type Err;
    type Asset: Asset;
    type OrderId: OrderId;
    type Transaction;

    fn create(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err>;

    fn update(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err>;

    fn delete_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> Result<(), Self::Err>;

    fn get_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> Result<Option<LimitOrder<Self::OrderId, Self::Asset>>, Self::Err>;

    fn next(
        &self,
        tx: &mut Self::Transaction,
    ) -> Result<Option<LimitOrder<Self::OrderId, Self::Asset>>, Self::Err>;
}
