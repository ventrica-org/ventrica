pub mod build;
pub mod client;
pub mod remote;

pub use build::build_repo;
pub use client::{check_updates, dep_store_paths, find_in_repos, install_from_repo, search_repos};
pub use remote::{fetch_manifest, fetch_manifest_cached, get_manifest, refresh_manifest_cache};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::kdl::Package;
use crate::schema::kdl::Repo;

pub const MANIFEST_FILE: &str = "manifest.msgpack";
pub const MANIFEST_HASH_FILE: &str = "manifest.msgpack.sha256";

pub fn mark_package_installed(package: &mut Package, package_hash: Option<String>) {
    package.is_installed = Some(true);
    package.package_hash = package_hash;
}

pub fn mark_package_not_installed(package: &mut Package) {
    package.is_installed = Some(false);
    package.package_hash = None;
}

pub fn run_dependencies(package: &Package) -> Vec<String> {
    package
        .dependencies
        .as_ref()
        .map(|deps| {
            deps.dep
                .iter()
                .filter(|d| !d.is_build.unwrap_or(false))
                .map(|d| d.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub repo: Repo,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone)]
pub struct UpdateCandidate {
    pub name: String,
    pub installed_version: String,
    pub available_version: String,
    pub repo_url: String,
    pub package: Package,
}

#[derive(Debug)]
pub struct SearchResult {
    pub repo_url: String,
    pub repo_name: String,
    pub package: Package,
}

pub fn encode_manifest(manifest: &Manifest) -> Result<Vec<u8>> {
    rmp_serde::to_vec_named(manifest).map_err(|e| Error::Msgpack(e.to_string()))
}

pub fn decode_manifest(bytes: &[u8]) -> Result<Manifest> {
    rmp_serde::from_slice(bytes).map_err(|e| Error::Msgpack(e.to_string()))
}
