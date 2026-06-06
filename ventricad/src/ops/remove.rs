use std::collections::HashSet;

use ventrica::models::Package;
use ventrica::store::simple_store_name;
use ventrica::store::simple_store_path;
use ventrica::store::{db::Database, live};
use ventrica::{Error, Result};

pub fn remove(names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }

    let db = Database::open()?;

    let mut resolved: Vec<Package> = Vec::new();
    for name in names {
        let pkg = db
            .find_package(name)?
            .ok_or_else(|| Error::PackageNotFound { name: name.clone() })?;
        resolved.push(pkg);
    }

    let all_installed = db.list_packages()?;

    let mut to_remove = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    // use simple_store_name instead of store_path to find packages

    for pkg in &resolved {
        seen.insert(simple_store_name(&pkg.name, &pkg.version));
    }

    let mut frontier = resolved
        .iter()
        .map(|pkg| simple_store_name(&pkg.name, &pkg.version))
        .collect::<Vec<_>>();
    while !frontier.is_empty() {
        let mut next_frontier = Vec::new();
        for candidate in &all_installed {
            if seen.contains(&simple_store_name(&candidate.name, &candidate.version)) {
                continue;
            }
            let candidate_deps =
                db.package_dependency_store_paths(&candidate.name, &candidate.version)?;
            if candidate_deps.iter().any(|(name, version)| {
                frontier.contains(&simple_store_path(name, version).display().to_string())
            }) {
                seen.insert(simple_store_name(&candidate.name, &candidate.version));
                next_frontier.push(simple_store_name(&candidate.name, &candidate.version));
                to_remove.push(candidate.clone());
            }
        }
        frontier = next_frontier;
    }

    to_remove.reverse();
    for pkg in &resolved {
        to_remove.push(pkg.clone());
    }

    for p in &to_remove {
        log::info!("removing {} {}...", p.name, p.version);
        db.remove_package(&p.name)?;
        log::info!("removed {} {}", p.name, p.version);
    }

    let remaining = db.list_packages()?;
    live::activate(
        &db,
        &remaining,
        Some(&format!("removed {}", names.join(", "))),
    )?;

    Ok(())
}
