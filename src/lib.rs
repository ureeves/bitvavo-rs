use std::error::Error as StdError;
use std::fmt;

use serde::de::{Error as SerdeError, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Time interval between each candlestick.
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

pub struct OHLCV {
    pub time: u64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

impl Serialize for OHLCV {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
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

/// Get candles for a particular market.
///
/// ```no_run
///
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

    #[tokio::test]
    async fn get_candles() {
        candles("BTC-EUR", CandleInterval::OneDay, Some(1), None, None)
            .await
            .expect("Getting the candles should succeed");
    }
}
