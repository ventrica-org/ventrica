use ventrica::repo::{PackageEntry, fetch_manifest_cached};
use ventrica::store::db::Database;

pub fn remove_repo(url: &str, log: &mut dyn FnMut(&str)) -> ventrica::Result<()> {
    let db = Database::open()?;
    db.remove_repo(url)?;
    log(&format!("removed repository '{url}'"));
    Ok(())
}

pub fn list_repo_packages(url: &str) -> ventrica::Result<Vec<PackageEntry>> {
    let manifest = fetch_manifest_cached(url)?;
    Ok(manifest.packages)
}
