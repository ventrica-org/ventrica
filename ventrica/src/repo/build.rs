use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use crate::build::Builder;
use crate::error::{Error, Result};
use crate::schema::FromYaml;
use crate::schema::package::Package;
use crate::schema::repo::RepoConfig;
use crate::store::var as var_fmt;
use crate::store::{LIVE_PREFIX, STORE_DIR, link_tree, seal, sha256_file};

use super::{MANIFEST_FILE, MANIFEST_HASH_FILE, Manifest, PackageEntry, encode_manifest};

pub fn build_repo(repo_dir: &Path, build_user: Option<(u32, u32)>) -> Result<()> {
    let config = RepoConfig::from_file(&repo_dir.join("repo.yml"))?;
    log::info!("==> Building repository: {}", config.meta.name);

    let recipes = scan_packages(&repo_dir.join("packages"))?;
    log::info!("found {} package(s)", recipes.len());

    let ordered = topo_sort(recipes)?;

    let cache_dir = repo_dir.join("cache");
    fs::create_dir_all(&cache_dir)?;

    // each package that lands in the store is linked here
    // so subsequent builds find their deps at /ventrica/live.
    let overlay_dir = Path::new(STORE_DIR).join(format!(".repo-build-{}", std::process::id()));
    fs::create_dir_all(&overlay_dir)?;
    let live_path = Path::new(LIVE_PREFIX);
    let prev_live: Option<PathBuf> = if live_path.is_symlink() {
        fs::read_link(live_path).ok()
    } else {
        None
    };

    // seed overlay with whatever tools already live in the current generation.
    if let Some(ref prev) = prev_live {
        if prev.is_dir() {
            if let Err(e) = link_tree(prev, &overlay_dir) {
                log::warn!("could not seed build overlay from live: {e}");
            }
        }
    }
    set_live(live_path, &overlay_dir)?;

    let result = build_repo_inner(
        &ordered,
        &cache_dir,
        build_user,
        &overlay_dir,
        &repo_dir.join("packages"),
    );

    // restore previous live symlink
    if let Some(ref prev) = prev_live {
        let _ = set_live(live_path, prev);
    } else {
        let _ = fs::remove_file(live_path);
    }
    let _ = fs::remove_dir_all(&overlay_dir);

    let entries = result?;

    write_manifest(&cache_dir, &config, &entries)?;

    log::info!(
        "==> Repository '{}' built - {} package(s) in {}",
        config.meta.name,
        entries.len(),
        cache_dir.display()
    );
    Ok(())
}

