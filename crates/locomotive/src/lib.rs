#[cfg(target_os = "windows")]
compile_error!("Ventrica is not supported on Windows.");

#[cfg(feature = "builder")]
pub mod builder;
pub mod db;
pub mod network;
pub mod platform;
pub mod store;
pub(crate) mod utils;

use concat_const::concat;

#[const_env::env_item]
pub const PREFIX: &'static str = "/opt/ventrica";
pub const VENTRICA_USR_PATH: &'static str = concat!(PREFIX, "/usr");
pub const VENTRICA_DB_PATH: &'static str = concat!(PREFIX, "/var/lib/ventrica/db.sqlite");
pub const VENTRICA_STATIONS_PATH: &'static str = concat!(PREFIX, "/usr/var/lib/ventrica/stations");

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    // builder
    #[cfg(feature = "builder")]
    #[error("Invalid repository path: {0}")]
    BuilderRepoInvalid(String),

    #[cfg(feature = "builder")]
    #[error("Failed to download package '{name}' with SHA256 '{sha256}'")]
    BuilderDownloadFailed { name: String, sha256: String },

    #[cfg(feature = "builder")]
    #[error("Invalid build type: {0}")]
    BuilderTypeInvalid(String),

    #[cfg(feature = "builder")]
    #[error("Failed to run builder command '{name}' with status '{status}'")]
    BuilderCommandFailed {
        name: String,
        status: std::process::ExitStatus,
    },

    #[cfg(feature = "builder")]
    #[error("MacOS application not found")]
    BuilderMacOSAppNotFound,

    #[cfg(feature = "builder")]
    #[error("Invalid data: {message}")]
    BuilderInvalidData { message: String },

    // network
    #[error("Station error: {0}")]
    NetworkInvalidUrl(String),

    #[error("Station hash mismatch: expected '{expected}', got '{actual}'")]
    NetworkHashMismatch { expected: String, actual: String },

    // store
    #[error("Store path is not registered: {0}")]
    StoreInvalidPath(String),

    #[error("No previous generations to roll back to")]
    StoreNoGenerations,

    #[error("Generation {0} does not exist")]
    StoreGenerationNotFound(u32),

    // station
    #[error("Invalid station reader path: {0}")]
    StationReaderInvalidPath(String),

    // crates
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Bincode decode error: {0}")]
    BincodeDecode(#[from] bincode_next::error::DecodeError),

    #[error("Bincode encode error: {0}")]
    BincodeEncode(#[from] bincode_next::error::EncodeError),

    #[error("KDL parse error")]
    KdlDecode(#[from] kdl::de::Error),

    #[error("KDL encode error")]
    KdlEncode(#[from] kdl::se::Error),

    #[error("Database error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
}
