use std::error::Error as StdError;
use std::fmt;

use reqwest::Response;
use serde::de::{DeserializeOwned, Error as SerdeError, SeqAccess, Unexpected, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Error type returned by the API.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Bitvavo { code: u64, message: String },
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err)
    }
}

async fn response_from_request<T: DeserializeOwned>(rsp: Response) -> Result<T, Error> {
    #[derive(Deserialize, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct BitvavoError {
        error_code: u64,
        error: String,
    }

    let status = rsp.status();
    let bytes = rsp.bytes().await?;

    if status.is_success() {
        Ok(serde_json::from_slice(&bytes)?)
    } else {
        let bitvavo_err: BitvavoError = serde_json::from_slice(&bytes)?;
        Err(Error::Bitvavo {
            code: bitvavo_err.error_code,
            message: bitvavo_err.error,
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Reqwest(err) => write!(f, "reqwest: {err}"),
            Error::Serde(err) => write!(f, "serde: {err}"),
            Error::Bitvavo { code, message } => {
                write!(f, "bitvavo: {code}: {message}")
            }
        }
    }
}

impl StdError for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Get the current time.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let t = bitvavo::time().await.unwrap();
/// println!("{t}");
/// # })
/// ```
pub async fn time() -> Result<u64> {
    #[derive(Deserialize, Serialize)]
    struct Response {
        time: u64,
    }

    let http_response = reqwest::get("https://api.bitvavo.com/v2/time").await?;
    let response = response_from_request::<Response>(http_response).await?;

    Ok(response.time)
}

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

#[derive(Debug)]
pub struct OHLCV {
    pub time: u64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

impl Serialize for OHLCV {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(6))?;

        seq.serialize_element(&self.time)?;
        seq.serialize_element(&self.open)?;
        seq.serialize_element(&self.high)?;
        seq.serialize_element(&self.low)?;
        seq.serialize_element(&self.close)?;
        seq.serialize_element(&self.volume)?;

        seq.end()
    }
}

impl<'de> Deserialize<'de> for OHLCV {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OHLCVVisitor;

        impl<'de> Visitor<'de> for OHLCVVisitor {
            type Value = OHLCV;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "OHLCV")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                macro_rules! next_seq_element {
                    ($seq:ident, $name:ident) => {
                        $seq.next_element()?
                            .ok_or_else(|| A::Error::missing_field(stringify!($name)))?
                    };
                }

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
#[derive(Debug, Deserialize, Serialize)]
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

impl Serialize for AssetStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AssetStatus::Ok => serializer.serialize_str("OK"),
            AssetStatus::Maintenance => serializer.serialize_str("MAINTENANCE"),
            AssetStatus::Delisted => serializer.serialize_str("DELISTED"),
        }
    }
}

impl<'de> Deserialize<'de> for AssetStatus {
    fn deserialize<D>(deserializer: D) -> Result<AssetStatus, D::Error>
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

/// Get all the assets.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let assets = bitvavo::assets().await.unwrap();
/// println!("Number of assets: {}", assets.len());
/// # })
pub async fn assets() -> Result<Vec<Asset>> {
    let http_response = reqwest::get("https://api.bitvavo.com/v2/assets").await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// Get the info of a particular asset.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let asset = bitvavo::asset("BTC").await.unwrap();
/// println!("Number of decimals used for BTC: {}", asset.decimals);
/// # })
pub async fn asset(symbol: &str) -> Result<Asset> {
    let http_response =
        reqwest::get(format!("https://api.bitvavo.com/v2/assets?symbol={symbol}")).await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

#[derive(Debug, Deserialize, Serialize)]
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

impl Serialize for MarketStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MarketStatus::Trading => serializer.serialize_str("trading"),
            MarketStatus::Halted => serializer.serialize_str("halted"),
            MarketStatus::Auction => serializer.serialize_str("auction"),
        }
    }
}

impl<'de> Deserialize<'de> for MarketStatus {
    fn deserialize<D>(deserializer: D) -> Result<MarketStatus, D::Error>
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

/// Get all the markets.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let markets = bitvavo::markets().await.unwrap();
/// println!("Number of markets: {}", markets.len());
/// # })
pub async fn markets() -> Result<Vec<Market>> {
    let http_response = reqwest::get("https://api.bitvavo.com/v2/markets").await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// Get market information for a specific market.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let market = bitvavo::market("BTC-EUR").await.unwrap();
/// println!("Price precision of BTC-EUR: {}", market.price_precision);
/// # })
pub async fn market(pair: &str) -> Result<Market> {
    let http_response =
        reqwest::get(format!("https://api.bitvavo.com/v2/markets?market={pair}")).await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// Order book for a particular market.
#[derive(Debug, Deserialize, Serialize)]
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

impl Serialize for Quote {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(3))?;

        seq.serialize_element(&self.price)?;
        seq.serialize_element(&self.amount)?;

        seq.end()
    }
}

impl<'de> Deserialize<'de> for Quote {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QuoteVisitor;

