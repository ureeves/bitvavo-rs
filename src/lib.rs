pub mod types;

use std::error::Error as StdError;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::Mac;
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use types::*;

/// Error type returned by the API.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
    Bitvavo { code: u64, message: String },
    InvalidSecret(BadSecret),
}

/// Error type for a bad secret.
#[derive(Debug)]
pub enum BadSecret {
    InvalidLength(hmac::digest::InvalidLength),
    Hex(hex::FromHexError),
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

impl From<hmac::digest::InvalidLength> for Error {
    fn from(err: hmac::digest::InvalidLength) -> Self {
        Self::InvalidSecret(BadSecret::InvalidLength(err))
    }
}

impl From<hex::FromHexError> for Error {
    fn from(err: hex::FromHexError) -> Self {
        Self::InvalidSecret(BadSecret::Hex(err))
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

    let s = String::from_utf8_lossy(&bytes);
    println!("{s}");

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
            Error::InvalidSecret(err) => match err {
                BadSecret::InvalidLength(err) => write!(f, "invalid secret: {err}"),
                BadSecret::Hex(err) => write!(f, "invalid secret: {err}"),
            },
        }
    }
}

impl StdError for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

struct Credentials {
    key: Zeroizing<String>,
    secret: Zeroizing<String>,
}

/// A client for the Bitvavo API.
pub struct Client {
    client: reqwest::Client,
    credentials: Option<Credentials>,
}

