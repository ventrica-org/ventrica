use ventrica::repo::{fetch_manifest_cached, mark_package_installed, mark_package_not_installed};
use ventrica::schema::kdl::Package;
use ventrica::store::db::Database;

pub fn remove_repo(url: &str) -> ventrica::Result<()> {
    let db = Database::open()?;
    db.remove_repo(url)?;
    log::info!("removed repository '{url}'");
    Ok(())
}

pub fn list_repo_packages(url: &str) -> ventrica::Result<Vec<Package>> {
    let db = Database::open()?;
    let installed = db.list_packages()?;
    let mut manifest = fetch_manifest_cached(url)?;

    for package in &mut manifest.packages {
        if let Some(pkg) = installed.iter().find(|p| p.name == package.name) {
            mark_package_installed(package, Some(pkg.store_name.clone()));
        } else {
            mark_package_not_installed(package);
        }
    }

    Ok(manifest.packages)
}