        impl<'de> Visitor<'de> for QuoteVisitor {
            type Value = Quote;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Quote")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                macro_rules! next_seq_element {
                    ($seq:ident, $name:ident) => {
                        $seq.next_element()?
                            .ok_or_else(|| A::Error::missing_field(stringify!($name)))?
                    };
                }

                Ok(Quote {
                    price: next_seq_element!(seq, price),
                    amount: next_seq_element!(seq, amount),
                })
            }
        }

        deserializer.deserialize_seq(QuoteVisitor)
    }
}

/// Get the order book for a particular market.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let ob = bitvavo::order_book("BTC-EUR", Some(2)).await.unwrap();
/// println!("Number of bids: {}", ob.bids.len());
/// # })
/// ```
pub async fn order_book(market: &str, depth: Option<u64>) -> Result<OrderBook> {
    let mut url = format!("https://api.bitvavo.com/v2/{market}/book");

    if let Some(depth) = depth {
        url.push_str(&format!("?depth={depth}"));
    }

    let http_response = reqwest::get(url).await?;
    let response = response_from_request(http_response).await?;

    Ok(response)
}

/// A trade performed on the exchange for a particular market.
#[derive(Debug, Deserialize, Serialize)]
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
        S: Serializer,
    {
        match self {
            TradeSide::Buy => serializer.serialize_str("buy"),
            TradeSide::Sell => serializer.serialize_str("sell"),
        }
    }
}

