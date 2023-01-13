use super::Asset;
use core::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AssetPair<A: Asset> {
    pub base_asset: A,
    pub quote_asset: A,
}

impl<A: Asset> AssetPair<A> {
    pub fn new(base_asset: A, quote_asset: A) -> Self {
        Self {
            base_asset,
            quote_asset,
        }
    }
}

impl<A: Asset> fmt::Display for AssetPair<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base_asset, self.quote_asset)
    }
}
