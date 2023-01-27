mod output;
pub use output::*;
pub type OrderProcessingResult<ID, P, Q> = MatchingEngineOutput<ID, P, Q>;

use crate::{
    model::{
        AmendOrderRequest, Asset, AssetPair, CancelOrderRequest, LimitOrder, MarketOrder, OrderId,
        OrderRequest, OrderSide, OrderType, Price, Quantity, StopLimitOrder, StopOrder,
    },
    repository::{
        LimitOrderRepository, MarketPriceRepository, PendingStopOrder, PendingStopOrderRepository,
    },
};
use std::time::{SystemTime, UNIX_EPOCH};

pub trait MatchingEngine {
    type Err;
    type Asset: Asset;
    type OrderId: OrderId;
    type Price: Price;
    type Quantity: Quantity;
    type Transaction;
    type BidLimitOrderRepository: LimitOrderRepository<
        Err = Self::Err,
        Asset = Self::Asset,
        OrderId = Self::OrderId,
        Price = Self::Price,
        Quantity = Self::Quantity,
        Transaction = Self::Transaction,
    >;
    type AskLimitOrderRepository: LimitOrderRepository<
        Err = Self::Err,
        Asset = Self::Asset,
        OrderId = Self::OrderId,
        Price = Self::Price,
        Quantity = Self::Quantity,
        Transaction = Self::Transaction,
    >;
    type HighPendingStopOrderRepository: PendingStopOrderRepository<
        Err = Self::Err,
        Asset = Self::Asset,
        OrderId = Self::OrderId,
        Price = Self::Price,
        Quantity = Self::Quantity,
        Transaction = Self::Transaction,
    >;
    type LowPendingStopOrderRepository: PendingStopOrderRepository<
        Err = Self::Err,
        Asset = Self::Asset,
        OrderId = Self::OrderId,
        Price = Self::Price,
        Quantity = Self::Quantity,
        Transaction = Self::Transaction,
    >;
    type MarketPriceRepository: MarketPriceRepository<
        Err = Self::Err,
        Price = Self::Price,
        Transaction = Self::Transaction,
    >;

    fn asset_pair(&self) -> &AssetPair<Self::Asset>;
    fn bid_limit_order_repository(&self) -> &Self::BidLimitOrderRepository;
    fn ask_limit_order_repository(&self) -> &Self::AskLimitOrderRepository;
    fn high_pending_stop_order_repository(&self) -> &Self::HighPendingStopOrderRepository;
    fn low_pending_stop_order_repository(&self) -> &Self::LowPendingStopOrderRepository;
    fn market_price_repository(&self) -> &Self::MarketPriceRepository;
    fn current_timestamp_ms(&self) -> u64 {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
        since_the_epoch.as_millis() as u64
    }

