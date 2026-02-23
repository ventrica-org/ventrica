use ventrica::error::{Error, Result};
use ventrica::store::{db::Database, live};

pub fn remove(
    name: &str,
    version: Option<&str>,
    log: &mut dyn FnMut(&str),
) -> Result<()> {
    let db = Database::open()?;
    let pkg = db
        .find_package(name, version)?
        .ok_or_else(|| Error::PackageNotFound { name: name.into() })?;

    log(&format!("removing {} {}...", pkg.name, pkg.version));

    db.remove_package(&pkg.name, &pkg.version)?;

    let remaining = db.list_packages()?;
    live::activate(&db, &remaining, Some(&format!("remove {name}")))?;

    log(&format!("removed {} {}", pkg.name, pkg.version));
    Ok(())
}
