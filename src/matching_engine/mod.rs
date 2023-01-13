mod failure;
mod output;

pub use failure::*;
pub use output::*;

pub type OrderProcessingResult<ID> =
    Vec<Result<MatchingEngineOutput<ID>, MatchingEngineFailure<ID>>>;

use crate::{
    model::{
        AmendOrder, Asset, AssetPair, CancelOrder, LimitOrder, MarketOrder, OrderId, OrderRequest,
        OrderSide, OrderType,
    },
    repository::LimitOrderRepositoryLike,
};
use std::time::SystemTime;

pub trait MatchingEngine {
    type Err;
    type Asset: Asset;
    type OrderId: OrderId;
    type Transaction;
    type BidLimitOrderRepository: LimitOrderRepositoryLike<
        Err = Self::Err,
        Asset = Self::Asset,
        OrderId = Self::OrderId,
        Transaction = Self::Transaction,
    >;
    type AskLimitOrderRepository: LimitOrderRepositoryLike<
        Err = Self::Err,
        Asset = Self::Asset,
        OrderId = Self::OrderId,
        Transaction = Self::Transaction,
    >;

    fn asset_pair(&self) -> &AssetPair<Self::Asset>;
    fn bid_limit_order_repository(&self) -> &Self::BidLimitOrderRepository;
    fn ask_limit_order_repository(&self) -> &Self::AskLimitOrderRepository;

    fn process_order(
        &mut self,
        tx: &mut Self::Transaction,
        order_request: OrderRequest<Self::OrderId, Self::Asset>,
    ) -> Result<OrderProcessingResult<Self::OrderId>, Self::Err> {
        let mut proc_result: OrderProcessingResult<Self::OrderId> = vec![];
        match order_request {
            OrderRequest::Market(market_order) => {
                assert_eq!(*self.asset_pair(), market_order.asset_pair);
                proc_result.push(Ok(MatchingEngineOutput::Accepted {
                    id: market_order.id,
                    order_type: OrderType::Market,
                    timestamp: SystemTime::now(),
                }));
                self.process_market_order(tx, &mut proc_result, &market_order)?;
            }
            OrderRequest::Limit(limit_order) => {
                assert_eq!(*self.asset_pair(), limit_order.asset_pair);
                proc_result.push(Ok(MatchingEngineOutput::Accepted {
                    id: limit_order.id,
                    order_type: OrderType::Limit,
                    timestamp: SystemTime::now(),
                }));
                self.process_limit_order(tx, &mut proc_result, &limit_order)?;
            }
            OrderRequest::Amend(amend_order) => {
                let is_amendable = match amend_order.target_order_type {
                    OrderType::Market => false,
                    OrderType::Limit => true,
                };
                assert!(is_amendable);
                self.process_amend_order(tx, &mut proc_result, &amend_order)?;
            }
            OrderRequest::Cancel(cancel_order) => {
                let is_cancelable = match cancel_order.target_order_type {
                    OrderType::Market => false,
                    OrderType::Limit => true,
                };
                assert!(is_cancelable);
                self.process_cancel_order(tx, &mut proc_result, &cancel_order)?;
            }
        }
        Ok(proc_result)
    }

    fn process_market_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId>,
        market_order: &MarketOrder<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err> {
        let opposite_order = match market_order.side {
            OrderSide::Bid => self.ask_limit_order_repository().next(tx),
            OrderSide::Ask => self.bid_limit_order_repository().next(tx),
        }?;
        if let Some(opposite_order) = opposite_order {
            let matching_complete = self.match_market_order_with_limit_order(
                tx,
                results,
                &market_order,
                &opposite_order,
            )?;
            let next_market_order = market_order.sub_quantity(opposite_order.quantity);
            if !matching_complete {
                self.process_market_order(tx, results, &next_market_order)?;
            }
        } else {
            results.push(Err(MatchingEngineFailure::NoMatch(market_order.id)));
        }
        Ok(())
    }

    fn process_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId>,
        limit_order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err> {
        let opposite_order = match limit_order.side {
            OrderSide::Bid => self.ask_limit_order_repository().next(tx),
            OrderSide::Ask => self.bid_limit_order_repository().next(tx),
        }?;
        if let Some(opposite_order) = opposite_order {
            let could_be_matched = match limit_order.side {
                // verify bid/ask price overlap
                OrderSide::Bid => limit_order.price >= opposite_order.price,
                OrderSide::Ask => limit_order.price <= opposite_order.price,
            };
            if could_be_matched {
                let matching_complete = self.match_limit_order_with_limit_order(
                    tx,
                    results,
                    &limit_order,
                    &opposite_order,
                )?;
                if !matching_complete {
                    let next_limit_order = limit_order.sub_quantity(opposite_order.quantity);
                    self.process_limit_order(tx, results, &next_limit_order)?;
                }
            } else {
                self.store_new_limit_order(tx, results, &limit_order)?;
            }
        } else {
            self.store_new_limit_order(tx, results, &limit_order)?;
        }
        Ok(())
    }

