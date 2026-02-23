use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

pub const STORE_ROOT: &str = "/ventrica";
pub const STORE_DIR: &str = "/ventrica/store";
pub const REPOS_DIR: &str = "/ventrica/repos";
pub const GENERATIONS_DIR: &str = "/ventrica/generations";
pub const LIVE_PREFIX: &str = "/ventrica/live";

/// `<name>@<version>`.
#[must_use]
pub fn simple_store_name(name: &str, version: &str) -> String {
    format!("{name}@{version}")
}

/// `/ventrica/store/<name>@<version>`.
#[must_use]
pub fn simple_store_path(name: &str, version: &str) -> PathBuf {
    Path::new(STORE_DIR).join(simple_store_name(name, version))
}

/// strip write bits
pub fn seal(path: &Path) -> Result<()> {
    chmod_tree(path, |mode| mode & !0o222)
}

/// restore write bits
pub fn unseal(path: &Path) -> Result<()> {
    chmod_tree(path, |mode| mode | 0o200)
}

fn chmod_tree(path: &Path, op: impl Fn(u32) -> u32 + Copy) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if path.is_symlink() {
        return Ok(());
    }
    let meta = fs::metadata(path)?;
    let mut perms = meta.permissions();
    perms.set_mode(op(perms.mode()));
    fs::set_permissions(path, perms)?;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            chmod_tree(&entry?.path(), op)?;
        }
    }
    Ok(())
}

pub fn atomic_move(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::rename(src, dest) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            copy_dir_all(src, dest)?;
            let _ = unseal(src);
            fs::remove_dir_all(src)?;
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn copy_dir_all(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_child = entry.path();
        let dest_child = dest.join(entry.file_name());
        if src_child.is_symlink() {
            let target = fs::read_link(&src_child)?;
            unix_fs::symlink(&target, &dest_child)?;
        } else if src_child.is_dir() {
            copy_dir_all(&src_child, &dest_child)?;
        } else {
            fs::copy(&src_child, &dest_child)?;
        }
    }
    Ok(())
}

/// Populate `dest` by **hard-linking** every file from `src`.
///
/// Directories become real directories (so multiple packages can merge under
/// `bin/`, `lib/`, etc.).  Symlinks are preserved as symlinks.  If a
/// destination file already exists from a previous package it is silently
/// skipped (first-writer-wins; packages should not ship conflicting paths).
pub fn link_tree(src: &Path, dest: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_child = entry.path();
        let dest_child = dest.join(entry.file_name());

        if src_child.is_symlink() {
            if dest_child.exists() || dest_child.is_symlink() {
                continue;
            }
            let target = fs::read_link(&src_child)?;
            unix_fs::symlink(&target, &dest_child)?;
        } else if src_child.is_dir() {
            fs::create_dir_all(&dest_child)?;
            link_tree(&src_child, &dest_child)?;
        } else {
            if dest_child.exists() || dest_child.is_symlink() {
                continue;
            }
            fs::hard_link(&src_child, &dest_child)?;
        }
    }
    Ok(())
}

/// Copy the generation-N symlink forest into generation-(N+1), excluding any
/// files whose hard-link target lives under one of the `exclude` store paths.
///
/// Used when removing a package: the new generation carries everything forward
/// except the removed package's files.
pub fn carry_forward(src: &Path, dest: &Path, exclude: &[PathBuf]) -> Result<()> {
    use std::collections::HashSet;
    use std::os::unix::fs::MetadataExt;

    // Precompute the complete set of inodes present in the excluded store
    // paths once, before the recursive tree walk.  The previous per-file O(m)
    // walk inside the loop made this O(n × m); now it is O(m) precompute +
    // O(n) traversal.
    let excluded_inodes: HashSet<u64> = if exclude.is_empty() {
        HashSet::new()
    } else {
        exclude
            .iter()
            .flat_map(|ex| walkdir::WalkDir::new(ex).into_iter().flatten())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| e.metadata().ok().map(|m| m.ino()))
            .collect()
    };

    carry_forward_inner(src, dest, exclude, &excluded_inodes)
}

fn carry_forward_inner(
    src: &Path,
    dest: &Path,
    exclude: &[PathBuf],
    excluded_inodes: &std::collections::HashSet<u64>,
) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_child = entry.path();
        let dest_child = dest.join(entry.file_name());

        if src_child.is_symlink() {
            let target = fs::read_link(&src_child)?;
            if exclude.iter().any(|ex| target.starts_with(ex)) {
                continue;
            }
            if !dest_child.exists() && !dest_child.is_symlink() {
                unix_fs::symlink(&target, &dest_child)?;
            }
        } else if src_child.is_dir() {
            fs::create_dir_all(&dest_child)?;
            carry_forward_inner(&src_child, &dest_child, exclude, excluded_inodes)?;
            let _ = fs::remove_dir(&dest_child);
        } else {
            if !excluded_inodes.is_empty() {
                if let Ok(m) = fs::metadata(&src_child) {
                    if excluded_inodes.contains(&m.ino()) {
                        continue;
                    }
                }
            }
            if !dest_child.exists() {
                fs::hard_link(&src_child, &dest_child)?;
            }
        }
    }
    Ok(())
}

pub fn sha256_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

pub fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(Error::HashMismatch {
            path: path.display().to_string(),
            expected: expected.into(),
            actual,
        })
    }
}
