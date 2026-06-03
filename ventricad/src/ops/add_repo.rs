use ventrica::repo::refresh_manifest_cache;
use ventrica::store::db::Database;

pub fn add_repo(url: &str) -> ventrica::Result<()> {
    log::info!("fetching manifest from {url}...");
    let manifest = refresh_manifest_cache(url)?;
    let db = Database::open()?;
    db.add_repo(&manifest, url)?;
    log::info!("added repository '{}' ({:?})", manifest.name, manifest.url);
    Ok(())
}
