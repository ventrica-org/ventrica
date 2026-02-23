use serde::{Deserialize, Serialize};
use ventrica::repo::search_repos;
use ventrica::store::db::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub repo: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub run_deps: Vec<String>,
    pub installed: bool,
}

pub fn search(
    query: &str,
) -> ventrica::Result<Vec<SearchResult>> {
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
            let is_installed = installed.iter().any(|p| p.name == r.entry.name);
            let description = Some(r.entry.description.clone()).filter(|s: &String| !s.is_empty());
            SearchResult {
                repo: r.repo_name,
                name: r.entry.name,
                version: r.entry.version,
                description,
                run_deps: r.entry.run_deps,
                installed: is_installed,
            }
        })
        .collect())
}
