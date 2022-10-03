mod elements;
pub use elements::*;

use crate::models::OrderId;
use std::time::SystemTime;

pub trait OrderQueue {
    type Element;
    fn peek(&mut self) -> Option<&Self::Element>;
    fn pop(&mut self) -> Option<Self::Element>;
    fn insert(&mut self, order: Self::Element, timestamp: SystemTime) -> bool;
    fn amend(&mut self, order: Self::Element, timestamp: SystemTime) -> bool;
    fn remove(&mut self, id: OrderId) -> bool;
    fn modify_current_order(&mut self, new_order: Self::Element) -> bool;
}