fn build_repo_inner(
    ordered: &[(PathBuf, Package)],
    cache_dir: &Path,
    build_user: Option<(u32, u32)>,
    overlay_dir: &Path,
    _packages_dir: &Path,
) -> Result<Vec<PackageEntry>> {
    let mut entries: Vec<PackageEntry> = Vec::new();
    let mut built_count = 0usize;
    let total = ordered.len();

    for (recipe_path, pkg) in ordered {
        built_count += 1;
        log::info!(
            "[{}/{}] {} {}",
            built_count,
            total,
            pkg.meta.name,
            pkg.meta.version
        );

        let store_name = format!("{}-{}", pkg.meta.name, pkg.meta.version);
        let var_filename = format!("{store_name}.var");
        let var_path = cache_dir.join(&var_filename);
        let store_entry_path = Path::new(STORE_DIR).join(&store_name);

        if pkg.meta.no_cache {
            log::info!("no_cache=true - embedding recipe in manifest (client builds)");
            let recipe_yaml = fs::read_to_string(recipe_path).unwrap_or_default();
            let mut entry = PackageEntry::from(pkg);
            entry.store_name = store_name;
            entry.recipe_content = Some(recipe_yaml);
            entries.push(entry);
            continue;
        }

        if var_path.exists() {
            log::info!("cache hit: {var_filename} - skipping build");
            if !store_entry_path.exists() {
                log::info!("unpacking cached .var into store");
                fs::create_dir_all(&store_entry_path)?;
                var_fmt::unpack(&var_path, &store_entry_path)?;
                seal(&store_entry_path)?;
            }
        } else {
            let built = Builder::new(pkg)
                .with_recipe_dir(recipe_path.parent().unwrap_or(Path::new("")).to_owned())
                .with_build_user_opt(build_user)
                .build_to_store()?;

            log::info!("packing => {}", var_path.display());
            var_fmt::pack(&built, &var_path)?;

            let packed_hash = sha256_file(&var_path)?;
            let _ = fs::write(
                cache_dir.join(format!("{var_filename}.sha256")),
                &packed_hash,
            );
        };

        let var_hash_hex = fs::read_to_string(cache_dir.join(format!("{var_filename}.sha256")))
            .map(|s| s.trim().to_owned())
            .unwrap_or_else(|_| sha256_file(&var_path).unwrap_or_default());

        let recipe_yaml = fs::read_to_string(recipe_path).unwrap_or_default();
        let mut entry = PackageEntry::from(pkg);
        entry.store_name = store_name;
        entry.filename = var_filename;
        entry.var_hash = format!("sha256:{var_hash_hex}");
        entry.recipe_content = Some(recipe_yaml);
        entries.push(entry);

        // merge package into the overlay so subsequent builds see it
        if store_entry_path.exists() {
            if let Err(e) = link_tree(&store_entry_path, overlay_dir) {
                log::warn!(
                    "could not merge {} into build overlay: {e}",
                    store_entry_path.display()
                );
            }
        }
    }

    Ok(entries)
}

///  redirect the `live` symlink to `target`.
fn set_live(live: &Path, target: &Path) -> std::io::Result<()> {
    let tmp = live.with_extension("new");
    if tmp.is_symlink() || tmp.exists() {
        fs::remove_file(&tmp)?;
    }
    unix_fs::symlink(target, &tmp)?;
    fs::rename(&tmp, live)
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
        } else if path.extension().map_or(false, |e| e == "yml") {
            match Package::from_file(&path) {
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
        .map(|(i, (_, p))| (p.meta.name.as_str(), i))
        .collect();

    let n = packages.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_degree: Vec<usize> = vec![0; n];

    for (i, (_, pkg)) in packages.iter().enumerate() {
        let mut seen: HashSet<usize> = HashSet::new();
        for dep in pkg.deps.build.iter().chain(pkg.deps.run.iter()) {
            let Some(&j) = name_to_idx.get(dep.as_str()) else {
                continue;
            };
            if seen.insert(j) {
                adj[j].push(i);
                in_degree[i] += 1;
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
            .map(|i| packages[i].1.meta.name.as_str())
            .collect();
        return Err(Error::DependencyResolution(format!(
            "dependency cycle detected among: {}",
            stuck.join(", ")
        )));
    }

    Ok(order.into_iter().map(|i| packages[i].clone()).collect())
}

fn write_manifest(cache_dir: &Path, config: &RepoConfig, entries: &[PackageEntry]) -> Result<()> {
    let manifest = Manifest {
        repo: crate::schema::repo::RepoMeta {
            name: config.meta.name.clone(),
            description: config.meta.description.clone(),
            icon: config.meta.icon.clone(),
            homepage: config.meta.homepage.clone(),
        },
        packages: entries.to_vec(),
    };

    let bytes = encode_manifest(&manifest)?;

    let manifest_path = cache_dir.join(MANIFEST_FILE);
    fs::write(&manifest_path, &bytes)?;
    log::info!("wrote {}", manifest_path.display());

    let hash = sha256_file(&manifest_path)?;
    let hash_path = cache_dir.join(MANIFEST_HASH_FILE);
    fs::write(&hash_path, &hash)?;
    log::info!("wrote {} (sha256: {}…)", hash_path.display(), &hash[..12]);

    Ok(())
}
