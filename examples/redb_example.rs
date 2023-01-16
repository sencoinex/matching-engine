use matching_engine::{
    AssetPair, LimitOrder, LimitOrderRepositoryLike, MatchingEngine, OrderRequest, OrderSide,
};
use redb::ReadableTable;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Sub;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum MyError {
    ReDbError(redb::Error),
    SerializationError(String),
}

pub type Result<T> = core::result::Result<T, MyError>;

impl From<redb::Error> for MyError {
    fn from(cause: redb::Error) -> Self {
        Self::ReDbError(cause)
    }
}

impl<T: Debug> From<ciborium::ser::Error<T>> for MyError {
    fn from(cause: ciborium::ser::Error<T>) -> Self {
        Self::SerializationError(format!("{}", cause))
    }
}

impl<T: Debug> From<ciborium::de::Error<T>> for MyError {
    fn from(cause: ciborium::de::Error<T>) -> Self {
        Self::SerializationError(format!("{}", cause))
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum MyAsset {
    USD,
    BTC,
}

impl core::fmt::Display for MyAsset {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::USD => write!(f, "USD"),
            Self::BTC => write!(f, "BTC"),
        }
    }
}

type MyAssetPair = AssetPair<MyAsset>;

type MyOrderId = ulid::Ulid;

const MAX_PRICE_PRECISION: u32 = 18;

#[derive(Debug, Copy, Clone, Eq, Deserialize, Serialize)]
pub struct MyPrice {
    pub num: u64,
    pub scale: u32,
}

/// implements into index key
impl Into<u128> for MyPrice {
    fn into(self) -> u128 {
        let int_part = self.num;
        let max_scale = MAX_PRICE_PRECISION - self.scale;
        Decimal::new(int_part as i64, max_scale).to_u128().unwrap()
    }
}

impl MyPrice {
    pub fn new(num: u64, scale: u32) -> Self {
        assert!(scale <= MAX_PRICE_PRECISION);
        Self { num, scale }
    }

    pub fn decimal(&self) -> Decimal {
        Decimal::new(self.num as i64, self.scale)
    }
}

impl PartialEq for MyPrice {
    fn eq(&self, other: &Self) -> bool {
        if self.scale == other.scale {
            self.num == other.num
        } else {
            let mine = self.decimal();
            let others = other.decimal();
            mine == others
        }
    }
}

impl Ord for MyPrice {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.scale == other.scale {
            self.num.cmp(&other.num)
        } else {
            let mine = self.decimal();
            let others = other.decimal();
            mine.cmp(&others)
        }
    }
}

impl PartialOrd for MyPrice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl core::fmt::Display for MyPrice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.decimal())
    }
}

#[derive(Debug, Copy, Clone, Eq, Deserialize, Serialize)]
pub struct MyQuantity {
    pub num: u64,
    pub scale: u32,
}

impl MyQuantity {
    pub fn new(num: u64, scale: u32) -> Self {
        Self { num, scale }
    }

    pub fn decimal(&self) -> Decimal {
        Decimal::new(self.num as i64, self.scale)
    }

    pub fn get_num_by_scale(&self, scale: u32) -> u64 {
        if scale == self.scale {
            self.num
        } else {
            if self.scale < scale {
                self.num * 10_u64.pow(scale - self.scale)
            } else {
                self.num / 10_u64.pow(self.scale - scale)
            }
        }
    }
}

impl Into<Decimal> for MyQuantity {
    fn into(self) -> Decimal {
        self.decimal()
    }
}

impl PartialEq for MyQuantity {
    fn eq(&self, other: &Self) -> bool {
        if self.scale == other.scale {
            self.num == other.num
        } else {
            let mine = self.decimal();
            let others = other.decimal();
            mine == others
        }
    }
}

impl Ord for MyQuantity {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.scale == other.scale {
            self.num.cmp(&other.num)
        } else {
            let mine = self.decimal();
            let others = other.decimal();
            mine.cmp(&others)
        }
    }
}

