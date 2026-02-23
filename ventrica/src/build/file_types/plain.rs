use std::path::{Path, PathBuf};

use super::ArchiveExtractor;
use crate::error::Result;

pub struct PlainExtractor;

impl ArchiveExtractor for PlainExtractor {
    fn extract(&self, archive: &Path, dest_dir: &Path) -> Result<PathBuf> {
        let basename = archive
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("source"));
        let out = dest_dir.join(basename);
        std::fs::copy(archive, &out)?;
        Ok(out)
    }
}
