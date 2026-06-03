use ventrica::error::{Error, Result};
use ventrica::repo::{dep_store_paths, find_in_repos, install_from_repo, run_dependencies};
use ventrica::schema::kdl::Package;
use ventrica::store::{db::Database, live};

use super::deps::ensure_dep_installed;

pub fn install(names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }

    let db = Database::open()?;
    let repos = db.list_repos()?;

    if repos.is_empty() {
        return Err(Error::PackageNotFound {
            name: format!(
                "{} (no repositories configured - use `repo add`)",
                names.join(", ")
            ),
        });
    }

    let repo_urls: Vec<String> = repos.iter().filter_map(|r| r.url.clone()).collect();

    let mut resolved: Vec<(String, Package)> = Vec::new();
    for name in names {
        let (base_url, entry) = find_in_repos(name, &repo_urls)?
            .ok_or_else(|| Error::PackageNotFound { name: name.clone() })?;

        if db
            .find_package_by_name_and_version(&entry.name, &entry.version)?
            .is_some()
        {
            return Err(Error::AlreadyInstalled {
                name: entry.name.clone(),
                version: entry.version.clone(),
            });
        }

        resolved.push((base_url, entry));
    }

    for (_, entry) in &resolved {
        for dep in run_dependencies(entry) {
            ensure_dep_installed(&dep, &repo_urls)?;
        }
    }

    for (base_url, entry) in &resolved {
        log::info!("installing {} {}...", entry.name, entry.version);

        install_from_repo(base_url, entry)?;

        if let Some(existing) = db.find_package(&entry.name)? {
            db.remove_package(&existing.name)?;
        }

        let run_deps = run_dependencies(entry);
        let dep_store_paths = dep_store_paths(&repo_urls, &run_deps);

        db.insert_package(entry, &dep_store_paths)?;
    }

    let all_pkgs = db.list_packages()?;

    let desc = format!(
        "install {}",
        resolved
            .iter()
            .map(|(_, e)| e.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    live::activate(&db, &all_pkgs, Some(&desc))?;

    for (_, entry) in &resolved {
        log::info!("installed {} {}", entry.name, entry.version);
    }
    Ok(())
}
