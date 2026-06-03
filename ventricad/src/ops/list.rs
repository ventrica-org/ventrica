use ventrica::schema::kdl::Package;
use ventrica::store::db::Database;
use ventrica::{GenerationRecord, PackageRecord, RepoRecord};

pub fn list_packages() -> ventrica::Result<Vec<Package>> {
    let db = Database::open()?;
    db.list_packages()
        .map(|rows| rows.into_iter().map(record_to_package).collect())
}

pub fn list_generations() -> ventrica::Result<Vec<GenerationRecord>> {
    let db = Database::open()?;
    let current = db.current_generation_number()?;
    let mut gens = db.list_generations()?;
    for g in &mut gens {
        g.current = g.number == current;
        g.packages = db.packages_in_generation(g.number)?;
    }
    Ok(gens)
}

pub fn list_repos() -> ventrica::Result<Vec<RepoRecord>> {
    let db = Database::open()?;
    db.list_repos()
}

fn record_to_package(rec: PackageRecord) -> Package {
    Package {
        is_installed: Some(true),
        is_cached: None,
        is_disabled: None,
        package_hash: Some(rec.store_name),
        name: rec.name,
        version: rec.version,
        description: rec.description,
        native_depiction: rec.native_description,
        license: None,
        homepage: None,
        category: Some(rec.category),
        icon: rec.icon,
        platforms: Vec::new(),
        dependencies: None,
        source: None,
        autobump: None,
        scripts: None,
        installed_at: None,
    }
}
