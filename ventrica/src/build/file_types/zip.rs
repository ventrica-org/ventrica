use std::path::{Path, PathBuf};

use super::ArchiveExtractor;
use crate::error::{Error, Result};

pub struct ZipExtractor;

impl ArchiveExtractor for ZipExtractor {
    fn extract(&self, _archive: &Path, _dest_dir: &Path) -> Result<PathBuf> {
        Err(Error::ExtractionFailed(
            "ZIP extraction is not yet implemented".into(),
        ))
    }
}
