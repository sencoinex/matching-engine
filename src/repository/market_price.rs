use crate::Price;

pub trait MarketPriceRepository: Send {
    type Err;
    type Price: Price;
    type Transaction;

    fn update(&self, tx: &mut Self::Transaction, price: &Self::Price) -> Result<(), Self::Err>;
    fn get(&self, tx: &mut Self::Transaction) -> Result<Option<Self::Price>, Self::Err>;
}
