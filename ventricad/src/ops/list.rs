use ventrica::schema::kdl::{Generation, Package, Repo};
use ventrica::store::db::Database;

pub fn list_packages() -> ventrica::Result<Vec<Package>> {
    let db = Database::open()?;
    db.list_packages_manifest()
}

pub fn list_generations() -> ventrica::Result<Vec<Generation>> {
    let db = Database::open()?;
    let current = db.current_generation_number()?;
    let mut gens = db.list_generations()?;
    for g in &mut gens {
        g.current = g.number == current;
        g.packages = db.packages_in_generation(g.number)?;
    }
    Ok(gens)
}

pub fn list_repos() -> ventrica::Result<Vec<Repo>> {
    let db = Database::open()?;
    db.list_repos()
}
