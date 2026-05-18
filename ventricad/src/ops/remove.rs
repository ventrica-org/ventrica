use std::collections::HashSet;

use ventrica::error::{Error, Result};
use ventrica::store::{db::Database, live};

pub fn remove(name: &str, version: Option<&str>, log: &mut dyn FnMut(&str)) -> Result<()> {
    let db = Database::open()?;
    let pkg = db
        .find_package(name, version)?
        .ok_or_else(|| Error::PackageNotFound { name: name.into() })?;

    let all_installed = db.list_packages()?;

    // BFS: collect the target package plus every installed package that
    // transitively lists one of the removed packages in its run_dep_store_paths.
    // We accumulate dependents in BFS order so that packages closest to the
    // leaves are removed first, leaving the original target last.
    let mut to_remove = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    seen.insert(pkg.store_path.clone());

    let mut frontier = vec![pkg.store_path.clone()];
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

    // Remove dependents first (reverse BFS order), then the target.
    to_remove.reverse();
    to_remove.push(pkg);

    for p in &to_remove {
        log(&format!("removing {} {}...", p.name, p.version));
        db.remove_package(&p.name, &p.version)?;
        log(&format!("removed {} {}", p.name, p.version));
    }

    let remaining = db.list_packages()?;
    live::activate(&db, &remaining, Some(&format!("remove {name}")))?;

    Ok(())
}
