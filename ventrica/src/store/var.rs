//! VAR1 - Ventrica ARchive format
//!
#![allow(unsafe_code)]
use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::error::{Error, Result};

const MAGIC: &[u8] = b"var-archive-1";

fn write_str(w: &mut impl Write, s: &[u8]) -> io::Result<()> {
    let len = s.len() as u64;
    w.write_all(&len.to_le_bytes())?;
    w.write_all(s)?;
    let pad = (8 - (s.len() % 8)) % 8;
    if pad > 0 {
        w.write_all(&[0u8; 7][..pad])?;
    }
    Ok(())
}

fn read_str(r: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut len_buf = [0u8; 8];
    r.read_exact(&mut len_buf)?;
    let len = u64::from_le_bytes(len_buf) as usize;
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    let pad = (8 - (len % 8)) % 8;
    if pad > 0 {
        let mut discard = [0u8; 7];
        r.read_exact(&mut discard[..pad])?;
    }
    Ok(buf)
}

fn expect_str(r: &mut impl Read, expected: &[u8]) -> Result<()> {
    let got = read_str(r).map_err(Error::Io)?;
    if got != expected {
        return Err(Error::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "var: expected {:?} got {:?}",
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&got)
            ),
        )));
    }
    Ok(())
}

/// pack dir
pub fn pack(src: &Path, dest: &Path) -> Result<()> {
    let file = fs::File::create(dest)?;
    let mut w = BufWriter::new(file);
    write_str(&mut w, MAGIC).map_err(Error::Io)?;
    write_node(&mut w, src)?;
    w.flush().map_err(Error::Io)?;
    Ok(())
}

fn write_node(w: &mut impl Write, path: &Path) -> Result<()> {
    write_str(w, b"(").map_err(Error::Io)?;

    let meta = fs::symlink_metadata(path)?;

    if meta.file_type().is_symlink() {
        let target = fs::read_link(path)?;
        write_str(w, b"type").map_err(Error::Io)?;
        write_str(w, b"symlink").map_err(Error::Io)?;
        write_str(w, b"target").map_err(Error::Io)?;
        write_str(w, target.as_os_str().as_encoded_bytes()).map_err(Error::Io)?;
    } else if meta.is_dir() {
        write_str(w, b"type").map_err(Error::Io)?;
        write_str(w, b"directory").map_err(Error::Io)?;

        // collect
        let mut entries: Vec<_> = fs::read_dir(path)?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<io::Result<_>>()?;
        entries.sort();

        for name in entries {
            let child = path.join(&name);
            write_str(w, b"entry").map_err(Error::Io)?;
            write_str(w, b"(").map_err(Error::Io)?;
            write_str(w, b"name").map_err(Error::Io)?;
            write_str(w, name.as_encoded_bytes()).map_err(Error::Io)?;
            write_str(w, b"node").map_err(Error::Io)?;
            write_node(w, &child)?;
            write_str(w, b")").map_err(Error::Io)?;
        }
    } else {
        // regular file
        let executable = meta.permissions().mode() & 0o111 != 0;
        write_str(w, b"type").map_err(Error::Io)?;
        write_str(w, b"regular").map_err(Error::Io)?;
        if executable {
            write_str(w, b"executable").map_err(Error::Io)?;
            write_str(w, b"").map_err(Error::Io)?;
        }
        let contents = fs::read(path)?;
        write_str(w, b"contents").map_err(Error::Io)?;
        write_str(w, &contents).map_err(Error::Io)?;
    }

    write_str(w, b")").map_err(Error::Io)?;
    Ok(())
}

pub fn unpack(src: &Path, dest: &Path) -> Result<()> {
    let mut f = fs::File::open(src)?;
    expect_str(&mut f, MAGIC)?;
    fs::create_dir_all(dest)?;
    read_node(&mut f, dest)?;
    Ok(())
}

fn read_node(r: &mut impl Read, dest: &Path) -> Result<()> {
    expect_str(r, b"(")?;
    expect_str(r, b"type")?;

    let kind = read_str(r).map_err(Error::Io)?;

    match kind.as_slice() {
        b"symlink" => {
            expect_str(r, b"target")?;
            let target_bytes = read_str(r).map_err(Error::Io)?;
            // SAFETY: bytes came from a path we wrote they are valid
            // OS-encoded bytes on this platform....
            let target = unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(&target_bytes) };
            std::os::unix::fs::symlink(Path::new(target), dest)?;
        }
        b"regular" => {
            // Peek at the next token - either "executable", "contents"
            let next = read_str(r).map_err(Error::Io)?;
            let executable;
            let contents = if next == b"executable" {
                // consume the empty string marker
                let _ = read_str(r).map_err(Error::Io)?;
                executable = true;
                expect_str(r, b"contents")?;
                read_str(r).map_err(Error::Io)?
            } else if next == b"contents" {
                executable = false;
                read_str(r).map_err(Error::Io)?
            } else {
                return Err(Error::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "var: unexpected token {:?} in regular file",
                        String::from_utf8_lossy(&next)
                    ),
                )));
            };

            fs::write(dest, &contents)?;
            let mode = if executable { 0o755 } else { 0o644 };
            fs::set_permissions(dest, fs::Permissions::from_mode(mode))?;
        }
        b"directory" => {
            fs::create_dir_all(dest)?;

            // Read entries until we see ")"
            loop {
                let token = read_str(r).map_err(Error::Io)?;
                if token == b")" {
                    return Ok(());
                }
                if token != b"entry" {
                    return Err(Error::Io(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "var: expected 'entry' or ')' got {:?}",
                            String::from_utf8_lossy(&token)
                        ),
                    )));
                }
                expect_str(r, b"(")?;
                expect_str(r, b"name")?;
                let name_bytes = read_str(r).map_err(Error::Io)?;
                let name = unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(&name_bytes) };
                // Reject any component that would escape the destination directory.
                let name_path = Path::new(name);
                if name_path
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(_)))
                {
                    return Err(Error::Io(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "var: unsafe path component in archive: {:?}",
                            name_path.display()
                        ),
                    )));
                }
                expect_str(r, b"node")?;
                let child = dest.join(name_path);
                read_node(r, &child)?;
                expect_str(r, b")")?;
            }
        }
        other => {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "var: unknown node type {:?}",
                    String::from_utf8_lossy(other)
                ),
            )));
        }
    }

    expect_str(r, b")")?;
    Ok(())
}

pub fn hash_file(path: &Path) -> Result<String> {
    crate::store::sha256_file(path)
}
