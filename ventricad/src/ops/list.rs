use ventrica::schema::kdl::Package;
use ventrica::store::db::Database;
use ventrica::{GenerationRecord, RepoRecord};

pub fn list_packages() -> ventrica::Result<Vec<Package>> {
    let db = Database::open()?;
    db.list_packages_manifest()
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
