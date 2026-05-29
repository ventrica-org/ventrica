use std::collections::HashSet;
use std::path::Path;

use ventrica::store::{GENERATIONS_DIR, STORE_DIR, db::Database, unseal};

pub fn gc() -> ventrica::Result<()> {
    let db = Database::open()?;

    let current = db.current_generation_number()?;
    for generation in db.list_generations()? {
        if generation.number == current {
            continue;
        }
        let gen_dir = Path::new(GENERATIONS_DIR).join(generation.number.to_string());
        if gen_dir.exists() {
            unseal(&gen_dir)?;
            std::fs::remove_dir_all(&gen_dir)?;
        }
        db.delete_generation(generation.number)?;
        log::info!("removed generation {}", generation.number);
    }

    let mut referenced: HashSet<String> = HashSet::new();
    if current > 0 {
        for pkg in db.packages_in_generation(current)? {
            referenced.insert(pkg.store_path.clone());
            referenced.extend(pkg.run_dep_store_paths);
        }
    }

    let packages_dir = std::path::Path::new(STORE_DIR);
    if !packages_dir.exists() {
        return Ok(());
    }

    let mut freed = 0u64;
    for entry in std::fs::read_dir(&packages_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let key = path.display().to_string();
        if referenced.contains(&key) {
            continue;
        }
        let size = dir_size(&path);
        unseal(&path)?;
        std::fs::remove_dir_all(&path)?;
        freed += size;
        log::info!("removed {key}");
    }

    log::info!("freed {:.1} MiB", freed as f64 / 1_048_576.0);
    Ok(())
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}