impl PartialOrd for MyQuantity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Sub for MyQuantity {
    type Output = MyQuantity;
    fn sub(self, other: Self) -> Self {
        if self.scale == other.scale {
            Self {
                num: self.num - other.num,
                scale: self.scale,
            }
        } else if self.scale < other.scale {
            let scale = other.scale;
            let my_num = self.get_num_by_scale(scale);
            Self {
                num: my_num - other.num,
                scale,
            }
        } else {
            let scale = self.scale;
            let others_num = other.get_num_by_scale(scale);
            Self {
                num: self.num - others_num,
                scale,
            }
        }
    }
}

impl core::fmt::Display for MyQuantity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.decimal())
    }
}

const BID_LIMIT_ORDER_PRICE_INDEX: redb::TableDefinition<u128, &[u8]> =
    redb::TableDefinition::new("bid_limit_order_prices");
const ASK_LIMIT_ORDER_PRICE_INDEX: redb::TableDefinition<u128, &[u8]> =
    redb::TableDefinition::new("ask_limit_order_prices");
const BID_LIMIT_ORDER_TABLE: redb::TableDefinition<u128, &[u8]> =
    redb::TableDefinition::new("bid_limit_orders");
const ASK_LIMIT_ORDER_TABLE: redb::TableDefinition<u128, &[u8]> =
    redb::TableDefinition::new("ask_limit_orders");

#[derive(Deserialize, Serialize, Default)]
struct PriceIndexValue(Vec<u128>);

impl PriceIndexValue {
    pub fn push(&mut self, id: &MyOrderId) {
        self.0.push(id.0)
    }
    pub fn remove(&mut self, id: &MyOrderId) {
        if let Some((index, _)) = self.0.iter().enumerate().find(|(_, i)| **i == id.0) {
            self.0.remove(index);
        }
    }
    pub fn encode<W: std::io::Write>(&self, w: W) -> Result<()> {
        ciborium::ser::into_writer(&self, w).map_err(Into::into)
    }
    pub fn decode(slice: &[u8]) -> Result<Self> {
        ciborium::de::from_reader(slice).map_err(Into::into)
    }
}

#[derive(Deserialize, Serialize)]
struct LimitOrderValue {
    pub price: MyPrice,
    pub quantity: MyQuantity,
    pub timestamp_ms: u64,
}

impl LimitOrderValue {
    pub fn encode<W: std::io::Write>(&self, w: W) -> Result<()> {
        ciborium::ser::into_writer(&self, w).map_err(Into::into)
    }
    pub fn decode(slice: &[u8]) -> Result<Self> {
        ciborium::de::from_reader(slice).map_err(Into::into)
    }
}

pub struct MyBidLimitOrderRepository<'db> {
    #[allow(dead_code)]
    database: &'db redb::Database,
    asset_pair: MyAssetPair,
}

impl<'db> MyBidLimitOrderRepository<'db> {
    fn new(database: &'db redb::Database, asset_pair: MyAssetPair) -> Self {
        Self {
            database,
            asset_pair,
        }
    }

    fn insert_or_update_order<'txn>(
        &self,
        order_table: &mut redb::Table<'db, 'txn, u128, &[u8]>,
        order: &LimitOrder<MyOrderId, MyAsset, MyPrice, MyQuantity>,
    ) -> Result<()> {
        let value = LimitOrderValue {
            price: order.price,
            quantity: order.quantity,
            timestamp_ms: order.timestamp_ms,
        };
        let mut bytes = Vec::new();
        value.encode(&mut bytes)?;
        order_table.insert(&order.id.0, &bytes)?;
        Ok(())
    }

    fn add_index_value<'txn>(
        &self,
        index: &mut redb::Table<'db, 'txn, u128, &[u8]>,
        price: &MyPrice,
        order_id: &MyOrderId,
    ) -> Result<()> {
        let key: u128 = (*price).into();
        let mut value = if let Some(ids) = index.get(&key)? {
            PriceIndexValue::decode(ids.value())?
        } else {
            PriceIndexValue::default()
        };
        value.push(&order_id);
        let mut index_value_bytes = Vec::new();
        value.encode(&mut index_value_bytes)?;
        index.insert(&key, &index_value_bytes)?;
        Ok(())
    }

    fn delete_index_value<'txn>(
        &self,
        index: &mut redb::Table<'db, 'txn, u128, &[u8]>,
        price: &MyPrice,
        order_id: &MyOrderId,
    ) -> Result<()> {
        let key: u128 = (*price).into();
        let mut value = if let Some(ids) = index.get(&key)? {
            PriceIndexValue::decode(ids.value())?
        } else {
            PriceIndexValue::default()
        };
        value.remove(&order_id);
        let mut index_value_bytes = Vec::new();
        value.encode(&mut index_value_bytes)?;
        index.insert(&key, &index_value_bytes)?;
        Ok(())
    }
}

