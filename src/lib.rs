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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_time() {
        time().await.expect("Getting the time should succeed");
    }
}
