use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use crate::error::{Error, Result};
use crate::store::db::{Database, PackageRecord};
use crate::store::{GENERATIONS_DIR, LIVE_PREFIX, link_tree, seal};

/// activates live with packages
pub fn activate(db: &Database, packages: &[PackageRecord], desc: Option<&str>) -> Result<u32> {
    let package_ids: Vec<i64> = packages.iter().map(|p| p.id).collect();
    let g = db.create_generation(&package_ids, desc)?;

    let gen_dir = Path::new(GENERATIONS_DIR).join(g.number.to_string());

    if gen_dir.exists() {
        crate::store::unseal(&gen_dir)?;
        fs::remove_dir_all(&gen_dir)?;
    }

    fs::create_dir_all(&gen_dir)?;

    for pkg in packages {
        let store = std::path::PathBuf::from(&pkg.store_path);
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

    let gen_dir = Path::new(GENERATIONS_DIR).join(number.to_string());
    if !gen_dir.exists() {
        return Err(Error::GenerationNotFound(number));
    }

    db.set_current_generation(number)?;
    swap_live(&gen_dir)?;

    log::info!("rolled back to generation {number}");
    Ok(())
}

fn swap_live(new_gen: &Path) -> Result<()> {
    let live = Path::new(LIVE_PREFIX);
    let live_tmp = Path::new("/ventrica/live.new");

    if live_tmp.is_symlink() || live_tmp.exists() {
        fs::remove_file(live_tmp)?;
    }

    unix_fs::symlink(new_gen, live_tmp)?;

    fs::rename(live_tmp, live)?;
    Ok(())
}