impl<'db> LimitOrderRepositoryLike for MyBidLimitOrderRepository<'db> {
    type Err = MyError;
    type Asset = MyAsset;
    type OrderId = MyOrderId;
    type Price = MyPrice;
    type Quantity = MyQuantity;
    type Transaction = redb::WriteTransaction<'db>;

    fn create(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> std::result::Result<(), Self::Err> {
        // insert order
        {
            let mut limit_order_table = tx.open_table(BID_LIMIT_ORDER_TABLE)?;
            self.insert_or_update_order(&mut limit_order_table, order)?;
        }
        // update index
        {
            let mut price_index = tx.open_table(BID_LIMIT_ORDER_PRICE_INDEX)?;
            self.add_index_value(&mut price_index, &order.price, &order.id)?;
        }
        Ok(())
    }

    fn update(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> std::result::Result<(), Self::Err> {
        if let Some(old_order) = self.get_by_order_id(tx, &order.id)? {
            if old_order.price != order.price {
                // delete & update index
                let mut price_index = tx.open_table(BID_LIMIT_ORDER_PRICE_INDEX)?;
                self.delete_index_value(&mut price_index, &old_order.price, &order.id)?;
                self.add_index_value(&mut price_index, &order.price, &order.id)?;
            } else {
                // no need to update index
            }
        } else {
            // update index
            let mut price_index = tx.open_table(BID_LIMIT_ORDER_PRICE_INDEX)?;
            self.add_index_value(&mut price_index, &order.price, &order.id)?;
        }
        // update or insert order
        {
            let mut limit_order_table = tx.open_table(BID_LIMIT_ORDER_TABLE)?;
            self.insert_or_update_order(&mut limit_order_table, order)?;
        }
        Ok(())
    }

    fn delete_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<(), Self::Err> {
        if let Some(order) = self.get_by_order_id(tx, order_id)? {
            // delete order
            {
                let mut limit_order_table = tx.open_table(BID_LIMIT_ORDER_TABLE)?;
                limit_order_table.remove(&order.id.0)?;
            }
            // delete from index
            {
                let mut price_index = tx.open_table(BID_LIMIT_ORDER_PRICE_INDEX)?;
                self.delete_index_value(&mut price_index, &order.price, &order.id)?;
            }
        }
        Ok(())
    }

    fn get_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<
        Option<LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>>,
        Self::Err,
    > {
        let limit_order_table = tx.open_table(BID_LIMIT_ORDER_TABLE)?;
        let order_bytes = limit_order_table
            .get(&order_id.0)?
            .expect("indexed key missing its reference...");
        let order = LimitOrderValue::decode(order_bytes.value())?;
        Ok(Some(LimitOrder {
            id: order_id.clone(),
            asset_pair: self.asset_pair.clone(),
            side: OrderSide::Bid,
            price: order.price,
            quantity: order.quantity,
            timestamp_ms: order.timestamp_ms,
        }))
    }

    fn next(
        &self,
        tx: &mut Self::Transaction,
    ) -> std::result::Result<
        Option<LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>>,
        Self::Err,
    > {
        let id = {
            let price_index = tx.open_table(BID_LIMIT_ORDER_PRICE_INDEX)?;
            let mut iter = price_index.iter()?;
            let id = {
                if let Some((_, id_array_bytes)) = iter.next() {
                    let ids = PriceIndexValue::decode(id_array_bytes.value())?;
                    let id = ids.0.first().expect("index node must have at least one id");
                    let id = MyOrderId::from(*id);
                    Some(id)
                } else {
                    None
                }
            };
            id
        };
        if let Some(id) = id {
            self.get_by_order_id(tx, &id)
        } else {
            Ok(None)
        }
    }
}

pub struct MyAskLimitOrderRepository<'db> {
    #[allow(dead_code)]
    database: &'db redb::Database,
    asset_pair: MyAssetPair,
}

impl<'db> MyAskLimitOrderRepository<'db> {
    fn new(database: &'db redb::Database, asset_pair: MyAssetPair) -> Self {
        Self {
            database,
            asset_pair,
        }
    }

