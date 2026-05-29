use ventrica::error::{Error, Result};
use ventrica::store::{db::Database, live};

pub fn rollback(target: Option<u32>) -> Result<()> {
    let db = Database::open()?;
    let current = db.current_generation_number()?;
    if current == 0 {
        return Err(Error::NoCurrentGeneration);
    }

    let target_gen = match target {
        Some(n) => n,
        None if current > 1 => current - 1,
        None => return Err(Error::GenerationNotFound(0)),
    };

    log::info!("rolling back generation {current} -> {target_gen}...");
    live::rollback(&db, target_gen)?;
    log::info!("active generation is now {target_gen}");
    Ok(())
}
