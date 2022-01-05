use anyhow::Result;
use serde::{Deserialize, Serialize};

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
    let rsp = self.http.post(&url).json(&req).send().await?;
    let status = rsp.status();
    if !status.is_success() {
      anyhow::bail!("server error: {} {}", status, rsp.text().await?);
    }
    let rsp: ApiSyncPullRsp = rsp.json().await?;
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