    fn process_amend_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId>,
        amend_order: &AmendOrder<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err> {
        match amend_order.target_order_type {
            OrderType::Limit => {
                let order = match amend_order.side {
                    OrderSide::Bid => self
                        .bid_limit_order_repository()
                        .get_by_order_id(tx, &amend_order.target_id),
                    OrderSide::Ask => self
                        .ask_limit_order_repository()
                        .get_by_order_id(tx, &amend_order.target_id),
                }?;
                if let Some(mut target_order) = order {
                    target_order.price = amend_order.price;
                    target_order.quantity = amend_order.quantity;
                    target_order.timestamp = amend_order.timestamp;
                    match amend_order.side {
                        OrderSide::Bid => {
                            self.bid_limit_order_repository().update(tx, &target_order)
                        }
                        OrderSide::Ask => {
                            self.ask_limit_order_repository().update(tx, &target_order)
                        }
                    }?;
                    results.push(Ok(MatchingEngineOutput::Amended {
                        id: amend_order.id,
                        target_id: amend_order.target_id,
                        price: amend_order.price,
                        quantity: amend_order.quantity,
                        timestamp: SystemTime::now(),
                    }));
                    // TODO process limit order?
                } else {
                    results.push(Err(MatchingEngineFailure::OrderNotFound {
                        order_id: amend_order.id,
                        target_order_id: amend_order.target_id,
                    }));
                }
            }
            _ => { /* ignore */ }
        }
        Ok(())
    }

    fn process_cancel_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId>,
        cancel_order: &CancelOrder<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err> {
        match cancel_order.target_order_type {
            OrderType::Limit => {
                match cancel_order.side {
                    OrderSide::Bid => self
                        .bid_limit_order_repository()
                        .delete_by_order_id(tx, &cancel_order.target_id),
                    OrderSide::Ask => self
                        .ask_limit_order_repository()
                        .delete_by_order_id(tx, &cancel_order.target_id),
                }?;
                results.push(Ok(MatchingEngineOutput::Cancelled {
                    id: cancel_order.id,
                    target_id: cancel_order.target_id,
                    timestamp: SystemTime::now(),
                }));
            }
            _ => { /* ignore */ }
        }
        Ok(())
    }

    fn store_new_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        _results: &mut OrderProcessingResult<Self::OrderId>,
        limit_order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err> {
        match limit_order.side {
            OrderSide::Bid => self.bid_limit_order_repository().create(tx, limit_order),
            OrderSide::Ask => self.ask_limit_order_repository().create(tx, limit_order),
        }
    }

    fn match_market_order_with_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId>,
        order: &MarketOrder<Self::OrderId, Self::Asset>,
        opposite_order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> Result<bool, Self::Err> {
        let deal_time = SystemTime::now();
        if order.quantity < opposite_order.quantity {
            // market order: fully filled / limit order: partially filled
            results.push(Ok(MatchingEngineOutput::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(MatchingEngineOutput::PartiallyFilled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));

            // modify unmatched part of the opposite limit order
            let new_limit_order = opposite_order.sub_quantity(order.quantity);
            match new_limit_order.side {
                OrderSide::Bid => self
                    .bid_limit_order_repository()
                    .update(tx, &new_limit_order),
                OrderSide::Ask => self
                    .ask_limit_order_repository()
                    .update(tx, &new_limit_order),
            }?;
            Ok(true)
        } else if order.quantity > opposite_order.quantity {
            // market order: partially filled / limit order: fully filled
            results.push(Ok(MatchingEngineOutput::PartiallyFilled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(MatchingEngineOutput::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match opposite_order.side {
                OrderSide::Bid => self
                    .bid_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
                OrderSide::Ask => self
                    .ask_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
            }?;
            Ok(false)
        } else {
            // exact match
            results.push(Ok(MatchingEngineOutput::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(MatchingEngineOutput::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match opposite_order.side {
                OrderSide::Bid => self
                    .bid_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
                OrderSide::Ask => self
                    .ask_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
            }?;
            Ok(true)
        }
    }

    fn match_limit_order_with_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId>,
        order: &LimitOrder<Self::OrderId, Self::Asset>,
        opposite_order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> Result<bool, Self::Err> {
        let deal_time = SystemTime::now();
        if order.quantity < opposite_order.quantity {
            // limit order: fully filled / limit order: partially filled
            results.push(Ok(MatchingEngineOutput::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(MatchingEngineOutput::PartiallyFilled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));

            // modify unmatched part of the opposite limit order
            let new_limit_order = opposite_order.sub_quantity(order.quantity);
            match new_limit_order.side {
                OrderSide::Bid => self
                    .bid_limit_order_repository()
                    .update(tx, &new_limit_order),
                OrderSide::Ask => self
                    .ask_limit_order_repository()
                    .update(tx, &new_limit_order),
            }?;
            Ok(true)
        } else if order.quantity > opposite_order.quantity {
            // market order: partially filled / limit order: fully filled
            results.push(Ok(MatchingEngineOutput::PartiallyFilled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(MatchingEngineOutput::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match opposite_order.side {
                OrderSide::Bid => self
                    .bid_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
                OrderSide::Ask => self
                    .ask_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
            }?;
            Ok(false)
        } else {
            // exact match
            results.push(Ok(MatchingEngineOutput::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(MatchingEngineOutput::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match opposite_order.side {
                OrderSide::Bid => self
                    .bid_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
                OrderSide::Ask => self
                    .ask_limit_order_repository()
                    .delete_by_order_id(tx, &opposite_order.id),
            }?;
            Ok(true)
        }
    }
}
