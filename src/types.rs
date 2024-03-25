use std::fmt;

use serde::de::{Error, SeqAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use uuid::Uuid;

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

impl Serialize for TradeSide {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TradeSide::Buy => serializer.serialize_str("buy"),
            TradeSide::Sell => serializer.serialize_str("sell"),
        }
    }
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

/// Fees charged for a market on an account.
#[derive(Debug, Deserialize)]
pub struct Fees {
    pub tier: u64,
    pub volume: String,
    pub taker: String,
    pub maker: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositInfo {
    pub address: String,
    pub payment_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deposit {
    pub timestamp: u64,
    pub symbol: String,
    pub amount: String,
    pub fee: String,
    pub status: DepositStatus,
    pub tx_id: Option<String>,
    pub address: Option<String>,
    pub payment_id: Option<String>,
}

/// The status of a deposit.
#[derive(Debug)]
pub enum DepositStatus {
    Completed,
    Canceled,
}

impl<'de> Deserialize<'de> for DepositStatus {
    fn deserialize<D>(deserializer: D) -> crate::Result<DepositStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "completed" => Ok(DepositStatus::Completed),
            "canceled" => Ok(DepositStatus::Canceled),
            s => Err(D::Error::invalid_value(
                Unexpected::Str(s),
                &"[completed, canceled]",
            )),
        }
    }
}

/// Information about a withdrawal.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Withdrawal {
    pub timestamp: u64,
    pub symbol: String,
    pub amount: String,
    pub address: Option<String>,
    pub payment_id: Option<String>,
    pub tx_id: Option<String>,
    pub fee: String,
    pub status: WithdrawalStatus,
}

/// The status of a withdrawal.
#[derive(Debug)]
pub enum WithdrawalStatus {
    AwaitingProcessing,
    AwaitingEmailConfirmation,
    AwaitingBitvavoInspection,
    Approved,
    Sending,
    InMempool,
    Processed,
    Completed,
    Canceled,
}

impl<'de> Deserialize<'de> for WithdrawalStatus {
    fn deserialize<D>(deserializer: D) -> crate::Result<WithdrawalStatus, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.as_str() {
            "awaiting_processing" => Ok(WithdrawalStatus::AwaitingProcessing),
            "awaiting_email_confirmation" => Ok(WithdrawalStatus::AwaitingEmailConfirmation),
            "awaiting_bitvavo_inspection" => Ok(WithdrawalStatus::AwaitingBitvavoInspection),
            "approved" => Ok(WithdrawalStatus::Approved),
            "sending" => Ok(WithdrawalStatus::Sending),
            "in_mempool" => Ok(WithdrawalStatus::InMempool),
            "processed" => Ok(WithdrawalStatus::Processed),
            "completed" => Ok(WithdrawalStatus::Completed),
            "canceled" => Ok(WithdrawalStatus::Canceled),
            s => Err(D::Error::invalid_value(
                Unexpected::Str(s),
                &"[awaiting_processing, awaiting_email_confirmation, awaiting_bitvavo_inspection, approved, sending, in_mempool, processed, completed, canceled]",
            )),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawOrder {
    pub symbol: String,
    pub amount: String,
    pub address: String,
    pub payment_id: Option<String>,
    pub internal: bool,
    pub add_withdrawal_fee: bool,
}

#[derive(Debug, Deserialize)]
pub struct WithdrawalOrderResponse {
    pub success: bool,
    pub symbol: String,
    pub amount: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub market: String,
    pub side: TradeSide,
    pub order_type: OrderType,
    pub client_order_id: Option<Uuid>,
    pub amount: Option<String>,
    pub amount_quote: Option<String>,
    pub price: Option<String>,
    pub trigger_amount: Option<String>,
    pub trigger_type: Option<TriggerType>,
    pub trigger_reference: Option<TriggerReference>,
    pub time_in_force: Option<TimeInForce>,
    pub post_only: Option<bool>,
    pub self_trade_prevention: Option<SelfTradePrevention>,
    pub disable_market_protection: bool,
    pub response_required: bool,
}

/// The type of order.
#[derive(Debug)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
}

impl Serialize for OrderType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            OrderType::Market => serializer.serialize_str("market"),
            OrderType::Limit => serializer.serialize_str("limit"),
            OrderType::StopLoss => serializer.serialize_str("stopLoss"),
            OrderType::StopLossLimit => serializer.serialize_str("stopLossLimit"),
            OrderType::TakeProfit => serializer.serialize_str("takeProfit"),
            OrderType::TakeProfitLimit => serializer.serialize_str("takeProfitLimit"),
        }
    }
}

/// The type of trigger that will cause an order to be filled.
#[derive(Debug)]
pub enum TriggerType {
    Price,
}

impl Serialize for TriggerType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TriggerType::Price => serializer.serialize_str("price"),
        }
    }
}

/// The price type that triggers an order to be filled.
#[derive(Debug)]
pub enum TriggerReference {
    LastTrade,
    BestBid,
    BestAsk,
    MidPrice,
}

impl Serialize for TriggerReference {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TriggerReference::LastTrade => serializer.serialize_str("lastTrade"),
            TriggerReference::BestBid => serializer.serialize_str("bestBid"),
            TriggerReference::BestAsk => serializer.serialize_str("bestAsk"),
            TriggerReference::MidPrice => serializer.serialize_str("midPrice"),
        }
    }
}

/// How long an order should remain active.
#[derive(Debug)]
pub enum TimeInForce {
    GoodTillCancelled,
    FillOrKill,
    ImmediateOrCancel,
}

impl Serialize for TimeInForce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TimeInForce::GoodTillCancelled => serializer.serialize_str("GTC"),
            TimeInForce::FillOrKill => serializer.serialize_str("FOK"),
            TimeInForce::ImmediateOrCancel => serializer.serialize_str("IOC"),
        }
    }
}

/// How to handle self trades.
#[derive(Debug)]
pub enum SelfTradePrevention {
    DecrementAndCancel,
    CancelBoth,
    CancelNewest,
    CancelOldest,
}

impl Serialize for SelfTradePrevention {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SelfTradePrevention::DecrementAndCancel => {
                serializer.serialize_str("decrementAndCancel")
            }
            SelfTradePrevention::CancelBoth => serializer.serialize_str("cancelBoth"),
            SelfTradePrevention::CancelNewest => serializer.serialize_str("cancelNewest"),
            SelfTradePrevention::CancelOldest => serializer.serialize_str("cancelOldest"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub market: String,
    pub order_id: Uuid,
    pub client_order_id: Option<Uuid>,
    pub created: u64,
    pub updated: u64,
}
