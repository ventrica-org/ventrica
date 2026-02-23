use std::io::BufReader;
use std::path::{Path, PathBuf};

use super::{ArchiveExtractor, unpack};
use crate::error::Result;

pub struct Bzip2Extractor;

impl ArchiveExtractor for Bzip2Extractor {
    fn extract(&self, archive: &Path, dest_dir: &Path) -> Result<PathBuf> {
        let file = BufReader::new(std::fs::File::open(archive)?);
        let decoder = ::bzip2::read::BzDecoder::new(file);
        Ok(unpack::unpack(decoder, dest_dir)?.unwrap_or_else(|| dest_dir.to_owned()))
    }
}
