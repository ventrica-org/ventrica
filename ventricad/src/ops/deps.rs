use ventrica::error::Result;
use ventrica::repo::{dep_store_paths, find_in_repos, install_from_repo, run_dependencies};
use ventrica::store::{STORE_DIR, db::Database, simple_store_name};

pub fn ensure_dep_installed(dep_name: &str, repo_urls: &[String]) -> Result<()> {
    let Some((base_url, entry)) = find_in_repos(dep_name, repo_urls)? else {
        log::info!("dependency '{dep_name}' not found in any repo, skipping");
        return Ok(());
    };

    for transitive in run_dependencies(&entry) {
        ensure_dep_installed(&transitive, repo_urls)?;
    }

    let store_name = simple_store_name(&entry.name, &entry.version);
    let store_path = std::path::Path::new(STORE_DIR).join(&store_name);

    if !store_path.exists() {
        log::info!("fetching dependency {} {}...", entry.name, entry.version);
        install_from_repo(&base_url, &entry)?;
    }

    let db = Database::open()?;
    if db
        .find_package_by_store_path(&store_path.display().to_string())?
        .is_none()
    {
        let run_deps = run_dependencies(&entry);
        let dep_paths = dep_store_paths(repo_urls, &run_deps);
        db.insert_package(
            &entry.name,
            &entry.version,
            &entry.description,
            entry.category.as_deref().unwrap_or_default(),
            &store_name,
            &store_path.display().to_string(),
            entry.icon.as_deref(),
            entry.native_depiction.as_deref(),
            &dep_paths,
        )?;
    }

    Ok(())
}
