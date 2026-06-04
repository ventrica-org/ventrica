use std::fs;
use std::path::{Path, PathBuf};

use crate::build::download::download;
use crate::error::{Error, Result};
use crate::models::Repo;
use crate::store::{REPOS_DIR, sha256_file};

use super::{MANIFEST_FILE, MANIFEST_HASH_FILE, decode_manifest};

pub fn refresh_manifest_cache(base_url: &str) -> Result<Repo> {
    let cache_dir = repo_cache_dir(base_url);
    fs::create_dir_all(&cache_dir)?;

    let base = base_url.trim_end_matches('/');
    let manifest_path = cache_dir.join(MANIFEST_FILE);
    let hash_path = cache_dir.join(MANIFEST_HASH_FILE);

    let _ = fs::remove_file(&manifest_path);
    let _ = fs::remove_file(&hash_path);

    download(&format!("{base}/{MANIFEST_FILE}"), &manifest_path, &[])?;
    download(&format!("{base}/{MANIFEST_HASH_FILE}"), &hash_path, &[])?;

    let expected = fs::read_to_string(&hash_path)?.trim().to_owned();
    let actual = sha256_file(&manifest_path)?;
    if expected != actual {
        let _ = fs::remove_file(&manifest_path);
        let _ = fs::remove_file(&hash_path);
        return Err(Error::HashMismatch {
            path: format!("{base}/{MANIFEST_FILE}"),
            expected,
            actual,
        });
    }

    read_manifest(&manifest_path)
}

pub fn fetch_manifest_cached(base_url: &str) -> Result<Repo> {
    get_manifest(base_url)
}

pub fn fetch_manifest(base_url: &str) -> Result<Repo> {
    let tmp = tempfile::tempdir()?;
    let base = base_url.trim_end_matches('/');
    let manifest_path = tmp.path().join(MANIFEST_FILE);
    let hash_path = tmp.path().join(MANIFEST_HASH_FILE);

    download(&format!("{base}/{MANIFEST_FILE}"), &manifest_path, &[])?;
    download(&format!("{base}/{MANIFEST_HASH_FILE}"), &hash_path, &[])?;

    let expected = fs::read_to_string(&hash_path)?.trim().to_owned();
    let actual = sha256_file(&manifest_path)?;
    if expected != actual {
        return Err(Error::HashMismatch {
            path: format!("{base}/{MANIFEST_FILE}"),
            expected,
            actual,
        });
    }

    read_manifest(&manifest_path)
}

pub fn get_manifest(base_url: &str) -> Result<Repo> {
    let cache_dir = repo_cache_dir(base_url);
    fs::create_dir_all(&cache_dir)?;

    let base = base_url.trim_end_matches('/');
    let manifest_path = cache_dir.join(MANIFEST_FILE);
    let hash_path = cache_dir.join(MANIFEST_HASH_FILE);

    let remote_hash = fetch_hash(&format!("{base}/{MANIFEST_HASH_FILE}"));

    let local_hash = fs::read_to_string(&hash_path)
        .map(|s| s.trim().to_owned())
        .unwrap_or_default();

    let needs_download = match &remote_hash {
        Some(rh) => rh != &local_hash || !manifest_path.exists(),
        None => !manifest_path.exists(),
    };

    if needs_download {
        match &remote_hash {
            Some(rh) => {
                let _ = fs::remove_file(&manifest_path);
                download(&format!("{base}/{MANIFEST_FILE}"), &manifest_path, &[])?;

                let actual = sha256_file(&manifest_path)?;
                if actual != *rh {
                    let _ = fs::remove_file(&manifest_path);
                    return Err(Error::HashMismatch {
                        path: format!("{base}/{MANIFEST_FILE}"),
                        expected: rh.clone(),
                        actual,
                    });
                }
                fs::write(&hash_path, rh)?;
            }
            None if !manifest_path.exists() => {
                return Err(Error::DownloadFailed {
                    url: format!("{base}/{MANIFEST_FILE}"),
                    reason: "server unreachable and no local cache available".into(),
                });
            }
            None => {
                log::warn!("could not reach {base_url} - using stale manifest cache");
            }
        }
    }

    read_manifest(&manifest_path)
}

pub(super) fn repo_cache_dir(base_url: &str) -> PathBuf {
    use sha2::{Digest, Sha256};

    let stripped = base_url.trim_end_matches('/');
    let no_scheme = stripped
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(stripped);
    let raw_slug = no_scheme
        .rsplit('/')
        .find(|s| !s.is_empty())
        .unwrap_or("repo");
    let slug: String = raw_slug
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    let mut h = Sha256::new();
    h.update(base_url.as_bytes());
    let hex = hex::encode(h.finalize());

    Path::new(REPOS_DIR).join(format!("{slug}-{hex}"))
}

fn read_manifest(path: &Path) -> Result<Repo> {
    let bytes = fs::read(path)?;
    decode_manifest(&bytes)
}

fn fetch_hash(hash_url: &str) -> Option<String> {
    let tmp = tempfile::tempdir().ok()?;
    let dest = tmp.path().join("hash");
    download(hash_url, &dest, &[]).ok()?;
    fs::read_to_string(&dest).ok().map(|s| s.trim().to_owned())
}
