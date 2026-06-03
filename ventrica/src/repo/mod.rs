pub mod build;
pub mod client;
pub mod remote;

pub use build::build_repo;
pub use client::{check_updates, dep_store_paths, find_in_repos, install_from_repo, search_repos};
pub use remote::{fetch_manifest, fetch_manifest_cached, get_manifest, refresh_manifest_cache};

use crate::error::{Error, Result};
use crate::schema::kdl::Package;
use crate::schema::kdl::Repo;

pub const MANIFEST_FILE: &str = "manifest.msgpack";
pub const MANIFEST_HASH_FILE: &str = "manifest.msgpack.sha256";

pub fn mark_package_installed(package: &mut Package) {
    package.is_installed = Some(true);
}

pub fn mark_package_not_installed(package: &mut Package) {
    package.is_installed = Some(false);
}

pub fn run_dependencies(package: &Package) -> Vec<String> {
    package
        .dependencies
        .as_ref()
        .map(|deps| {
            deps.iter()
                .filter(|d| !d.is_build.unwrap_or(false))
                .filter_map(|d| d.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

pub fn encode_manifest(manifest: &Repo) -> Result<Vec<u8>> {
    rmp_serde::to_vec_named(manifest).map_err(|e| Error::Msgpack(e.to_string()))
}

pub fn decode_manifest(bytes: &[u8]) -> Result<Repo> {
    rmp_serde::from_slice(bytes).map_err(|e| Error::Msgpack(e.to_string()))
}
