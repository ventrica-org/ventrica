use std::io::BufReader;
use std::path::{Path, PathBuf};

use super::{ArchiveExtractor, unpack};
use crate::error::Result;

pub struct GzipExtractor;

impl ArchiveExtractor for GzipExtractor {
    fn extract(&self, archive: &Path, dest_dir: &Path) -> Result<PathBuf> {
        let file = BufReader::new(std::fs::File::open(archive)?);
        let decoder = flate2::read::GzDecoder::new(file);
        Ok(unpack::unpack(decoder, dest_dir)?.unwrap_or_else(|| dest_dir.to_owned()))
    }
}
