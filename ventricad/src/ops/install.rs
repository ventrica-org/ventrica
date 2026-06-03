use ventrica::error::{Error, Result};
use ventrica::repo::{dep_store_paths, find_in_repos, install_from_repo, run_dependencies};
use ventrica::schema::kdl::Package;
use ventrica::store::simple_store_name;
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

    let repo_urls: Vec<String> = repos.iter().map(|r| r.url.clone()).collect();

    let mut resolved: Vec<(String, Package)> = Vec::new();
    for name in names {
        let (base_url, entry) = find_in_repos(name, &repo_urls)?
            .ok_or_else(|| Error::PackageNotFound { name: name.clone() })?;

        let expected = ventrica::store::simple_store_path(&entry.name, &entry.version);
        if db
            .find_package_by_store_path(&expected.display().to_string())?
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

    let mut new_records = Vec::new();
    for (base_url, entry) in &resolved {
        log::info!("installing {} {}...", entry.name, entry.version);

        let store_path = install_from_repo(base_url, entry)?;

        if let Some(existing) = db.find_package(&entry.name)? {
            db.remove_package(&existing.name)?;
        }

        let run_deps = run_dependencies(entry);
        let dep_store_paths = dep_store_paths(&repo_urls, &run_deps);
        let store_name = simple_store_name(&entry.name, &entry.version);

        let record = db.insert_package(
            &entry.name,
            &entry.version,
            &entry.description,
            entry.category.as_deref().unwrap_or_default(),
            &store_name,
            &store_path.display().to_string(),
            entry.icon.as_deref(),
            entry.native_depiction.as_deref(),
            &dep_store_paths,
        )?;
        new_records.push(record);
    }

    let mut all_pkgs = db.list_packages()?;
    for rec in &new_records {
        if !all_pkgs.iter().any(|p| p.id == rec.id) {
            all_pkgs.push(rec.clone());
        }
    }

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
