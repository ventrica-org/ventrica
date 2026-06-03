use serde::{Deserialize, Serialize};
use ventrica::repo::{
    mark_package_installed, mark_package_not_installed, run_dependencies, search_repos,
};
use ventrica::schema::kdl::Package;
use ventrica::store::db::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub repo: String,
    pub package: Package,
    pub run_deps: Vec<String>,
}

pub fn search(query: &str) -> ventrica::Result<Vec<SearchResult>> {
    let db = Database::open()?;
    let repos = db.list_repos()?;

    if repos.is_empty() {
        return Ok(Vec::new());
    }

    let repo_urls: Vec<String> = repos.iter().map(|r| r.url.clone()).collect();
    let results = search_repos(query, &repo_urls)?;
    let installed = db.list_packages()?;

    Ok(results
        .into_iter()
        .map(|r| {
            let mut package = r.package;
            if let Some(installed_pkg) = installed.iter().find(|p| p.name == package.name) {
                mark_package_installed(&mut package, Some(installed_pkg.store_name.clone()));
            } else {
                mark_package_not_installed(&mut package);
            }

            SearchResult {
                repo: r.repo_name,
                run_deps: run_dependencies(&package),
                package,
            }
        })
        .collect())
}
