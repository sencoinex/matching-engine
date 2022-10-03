mod failure;
mod output;

pub use failure::*;
pub use output::*;

pub type OrderProcessingResult = Vec<Result<OrderBookOutput, OrderBookFailure>>;

use crate::models::{Asset, AssetPair, Order, OrderId, OrderSide, OrderType, Quantity};
use crate::order_queue::{LimitOrder, MarketOrder, OrderQueue};
use crate::Price;
use std::time::SystemTime;

pub struct OrderBook<A: Asset, LQ: OrderQueue<Element = LimitOrder>> {
    asset_pair: AssetPair<A>,
    bid_limit_queue: LQ,
    ask_limit_queue: LQ,
}

impl<A: Asset, LQ: OrderQueue<Element = LimitOrder>> OrderBook<A, LQ> {
    pub fn process_order(&mut self, order: Order<A>) -> OrderProcessingResult {
        let mut proc_result: OrderProcessingResult = vec![];
        match order {
            Order::Market {
                id,
                asset_pair,
                side,
                quantity,
                timestamp,
            } => {
                assert_eq!(self.asset_pair, asset_pair);
                proc_result.push(Ok(OrderBookOutput::Accepted {
                    id,
                    order_type: OrderType::Market,
                    timestamp: SystemTime::now(),
                }));
                self.process_market_order(&mut proc_result, id, side, quantity, timestamp);
            }
            Order::Limit {
                id,
                asset_pair,
                side,
                price,
                quantity,
                timestamp,
            } => {
                assert_eq!(self.asset_pair, asset_pair);
                proc_result.push(Ok(OrderBookOutput::Accepted {
                    id,
                    order_type: OrderType::Limit,
                    timestamp: SystemTime::now(),
                }));
                self.process_limit_order(&mut proc_result, id, side, price, quantity, timestamp);
            }
            Order::Amend {
                id,
                target_id,
                target_order_type,
                side,
                price,
                quantity,
                timestamp,
            } => {
                let is_amendable = match target_order_type {
                    OrderType::Market => false,
                    OrderType::Limit => true,
                };
                assert!(is_amendable);
                self.process_amend_order(
                    &mut proc_result,
                    id,
                    target_id,
                    target_order_type,
                    side,
                    price,
                    quantity,
                    timestamp,
                );
            }
            Order::Cancel {
                id,
                target_id,
                target_order_type,
                side,
            } => {
                let is_cancelable = match target_order_type {
                    OrderType::Market => false,
                    OrderType::Limit => true,
                };
                assert!(is_cancelable);
                self.process_cancel_order(&mut proc_result, id, target_id, target_order_type, side);
            }
        }
        proc_result
    }

