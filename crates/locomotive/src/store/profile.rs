use std::fs;
use std::os::unix::fs::{self as unix_fs, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::Error;

// TODO: remove later
pub struct GenerationInfo {
    pub number: u32,
    pub path: PathBuf,
    pub hash: String,
    pub created_at: u64,
    pub is_current: bool,
}

const PROFILES_REL: &str = "var/lib/ventrica/profiles";
const CURRENT: &str = "current";

pub fn build(prefix: &Path, packages: &[PathBuf]) -> Result<PathBuf, Error> {
    let hash = generation_hash(packages);
    let gen_path = prefix.join("store").join(format!("gen-{}", hash));

    if !gen_path.exists() {
        fs::create_dir_all(&gen_path)?;
        for pkg in packages {
            hardlink_tree(pkg, &gen_path)?;
        }
        make_readonly(&gen_path)?;
    }

    Ok(gen_path)
}

pub fn activate(prefix: &Path, gen_path: &Path) -> Result<(), Error> {
    let profiles_dir = prefix.join(PROFILES_REL);
    fs::create_dir_all(&profiles_dir)?;

    let current = profiles_dir.join(CURRENT);
    archive_current(&profiles_dir, &current)?;
    unix_fs::symlink(gen_path, &current)?;

    let usr = prefix.join("usr");
    if usr.is_symlink() {
        fs::remove_file(&usr)?;
    }
    if !usr.exists() {
        unix_fs::symlink(&current, &usr)?;
    }

    Ok(())
}

pub fn rollback(prefix: &Path, number: Option<u32>) -> Result<(), Error> {
    let profiles_dir = prefix.join(PROFILES_REL);
    let current = profiles_dir.join(CURRENT);

    let generations = list(prefix)?;
    if generations.is_empty() {
        return Err(Error::StoreNoGenerations);
    }

    let target = match number {
        None => generations.last().unwrap(),
        Some(n) => generations
            .iter()
            .find(|g| g.number == n)
            .ok_or(Error::StoreGenerationNotFound(n))?,
    };

    let gen_path = fs::read_link(&target.path).unwrap_or_else(|_| target.path.clone());

    archive_current(&profiles_dir, &current)?;
    unix_fs::symlink(&gen_path, &current)?;

    Ok(())
}

pub fn list(prefix: &Path) -> Result<Vec<GenerationInfo>, Error> {
    let profiles_dir = prefix.join(PROFILES_REL);
    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let current_target = profiles_dir
        .join(CURRENT)
        .canonicalize()
        .unwrap_or_default();

    let mut generations: Vec<GenerationInfo> = fs::read_dir(&profiles_dir)?
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let created_at = name_str.strip_prefix("profile-")?.parse::<u64>().ok()?;
            let path = entry.path();
            let gen_path = fs::read_link(&path).unwrap_or_else(|_| path.clone());
            let hash = gen_path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix("gen-"))
                .unwrap_or("")
                .to_owned();
            let is_current = gen_path
                .canonicalize()
                .map(|p| p == current_target)
                .unwrap_or(false);
            Some(GenerationInfo {
                number: 0,
                path,
                hash,
                created_at,
                is_current,
            })
        })
        .collect();

    generations.sort_by_key(|g| g.created_at);
    for (i, g) in generations.iter_mut().enumerate() {
        g.number = (i + 1) as u32;
    }

    Ok(generations)
}

fn generation_hash(packages: &[PathBuf]) -> String {
    let mut sorted: Vec<&str> = packages.iter().filter_map(|p| p.to_str()).collect();
    sorted.sort_unstable();

    let mut hasher = Sha256::new();
    for p in &sorted {
        hasher.update(p.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(&hasher.finalize()[..20])
}

fn archive_current(profiles_dir: &Path, current: &Path) -> Result<(), Error> {
    if current.is_symlink() || current.exists() {
        let archive = profiles_dir.join(format!("profile-{}", unix_now()));
        fs::rename(current, archive)?;
    }
    Ok(())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn make_readonly(path: &Path) -> Result<(), Error> {
    let meta = fs::metadata(path)?;
    let readonly_mode = meta.permissions().mode() & !0o222;

    if meta.is_dir() {
        for entry in fs::read_dir(path)? {
            make_readonly(&entry?.path())?;
        }
    }

    fs::set_permissions(path, fs::Permissions::from_mode(readonly_mode))?;
    Ok(())
}

fn hardlink_tree(src: &Path, dst: &Path) -> Result<(), Error> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_child = entry.path();
        let dst_child = dst.join(entry.file_name());

        if src_child.is_dir() {
            fs::create_dir_all(&dst_child)?;
            hardlink_tree(&src_child, &dst_child)?;
        } else {
            fs::hard_link(&src_child, &dst_child)?;
        }
    }
    Ok(())
}
