use ventrica::error::Result;
use ventrica::repo::{
    check_updates, dep_store_paths, find_in_repos, install_from_repo, run_dependencies,
};
use ventrica::store::{db::Database, live};

use super::deps::ensure_dep_installed;

pub fn upgrade(names: &[String]) -> Result<()> {
    let db = Database::open()?;
    let repos = db.list_repos()?;

    if repos.is_empty() {
        log::info!("no repositories configured - use `repo add`");
        return Ok(());
    }

    let repo_urls: Vec<String> = repos.iter().filter_map(|r| r.url.clone()).collect();

    let all_installed = db.list_packages_manifest()?;
    let installed: std::collections::HashMap<String, String> = all_installed
        .iter()
        .filter(|p| names.is_empty() || names.iter().any(|n| n == &p.name))
        .map(|p| (p.name.clone(), p.version.clone()))
        .collect();

    if installed.is_empty() {
        log::info!("no matching packages installed");
        return Ok(());
    }

    let candidates = check_updates(&installed, &repo_urls)?;

    if candidates.is_empty() {
        log::info!("all packages are up to date");
        return Ok(());
    }

    for candidate in &candidates {
        let installed_version = installed.get(&candidate.name).cloned().unwrap_or_default();
        log::info!(
            "upgrading {} {} -> {}...",
            candidate.name,
            installed_version,
            candidate.version
        );

        for dep in run_dependencies(&candidate) {
            ensure_dep_installed(&dep, &repo_urls)?;
        }

        let (repo_url, _) = find_in_repos(&candidate.name, &repo_urls)?.ok_or_else(|| {
            ventrica::Error::PackageNotFound {
                name: candidate.name.clone(),
            }
        })?;

        install_from_repo(&repo_url, &candidate)?;

        db.remove_package(&candidate.name)?;

        let run_deps = run_dependencies(&candidate);
        let dep_store_paths = dep_store_paths(&repo_urls, &run_deps);

        db.insert_package(candidate, &dep_store_paths)?;

        log::info!(
            "upgraded {} {} -> {}",
            candidate.name,
            installed_version,
            candidate.version
        );
    }

    let all_pkgs = db.list_packages()?;
    let desc = format!(
        "upgrade {}",
        candidates
            .iter()
            .map(|c| format!("{} -> {}", c.name, c.version))
            .collect::<Vec<_>>()
            .join(", ")
    );
    live::activate(&db, &all_pkgs, Some(&desc))?;

    Ok(())
}
