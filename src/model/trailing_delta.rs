pub type BasisPoints = u64;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct TrailingDelta(BasisPoints);

impl TrailingDelta {
    pub fn new(basis_points: BasisPoints) -> Self {
        Self(basis_points)
    }
}

impl AsRef<BasisPoints> for TrailingDelta {
    fn as_ref(&self) -> &BasisPoints {
        &self.0
    }
}
