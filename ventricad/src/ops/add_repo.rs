use ventrica::repo::refresh_manifest_cache;
use ventrica::store::db::Database;

pub fn add_repo(
    url: &str,
    log: &mut dyn FnMut(&str),
) -> ventrica::Result<String> {
    log(&format!("fetching manifest from {url}..."));
    let manifest = refresh_manifest_cache(url)?;
    let name = manifest.repo.name;
    let db = Database::open()?;
    db.add_repo(&name, url)?;
    log(&format!("added repository '{name}' ({url})"));
    Ok(name)
}
