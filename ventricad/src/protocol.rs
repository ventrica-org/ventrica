use serde::{Deserialize, Serialize};

pub const DEFAULT_SOCKET: &str = "/var/run/ventricad.sock";
pub const SOCKET_ENV: &str = "VENTRICA_SOCKET";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Install { names: Vec<String> },
    Remove { names: Vec<String> },
    Upgrade { names: Vec<String> },
    Rollback { generation: Option<u32> },
    ListPackages,
    ListGenerations,
    Gc,
    AddRepo { url: String },
    UpdateRepos,
    Search { query: String },
    BuildRepo { repo_dir: String },
    ListRepos,
    RemoveRepo { url: String },
    ListRepoPackages { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Message {
    Success(String),
    Error(String),
    Data(serde_json::Value),
    Done,
}

impl Message {
    pub fn success(s: impl Into<String>) -> Self {
        Self::Success(s.into())
    }
    pub fn error(s: impl Into<String>) -> Self {
        Self::Error(s.into())
    }
}
