use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use super::{ArchiveExtractor, unpack};
use crate::error::{Error, Result};

pub struct LzipExtractor;

impl ArchiveExtractor for LzipExtractor {
    fn extract(&self, archive: &Path, dest_dir: &Path) -> Result<PathBuf> {
        let file = std::fs::File::open(archive)?;
        let mut tmp = tempfile::tempfile()?;
        decompress(file, &mut tmp)?;
        tmp.seek(SeekFrom::Start(0))?;
        Ok(unpack::unpack(BufReader::new(tmp), dest_dir)?.unwrap_or_else(|| dest_dir.to_owned()))
    }
}

fn decompress<R: Read, W: std::io::Write>(mut src: R, dst: &mut W) -> Result<()> {
    let mut hdr = [0u8; 6];
    src.read_exact(&mut hdr)
        .map_err(|e| Error::ExtractionFailed(e.to_string()))?;

    if &hdr[0..4] != b"LZIP" || hdr[4] != 1 {
        return Err(Error::ExtractionFailed("not a valid lzip v1 member".into()));
    }

    let coded = hdr[5];
    let base = 1u32 << (coded >> 3);
    let dict_size: u32 = if base > (1 << 12) {
        base - (base / 8) * u32::from(coded & 7)
    } else {
        1 << 12
    };

    // strip the 20-byte trailer
    let mut body = Vec::new();
    src.read_to_end(&mut body)
        .map_err(|e| Error::ExtractionFailed(e.to_string()))?;

    const TRAILER: usize = 20;
    if body.len() < TRAILER {
        return Err(Error::ExtractionFailed(
            "lzip member too short (missing trailer)".into(),
        ));
    }
    let lzma_data = &body[..body.len() - TRAILER];

    // lc=3, lp=0, pb=2 => props byte = (pb*5+lp)*9+lc = 93 = 0x5D
    let mut lzma_hdr = [0u8; 13];
    lzma_hdr[0] = 0x5D;
    lzma_hdr[1..5].copy_from_slice(&dict_size.to_le_bytes());
    lzma_hdr[5..13].copy_from_slice(&u64::MAX.to_le_bytes());

    let mut chained =
        BufReader::new(std::io::Cursor::new(lzma_hdr).chain(std::io::Cursor::new(lzma_data)));
    lzma_rs::lzma_decompress(&mut chained, dst).map_err(|e| Error::ExtractionFailed(e.to_string()))
}
