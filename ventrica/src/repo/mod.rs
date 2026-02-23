pub mod build;
pub mod client;
pub mod remote;

pub use build::build_repo;
pub use client::{check_updates, dep_store_paths, find_in_repos, install_from_repo, search_repos};
pub use remote::{fetch_manifest, fetch_manifest_cached, get_manifest, refresh_manifest_cache};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::package::Package;
use crate::schema::repo::RepoMeta;

pub const MANIFEST_FILE: &str = "manifest.msgpack";
pub const MANIFEST_HASH_FILE: &str = "manifest.msgpack.sha256";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub category: String,
    pub store_name: String,
    /// Empty for source-only packages.
    pub filename: String,
    /// Empty for source-only packages.
    pub var_hash: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run_deps: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_content: Option<String>,
}

impl From<&Package> for PackageEntry {
    fn from(p: &Package) -> Self {
        PackageEntry {
            name: p.meta.name.clone(),
            version: p.meta.version.clone(),
            description: p.meta.description.clone(),
            category: p.meta.category.clone(),
            store_name: p.store_name.clone().unwrap_or_default(),
            filename: p.filename.clone().unwrap_or_default(),
            var_hash: p.var_hash.clone().unwrap_or_default(),
            run_deps: p.deps.run.clone(),
            icon: p.meta.icon.clone(),
            recipe_content: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub repo: RepoMeta,
    pub packages: Vec<PackageEntry>,
}

#[derive(Debug, Clone)]
pub struct UpdateCandidate {
    pub name: String,
    pub installed_version: String,
    pub available_version: String,
    pub repo_url: String,
    pub entry: PackageEntry,
}

#[derive(Debug)]
pub struct SearchResult {
    pub repo_url: String,
    pub repo_name: String,
    pub entry: PackageEntry,
}

pub fn encode_manifest(manifest: &Manifest) -> Result<Vec<u8>> {
    rmp_serde::to_vec_named(manifest).map_err(|e| Error::Msgpack(e.to_string()))
}

pub fn decode_manifest(bytes: &[u8]) -> Result<Manifest> {
    rmp_serde::from_slice(bytes).map_err(|e| Error::Msgpack(e.to_string()))
}
