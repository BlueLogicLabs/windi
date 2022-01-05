use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct LogEntry {
  pub ts: u64,
  pub data: LogContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum LogContent {
  CreateNote {
    id: String,
    user: String,
    note: StoredNote,
    origin: Option<LogContentCreateNoteOrigin>,
  },
  UpdateNote {
    id: String,
    user: String,
    note: StoredNote,
  },
  DeleteNote {
    id: String,
    user: String,
    note: StoredNote,
  },
  CreateIngress {
    user: String,
    ingress: String,
    sub: String,
  },
  DeleteIngress {
    user: String,
    ingress: String,
    sub: String,
  },
  UserSubscriptionPaid {
    user: String,
    #[serde(rename = "subId")]
    sub_id: String,
  },
  UserSubscriptionCancel {
    user: String,
    #[serde(rename = "subId")]
    sub_id: String,
  },
  UserSubscriptionEnd {
    user: String,
    #[serde(rename = "subId")]
    sub_id: String,
  },
  UserSyncPull {
    user: String,
    #[serde(rename = "tokenTs")]
    token_ts: u64,
    #[serde(rename = "fromSeq")]
    from_seq: String,
  },
  UserSyncCreateToken {
    user: String,
    #[serde(rename = "tokenTs")]
    token_ts: u64,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum LogContentCreateNoteOrigin {
  Web,
  Telegram {
    #[serde(rename = "userId")]
    user_id: u64,
  },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StoredNote {
  pub real_ts: u64,
  pub content: String,
  pub private: Option<bool>,
  pub deliverable_ts: Option<u64>,
  pub forward_links: Option<Vec<NoteLinkWithPosition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NoteLinkWithPosition {
  pub username: String,
  pub full_id: String,
  pub position: u64,
  pub text: String,
}
