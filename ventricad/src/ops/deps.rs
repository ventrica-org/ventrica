use ventrica::error::Result;
use ventrica::repo::{dep_store_paths, find_in_repos, install_from_repo, run_dependencies};
use ventrica::store::db::Database;
use ventrica::store::simple_store_path;

pub fn ensure_dep_installed(dep_name: &str, repo_urls: &[String]) -> Result<()> {
    let Some((base_url, entry)) = find_in_repos(dep_name, repo_urls)? else {
        log::info!("dependency '{dep_name}' not found in any repo, skipping");
        return Ok(());
    };

    for transitive in run_dependencies(&entry) {
        ensure_dep_installed(&transitive, repo_urls)?;
    }

    let store_path = simple_store_path(&entry.name, &entry.version);

    if !store_path.exists() {
        log::info!("fetching dependency {} {}...", entry.name, entry.version);
        install_from_repo(&base_url, &entry)?;
    }

    let db = Database::open()?;
    if db
        .find_package_by_name_and_version(&entry.name, &entry.version)?
        .is_none()
    {
        let run_deps = run_dependencies(&entry);
        let dep_paths = dep_store_paths(repo_urls, &run_deps);
        db.insert_package(&entry, &dep_paths)?;
    }

    Ok(())
}
