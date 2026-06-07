use ventrica::repo::refresh_manifest_cache;
use ventrica::store::db::Database;

pub fn update_repos() -> ventrica::Result<()> {
    let db = Database::open()?;
    let repos = db.list_repos()?;

    if repos.is_empty() {
        log::info!("no repositories configured - use `repo add`");
        return Ok(());
    }

    let mut errors = 0usize;
    for repo in &repos {
        log::info!("updating '{}' ({:?})...", repo.name, repo.url);
        if let Some(url) = &repo.url {
            match refresh_manifest_cache(url) {
                Ok(m) => log::info!(
                    "    {} package(s) cached",
                    m.packages.as_ref().map_or(0, std::vec::Vec::len)
                ),
                Err(e) => {
                    log::info!("    {e}");
                    errors += 1;
                }
            }
        } else {
            log::info!("    no URL configured");
            errors += 1;
        }
    }

    if errors == 0 {
        log::info!("all repositories updated");
    } else {
        log::info!("{errors} repository/repositories could not be updated");
    }

    Ok(())
}
