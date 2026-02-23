use ventrica::error::Result;
use ventrica::repo::{dep_store_paths, find_in_repos, install_from_repo};
use ventrica::store::{STORE_DIR, db::Database};

pub fn ensure_dep_installed(
    dep_name: &str,
    repo_urls: &[String],
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let Some((base_url, entry)) = find_in_repos(dep_name, repo_urls)? else {
        log(&format!(
            "dependency '{dep_name}' not found in any repo, skipping"
        ));
        return Ok(());
    };

    for transitive in entry.run_deps.clone() {
        ensure_dep_installed(&transitive, repo_urls, log)?;
    }

    let store_path = std::path::Path::new(STORE_DIR).join(&entry.store_name);

    if !store_path.exists() {
        log(&format!(
            "fetching dependency {} {}...",
            entry.name, entry.version
        ));
        install_from_repo(&base_url, &entry)?;
    }

    let db = Database::open()?;
    if db
        .find_package_by_store_path(&store_path.display().to_string())?
        .is_none()
    {
        let dep_paths = dep_store_paths(repo_urls, &entry.run_deps);
        db.insert_package(
            &entry.name,
            &entry.version,
            &entry.description,
            &entry.category,
            &entry.store_name,
            &store_path.display().to_string(),
            entry.icon.as_deref(),
            None,
            &dep_paths,
        )?;
    }

    Ok(())
}
