use matching_engine::{
    AssetPair, LimitOrder, LimitOrderRepositoryLike, MatchingEngine, OrderRequest, OrderSide,
    Price, Quantity,
};
use std::time::SystemTime;

#[derive(Debug)]
pub enum MyError {
    ReDbError(redb::Error),
}

pub type Result<T> = core::result::Result<T, MyError>;

impl From<redb::Error> for MyError {
    fn from(cause: redb::Error) -> Self {
        Self::ReDbError(cause)
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

struct MyBidLimitOrderEntity {
    pub id: String,
    pub quantity: Quantity,
    pub timestamp: SystemTime,
}

pub struct MyBidLimitOrderRepository<'db> {
    database: &'db redb::Database,
}

impl<'db> MyBidLimitOrderRepository<'db> {
    fn new(database: &'db redb::Database) -> Self {
        Self { database }
    }
}

impl<'db> LimitOrderRepositoryLike for MyBidLimitOrderRepository<'db> {
    type Err = MyError;
    type Asset = MyAsset;
    type OrderId = MyOrderId;
    type Transaction = redb::WriteTransaction<'db>;

    fn create(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> std::result::Result<(), Self::Err> {
        todo!()
    }

    fn update(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> std::result::Result<(), Self::Err> {
        todo!()
    }

    fn delete_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<(), Self::Err> {
        todo!()
    }

    fn get_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<Option<LimitOrder<Self::OrderId, Self::Asset>>, Self::Err> {
        todo!()
    }

    fn next(
        &self,
        tx: &mut Self::Transaction,
    ) -> std::result::Result<Option<LimitOrder<Self::OrderId, Self::Asset>>, Self::Err> {
        todo!()
    }
}

pub struct MyAskLimitOrderRepository<'db> {
    database: &'db redb::Database,
}

impl<'db> MyAskLimitOrderRepository<'db> {
    fn new(database: &'db redb::Database) -> Self {
        Self { database }
    }
}

impl<'db> LimitOrderRepositoryLike for MyAskLimitOrderRepository<'db> {
    type Err = MyError;
    type Asset = MyAsset;
    type OrderId = MyOrderId;
    type Transaction = redb::WriteTransaction<'db>;

    fn create(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> std::result::Result<(), Self::Err> {
        todo!()
    }

    fn update(
        &self,
        tx: &mut Self::Transaction,
        order: &LimitOrder<Self::OrderId, Self::Asset>,
    ) -> std::result::Result<(), Self::Err> {
        todo!()
    }

    fn delete_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<(), Self::Err> {
        todo!()
    }

    fn get_by_order_id(
        &self,
        tx: &mut Self::Transaction,
        order_id: &Self::OrderId,
    ) -> std::result::Result<Option<LimitOrder<Self::OrderId, Self::Asset>>, Self::Err> {
        todo!()
    }

    fn next(
        &self,
        tx: &mut Self::Transaction,
    ) -> std::result::Result<Option<LimitOrder<Self::OrderId, Self::Asset>>, Self::Err> {
        todo!()
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
    let database = redb::Database::create("rbdb_example.redb")?;
    let asset_pair = AssetPair::new(MyAsset::BTC, MyAsset::USD);
    let mut my_engine = MyMatchingEngine {
        database: &database,
        asset_pair: asset_pair.clone(),
        bid_limit_order_repository: MyBidLimitOrderRepository::new(&database),
        ask_limit_order_repository: MyAskLimitOrderRepository::new(&database),
    };
    let order_requests = vec![OrderRequest::Limit(LimitOrder {
        id: MyOrderId::new(),
        asset_pair: asset_pair.clone(),
        side: OrderSide::Bid,
        price: Price::new(98, 2),
        quantity: Quantity::new(50, 1),
        timestamp: SystemTime::now(),
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
