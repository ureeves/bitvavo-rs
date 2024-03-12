pub mod types;

use std::error::Error as StdError;
use std::fmt;

use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use types::*;

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
/// use bitvavo::types::CandleInterval;
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

    #[tokio::test]
    async fn error_handling() {
        let err = ticker_price("BAD-MARKET")
            .await
            .expect_err("Getting an invalid market should fail");
        assert!(matches!(err, Error::Bitvavo { .. }));
    }
}