impl<'de> Deserialize<'de> for TradeSide {
    fn deserialize<D>(deserializer: D) -> Result<TradeSide, D::Error>
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

/// Get the trades for a particular market.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let trades = bitvavo::trades("BTC-EUR", None, None, None, None, None).await.unwrap();
/// println!("Number of trades: {}", trades.len());
/// # })
/// ```
pub async fn trades(
    market: &str,
    limit: Option<u64>,
    start: Option<u64>,
    end: Option<u64>,
    trade_id_from: Option<String>,
    trade_id_to: Option<String>,
) -> Result<Vec<Trade>> {
    let mut url = format!("https://api.bitvavo.com/v2/{market}/trades");

    if let Some(limit) = limit {
        url.push_str(&format!("?limit={limit}"));
    }
    if let Some(start) = start {
        url.push_str(&format!("&start={start}"));
    }
    if let Some(end) = end {
        url.push_str(&format!("&end={end}"));
    }
    if let Some(trade_id_from) = trade_id_from {
        url.push_str(&format!("&tradeIdFrom={trade_id_from}"));
    }
    if let Some(trade_id_to) = trade_id_to {
        url.push_str(&format!("&tradeIdTo={trade_id_to}"));
    }

    let http_response = reqwest::get(url).await?;
    let response = response_from_request(http_response).await?;

    Ok(response)
}

/// Get candles for a particular market.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
/// use bitvavo::CandleInterval;
///
/// let cs = bitvavo::candles("BTC-EUR", CandleInterval::OneDay, Some(1), None, None).await.unwrap();
/// println!("High for BTC-EUR at {} was {}", cs[0].time, cs[0].high);
/// # })
/// ```
pub async fn candles(
    market: &str,
    interval: CandleInterval,
    limit: Option<u16>,
    start: Option<u64>,
    end: Option<u64>,
) -> Result<Vec<OHLCV>> {
    let mut url = format!("https://api.bitvavo.com/v2/{market}/candles?interval={interval}");

    if let Some(limit) = limit {
        url.push_str(&format!("&limit={limit}"));
    }
    if let Some(start) = start {
        url.push_str(&format!("&start={start}"));
    }
    if let Some(end) = end {
        url.push_str(&format!("&end={end}"));
    }

    let http_response = reqwest::get(url).await?;
    let response = response_from_request(http_response).await?;

    Ok(response)
}

/// A ticker for a given market pair.
#[derive(Debug, Deserialize, Serialize)]
pub struct TickerPrice {
    pub market: String,
    pub price: Option<String>,
}

/// Get all the tickers.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let ms = bitvavo::ticker_prices().await.unwrap();
/// println!("Number of markets: {}", ms.len());
/// # })
/// ```
pub async fn ticker_prices() -> Result<Vec<TickerPrice>> {
    let http_response = reqwest::get("https://api.bitvavo.com/v2/ticker/price").await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// Get the ticker for a particular market.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let m = bitvavo::ticker_price("BTC-EUR").await.unwrap();
/// println!("Price for BTC-EUR: {}", m.price.unwrap_or_default());
/// # })
/// ```
pub async fn ticker_price(pair: &str) -> Result<TickerPrice> {
    let http_response = reqwest::get(format!(
        "https://api.bitvavo.com/v2/ticker/price?market={pair}"
    ))
    .await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// Highest buy and lowest sell prices currently available for a market.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickerBook {
    pub market: Option<String>,
    pub bid: Option<String>,
    pub bid_size: Option<String>,
    pub ask: Option<String>,
    pub ask_size: Option<String>,
}

/// Retrieve the highest buy and lowest sell prices currently available for all markets.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let tb = bitvavo::ticker_books().await.unwrap();
/// println!("Number of tickers: {}", tb.len());
/// # })
/// ```
pub async fn ticker_books() -> Result<Vec<TickerBook>> {
    let http_response = reqwest::get("https://api.bitvavo.com/v2/ticker/book").await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// Retrieve the highest buy and lowest sell prices currently available for a given market.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let tb = bitvavo::ticker_book("BTC-EUR").await.unwrap();
/// println!("Highest buy price for BTC-EUR: {}", tb.ask.unwrap());
/// # })
/// ```
pub async fn ticker_book(market: &str) -> Result<TickerBook> {
    let http_response = reqwest::get(format!(
        "https://api.bitvavo.com/v2/ticker/book?market={market}"
    ))
    .await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// High, low, open, last, and volume information for trades for a given market over the previous 24h.
#[derive(Debug, Deserialize, Serialize)]
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

/// Retrieve high, low, open, last, and volume information for trades for all markets over the previous 24h.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let t24h = bitvavo::tickers_24h().await.unwrap();
/// println!("Number of tickers: {}", t24h.len());
/// # })
/// ```
pub async fn tickers_24h() -> Result<Vec<Ticker24h>> {
    let http_response = reqwest::get("https://api.bitvavo.com/v2/ticker/24h").await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

/// Retrieve high, low, open, last, and volume information for trades for a given market over the previous 24h.
///
/// ```no_run
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let t24h = bitvavo::ticker_24h("BTC-EUR").await.unwrap();
/// println!("24h ask for BTC-EUR: {}", t24h.ask.unwrap());
/// # })
/// ```
pub async fn ticker_24h(market: &str) -> Result<Ticker24h> {
    let http_response = reqwest::get(format!(
        "https://api.bitvavo.com/v2/ticker/24h?market={market}"
    ))
    .await?;
    let response = response_from_request(http_response).await?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_time() {
        time().await.expect("Getting the time should succeed");
    }

    #[tokio::test]
    async fn get_assets() {
        assets().await.expect("Getting the assets should succeed");
    }

    #[tokio::test]
    async fn get_asset() {
        asset("BTC")
            .await
            .expect("Getting the asset should succeed");
    }

    #[tokio::test]
    async fn get_markets() {
        markets().await.expect("Getting the markets should succeed");
    }

    #[tokio::test]
    async fn get_market() {
        market("BTC-EUR")
            .await
            .expect("Getting the market should succeed");
    }

    #[tokio::test]
    async fn get_order_book() {
        order_book("BTC-EUR", Some(2))
            .await
            .expect("Getting the order book should succeed");
    }

    #[tokio::test]
    async fn get_trades() {
        trades("BTC-EUR", None, None, None, None, None)
            .await
            .expect("Getting the order book should succeed");
    }

    #[tokio::test]
    async fn get_candles() {
        candles("BTC-EUR", CandleInterval::OneDay, Some(1), None, None)
            .await
            .expect("Getting the candles should succeed");
    }

    #[tokio::test]
    async fn get_ticker_prices() {
        ticker_prices()
            .await
            .expect("Getting the markets should succeed");
    }

    #[tokio::test]
    async fn get_ticker_price() {
        ticker_price("BTC-EUR")
            .await
            .expect("Getting the market should succeed");
    }

    #[tokio::test]
    async fn get_ticker_books() {
        ticker_books()
            .await
            .expect("Getting the ticker books should succeed");
    }

    #[tokio::test]
    async fn get_ticker_book() {
        ticker_book("BTC-EUR")
            .await
            .expect("Getting the ticker book should succeed");
    }

    #[tokio::test]
    async fn get_tickers_24h() {
        tickers_24h()
            .await
            .expect("Getting the 24h tickers should succeed");
    }

    #[tokio::test]
    async fn get_ticker_24h() {
        ticker_24h("BTC-EUR")
            .await
            .expect("Getting the 24h tickers should succeed");
    }
}
