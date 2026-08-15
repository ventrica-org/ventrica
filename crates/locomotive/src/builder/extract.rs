use std::{
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Component, Path},
};

use lzma_rust2::{LzipReader, LzmaReader, XzReader};

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Tar,
    Gzip,
    Bzip2,
    Xz,
    Lzma,
    Lzip,
}

fn detect_format(data: &[u8]) -> Option<Format> {
    if data.starts_with(&[0x1f, 0x8b]) {
        return Some(Format::Gzip);
    }
    if data.starts_with(b"BZh") {
        return Some(Format::Bzip2);
    }
    if data.starts_with(&[0xfd, b'7', b'z', b'X', b'Z', 0x00]) {
        return Some(Format::Xz);
    }
    if data.starts_with(b"LZIP") {
        return Some(Format::Lzip);
    }

    // Tar magic bytes are at offset 257
    if data.len() >= 263 && (&data[257..263] == b"ustar\0" || &data[257..263] == b"ustar ") {
        return Some(Format::Tar);
    }

    // LZMA .lzma header heuristic (moved to bottom as it's a weak heuristic)
    if data.len() >= 13 && data[0] <= 224 {
        return Some(Format::Lzma);
    }

    None
}

fn safe_path(path: &Path) -> Result<(), Error> {
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(Error::BuilderInvalidData {
            message: format!("unsafe path in archive: {}", path.display()),
        });
    }
    Ok(())
}

/// Extracts a Tar archive, optionally stripping a number of top-level directory components.
fn extract_tar<R: Read>(reader: R, dest: &Path, strip_components: usize) -> Result<(), Error> {
    let mut archive = tar::Archive::new(reader);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let original = entry.path()?.to_path_buf();

        safe_path(&original)?;

        // Strip the specified number of top-level directories
        let mut components = original.components();
        for _ in 0..strip_components {
            components.next();
        }
        let path = components.as_path();

        // If the path is now empty (e.g. it was the root folder itself), skip it
        if path.as_os_str().is_empty() {
            continue;
        }

        let output = dest.join(path);

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }

        entry.unpack(output)?;
    }

    Ok(())
}

pub(crate) fn extract_archive(
    archive: &Path,
    dest: &Path,
    strip_components: usize,
) -> Result<(), Error> {
    std::fs::create_dir_all(dest)?;

    let file = File::open(archive)?;

    // We start with a file stream, and dynamically wrap it in decoders
    let mut stream: Box<dyn Read> = Box::new(file);

    loop {
        // Use BufReader to peek at the magic bytes without consuming them
        let mut buf = BufReader::new(stream);
        let magic = {
            let m = buf.fill_buf()?;
            if m.is_empty() {
                return Err(Error::BuilderInvalidData {
                    message: "empty archive".into(),
                });
            }
            m.to_vec() // Clone to end the borrow so we can move `buf` later
        };

        match detect_format(&magic) {
            Some(Format::Tar) => {
                return extract_tar(buf, dest, strip_components);
            }
            Some(Format::Gzip) => {
                stream = Box::new(flate2::read::MultiGzDecoder::new(buf));
            }
            Some(Format::Bzip2) => {
                stream = Box::new(bzip2::read::BzDecoder::new(buf));
            }
            Some(Format::Xz) => {
                stream = Box::new(XzReader::new(buf, false));
            }
            Some(Format::Lzip) => {
                stream = Box::new(LzipReader::new(buf));
            }
            Some(Format::Lzma) => {
                // LZMA requires consuming the 13-byte header before passing to LzmaReader
                let mut header = [0u8; 13];
                buf.read_exact(&mut header)?;

                let props = header[0] as u32;
                let lc = props % 9;
                let props = props / 9;
                let lp = props % 5;
                let pb = props / 5;

                let dict_size = u32::from_le_bytes(header[1..5].try_into().unwrap());
                let uncompressed_size = u64::from_le_bytes(header[5..13].try_into().unwrap());

                let size = if uncompressed_size == u64::MAX {
                    u64::MAX
                } else {
                    uncompressed_size
                };

                stream = Box::new(LzmaReader::new(buf, size, lc, lp, pb, dict_size, None)?);
            }
            None => {
                return Err(Error::BuilderInvalidData {
                    message: "unsupported or invalid archive format".into(),
                });
            }
        }
    }
}
