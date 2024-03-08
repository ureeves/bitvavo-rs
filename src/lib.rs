use std::error::Error as StdError;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// Error type returned by the API.
#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Reqwest(err) => write!(f, "reqwest: {err}"),
            Error::Serde(err) => write!(f, "serde: {err}"),
        }
    }
}

impl StdError for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Get the current time.
///
/// ```no_run
///
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
    let body_bytes = http_response.bytes().await?;

    let response = serde_json::from_slice::<Response>(&body_bytes)?;

    Ok(response.time)
}

/// A ticker for a given market pair.
#[derive(Deserialize, Serialize)]
pub struct Ticker {
    pub market: String,
    pub price: Option<String>,
}

/// Get all the tickers.
///
/// ```no_run
///
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let ms = bitvavo::tickers().await.unwrap();
/// println!("Number of markets: {}", ms.len());
/// # })
/// ```
pub async fn tickers() -> Result<Vec<Ticker>> {
    let http_response = reqwest::get("https://api.bitvavo.com/v2/ticker/price").await?;
    let body_bytes = http_response.bytes().await?;

    let response = serde_json::from_slice(&body_bytes)?;

    Ok(response)
}

/// Get the ticker for a particular market.
///
/// ```no_run
///
/// # tokio_test::block_on(async {
/// use bitvavo_api as bitvavo;
///
/// let m = bitvavo::ticker("BTC-EUR").await.unwrap();
/// println!("Price for BTC-EUR: {}", m.price.unwrap_or_default());
/// # })
/// ```
pub async fn ticker(pair: &str) -> Result<Ticker> {
    let http_response = reqwest::get(format!(
        "https://api.bitvavo.com/v2/ticker/price?market={pair}"
    ))
    .await?;
    let body_bytes = http_response.bytes().await?;

    let response = serde_json::from_slice(&body_bytes)?;

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
    async fn get_tickers() {
        tickers().await.expect("Getting the markets should succeed");
    }

    #[tokio::test]
    async fn get_ticker() {
        ticker("BTC-EUR")
            .await
            .expect("Getting the markets should succeed");
    }
}
