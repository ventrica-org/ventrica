use ventrica::repo::refresh_manifest_cache;
use ventrica::store::db::Database;

pub fn update_repos(log: &mut dyn FnMut(&str)) -> ventrica::Result<()> {
    let db = Database::open()?;
    let repos = db.list_repos()?;

    if repos.is_empty() {
        log("no repositories configured - use `repo add`");
        return Ok(());
    }

    let mut errors = 0usize;
    for repo in &repos {
        log(&format!("updating '{}' ({})...", repo.name, repo.url));
        match refresh_manifest_cache(&repo.url) {
            Ok(m) => log(&format!("    {} package(s) cached", m.packages.len())),
            Err(e) => {
                log(&format!("    {e}"));
                errors += 1;
            }
        }
    }

    if errors == 0 {
        log("all repositories updated");
    } else {
        log(&format!(
            "{errors} repository/repositories could not be updated"
        ));
    }

    Ok(())
}
