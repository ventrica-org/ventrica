use ventrica::models::Package;
use ventrica::repo::{fetch_manifest_cached, mark_package_installed, mark_package_not_installed};
use ventrica::store::db::Database;

pub fn remove_repo(url: &str) -> ventrica::Result<()> {
    let db = Database::open()?;
    db.remove_repo(url)?;
    log::info!("removed repository '{url}'");
    Ok(())
}

pub fn list_repo_packages(url: &str) -> ventrica::Result<Vec<Package>> {
    let db = Database::open()?;
    let installed = db.list_packages_manifest()?;
    let mut manifest = fetch_manifest_cached(url)?;

    let mut packages = manifest.packages.take().unwrap_or_default();

    for package in &mut packages {
        if installed.iter().any(|p| p.name == package.name) {
            mark_package_installed(package);
        } else {
            mark_package_not_installed(package);
        }
    }

    Ok(packages)
}