    fn insert_or_update_order<'txn>(
        &self,
        order_table: &mut redb::Table<'db, 'txn, u128, &[u8]>,
        order: &LimitOrder<MyOrderId, MyAsset, MyPrice, MyQuantity>,
    ) -> Result<()> {
        let value = LimitOrderValue {
            price: order.price,
            quantity: order.quantity,
            timestamp_ms: order.timestamp_ms,
        };
        let mut bytes = Vec::new();
        value.encode(&mut bytes)?;
        order_table.insert(&order.id.0, &bytes)?;
        Ok(())
    }

    fn add_index_value<'txn>(
        &self,
        index: &mut redb::Table<'db, 'txn, u128, &[u8]>,
        price: &MyPrice,
        order_id: &MyOrderId,
    ) -> Result<()> {
        let key: u128 = (*price).into();
        let mut value = if let Some(ids) = index.get(&key)? {
            PriceIndexValue::decode(ids.value())?
        } else {
            PriceIndexValue::default()
        };
        value.push(&order_id);
        let mut index_value_bytes = Vec::new();
        value.encode(&mut index_value_bytes)?;
        index.insert(&key, &index_value_bytes)?;
        Ok(())
    }

    fn delete_index_value<'txn>(
        &self,
        index: &mut redb::Table<'db, 'txn, u128, &[u8]>,
        price: &MyPrice,
        order_id: &MyOrderId,
    ) -> Result<()> {
        let key: u128 = (*price).into();
        let mut value = if let Some(ids) = index.get(&key)? {
            PriceIndexValue::decode(ids.value())?
        } else {
            PriceIndexValue::default()
        };
        value.remove(&order_id);
        let mut index_value_bytes = Vec::new();
        value.encode(&mut index_value_bytes)?;
        index.insert(&key, &index_value_bytes)?;
        Ok(())
    }
}

impl<'db> LimitOrderRepositoryLike for MyAskLimitOrderRepository<'db> {
    type Err = MyError;
    type Asset = MyAsset;
    type OrderId = MyOrderId;
    type Price = MyPrice;
    type Quantity = MyQuantity;
    type Transaction = redb::WriteTransaction<'db>;