    fn process_market_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: OrderId,
        side: OrderSide,
        quantity: Quantity,
        timestamp: SystemTime,
    ) {
        let order = MarketOrder { order_id, quantity };
        let opposite_order = match side {
            OrderSide::Bid => self.ask_limit_queue.peek(),
            OrderSide::Ask => self.bid_limit_queue.peek(),
        }
        .map(Clone::clone);
        if let Some(opposite_order) = opposite_order {
            let matching_complete =
                self.match_market_order_with_limit_order(results, &order, &opposite_order, side);
            if !matching_complete {
                self.process_market_order(
                    results,
                    order_id,
                    side,
                    quantity - opposite_order.quantity,
                    timestamp,
                );
            }
        } else {
            results.push(Err(OrderBookFailure::NoMatch(order_id)));
        }
    }

    fn process_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: OrderId,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) {
        let order = LimitOrder {
            order_id,
            price,
            quantity,
        };
        let opposite_order = match side {
            OrderSide::Bid => self.ask_limit_queue.peek(),
            OrderSide::Ask => self.bid_limit_queue.peek(),
        }
        .map(Clone::clone);
        if let Some(opposite_order) = opposite_order {
            let could_be_matched = match side {
                // verify bid/ask price overlap
                OrderSide::Bid => price >= opposite_order.price,
                OrderSide::Ask => price <= opposite_order.price,
            };
            if could_be_matched {
                let matching_complete =
                    self.match_limit_order_with_limit_order(results, &order, &opposite_order, side);
                if !matching_complete {
                    self.process_limit_order(
                        results,
                        order_id,
                        side,
                        price,
                        quantity - opposite_order.quantity,
                        timestamp,
                    );
                }
            } else {
                self.store_new_limit_order(results, order, side, timestamp);
            }
        } else {
            self.store_new_limit_order(results, order, side, timestamp);
        }
    }

    fn process_amend_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: OrderId,
        target_order_id: OrderId,
        target_order_type: OrderType,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        timestamp: SystemTime,
    ) {
        match target_order_type {
            OrderType::Limit => {
                let order = LimitOrder {
                    order_id: target_order_id,
                    price,
                    quantity,
                };
                if match side {
                    OrderSide::Bid => self.bid_limit_queue.amend(order, timestamp),
                    OrderSide::Ask => self.ask_limit_queue.amend(order, timestamp),
                } {
                    results.push(Ok(OrderBookOutput::Amended {
                        id: order_id,
                        target_id: target_order_id,
                        price,
                        quantity,
                        timestamp: SystemTime::now(),
                    }));
                    // TODO process limit order?
                } else {
                    results.push(Err(OrderBookFailure::OrderNotFound {
                        order_id,
                        target_order_id,
                    }));
                }
            }
            _ => { /* ignore */ }
        }
    }

    fn process_cancel_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order_id: OrderId,
        target_order_id: OrderId,
        target_order_type: OrderType,
        side: OrderSide,
    ) {
        match target_order_type {
            OrderType::Limit => {
                if match side {
                    OrderSide::Bid => self.bid_limit_queue.remove(target_order_id),
                    OrderSide::Ask => self.ask_limit_queue.remove(target_order_id),
                } {
                    results.push(Ok(OrderBookOutput::Cancelled {
                        id: order_id,
                        target_id: target_order_id,
                        timestamp: SystemTime::now(),
                    }));
                } else {
                    results.push(Err(OrderBookFailure::OrderNotFound {
                        order_id,
                        target_order_id,
                    }));
                }
            }
            _ => { /* ignore */ }
        }
    }

    fn store_new_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order: LimitOrder,
        side: OrderSide,
        timestamp: SystemTime,
    ) {
        let order_id = order.order_id;
        if !match side {
            OrderSide::Bid => self.bid_limit_queue.insert(order, timestamp),
            OrderSide::Ask => self.ask_limit_queue.insert(order, timestamp),
        } {
            results.push(Err(OrderBookFailure::FailedToEnqueueOrder(order_id)))
        }
    }

    fn match_market_order_with_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order: &MarketOrder,
        opposite_order: &LimitOrder,
        side: OrderSide,
    ) -> bool {
        let deal_time = SystemTime::now();
        if order.quantity < opposite_order.quantity {
            // market order: fully filled / limit order: partially filled
            results.push(Ok(OrderBookOutput::Filled {
                id: order.order_id,
                side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(OrderBookOutput::PartiallyFilled {
                id: opposite_order.order_id,
                side: side.opposite(),
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));

            // modify unmatched part of the opposite limit order
            let new_limit_order = LimitOrder {
                order_id: opposite_order.order_id,
                price: opposite_order.price,
                quantity: opposite_order.quantity - order.quantity,
            };
            match side {
                OrderSide::Bid => self.ask_limit_queue.modify_current_order(new_limit_order),
                OrderSide::Ask => self.bid_limit_queue.modify_current_order(new_limit_order),
            };
            true
        } else if order.quantity > opposite_order.quantity {
            // market order: partially filled / limit order: fully filled
            results.push(Ok(OrderBookOutput::PartiallyFilled {
                id: order.order_id,
                side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(OrderBookOutput::Filled {
                id: opposite_order.order_id,
                side: side.opposite(),
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match side {
                OrderSide::Bid => self.ask_limit_queue.pop(),
                OrderSide::Ask => self.bid_limit_queue.pop(),
            };
            false
        } else {
            // exact match
            results.push(Ok(OrderBookOutput::Filled {
                id: order.order_id,
                side,
                order_type: OrderType::Market,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(OrderBookOutput::Filled {
                id: opposite_order.order_id,
                side: side.opposite(),
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match side {
                OrderSide::Bid => self.ask_limit_queue.pop(),
                OrderSide::Ask => self.bid_limit_queue.pop(),
            };
            true
        }
    }

    fn match_limit_order_with_limit_order(
        &mut self,
        results: &mut OrderProcessingResult,
        order: &LimitOrder,
        opposite_order: &LimitOrder,
        side: OrderSide,
    ) -> bool {
        let deal_time = SystemTime::now();
        if order.quantity < opposite_order.quantity {
            // limit order: fully filled / limit order: partially filled
            results.push(Ok(OrderBookOutput::Filled {
                id: order.order_id,
                side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(OrderBookOutput::PartiallyFilled {
                id: opposite_order.order_id,
                side: side.opposite(),
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));

            // modify unmatched part of the opposite limit order
            let new_limit_order = LimitOrder {
                order_id: opposite_order.order_id,
                price: opposite_order.price,
                quantity: opposite_order.quantity - order.quantity,
            };
            match side {
                OrderSide::Bid => self.ask_limit_queue.modify_current_order(new_limit_order),
                OrderSide::Ask => self.bid_limit_queue.modify_current_order(new_limit_order),
            };
            true
        } else if order.quantity > opposite_order.quantity {
            // market order: partially filled / limit order: fully filled
            results.push(Ok(OrderBookOutput::PartiallyFilled {
                id: order.order_id,
                side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(OrderBookOutput::Filled {
                id: opposite_order.order_id,
                side: side.opposite(),
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match side {
                OrderSide::Bid => self.ask_limit_queue.pop(),
                OrderSide::Ask => self.bid_limit_queue.pop(),
            };
            false
        } else {
            // exact match
            results.push(Ok(OrderBookOutput::Filled {
                id: order.order_id,
                side,
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: order.quantity,
                timestamp: deal_time,
            }));
            results.push(Ok(OrderBookOutput::Filled {
                id: opposite_order.order_id,
                side: side.opposite(),
                order_type: OrderType::Limit,
                price: opposite_order.price,
                quantity: opposite_order.quantity,
                timestamp: deal_time,
            }));

            // remove filled limit order from the queue
            match side {
                OrderSide::Bid => self.ask_limit_queue.pop(),
                OrderSide::Ask => self.bid_limit_queue.pop(),
            };
            true
        }
    }
}
