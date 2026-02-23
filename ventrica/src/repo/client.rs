use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::build::download::download;
use crate::error::{Error, Result};
use crate::store::var as var_fmt;
use crate::store::{STORE_DIR, seal, sha256_file};

use super::remote::get_manifest;
use super::{PackageEntry, SearchResult, UpdateCandidate};

pub fn dep_store_paths(repo_urls: &[String], run_deps: &[String]) -> Vec<String> {
    fn walk(repo_urls: &[String], name: &str, seen: &mut HashSet<String>, out: &mut Vec<String>) {
        if !seen.insert(name.to_owned()) {
            return;
        }
        if let Ok(Some((_, e))) = find_in_repos(name, repo_urls) {
            out.push(
                Path::new(STORE_DIR)
                    .join(&e.store_name)
                    .to_string_lossy()
                    .into_owned(),
            );
            for d in e.run_deps.clone() {
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
) -> Result<Option<(String, PackageEntry)>> {
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
            .find(|e| e.name.to_lowercase() == name_lower)
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
        for entry in manifest.packages {
            if entry.name.to_lowercase().contains(&q)
                || entry.description.to_lowercase().contains(&q)
            {
                results.push(SearchResult {
                    repo_url: url.clone(),
                    repo_name: repo_name.clone(),
                    entry,
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

        for entry in manifest.packages {
            let name = &entry.name;
            if resolved.contains(name.as_str()) {
                continue;
            }
            if let Some(installed_ver) = installed.get(name) {
                resolved.insert(name.clone());
                if entry.version != *installed_ver {
                    candidates.push(UpdateCandidate {
                        name: name.clone(),
                        installed_version: installed_ver.clone(),
                        available_version: entry.version.clone(),
                        repo_url: url.clone(),
                        entry,
                    });
                }
            }
        }
    }

    Ok(candidates)
}

pub fn install_from_repo(base_url: &str, entry: &PackageEntry) -> Result<PathBuf> {
    let dest = Path::new(STORE_DIR).join(&entry.store_name);
    if dest.exists() {
        log::info!("{} already in store - skipping download", entry.name);
        return Ok(dest);
    }

    if entry.filename.is_empty() {
        return install_from_source(entry);
    }

    let base = base_url.trim_end_matches('/');
    let var_url = format!("{base}/{}", entry.filename);

    let tmp_dir = tempfile::tempdir()?;
    let var_path = tmp_dir.path().join(&entry.filename);

    log::info!("downloading {} from {base_url}", entry.name);
    download(&var_url, &var_path, &[])?;

    let actual = format!("sha256:{}", sha256_file(&var_path)?);
    if actual != entry.var_hash {
        return Err(Error::HashMismatch {
            path: var_url,
            expected: entry.var_hash.clone(),
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

fn install_from_source(entry: &PackageEntry) -> Result<PathBuf> {
    use crate::build::Builder;
    use crate::schema::FromYaml;
    use crate::schema::package::Package;

    let recipe_yaml = entry
        .recipe_content
        .as_deref()
        .ok_or_else(|| Error::PackageNotFound {
            name: format!(
                "{} (source-only package has no embedded recipe)",
                entry.name
            ),
        })?;

    log::info!("building {} from embedded recipe", entry.name);
    let pkg = Package::from_str_content(recipe_yaml)?;
    Builder::new(&pkg).build_to_store()
}
