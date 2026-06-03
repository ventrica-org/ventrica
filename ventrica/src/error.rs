use std::path::PathBuf;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("path error '{path}': {reason}")]
    Path { path: PathBuf, reason: String },

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("package '{name}' not found")]
    PackageNotFound { name: String },

    #[error("package '{name}-{version}' not found in store")]
    PackageVersionNotFound { name: String, version: String },

    #[error("package '{name}-{version}' is already installed")]
    AlreadyInstalled { name: String, version: String },

    #[error("version conflict for '{name}': {reason}")]
    VersionConflict { name: String, reason: String },

    #[error("version parse error: {0}")]
    VersionParse(String),

    #[error("build failed for '{name}': {reason}")]
    BuildFailed { name: String, reason: String },

    #[error("hash mismatch for '{path}': expected {expected}, got {actual}")]
    HashMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("sandbox initialisation failed: {0}")]
    Sandbox(String),

    #[error("download failed for '{url}': {reason}")]
    DownloadFailed { url: String, reason: String },

    #[error("archive extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("YAML parse error: {0}")]
    YamlParse(String),

    #[error("invalid package schema: {0}")]
    InvalidSchema(String),

    #[error("generation {0} not found")]
    GenerationNotFound(u32),

    #[error("no current generation exists")]
    NoCurrentGeneration,

    #[error("dependency resolution failed: {0}")]
    DependencyResolution(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("KDL parse error")]
    Kdl(#[from] kdl::de::Error),

    #[error("KDL encode error")]
    SKdl(#[from] kdl::se::Error),

    #[error("LZMA error: {0}")]
    Lzma(#[from] lzma_rs::error::Error),

    #[error("MessagePack error: {0}")]
    Msgpack(String),
}

impl Error {
    #[must_use]
    pub fn path(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::Path {
            path: path.into(),
            reason: reason.into(),
        }
    }
}
