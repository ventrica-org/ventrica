use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub fn unpack<R: std::io::Read>(reader: R, dest: &Path) -> Result<Option<PathBuf>> {
    let mut archive = tar::Archive::new(reader);
    archive.set_preserve_permissions(true);
    archive.set_preserve_mtime(true);

    let mut top_dir: Option<PathBuf> = None;
    for entry in archive
        .entries()
        .map_err(|e| Error::ExtractionFailed(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| Error::ExtractionFailed(e.to_string()))?;

        if top_dir.is_none() {
            let kind = entry.header().entry_type();
            if !matches!(
                kind,
                tar::EntryType::XGlobalHeader | tar::EntryType::XHeader
            ) {
                if let Ok(path) = entry.path() {
                    if let Some(first) = path.components().next() {
                        top_dir = Some(dest.join(first.as_os_str()));
                    }
                }
            }
        }

        entry
            .unpack_in(dest)
            .map_err(|e| Error::ExtractionFailed(e.to_string()))?;
    }

    Ok(top_dir)
}
