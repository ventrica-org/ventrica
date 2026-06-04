use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::build::Builder;
use crate::error::{Error, Result};
use crate::models::{Package, Repo, Source};
use crate::store::sha256_file;
use crate::store::var;

use super::{MANIFEST_FILE, MANIFEST_HASH_FILE, encode_manifest, mark_package_not_installed};

pub fn build_repo(repo_dir: &Path, _build_user: Option<(u32, u32)>) -> Result<()> {
    let packages_dir = repo_dir.join("packages");
    let packages = scan_packages(&packages_dir)?;
    log::info!("found {} package(s)", packages.len());

    let ordered = topo_sort(packages)?;

    let cache_dir = repo_dir.join("cache");
    fs::create_dir_all(&cache_dir)?;

    let mut manifest_packages = Vec::with_capacity(ordered.len());
    for (recipe_path, mut package) in ordered {
        let recipe_dir = recipe_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| repo_dir.join("packages"));

        let store_path = Builder::new(&package)
            .with_build_user_opt(_build_user)
            .with_recipe_dir(recipe_dir)
            .build_to_store()?;

        let archive_name = format!("{}-{}.var", package.name, package.version);
        let archive_path = cache_dir.join(&archive_name);
        if archive_path.exists() {
            fs::remove_file(&archive_path)?;
        }
        var::pack(&store_path, &archive_path)?;
        let archive_hash = sha256_file(&archive_path)?;

        package.source = Some(Source {
            url: vec![archive_name],
            sha256: archive_hash,
        });

        mark_package_not_installed(&mut package);
        manifest_packages.push(package);
    }

    let mut repo = repo_from_dir(repo_dir)?;
    write_manifest(&cache_dir, &mut repo, &manifest_packages)?;

    log::info!(
        "==> Repository '{}' built - {} package(s) in {}",
        repo.name,
        manifest_packages.len(),
        cache_dir.display()
    );
    Ok(())
}

fn repo_from_dir(repo_dir: &Path) -> Result<Repo> {
    let meta_path = repo_dir.join("repo.kdl");
    if !meta_path.exists() {
        return Err(Error::InvalidSchema(format!(
            "missing required repository metadata file: {}",
            meta_path.display()
        )));
    }

    let content = fs::read_to_string(&meta_path)?;
    let repo: Repo = kdl::de::from_str(&content)?;
    log::info!("{:?}", repo);
    Ok(repo)
}

pub fn scan_packages(packages_dir: &Path) -> Result<Vec<(PathBuf, Package)>> {
    let mut out = Vec::new();
    scan_dir(packages_dir, &mut out)?;
    Ok(out)
}

fn scan_dir(dir: &Path, out: &mut Vec<(PathBuf, Package)>) -> Result<()> {
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };

    let mut entries: Vec<_> = rd.collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "kdl") {
            match Package::from_path(&path) {
                Ok(pkg) => out.push((path, pkg)),
                Err(e) => log::warn!("skipping {} - {e}", path.display()),
            }
        }
    }
    Ok(())
}

pub fn topo_sort(packages: Vec<(PathBuf, Package)>) -> Result<Vec<(PathBuf, Package)>> {
    let name_to_idx: HashMap<&str, usize> = packages
        .iter()
        .enumerate()
        .map(|(i, (_, p))| (p.name.as_str(), i))
        .collect();

    let n = packages.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_degree: Vec<usize> = vec![0; n];

    for (i, (_, pkg)) in packages.iter().enumerate() {
        let mut seen: HashSet<usize> = HashSet::new();
        if let Some(deps) = &pkg.dependencies {
            for dep in deps {
                let Some(&j) = name_to_idx.get(dep.name.as_deref().unwrap_or_default()) else {
                    continue;
                };
                if seen.insert(j) {
                    adj[j].push(i);
                    in_degree[i] += 1;
                }
            }
        }
    }

    let mut queue: VecDeque<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut order: Vec<usize> = Vec::with_capacity(n);

    while let Some(i) = queue.pop_front() {
        order.push(i);
        for &j in &adj[i] {
            in_degree[j] -= 1;
            if in_degree[j] == 0 {
                queue.push_back(j);
            }
        }
    }

    if order.len() != n {
        let ordered_set: HashSet<usize> = order.iter().copied().collect();
        let stuck: Vec<&str> = (0..n)
            .filter(|i| !ordered_set.contains(i))
            .map(|i| packages[i].1.name.as_str())
            .collect();
        return Err(Error::DependencyResolution(format!(
            "dependency cycle detected among: {}",
            stuck.join(", ")
        )));
    }

    Ok(order.into_iter().map(|i| packages[i].clone()).collect())
}

fn write_manifest(cache_dir: &Path, repo: &mut Repo, packages: &[Package]) -> Result<()> {
    repo.packages = packages.to_vec();

    let bytes = encode_manifest(&repo)?;

    let manifest_path = cache_dir.join(MANIFEST_FILE);
    fs::write(&manifest_path, &bytes)?;
    log::info!("wrote {}", manifest_path.display());

    let hash = sha256_file(&manifest_path)?;
    let hash_path = cache_dir.join(MANIFEST_HASH_FILE);
    fs::write(&hash_path, &hash)?;
    log::info!("wrote {} (sha256: {}...)", hash_path.display(), &hash[..12]);

    Ok(())
}
