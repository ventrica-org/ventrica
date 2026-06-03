use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::build::download::download;
use crate::error::{Error, Result};
use crate::schema::kdl::Package;
use crate::store::var as var_fmt;
use crate::store::{STORE_DIR, seal, sha256_file, simple_store_name};

use super::remote::get_manifest;
use super::{SearchResult, UpdateCandidate, run_dependencies};

pub fn dep_store_paths(repo_urls: &[String], run_deps: &[String]) -> Vec<String> {
    fn walk(repo_urls: &[String], name: &str, seen: &mut HashSet<String>, out: &mut Vec<String>) {
        if !seen.insert(name.to_owned()) {
            return;
        }
        if let Ok(Some((_, package))) = find_in_repos(name, repo_urls) {
            let store_name = simple_store_name(&package.name, &package.version);
            out.push(
                Path::new(STORE_DIR)
                    .join(store_name)
                    .to_string_lossy()
                    .into_owned(),
            );
            for d in run_dependencies(&package) {
                walk(repo_urls, &d, seen, out);
            }
        }
    }
    let (mut seen, mut out) = (HashSet::new(), Vec::new());
    for dep in run_deps {
        walk(repo_urls, dep, &mut seen, &mut out);
    }
    out
}

pub fn find_in_repos(
    package_name: &str,
    repo_urls: &[String],
) -> Result<Option<(String, Package)>> {
    for url in repo_urls {
        let manifest = match get_manifest(url) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("could not load manifest from {url}: {e}");
                continue;
            }
        };
        let name_lower = package_name.to_lowercase();
        if let Some(entry) = manifest
            .packages
            .into_iter()
            .find(|p| p.name.to_lowercase() == name_lower)
        {
            return Ok(Some((url.clone(), entry)));
        }
    }
    Ok(None)
}

pub fn search_repos(query: &str, repo_urls: &[String]) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();
    for url in repo_urls {
        let manifest = match get_manifest(url) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("could not load manifest from {url}: {e}");
                continue;
            }
        };
        let q = query.to_lowercase();
        let repo_name = manifest.repo.name.clone();
        for package in manifest.packages {
            if package.name.to_lowercase().contains(&q)
                || package.description.to_lowercase().contains(&q)
            {
                results.push(SearchResult {
                    repo_url: url.clone(),
                    repo_name: repo_name.clone(),
                    package,
                });
            }
        }
    }
    Ok(results)
}

pub fn check_updates(
    installed: &HashMap<String, String>,
    repo_urls: &[String],
) -> Result<Vec<UpdateCandidate>> {
    if installed.is_empty() || repo_urls.is_empty() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    let mut resolved: HashSet<String> = HashSet::new();

    for url in repo_urls {
        let manifest = match get_manifest(url) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("could not load manifest from {url}: {e}");
                continue;
            }
        };

        for package in manifest.packages {
            let name = &package.name;
            if resolved.contains(name.as_str()) {
                continue;
            }
            if let Some(installed_ver) = installed.get(name) {
                resolved.insert(name.clone());
                if package.version != *installed_ver {
                    candidates.push(UpdateCandidate {
                        name: name.clone(),
                        installed_version: installed_ver.clone(),
                        available_version: package.version.clone(),
                        repo_url: url.clone(),
                        package,
                    });
                }
            }
        }
    }

    Ok(candidates)
}

pub fn install_from_repo(base_url: &str, package: &Package) -> Result<PathBuf> {
    let store_name = simple_store_name(&package.name, &package.version);
    let dest = Path::new(STORE_DIR).join(&store_name);
    if dest.exists() {
        log::info!("{} already in store - skipping download", package.name);
        return Ok(dest);
    }

    let source = package.source.as_ref().ok_or_else(|| Error::BuildFailed {
        name: package.name.clone(),
        reason: "package has no source URLs in manifest".into(),
    })?;

    let source_url = source.url.first().ok_or_else(|| Error::BuildFailed {
        name: package.name.clone(),
        reason: "package source URLs are empty in manifest".into(),
    })?;

    let base = base_url.trim_end_matches('/');
    let download_url = if source_url.starts_with("http://") || source_url.starts_with("https://") {
        source_url.clone()
    } else {
        format!("{base}/{}", source_url.trim_start_matches('/'))
    };

    let tmp_dir = tempfile::tempdir()?;
    let filename = source_url
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("package.var");
    let var_path = tmp_dir.path().join(filename);

    log::info!("downloading {} from {download_url}", package.name);
    download(&download_url, &var_path, &[])?;

    let actual = sha256_file(&var_path)?;
    let expected = source.sha256.trim();
    let expected_no_prefix = expected.strip_prefix("sha256:").unwrap_or(expected);
    if actual != expected_no_prefix {
        return Err(Error::HashMismatch {
            path: download_url,
            expected: expected.to_owned(),
            actual,
        });
    }

    fs::create_dir_all(&dest)?;
    log::info!("unpacking into {}", dest.display());
    var_fmt::unpack(&var_path, &dest)?;
    drop(tmp_dir);

    seal(&dest)?;
    log::info!("committed {}", dest.display());
    Ok(dest)
}
