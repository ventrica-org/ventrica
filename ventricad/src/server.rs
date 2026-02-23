use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

use ventricad::protocol::{Message, Request};

pub fn run(socket_path: &Path) -> std::io::Result<()> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }

    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(socket_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o666))?;
    }

    eprintln!("ventricad: listening on {}", socket_path.display());

    for incoming in listener.incoming() {
        match incoming {
            Ok(stream) => {
                std::thread::spawn(move || {
                    if let Err(e) = handle_connection(stream) {
                        eprintln!("ventricad: connection error: {e}");
                    }
                });
            }
            Err(e) => eprintln!("ventricad: accept error: {e}"),
        }
    }

    Ok(())
}

#[cfg(unix)]
#[allow(unsafe_code)]
fn peer_uid_gid(stream: &UnixStream) -> Option<(u32, u32)> {
    use std::os::unix::io::AsRawFd;
    let fd = stream.as_raw_fd();

    #[cfg(target_os = "macos")]
    {
        let mut uid: libc::uid_t = 0;
        let mut gid: libc::gid_t = 0;
        if unsafe { libc::getpeereid(fd, &mut uid, &mut gid) } == 0 {
            return Some((uid, gid));
        }
        return None;
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut cred = libc::ucred {
            pid: 0,
            uid: 0,
            gid: 0,
        };
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                &mut cred as *mut libc::ucred as *mut libc::c_void,
                &mut len,
            )
        } == 0
        {
            return Some((cred.uid, cred.gid));
        }
        None
    }
}

fn handle_connection(stream: UnixStream) -> std::io::Result<()> {

    #[cfg(unix)]
    let build_user = peer_uid_gid(&stream);
    #[cfg(not(unix))]
    let build_user: Option<(u32, u32)> = None;

    let mut writer = stream.try_clone()?;
    let reader = BufReader::new(stream);

    let mut send = |msg: &Message| -> std::io::Result<()> {
        let mut line = serde_json::to_string(msg)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        line.push('\n');
        writer.write_all(line.as_bytes())?;
        writer.flush()
    };

    for raw_line in reader.lines() {
        let raw_line = raw_line?;
        if raw_line.trim().is_empty() {
            continue;
        }

        let req: Request = match serde_json::from_str(&raw_line) {
            Ok(r) => r,
            Err(e) => {
                let _ = send(&Message::error(format!("malformed request: {e}")));
                let _ = send(&Message::Done);
                continue;
            }
        };

        crate::handler::dispatch(&req, build_user, &mut |msg| {
            let _ = send(msg);
        });

        let _ = send(&Message::Done);
    }

    Ok(())
}