    fn create(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> std::result::Result<(), Self::Err> {
        // insert order
        {
            let mut limit_order_table = tx.open_table(ASK_LIMIT_ORDER_TABLE)?;
            self.insert_or_update_order(&mut limit_order_table, order)?;
        }
        // update index
        {
            let mut price_index = tx.open_table(ASK_LIMIT_ORDER_PRICE_INDEX)?;
            self.add_index_value(&mut price_index, &order.price, &order.id)?;
        }
        Ok(())
    }

    fn update(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>,
    ) -> std::result::Result<(), Self::Err> {
        if let Some(old_order) = self.get_by_order_id(tx, &order.id)? {
            if old_order.price != order.price {
                // delete & update index
                let mut price_index = tx.open_table(ASK_LIMIT_ORDER_PRICE_INDEX)?;
                self.delete_index_value(&mut price_index, &old_order.price, &order.id)?;
                self.add_index_value(&mut price_index, &order.price, &order.id)?;
            } else {
                // no need to update index
            }
        } else {
            // update index
            let mut price_index = tx.open_table(ASK_LIMIT_ORDER_PRICE_INDEX)?;
            self.add_index_value(&mut price_index, &order.price, &order.id)?;
        }
        // update or insert order
        {
            let mut limit_order_table = tx.open_table(ASK_LIMIT_ORDER_TABLE)?;
            self.insert_or_update_order(&mut limit_order_table, order)?;
        }
        Ok(())
    }

    fn delete_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<(), Self::Err> {
        if let Some(order) = self.get_by_order_id(tx, order_id)? {
            // delete order
            {
                let mut limit_order_table = tx.open_table(ASK_LIMIT_ORDER_TABLE)?;
                limit_order_table.remove(&order.id.0)?;
            }
            // delete from index
            {
                let mut price_index = tx.open_table(ASK_LIMIT_ORDER_PRICE_INDEX)?;
                self.delete_index_value(&mut price_index, &order.price, &order.id)?;
            }
        }
        Ok(())
    }

    fn get_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<
        Option<LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>>,
        Self::Err,
    > {
        let limit_order_table = tx.open_table(ASK_LIMIT_ORDER_TABLE)?;
        let order_bytes = limit_order_table
            .get(&order_id.0)?
            .expect("indexed key missing its reference...");
        let order = LimitOrderValue::decode(order_bytes.value())?;
        Ok(Some(LimitOrder {
            id: order_id.clone(),
            asset_pair: self.asset_pair.clone(),
            side: OrderSide::Ask,
            price: order.price,
            quantity: order.quantity,
            timestamp_ms: order.timestamp_ms,
        }))
    }

    fn next(
        &self,
        tx: &mut Self::Transaction,
    ) -> std::result::Result<
        Option<LimitOrder<Self::OrderId, Self::Asset, Self::Price, Self::Quantity>>,
        Self::Err,
    > {
        let id = {
            let price_index = tx.open_table(ASK_LIMIT_ORDER_PRICE_INDEX)?;
            let mut iter = price_index.iter()?;
            let id = {
                if let Some((_, id_array_bytes)) = iter.next_back() {
                    let ids = PriceIndexValue::decode(id_array_bytes.value())?;
                    let id = ids.0.first().expect("index node must have at least one id");
                    let id = MyOrderId::from(*id);
                    Some(id)
                } else {
                    None
                }
            };
            id
        };
        if let Some(id) = id {
            self.get_by_order_id(tx, &id)
        } else {
            Ok(None)
        }
    }
}

pub struct MyMatchingEngine<'db> {
    database: &'db redb::Database,
    asset_pair: MyAssetPair,
    bid_limit_order_repository: MyBidLimitOrderRepository<'db>,
    ask_limit_order_repository: MyAskLimitOrderRepository<'db>,
}

impl<'db> MyMatchingEngine<'db> {
    fn start_tx(&self) -> Result<redb::WriteTransaction<'db>> {
        self.database.begin_write().map_err(Into::into)
    }
}

impl<'db> MatchingEngine for MyMatchingEngine<'db> {
    type Err = MyError;
    type Asset = MyAsset;
    type OrderId = MyOrderId;
    type Price = MyPrice;
    type Quantity = MyQuantity;
    type Transaction = redb::WriteTransaction<'db>;
    type BidLimitOrderRepository = MyBidLimitOrderRepository<'db>;
    type AskLimitOrderRepository = MyAskLimitOrderRepository<'db>;

    fn asset_pair(&self) -> &AssetPair<Self::Asset> {
        &self.asset_pair
    }

    fn bid_limit_order_repository(&self) -> &Self::BidLimitOrderRepository {
        &self.bid_limit_order_repository
    }

    fn ask_limit_order_repository(&self) -> &Self::AskLimitOrderRepository {
        &self.ask_limit_order_repository
    }
}

fn main() -> Result<()> {
    let database = redb::Database::create("redb_example.redb")?;
    let asset_pair = AssetPair::new(MyAsset::BTC, MyAsset::USD);
    let mut my_engine = MyMatchingEngine {
        database: &database,
        asset_pair: asset_pair.clone(),
        bid_limit_order_repository: MyBidLimitOrderRepository::new(&database, asset_pair.clone()),
        ask_limit_order_repository: MyAskLimitOrderRepository::new(&database, asset_pair.clone()),
    };
    let order_requests = vec![OrderRequest::Limit(LimitOrder {
        id: MyOrderId::new(),
        asset_pair: asset_pair.clone(),
        side: OrderSide::Bid,
        price: MyPrice::new(98, 2),
        quantity: MyQuantity::new(50, 1),
        timestamp_ms: current_timestamp_ms(),
    })];

    // processing
    for order_request in order_requests {
        println!("Order => {:?}", &order_request);
        let mut tx = my_engine.start_tx()?;
        {
            let res = my_engine.process_order(&mut tx, order_request)?;
            println!("Processing => {:?}", res);
        }
        tx.commit()?;
    }
    Ok(())
}

fn current_timestamp_ms() -> u64 {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_millis() as u64
}