impl Client {
    /// Create a new client for the Bitvavo API.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            credentials: None,
        }
    }

    /// Create a new client for the Bitvavo API with credentials.
    pub fn with_credentials(key: String, secret: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            credentials: Some(Credentials {
                key: Zeroizing::new(key),
                secret: Zeroizing::new(secret),
            }),
        }
    }

    fn get(&self, endpoint: impl AsRef<str>) -> Result<reqwest::RequestBuilder> {
        let endpoint = endpoint.as_ref();
        let slug = format!("/v2/{endpoint}");

        let mut req = self.client.get(format!("https://api.bitvavo.com{slug}"));

        if let Some(credentials) = &self.credentials {
            let key = &*credentials.key;
            let secret = &*credentials.secret;

            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
                .to_string();

            type Hmac = hmac::Hmac<sha2::Sha256>;

            let mut hmac = Hmac::new_from_slice(secret.as_bytes())?;

            hmac = hmac
                .chain_update(&timestamp)
                .chain_update("GET")
                .chain_update(slug);

            let signature = hex::encode(hmac.finalize().into_bytes());

            req = req.header("Bitvavo-Access-Key", key);
            req = req.header("Bitvavo-Access-Timestamp", timestamp);
            req = req.header("Bitvavo-Access-Signature", signature);
        }

        Ok(req)
    }

    /// Get the current time.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let t = c.time().await.unwrap();
    ///
    /// println!("{t}");
    /// # })
    /// ```
    pub async fn time(&self) -> Result<u64> {
        #[derive(Deserialize, Serialize)]
        struct Response {
            time: u64,
        }

        let request = self.get("time")?;

        let http_response = request.send().await?;
        let response = response_from_request::<Response>(http_response).await?;

        Ok(response.time)
    }

    /// Get all the assets.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let assets = c.assets().await.unwrap();
    ///
    /// println!("Number of assets: {}", assets.len());
    /// # })
    pub async fn assets(&self) -> Result<Vec<Asset>> {
        let request = self.get("assets")?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Get the info of a particular asset.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let asset = c.asset("BTC").await.unwrap();
    ///
    /// println!("Number of decimals used for BTC: {}", asset.decimals);
    /// # })
    pub async fn asset(&self, symbol: &str) -> Result<Asset> {
        let request = self.get(format!("assets?symbol={symbol}"))?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Get all the markets.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let markets = c.markets().await.unwrap();
    ///
    /// println!("Number of markets: {}", markets.len());
    /// # })
    pub async fn markets(&self) -> Result<Vec<Market>> {
        let request = self.get("markets")?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Get market information for a specific market.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let market = c.market("BTC-EUR").await.unwrap();
    ///
    /// println!("Price precision of BTC-EUR: {}", market.price_precision);
    /// # })
    pub async fn market(&self, pair: &str) -> Result<Market> {
        let request = self.get(format!("markets?market={pair}"))?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Get the order book for a particular market.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let ob = c.order_book("BTC-EUR", Some(2)).await.unwrap();
    ///
    /// println!("Number of bids: {}", ob.bids.len());
    /// # })
    /// ```
    pub async fn order_book(&self, market: &str, depth: Option<u64>) -> Result<OrderBook> {
        let mut url = format!("{market}/book");

        if let Some(depth) = depth {
            url.push_str(&format!("?depth={depth}"));
        }

        let request = self.get(url)?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Get the trades for a particular market.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let trades = c.trades("BTC-EUR", None, None, None, None, None).await.unwrap();
    ///
    /// println!("Number of trades: {}", trades.len());
    /// # })
    /// ```
    pub async fn trades(
        &self,
        market: &str,
        limit: Option<u64>,
        start: Option<u64>,
        end: Option<u64>,
        trade_id_from: Option<String>,
        trade_id_to: Option<String>,
    ) -> Result<Vec<Trade>> {
        let mut url = format!("{market}/trades");

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

        let request = self.get(url)?;

        let http_response = request.send().await?;
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
    /// let c = bitvavo::Client::new();
    /// let cs = c.candles("BTC-EUR", CandleInterval::OneDay, Some(1), None, None).await.unwrap();
    ///
    /// println!("High for BTC-EUR at {} was {}", cs[0].time, cs[0].high);
    /// # })
    /// ```
    pub async fn candles(
        &self,
        market: &str,
        interval: CandleInterval,
        limit: Option<u16>,
        start: Option<u64>,
        end: Option<u64>,
    ) -> Result<Vec<OHLCV>> {
        let mut url = format!("{market}/candles?interval={interval}");

        if let Some(limit) = limit {
            url.push_str(&format!("&limit={limit}"));
        }
        if let Some(start) = start {
            url.push_str(&format!("&start={start}"));
        }
        if let Some(end) = end {
            url.push_str(&format!("&end={end}"));
        }

        let request = self.get(url)?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Get all the tickers.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let ms = c.ticker_prices().await.unwrap();
    ///
    /// println!("Number of markets: {}", ms.len());
    /// # })
    /// ```
    pub async fn ticker_prices(&self) -> Result<Vec<TickerPrice>> {
        let request = self.get("ticker/price")?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Get the ticker for a particular market.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let m = c.ticker_price("BTC-EUR").await.unwrap();
    ///
    /// println!("Price for BTC-EUR: {}", m.price.unwrap_or_default());
    /// # })
    /// ```
    pub async fn ticker_price(&self, pair: &str) -> Result<TickerPrice> {
        let request = self.get(format!("ticker/price?market={pair}"))?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Retrieve the highest buy and lowest sell prices currently available for all markets.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let tb = c.ticker_books().await.unwrap();
    ///
    /// println!("Number of tickers: {}", tb.len());
    /// # })
    /// ```
    pub async fn ticker_books(&self) -> Result<Vec<TickerBook>> {
        let request = self.get("ticker/book")?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Retrieve the highest buy and lowest sell prices currently available for a given market.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let tb = c.ticker_book("BTC-EUR").await.unwrap();
    ///
    /// println!("Highest buy price for BTC-EUR: {}", tb.ask.unwrap());
    /// # })
    /// ```
    pub async fn ticker_book(&self, market: &str) -> Result<TickerBook> {
        let request = self.get(format!("ticker/book?market={market}"))?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Retrieve high, low, open, last, and volume information for trades for all markets over the previous 24h.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let t24h = c.tickers_24h().await.unwrap();
    ///
    /// println!("Number of tickers: {}", t24h.len());
    /// # })
    /// ```
    pub async fn tickers_24h(&self) -> Result<Vec<Ticker24h>> {
        let request = self.get("ticker/24h")?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Retrieve high, low, open, last, and volume information for trades for a given market over the previous 24h.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let c = bitvavo::Client::new();
    /// let t24h = c.ticker_24h("BTC-EUR").await.unwrap();
    ///
    /// println!("24h ask for BTC-EUR: {}", t24h.ask.unwrap());
    /// # })
    /// ```
    pub async fn ticker_24h(&self, market: &str) -> Result<Ticker24h> {
        let request = self.get(format!("ticker/24h?market={market}"))?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Retrieve information about the account.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let key = String::from("YOUR_API_KEY");
    /// let secret = String::from("YOUR_API_SECRET");
    ///
    /// let c = bitvavo::Client::with_credentials(key, secret);
    /// let account = c.account().await.unwrap();
    ///
    /// println!("Fee for maker orders: {}", account.fees.maker);
    /// # })
    pub async fn account(&self) -> Result<Account> {
        let request = self.get("account")?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Retrieve balances for all assets in the account.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let key = String::from("YOUR_API_KEY");
    /// let secret = String::from("YOUR_API_SECRET");
    ///
    /// let c = bitvavo::Client::with_credentials(key, secret);
    /// let balances = c.balances().await.unwrap();
    ///
    /// println!("Number of assets held: {}", balances.len());
    /// # })
    pub async fn balances(&self) -> Result<Vec<Balance>> {
        let request = self.get("balance")?;

        let http_response = request.send().await?;
        let response = response_from_request(http_response).await?;

        Ok(response)
    }

    /// Retrieve balances for a particular asset in the account.
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use bitvavo_api as bitvavo;
    ///
    /// let key = String::from("YOUR_API_KEY");
    /// let secret = String::from("YOUR_API_SECRET");
    ///
    /// let c = bitvavo::Client::with_credentials(key, secret);
    /// let balance = c.balance("BTC").await.unwrap();
    ///
    /// println!("BTC available: {}", balance.available);
    /// # })
    pub async fn balance(&self, symbol: &str) -> Result<Balance> {
        let request = self.get(format!("balance?symbol={symbol}"))?;

        let http_response = request.send().await?;
        let response = response_from_request::<Vec<Balance>>(http_response).await?;

        Ok(response.into_iter().next().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_time() {
        let client = Client::new();
        client
            .time()
            .await
            .expect("Getting the time should succeed");
    }

    #[tokio::test]
    async fn get_assets() {
        let client = Client::new();
        client
            .assets()
            .await
            .expect("Getting the assets should succeed");
    }

    #[tokio::test]
    async fn get_asset() {
        let client = Client::new();
        client
            .asset("BTC")
            .await
            .expect("Getting the asset should succeed");
    }

    #[tokio::test]
    async fn get_markets() {
        let client = Client::new();
        client
            .markets()
            .await
            .expect("Getting the markets should succeed");
    }

    #[tokio::test]
    async fn get_market() {
        let client = Client::new();
        client
            .market("BTC-EUR")
            .await
            .expect("Getting the market should succeed");
    }

    #[tokio::test]
    async fn get_order_book() {
        let client = Client::new();
        client
            .order_book("BTC-EUR", Some(2))
            .await
            .expect("Getting the order book should succeed");
    }

    #[tokio::test]
    async fn get_trades() {
        let client = Client::new();
        client
            .trades("BTC-EUR", None, None, None, None, None)
            .await
            .expect("Getting the order book should succeed");
    }

    #[tokio::test]
    async fn get_candles() {
        let client = Client::new();
        client
            .candles("BTC-EUR", CandleInterval::OneDay, Some(1), None, None)
            .await
            .expect("Getting the candles should succeed");
    }

    #[tokio::test]
    async fn get_ticker_prices() {
        let client = Client::new();
        client
            .ticker_prices()
            .await
            .expect("Getting the markets should succeed");
    }

    #[tokio::test]
    async fn get_ticker_price() {
        let client = Client::new();
        client
            .ticker_price("BTC-EUR")
            .await
            .expect("Getting the market should succeed");
    }

    #[tokio::test]
    async fn get_ticker_books() {
        let client = Client::new();
        client
            .ticker_books()
            .await
            .expect("Getting the ticker books should succeed");
    }

    #[tokio::test]
    async fn get_ticker_book() {
        let client = Client::new();
        client
            .ticker_book("BTC-EUR")
            .await
            .expect("Getting the ticker book should succeed");
    }

    #[tokio::test]
    async fn get_tickers_24h() {
        let client = Client::new();
        client
            .tickers_24h()
            .await
            .expect("Getting the 24h tickers should succeed");
    }

    #[tokio::test]
    async fn get_ticker_24h() {
        let client = Client::new();
        client
            .ticker_24h("BTC-EUR")
            .await
            .expect("Getting the 24h tickers should succeed");
    }

    #[tokio::test]
    async fn error_handling() {
        let client = Client::new();

        let err = client
            .ticker_price("BAD-MARKET")
            .await
            .expect_err("Getting an invalid market should fail");

        assert!(matches!(err, Error::Bitvavo { .. }));
    }

    #[cfg(feature = "auth-tests")]
    mod auth {
        use super::*;

        const API_KEY: &str = env!(
            "BITVAVO_API_KEY",
            "Testing authenticated endpoints requires an API key"
        );

        const API_SECRET: &str = env!(
            "BITVAVO_API_SECRET",
            "Testing authenticated endpoints requires an API secret"
        );

        #[tokio::test]
        async fn get_account() -> Result<()> {
            let client = Client::with_credentials(API_KEY.to_string(), API_SECRET.to_string());

            client
                .account()
                .await
                .expect("Getting the account should succeed");

            Ok(())
        }

        #[tokio::test]
        async fn get_balances() -> Result<()> {
            let client = Client::with_credentials(API_KEY.to_string(), API_SECRET.to_string());

            client
                .balances()
                .await
                .expect("Getting the balances of the account should succeed");

            Ok(())
        }

        #[tokio::test]
        async fn get_balance() -> Result<()> {
            let client = Client::with_credentials(API_KEY.to_string(), API_SECRET.to_string());

            client
                .balance("BTC")
                .await
                .expect("Getting the balance of the account in a given asset should succeed");

            Ok(())
        }
    }
}
