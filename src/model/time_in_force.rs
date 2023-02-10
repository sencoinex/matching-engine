use std::fmt;
use std::str::FromStr;

const GOOD_TILL_CANCELLED: &'static str = "GTC";
const IMMEDIATE_OR_CANCEL: &'static str = "IOC";
const FILL_OR_KILL: &'static str = "FOK";

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum TimeInForce {
    GoodTillCancelled,
    ImmediateOrCancel,
    FillOrKill,
}

impl TimeInForce {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl Default for TimeInForce {
    fn default() -> Self {
        Self::GoodTillCancelled
    }
}

impl AsRef<str> for TimeInForce {
    fn as_ref(&self) -> &str {
        match self {
            Self::GoodTillCancelled => GOOD_TILL_CANCELLED,
            Self::ImmediateOrCancel => IMMEDIATE_OR_CANCEL,
            Self::FillOrKill => FILL_OR_KILL,
        }
    }
}

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Into<String> for TimeInForce {
    fn into(self) -> String {
        self.to_string()
    }
}

impl FromStr for TimeInForce {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            GOOD_TILL_CANCELLED => Ok(Self::GoodTillCancelled),
            IMMEDIATE_OR_CANCEL => Ok(Self::ImmediateOrCancel),
            FILL_OR_KILL => Ok(Self::FillOrKill),
            _ => Err(format!("unknown time in force value: {}", s)),
        }
    }
}