    fn process_order_request(
        &mut self,
        tx: &mut Self::Transaction,
        market_price: Option<Self::Price>,
        order_request: OrderRequest<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>, Self::Err> {
        let initial_market_price = if let Some(p) = market_price {
            Some(p)
        } else {
            // if market price is not specified, get market price from repository.
            self.market_price_repository().get(tx)?
        };
        let mut proc_result: OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity> =
            OrderProcessingResult::new(initial_market_price);
        match order_request {
            OrderRequest::Market(request) => {
                assert_eq!(*self.asset_pair(), request.asset_pair);
                let timestamp_ms = self.current_timestamp_ms();
                let market_order = MarketOrder::new(
                    request.id,
                    request.asset_pair,
                    request.side,
                    request.quantity,
                    timestamp_ms,
                );
                proc_result.push(Ok(MatchingEngineEvent::Accepted {
                    id: market_order.id,
                    order_type: OrderType::Market,
                    timestamp_ms,
                }));
                self.process_market_order(tx, &mut proc_result, market_order)?;
            }
            OrderRequest::Limit(request) => {
                assert_eq!(*self.asset_pair(), request.asset_pair);
                let timestamp_ms = self.current_timestamp_ms();
                let limit_order = LimitOrder::new(
                    request.id,
                    request.asset_pair,
                    request.side,
                    request.time_in_force,
                    request.price,
                    request.quantity,
                    timestamp_ms,
                );
                proc_result.push(Ok(MatchingEngineEvent::Accepted {
                    id: limit_order.id,
                    order_type: OrderType::Limit,
                    timestamp_ms,
                }));
                self.process_limit_order(tx, &mut proc_result, limit_order)?;
            }
            OrderRequest::Stop(request) => {
                assert_eq!(*self.asset_pair(), request.asset_pair);
                let timestamp_ms = self.current_timestamp_ms();
                let stop_order = StopOrder::new(
                    request.id,
                    request.asset_pair,
                    request.side,
                    request.stop_price,
                    request.quantity,
                    timestamp_ms,
                );
                proc_result.push(Ok(MatchingEngineEvent::Accepted {
                    id: stop_order.id,
                    order_type: OrderType::Stop,
                    timestamp_ms,
                }));
                self.process_stop_order(tx, &mut proc_result, stop_order)?;
            }
            OrderRequest::StopLimit(request) => {
                assert_eq!(*self.asset_pair(), request.asset_pair);
                let timestamp_ms = self.current_timestamp_ms();
                let stop_limit_order = StopLimitOrder::new(
                    request.id,
                    request.asset_pair,
                    request.side,
                    request.stop_price,
                    request.time_in_force,
                    request.price,
                    request.quantity,
                    timestamp_ms,
                );
                proc_result.push(Ok(MatchingEngineEvent::Accepted {
                    id: stop_limit_order.id,
                    order_type: OrderType::StopLimit,
                    timestamp_ms,
                }));
                self.process_stop_limit_order(tx, &mut proc_result, stop_limit_order)?;
            }
            OrderRequest::Amend(request) => {
                assert_ne!(request.target_order_type, OrderType::Market);
                self.process_amend_order_request(tx, &mut proc_result, &request)?;
            }
            OrderRequest::Cancel(request) => {
                assert_ne!(request.target_order_type, OrderType::Market);
                self.process_cancel_order_request(tx, &mut proc_result, &request)?;
            }
        }

        // watch market price change & handle stop price trigger.
        self.handle_price_change(tx, &mut proc_result, initial_market_price.as_ref())?;

        // save latest market price
        if let Some(latest_market_price) = &proc_result.market_price {
            let diff = match initial_market_price {
                Some(initial_market_price) => initial_market_price != *latest_market_price,
                None => true,
            };
            if diff {
                self.market_price_repository()
                    .update(tx, latest_market_price)?;
            }
        }

        Ok(proc_result)
    }

    fn process_market_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        market_order: MarketOrder<Self::OrderId, Self::Asset, Self::Quantity>,
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
                self.process_market_order(tx, results, next_market_order)?;
            }
        } else {
            results.push(Err(MatchingEngineFailure::NoMatch(market_order.id)));
        }
        Ok(())
    }

    fn process_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        limit_order: LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
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
                    self.process_limit_order(tx, results, next_limit_order)?;
                }
            } else {
                self.store_new_limit_order(tx, results, &limit_order)?;
            }
        } else {
            self.store_new_limit_order(tx, results, &limit_order)?;
        }
        Ok(())
    }

    fn process_amend_order_request(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        request: &AmendOrderRequest<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<(), Self::Err> {
        match request.target_order_type {
            OrderType::Limit => {
                let order = match request.side {
                    OrderSide::Bid => self
                        .bid_limit_order_repository()
                        .get_by_order_id(tx, &request.target_id),
                    OrderSide::Ask => self
                        .ask_limit_order_repository()
                        .get_by_order_id(tx, &request.target_id),
                }?;
                if let Some(mut target_order) = order {
                    let timestamp_ms = self.current_timestamp_ms();
                    target_order.price = request.price;
                    target_order.quantity = request.quantity;
                    target_order.timestamp_ms = timestamp_ms;
                    match request.side {
                        OrderSide::Bid => {
                            self.bid_limit_order_repository().update(tx, &target_order)
                        }
                        OrderSide::Ask => {
                            self.ask_limit_order_repository().update(tx, &target_order)
                        }
                    }?;
                    results.push(Ok(MatchingEngineEvent::Amended {
                        id: request.id,
                        target_id: request.target_id,
                        price: request.price,
                        quantity: request.quantity,
                        timestamp_ms,
                    }));
                    // TODO process limit order?
                } else {
                    results.push(Err(MatchingEngineFailure::OrderNotFound {
                        order_id: request.id,
                        target_order_id: request.target_id,
                    }));
                }
            }
            _ => { /* ignore */ }
        }
        Ok(())
    }

    fn process_cancel_order_request(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        request: &CancelOrderRequest<Self::OrderId, Self::Asset>,
    ) -> Result<(), Self::Err> {
        match request.target_order_type {
            OrderType::Limit => {
                // TODO check missed
                let deleted = match request.side {
                    OrderSide::Bid => self
                        .bid_limit_order_repository()
                        .delete_by_order_id(tx, &request.target_id),
                    OrderSide::Ask => self
                        .ask_limit_order_repository()
                        .delete_by_order_id(tx, &request.target_id),
                }?;
                if deleted {
                    results.push(Ok(MatchingEngineEvent::Cancelled {
                        id: request.id,
                        target_id: request.target_id,
                        timestamp_ms: self.current_timestamp_ms(),
                    }));
                } else {
                    results.push(Err(MatchingEngineFailure::OrderNotFound {
                        order_id: request.id,
                        target_order_id: request.target_id,
                    }));
                }
            }
            _ => { /* ignore */ }
        }
        Ok(())
    }

    fn store_new_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        _results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        limit_order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<(), Self::Err> {
        match limit_order.side {
            OrderSide::Bid => self.bid_limit_order_repository().create(tx, limit_order),
            OrderSide::Ask => self.ask_limit_order_repository().create(tx, limit_order),
        }
    }

    fn match_market_order_with_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        order: &MarketOrder<Self::OrderId, Self::Asset, Self::Quantity>,
        opposite_order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<bool, Self::Err> {
        let deal_time = self.current_timestamp_ms();
        if order.quantity < opposite_order.quantity {
            // market order: fully filled / limit order: partially filled
            results.push(Ok(MatchingEngineEvent::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp_ms: deal_time,
            }));
            results.push(Ok(MatchingEngineEvent::PartiallyFilled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp_ms: deal_time,
            }));
            results.set_market_price(opposite_order.price);

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
            results.push(Ok(MatchingEngineEvent::PartiallyFilled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp_ms: deal_time,
            }));
            results.push(Ok(MatchingEngineEvent::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp_ms: deal_time,
            }));
            results.set_market_price(opposite_order.price);

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
            results.push(Ok(MatchingEngineEvent::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp_ms: deal_time,
            }));
            results.push(Ok(MatchingEngineEvent::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp_ms: deal_time,
            }));
            results.set_market_price(opposite_order.price);

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
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
        opposite_order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<bool, Self::Err> {
        let deal_time = self.current_timestamp_ms();
        if order.quantity < opposite_order.quantity {
            // limit order: fully filled / limit order: partially filled
            results.push(Ok(MatchingEngineEvent::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp_ms: deal_time,
            }));
            results.push(Ok(MatchingEngineEvent::PartiallyFilled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp_ms: deal_time,
            }));
            results.set_market_price(opposite_order.price);

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
            results.push(Ok(MatchingEngineEvent::PartiallyFilled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp_ms: deal_time,
            }));
            results.push(Ok(MatchingEngineEvent::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp_ms: deal_time,
            }));
            results.set_market_price(opposite_order.price);

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
            results.push(Ok(MatchingEngineEvent::Filled {
                id: order.id,
                side: order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp_ms: deal_time,
            }));
            results.push(Ok(MatchingEngineEvent::Filled {
                id: opposite_order.id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp_ms: deal_time,
            }));
            results.set_market_price(opposite_order.price);

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

    fn process_stop_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        stop_order: StopOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<(), Self::Err> {
        if results.market_price.is_none() {
            results.push(Err(MatchingEngineFailure::MissingMarketPriceForStopOrder(
                stop_order.id,
            )));
            return Ok(());
        }
        let market_price = results.market_price.as_ref().unwrap();
        if stop_order.stop_price < *market_price {
            self.low_pending_stop_order_repository()
                .create(tx, &PendingStopOrder::StopOrder(stop_order))
        } else if *market_price < stop_order.stop_price {
            self.high_pending_stop_order_repository()
                .create(tx, &PendingStopOrder::StopOrder(stop_order))
        } else {
            // issue market order
            let timestamp_ms = self.current_timestamp_ms();
            let market_order = stop_order.issue_market_order(timestamp_ms);
            results.push(Ok(MatchingEngineEvent::StopOrderIssueMarketOrder {
                id: market_order.id,
                timestamp_ms,
            }));
            self.process_market_order(tx, results, market_order)
        }
    }

    fn process_stop_limit_order(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        stop_limit_order: StopLimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> Result<(), Self::Err> {
        if results.market_price.is_none() {
            results.push(Err(MatchingEngineFailure::MissingMarketPriceForStopOrder(
                stop_limit_order.id,
            )));
            return Ok(());
        }
        let market_price = results.market_price.as_ref().unwrap();
        if stop_limit_order.stop_price < *market_price {
            self.low_pending_stop_order_repository()
                .create(tx, &PendingStopOrder::StopLimitOrder(stop_limit_order))
        } else if *market_price < stop_limit_order.stop_price {
            self.high_pending_stop_order_repository()
                .create(tx, &PendingStopOrder::StopLimitOrder(stop_limit_order))
        } else {
            // issue limit order
            let timestamp_ms = self.current_timestamp_ms();
            let limit_order = stop_limit_order.issue_limit_order(timestamp_ms);
            results.push(Ok(MatchingEngineEvent::StopLimitOrderIssueLimitOrder {
                id: limit_order.id,
                timestamp_ms,
            }));
            self.process_limit_order(tx, results, limit_order)
        }
    }

    /// If the market price has changed, then search stop(or stop limit) orders whose stop price has reached its threshold.
    /// If there exists stop(or stop limit) orders to be processed, issue market(or limit) orders.
    fn handle_price_change(
        &mut self,
        tx: &mut Self::Transaction,
        results: &mut OrderProcessingResult<Self::OrderId, Self::Price, Self::Quantity>,
        initial_market_price: Option<&Self::Price>,
    ) -> Result<(), Self::Err> {
        let increased = if let Some(latest_market_price) = &results.market_price {
            if let Some(initial_market_price) = initial_market_price {
                if *initial_market_price < *latest_market_price {
                    true
                } else if *latest_market_price < *initial_market_price {
                    false
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        let batch_size = 100;
        let market_price = results.market_price.unwrap();
        loop {
            let pending_orders = if increased {
                self.high_pending_stop_order_repository()
                    .get_list_by_market_price(tx, &market_price, batch_size)?
            } else {
                self.low_pending_stop_order_repository()
                    .get_list_by_market_price(tx, &market_price, batch_size)?
            };
            let fetch_len = pending_orders.len();
            for pending_order in pending_orders {
                match &pending_order {
                    PendingStopOrder::StopOrder(stop_order) => {
                        let timestamp_ms = self.current_timestamp_ms();
                        let market_order = stop_order.issue_market_order(timestamp_ms);
                        results.push(Ok(MatchingEngineEvent::StopOrderIssueMarketOrder {
                            id: market_order.id,
                            timestamp_ms,
                        }));
                        self.process_market_order(tx, results, market_order)?;
                    }
                    PendingStopOrder::StopLimitOrder(stop_limit_order) => {
                        let timestamp_ms = self.current_timestamp_ms();
                        let limit_order = stop_limit_order.issue_limit_order(timestamp_ms);
                        results.push(Ok(MatchingEngineEvent::StopLimitOrderIssueLimitOrder {
                            id: limit_order.id,
                            timestamp_ms,
                        }));
                        self.process_limit_order(tx, results, limit_order)?;
                    }
                }
                if increased {
                    self.high_pending_stop_order_repository()
                        .delete(tx, &pending_order)?;
                } else {
                    self.low_pending_stop_order_repository()
                        .delete(tx, &pending_order)?;
                }
            }
            if fetch_len < batch_size as usize {
                break;
            }
        }
        Ok(())
    }
}
