use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use super::{ArchiveExtractor, unpack};
use crate::error::Result;

pub struct XzExtractor;

impl ArchiveExtractor for XzExtractor {
    fn extract(&self, archive: &Path, dest_dir: &Path) -> Result<PathBuf> {
        let mut input = BufReader::new(File::open(archive)?);
        let mut decompressed = Vec::new();
        lzma_rs::xz_decompress(&mut input, &mut decompressed)?;
        let cursor = std::io::Cursor::new(decompressed);
        Ok(unpack::unpack(cursor, dest_dir)?.unwrap_or_else(|| dest_dir.to_owned()))
    }
}
