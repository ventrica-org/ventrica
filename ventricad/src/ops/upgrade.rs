use ventrica::error::Result;
use ventrica::repo::{check_updates, dep_store_paths, install_from_repo, run_dependencies};
use ventrica::store::simple_store_name;
use ventrica::store::{db::Database, live};

use super::deps::ensure_dep_installed;

pub fn upgrade(names: &[String]) -> Result<()> {
    let db = Database::open()?;
    let repos = db.list_repos()?;

    if repos.is_empty() {
        log::info!("no repositories configured - use `repo add`");
        return Ok(());
    }

    let repo_urls: Vec<String> = repos.iter().map(|r| r.url.clone()).collect();

    let all_installed = db.list_packages()?;
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
        log::info!(
            "upgrading {} {} -> {}...",
            candidate.name,
            candidate.installed_version,
            candidate.available_version
        );

        for dep in run_dependencies(&candidate.package) {
            ensure_dep_installed(&dep, &repo_urls)?;
        }

        let store_path = install_from_repo(&candidate.repo_url, &candidate.package)?;

        db.remove_package(&candidate.name)?;

        let run_deps = run_dependencies(&candidate.package);
        let dep_store_paths = dep_store_paths(&repo_urls, &run_deps);
        let store_name = simple_store_name(&candidate.package.name, &candidate.package.version);

        db.insert_package(
            &candidate.package.name,
            &candidate.package.version,
            &candidate.package.description,
            candidate.package.category.as_deref().unwrap_or_default(),
            &store_name,
            &store_path.display().to_string(),
            candidate.package.icon.as_deref(),
            candidate.package.native_depiction.as_deref(),
            &dep_store_paths,
        )?;

        log::info!(
            "upgraded {} {} -> {}",
            candidate.name,
            candidate.installed_version,
            candidate.available_version
        );
    }

    let all_pkgs = db.list_packages()?;
    let desc = format!(
        "upgrade {}",
        candidates
            .iter()
            .map(|c| format!("{} -> {}", c.name, c.available_version))
            .collect::<Vec<_>>()
            .join(", ")
    );
    live::activate(&db, &all_pkgs, Some(&desc))?;

    Ok(())
}
