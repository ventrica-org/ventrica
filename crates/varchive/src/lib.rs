use std::fs;
use std::io::{self, BufWriter, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const MAGIC: &[u8] = b"var-archive-1";
const META: &[u8] = b"meta";

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

fn expect_str(r: &mut impl Read, expected: &[u8]) -> io::Result<()> {
    let got = read_str(r)?;

    if got != expected {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "var: expected {:?} got {:?}",
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&got)
            ),
        ));
    }

    Ok(())
}

pub fn pack(src: &Path, dest: &Path) -> io::Result<()> {
    pack_with_metadata::<()>(src, dest, None)
}

pub fn pack_with_metadata<T: serde::Serialize>(
    src: &Path,
    dest: &Path,
    metadata: Option<&T>,
) -> io::Result<()> {
    let file = fs::File::create(dest)?;
    let mut w = BufWriter::new(file);

    write_str(&mut w, MAGIC)?;
    write_str(&mut w, META)?;

    let metadata = match metadata {
        Some(value) => rmp_serde::to_vec_named(value).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to serialize metadata: {e}"),
            )
        })?,
        None => Vec::new(),
    };

    write_str(&mut w, &metadata)?;

    write_node(&mut w, src)?;
    w.flush()?;

    Ok(())
}

fn write_node(w: &mut impl Write, path: &Path) -> io::Result<()> {
    write_str(w, b"(")?;

    let meta = fs::symlink_metadata(path)?;

    if meta.file_type().is_symlink() {
        let target = fs::read_link(path)?;

        write_str(w, b"type")?;
        write_str(w, b"symlink")?;
        write_str(w, b"target")?;
        write_str(w, target.as_os_str().as_encoded_bytes())?;
    } else if meta.is_dir() {
        write_str(w, b"type")?;
        write_str(w, b"directory")?;

        let mut entries: Vec<_> = fs::read_dir(path)?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<io::Result<_>>()?;

        entries.sort();

        for name in entries {
            let child = path.join(&name);

            write_str(w, b"entry")?;
            write_str(w, b"(")?;
            write_str(w, b"name")?;
            write_str(w, name.as_encoded_bytes())?;
            write_str(w, b"node")?;

            write_node(w, &child)?;

            write_str(w, b")")?;
        }
    } else {
        let executable = meta.permissions().mode() & 0o111 != 0;

        write_str(w, b"type")?;
        write_str(w, b"regular")?;

        if executable {
            write_str(w, b"executable")?;
            write_str(w, b"")?;
        }

        let contents = fs::read(path)?;

        write_str(w, b"contents")?;
        write_str(w, &contents)?;
    }

    write_str(w, b")")?;

    Ok(())
}

pub fn unpack(src: &Path, dest: &Path) -> io::Result<()> {
    let mut f = fs::File::open(src)?;

    expect_str(&mut f, MAGIC)?;
    fs::create_dir_all(dest)?;

    let token = read_str(&mut f)?;

    if token == META {
        read_str(&mut f)?;
        read_node(&mut f, dest)?;
    } else if token == b"(" {
        read_node_after_open(&mut f, dest)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "var: expected {:?} or '(' got {:?}",
                String::from_utf8_lossy(META),
                String::from_utf8_lossy(&token)
            ),
        ));
    }

    Ok(())
}

pub fn read_metadata<T: serde::de::DeserializeOwned>(src: &Path) -> io::Result<Option<T>> {
    let mut f = fs::File::open(src)?;

    expect_str(&mut f, MAGIC)?;

    let token = read_str(&mut f)?;

    if token == b"(" {
        return Ok(None);
    }

    if token != META {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "var: expected {:?} or '(' got {:?}",
                String::from_utf8_lossy(META),
                String::from_utf8_lossy(&token)
            ),
        ));
    }

    let bytes = read_str(&mut f)?;

    if bytes.is_empty() {
        return Ok(None);
    }

    let metadata = rmp_serde::from_slice::<T>(&bytes).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to deserialize metadata: {e}"),
        )
    })?;

    Ok(Some(metadata))
}

fn read_node(r: &mut impl Read, dest: &Path) -> io::Result<()> {
    expect_str(r, b"(")?;
    read_node_after_open(r, dest)
}

fn read_node_after_open(r: &mut impl Read, dest: &Path) -> io::Result<()> {
    expect_str(r, b"type")?;

    let kind = read_str(r)?;

    match kind.as_slice() {
        b"symlink" => {
            expect_str(r, b"target")?;

            let target_bytes = read_str(r)?;

            let target = unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(&target_bytes) };

            std::os::unix::fs::symlink(Path::new(target), dest)?;
        }

        b"regular" => {
            let next = read_str(r)?;

            let (executable, contents) = if next == b"executable" {
                read_str(r)?;

                expect_str(r, b"contents")?;

                (true, read_str(r)?)
            } else if next == b"contents" {
                (false, read_str(r)?)
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "var: unexpected token {:?} in regular file",
                        String::from_utf8_lossy(&next)
                    ),
                ));
            };

            fs::write(dest, &contents)?;

            let mode = if executable { 0o755 } else { 0o644 };
            fs::set_permissions(dest, fs::Permissions::from_mode(mode))?;
        }

        b"directory" => {
            fs::create_dir_all(dest)?;

            loop {
                let token = read_str(r)?;

                if token == b")" {
                    return Ok(());
                }

                if token != b"entry" {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "var: expected 'entry' or ')' got {:?}",
                            String::from_utf8_lossy(&token)
                        ),
                    ));
                }

                expect_str(r, b"(")?;
                expect_str(r, b"name")?;

                let name_bytes = read_str(r)?;

                let name = unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(&name_bytes) };

                let name_path = Path::new(name);

                if name_path
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(_)))
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "var: unsafe path component in archive: {:?}",
                            name_path.display()
                        ),
                    ));
                }

                expect_str(r, b"node")?;

                let child = dest.join(name_path);
                read_node(r, &child)?;

                expect_str(r, b")")?;
            }
        }

        other => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "var: unknown node type {:?}",
                    String::from_utf8_lossy(other)
                ),
            ));
        }
    }

    expect_str(r, b")")?;

    Ok(())
}
