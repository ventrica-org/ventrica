mod unpack;

pub mod bzip2;
pub mod gzip;
pub mod lzip;
pub mod plain;
pub mod xz;
pub mod zip;

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub trait ArchiveExtractor {
    fn extract(&self, archive: &Path, dest_dir: &Path) -> Result<PathBuf>;
}

/// | Format | Magic                             |
/// |--------|-----------------------------------|
/// | gzip   | `1F 8B`                           |
/// | bzip2  | `42 5A 68` (`BZh`)                |
/// | xz     | `FD 37 7A 58 5A 00`               |
/// | lzip   | `4C 5A 49 50` (`LZIP`)            |
/// | zip    | `50 4B 03 04` (`PK\x03\x04`)      |
pub fn detect(archive: &Path) -> Result<Box<dyn ArchiveExtractor>> {
    use std::io::Read;

    let mut buf = [0u8; 8];
    let n = std::fs::File::open(archive)
        .and_then(|mut f| f.read(&mut buf))
        .map_err(|e| {
            Error::ExtractionFailed(format!("cannot read header of {}: {e}", archive.display()))
        })?;
    let h = &buf[..n];

    if h.len() >= 2 && h[0] == 0x1f && h[1] == 0x8b {
        return Ok(Box::new(gzip::GzipExtractor));
    }
    if h.starts_with(b"BZh") {
        return Ok(Box::new(bzip2::Bzip2Extractor));
    }
    if h.len() >= 6 && h[..6] == [0xfd, b'7', b'z', b'X', b'Z', 0x00] {
        return Ok(Box::new(xz::XzExtractor));
    }
    if h.starts_with(b"LZIP") {
        return Ok(Box::new(lzip::LzipExtractor));
    }
    if h.starts_with(b"PK\x03\x04") {
        return Ok(Box::new(zip::ZipExtractor));
    }

    Ok(Box::new(plain::PlainExtractor))
}

#[must_use]
pub fn from_hint(hint: &str) -> Option<Box<dyn ArchiveExtractor>> {
    match hint.to_ascii_lowercase().as_str() {
        "tar.gz" | "tgz" => Some(Box::new(gzip::GzipExtractor)),
        "tar.bz2" | "tbz2" => Some(Box::new(bzip2::Bzip2Extractor)),
        "tar.xz" | "txz" => Some(Box::new(xz::XzExtractor)),
        "tar.lz" | "tlz" => Some(Box::new(lzip::LzipExtractor)),
        "zip" => Some(Box::new(zip::ZipExtractor)),
        _ => None,
    }
}
