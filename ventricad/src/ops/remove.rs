use std::collections::HashSet;

use ventrica::PackageRecord;
use ventrica::error::{Error, Result};
use ventrica::store::{db::Database, live};

pub fn remove(names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }

    let db = Database::open()?;

    let mut resolved: Vec<PackageRecord> = Vec::new();
    for name in names {
        let pkg = db
            .find_package(name)?
            .ok_or_else(|| Error::PackageNotFound { name: name.clone() })?;
        resolved.push(pkg);
    }

    let all_installed = db.list_packages()?;

    let mut to_remove = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for pkg in &resolved {
        seen.insert(pkg.store_path.clone());
    }

    let mut frontier = resolved
        .iter()
        .map(|pkg| pkg.store_path.clone())
        .collect::<Vec<_>>();
    while !frontier.is_empty() {
        let mut next_frontier = Vec::new();
        for candidate in &all_installed {
            if seen.contains(&candidate.store_path) {
                continue;
            }
            if candidate
                .run_dep_store_paths
                .iter()
                .any(|d| frontier.contains(d))
            {
                seen.insert(candidate.store_path.clone());
                next_frontier.push(candidate.store_path.clone());
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
