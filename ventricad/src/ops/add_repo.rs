use ventrica::repo::refresh_manifest_cache;
use ventrica::store::db::Database;

pub fn add_repo(url: &str) -> ventrica::Result<String> {
    log::info!("fetching manifest from {url}...");
    let manifest = refresh_manifest_cache(url)?;
    let name = manifest.repo.name;
    let db = Database::open()?;
    db.add_repo(&name, url)?;
    log::info!("added repository '{name}' ({url})");
    Ok(name)
}
