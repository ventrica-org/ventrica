use ventrica::models::Package;
use ventrica::repo::{mark_package_installed, mark_package_not_installed, search_repos};
use ventrica::store::db::Database;

pub fn search(query: &str) -> ventrica::Result<Vec<Package>> {
    let db = Database::open()?;
    let repos = db.list_repos()?;

    if repos.is_empty() {
        return Ok(Vec::new());
    }

    let repo_urls: Vec<String> = repos.iter().filter_map(|r| r.url.clone()).collect();
    let results = search_repos(query, &repo_urls)?;
    let installed = db.list_packages_manifest()?;

    Ok(results
        .into_iter()
        .map(|r| {
            let mut package = r;
            if installed.iter().any(|p| p.name == package.name) {
                mark_package_installed(&mut package);
            } else {
                mark_package_not_installed(&mut package);
            }

            package
        })
        .collect())
}
