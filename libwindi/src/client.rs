use anyhow::Result;
use backoff::ExponentialBackoff;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::LogEntry;

static DEFAULT_SERVICE_URL: &'static str = "https://bluebird.ink";

#[derive(Default, Clone, Debug)]
#[non_exhaustive]
pub struct ClientConfig {
  pub url: Option<String>,
  pub token: String,
}

pub struct Client {
  config: ClientConfig,
  http: reqwest::Client,
}

pub struct LogEntryWithSeq {
  pub seq: u128,
  pub log: Result<LogEntry, String>,
}

#[derive(Serialize)]
struct ApiSyncPullReq {
  #[serde(rename = "fromSeq")]
  from_seq: String,
}

#[derive(Deserialize)]
struct ApiSyncPullRsp {
  data: Vec<RawLogEntryWithSeq>,
}

#[derive(Deserialize)]
pub struct RawLogEntryWithSeq {
  pub seq: String,
  pub value: String,
}

impl Client {
  pub fn new(config: ClientConfig) -> Result<Self> {
    // Create reqwest client with "Authorization" header
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
      reqwest::header::AUTHORIZATION,
      reqwest::header::HeaderValue::from_str(&format!("Bearer {}", config.token))?,
    );
    let http = reqwest::Client::builder()
      .default_headers(headers)
      .build()?;
    Ok(Self { config, http })
  }

  pub async fn pull(&self, from_seq: u128) -> Result<Vec<LogEntryWithSeq>> {
    let from_seq = hex::encode(&from_seq.to_be_bytes()[..]);
    let url = self
      .config
      .url
      .clone()
      .unwrap_or_else(|| DEFAULT_SERVICE_URL.to_string());
    let url = format!("{}/api/v1/sync/pull", url);
    let req = ApiSyncPullReq { from_seq };
    let rsp: ApiSyncPullRsp = backoff::future::retry(ExponentialBackoff::default(), || {
      #[derive(Error, Debug)]
      #[error("give up")]
      struct GiveUp;
      async {
        let rsp = self.http.post(&url).json(&req).send().await?;
        let status = rsp.status();
        if !status.is_success() {
          let text = rsp.text().await?;
          if status.is_client_error() {
            log::error!("client error, not retrying: {} {}", status, text);
            Err(GiveUp.into())
          } else {
            anyhow::bail!("server error: {} {}", status, text);
          }
        } else {
          Ok(rsp.json().await?)
        }
      }
      .map_err(|e| {
        if e.downcast_ref::<GiveUp>().is_some() {
          backoff::Error::Permanent(e)
        } else {
          log::error!("retryable pull error: {}", e);
          backoff::Error::Transient {
            err: anyhow::anyhow!("unable to pull logs from service"),
            retry_after: None,
          }
        }
      })
    })
    .await?;
    let mut out: Vec<LogEntryWithSeq> = vec![];
    for raw in rsp.data {
      let mut seq = hex::decode(&raw.seq)?;
      seq.reverse();
      seq.resize(16, 0);
      let seq = u128::from_le_bytes(<[u8; 16]>::try_from(seq).unwrap());
      let log: Result<LogEntry, String> = serde_json::from_str(&raw.value).map_err(|_| raw.value);
      out.push(LogEntryWithSeq { seq, log });
    }
    Ok(out)
  }
}
