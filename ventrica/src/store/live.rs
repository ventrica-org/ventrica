use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use crate::env::{VENTRICA_GENERATIONS_PATH, VENTRICA_LIVE_PATH, VENTRICA_LIVE_TMP_PATH};
use crate::error::{Error, Result};
use crate::models::Package;
use crate::store::db::Database;
use crate::store::{link_tree, seal, simple_store_path};

/// activates live with packages
pub fn activate(db: &Database, packages: &[Package], desc: Option<&str>) -> Result<u32> {
    let package_ids: Vec<i64> = packages.iter().filter_map(|p| p.id).collect();
    let g = db.create_generation(&package_ids, desc)?;

    let gen_dir = Path::new(VENTRICA_GENERATIONS_PATH).join(g.number.to_string());

    if gen_dir.exists() {
        crate::store::unseal(&gen_dir)?;
        fs::remove_dir_all(&gen_dir)?;
    }

    fs::create_dir_all(&gen_dir)?;

    for pkg in packages {
        let store = simple_store_path(&pkg.name, &pkg.version);
        if store.exists() {
            link_tree(&store, &gen_dir)?;
        }
    }

    seal(&gen_dir)?;

    swap_live(&gen_dir)?;

    log::info!("activated generation {}", g.number);
    Ok(g.number)
}

/// rollback to a previous generation
pub fn rollback(db: &Database, number: u32) -> Result<()> {
    db.get_generation(number)?;

    let gen_dir = Path::new(VENTRICA_GENERATIONS_PATH).join(number.to_string());
    if !gen_dir.exists() {
        return Err(Error::GenerationNotFound(number));
    }

    db.set_current_generation(number)?;
    swap_live(&gen_dir)?;

    log::info!("rolled back to generation {number}");
    Ok(())
}

fn swap_live(new_gen: &Path) -> Result<()> {
    let live = Path::new(VENTRICA_LIVE_PATH);
    let live_tmp = Path::new(VENTRICA_LIVE_TMP_PATH);

    if live_tmp.is_symlink() || live_tmp.exists() {
        fs::remove_file(live_tmp)?;
    }

    unix_fs::symlink(new_gen, live_tmp)?;

    fs::rename(live_tmp, live)?;
    Ok(())
}
