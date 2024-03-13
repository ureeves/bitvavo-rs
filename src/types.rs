use std::fmt;

use serde::de::{Error, SeqAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

/// Time interval between each candlestick.
#[derive(Debug)]
pub enum CandleInterval {
    OneMinute,
    FiveMinutes,
    FifteenMinutes,
    ThirtyMinutes,
    OneHour,
    TwoHours,
    FourHours,
    SixHours,
    EightHours,
    TwelveHours,
    OneDay,
}

impl fmt::Display for CandleInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CandleInterval::OneMinute => write!(f, "1m"),
            CandleInterval::FiveMinutes => write!(f, "5m"),
            CandleInterval::FifteenMinutes => write!(f, "15m"),
            CandleInterval::ThirtyMinutes => write!(f, "30m"),
            CandleInterval::OneHour => write!(f, "1h"),
            CandleInterval::TwoHours => write!(f, "2h"),
            CandleInterval::FourHours => write!(f, "4h"),
            CandleInterval::SixHours => write!(f, "6h"),
            CandleInterval::EightHours => write!(f, "8h"),
            CandleInterval::TwelveHours => write!(f, "12h"),
            CandleInterval::OneDay => write!(f, "1d"),
        }
    }
}

/// A candlestick for a given market over a given time interval.
#[derive(Debug)]
pub struct OHLCV {
    pub time: u64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

macro_rules! next_seq_element {
    ($seq:ident, $name:ident) => {
        $seq.next_element()?
            .ok_or_else(|| A::Error::missing_field(stringify!($name)))?
    };
}

impl<'de> Deserialize<'de> for OHLCV {
    fn deserialize<D>(deserializer: D) -> crate::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OHLCVVisitor;

        impl<'de> Visitor<'de> for OHLCVVisitor {
            type Value = OHLCV;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "OHLCV")
            }

            fn visit_seq<A>(self, mut seq: A) -> crate::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(OHLCV {
                    time: next_seq_element!(seq, time),
                    open: next_seq_element!(seq, open),
                    high: next_seq_element!(seq, high),
                    low: next_seq_element!(seq, low),
                    close: next_seq_element!(seq, close),
                    volume: next_seq_element!(seq, volume),
                })
            }
        }

        deserializer.deserialize_seq(OHLCVVisitor)
    }
}

/// Asset supported by Bitvavo.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub symbol: String,
    pub name: String,
    pub decimals: u64,
    pub deposit_fee: String,
    pub deposit_confirmations: u64,
    pub deposit_status: AssetStatus,
    pub withdrawal_fee: String,
    pub withdrawal_min_amount: String,
    pub withdrawal_status: AssetStatus,
    pub networks: Vec<String>,
    pub message: Option<String>,
}

/// The status of an asset.
#[derive(Debug)]
pub enum AssetStatus {
    Ok,
    Maintenance,
    Delisted,
}

impl<'de> Deserialize<'de> for AssetStatus {
    fn deserialize<D>(deserializer: D) -> crate::Result<AssetStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "OK" => Ok(AssetStatus::Ok),
            "MAINTENANCE" => Ok(AssetStatus::Maintenance),
            "DELISTED" => Ok(AssetStatus::Delisted),
            s => Err(D::Error::invalid_value(
                Unexpected::Str(s),
                &"[OK, MAINTENANCE, DELISTED]",
            )),
        }
    }
}

/// Information about a market on Bitvavo.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
    #[serde(rename = "market")]
    pub pair: String,
    pub status: MarketStatus,
    pub base: String,
    pub quote: String,
    pub price_precision: u64,
    pub min_order_in_base_asset: String,
    pub min_order_in_quote_asset: String,
    pub max_order_in_base_asset: String,
    pub max_order_in_quote_asset: String,
    pub order_types: Vec<String>,
}

/// The status of a market.
#[derive(Debug)]
pub enum MarketStatus {
    Trading,
    Halted,
    Auction,
}

impl<'de> Deserialize<'de> for MarketStatus {
    fn deserialize<D>(deserializer: D) -> crate::Result<MarketStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "trading" => Ok(MarketStatus::Trading),
            "halted" => Ok(MarketStatus::Halted),
            "auction" => Ok(MarketStatus::Auction),
            s => Err(D::Error::invalid_value(
                Unexpected::Str(s),
                &"[trading, halted, auction]",
            )),
        }
    }
}

/// Order book for a particular market.
#[derive(Debug, Deserialize)]
pub struct OrderBook {
    pub market: String,
    pub nonce: u64,
    pub bids: Vec<Quote>,
    pub asks: Vec<Quote>,
}

/// A quote in the order book.
#[derive(Debug)]
pub struct Quote {
    pub price: String,
    pub amount: String,
}

impl<'de> Deserialize<'de> for Quote {
    fn deserialize<D>(deserializer: D) -> crate::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QuoteVisitor;

        impl<'de> Visitor<'de> for QuoteVisitor {
            type Value = Quote;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Quote")
            }

            fn visit_seq<A>(self, mut seq: A) -> crate::Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                Ok(Quote {
                    price: next_seq_element!(seq, price),
                    amount: next_seq_element!(seq, amount),
                })
            }
        }

        deserializer.deserialize_seq(QuoteVisitor)
    }
}

/// A trade performed on the exchange for a particular market.
#[derive(Debug, Deserialize)]
pub struct Trade {
    pub id: String,
    pub timestamp: u64,
    pub amount: String,
    pub price: String,
    pub side: TradeSide,
}

/// The side of a trade.
#[derive(Debug)]
pub enum TradeSide {
    Buy,
    Sell,
}

impl<'de> Deserialize<'de> for TradeSide {
    fn deserialize<D>(deserializer: D) -> crate::Result<TradeSide, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "buy" => Ok(TradeSide::Buy),
            "sell" => Ok(TradeSide::Sell),
            s => Err(D::Error::invalid_value(Unexpected::Str(s), &"[buy, sell]")),
        }
    }
}

/// A ticker for a given market pair.
#[derive(Debug, Deserialize)]
pub struct TickerPrice {
    pub market: String,
    pub price: Option<String>,
}

/// Highest buy and lowest sell prices currently available for a market.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerBook {
    pub market: Option<String>,
    pub bid: Option<String>,
    pub bid_size: Option<String>,
    pub ask: Option<String>,
    pub ask_size: Option<String>,
}

/// High, low, open, last, and volume information for trades for a given market over the previous 24h.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24h {
    pub market: String,
    pub start_timestamp: Option<u64>,
    pub timestamp: Option<u64>,
    pub open: Option<String>,
    pub open_timestamp: Option<u64>,
    pub high: Option<String>,
    pub low: Option<String>,
    pub last: Option<String>,
    pub close_timestamp: Option<u64>,
    pub bid: Option<String>,
    pub bid_size: Option<String>,
    pub ask: Option<String>,
    pub ask_size: Option<String>,
    pub volume: Option<String>,
    pub volume_quote: Option<String>,
}

/// The fees for an account.
#[derive(Debug, Deserialize)]
pub struct Account {
    pub fees: AccountFees,
}

/// The fees in use for an account.
#[derive(Debug, Deserialize)]
pub struct AccountFees {
    pub taker: String,
    pub maker: String,
    pub volume: String,
}

/// The balance of an account in a particular asset.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub symbol: String,
    pub available: String,
    pub in_order: String,
}
