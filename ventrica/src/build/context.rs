use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(unix)]
use std::os::unix::process::CommandExt as _;

use crate::error::{Error, Result};
use crate::schema::package::Build;

#[derive(Debug)]
pub struct BuildContext<'a> {
    /// Extracted source root (or `working_dir` override inside it).
    pub src: &'a Path,
    /// Out-of-tree build directory (may equal `src` for in-tree builds).
    pub build: &'a Path,
    /// Staging `DESTDIR`; install commands write here.
    pub dest: &'a Path,
    /// Top-level scratch directory (used by the sandbox profile).
    pub scratch: &'a Path,
    /// Fully-resolved environment for child processes.
    pub env: &'a HashMap<String, String>,
    /// Resolved recipe `build` section.
    pub spec: &'a Build,
    /// Package name (used in error messages and log output).
    pub name: &'a str,
    /// Install prefix embedded in binaries/dylibs at build time (`/ventrica/live`).
    pub prefix: &'a str,
    /// When `true`, wrap child processes with `sandbox-exec` on macOS.
    pub sandboxed: bool,
    /// UID/GID to drop build subprocesses to when running as root.
    pub build_user: Option<(u32, u32)>,
}

pub(crate) fn run_cmd_in(
    ctx: &BuildContext<'_>,
    cwd: &Path,
    prog: &str,
    args: &[String],
) -> Result<()> {
    run_cmd_with_env(ctx, cwd, prog, args, ctx.env)
}

pub(crate) fn run_cmd_with_env(
    ctx: &BuildContext<'_>,
    cwd: &Path,
    prog: &str,
    args: &[String],
    env: &HashMap<String, String>,
) -> Result<()> {
    let mut base = Command::new(prog);
    base.args(args);

    let mut cmd = if ctx.sandboxed {
        crate::build::sandbox::wrap(base, ctx.scratch).map_err(|e| Error::BuildFailed {
            name: ctx.name.into(),
            reason: e.to_string(),
        })?
    } else {
        base
    };

    cmd.current_dir(cwd)
        .env_clear()
        .envs(env)
        .env("DESTDIR", ctx.dest.display().to_string())
        .env("PREFIX", ctx.prefix)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    log::debug!("running command: {:?}", cmd);

    // each build command runs in its own process group so Ctrl+C is allowed
    #[cfg(unix)]
    #[allow(unsafe_code)]
    unsafe {
        cmd.pre_exec(|| {
            libc::setpgid(0, 0);
            Ok(())
        });
    }

    #[cfg(unix)]
    drop_privileges(&mut cmd, ctx.build_user);

    run_command(cmd, ctx.name)
}

fn run_command(mut cmd: Command, name: &str) -> Result<()> {
    let status = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::BuildFailed {
            name: name.into(),
            reason: format!("command exited with {status}"),
        })
    }
}

#[cfg(unix)]
#[allow(unsafe_code)]
fn drop_privileges(cmd: &mut Command, build_user: Option<(u32, u32)>) {
    let Some((uid, gid)) = build_user else { return };
    unsafe {
        cmd.pre_exec(move || {
            if libc::setgid(gid) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::setuid(uid) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

#[cfg(unix)]
#[allow(unsafe_code)]
pub fn chown_scratch(dir: &Path, build_user: Option<(u32, u32)>) {
    let Some((uid, gid)) = build_user else { return };
    chown_recursive(dir, uid, gid);
}

#[cfg(unix)]
#[allow(unsafe_code)]
fn chown_recursive(dir: &Path, uid: u32, gid: u32) {
    use std::os::unix::ffi::OsStrExt;

    let Ok(cstr) = std::ffi::CString::new(dir.as_os_str().as_bytes()) else {
        return;
    };
    unsafe { libc::lchown(cstr.as_ptr(), uid, gid) };

    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        let Ok(cstr) = std::ffi::CString::new(p.as_os_str().as_bytes()) else {
            continue;
        };
        unsafe { libc::lchown(cstr.as_ptr(), uid, gid) };
        if p.is_dir() && !p.is_symlink() {
            chown_recursive(&p, uid, gid);
        }
    }
}

#[must_use]
pub fn build_env(recipe_env: &HashMap<String, String>) -> HashMap<String, String> {
    use crate::store::LIVE_PREFIX;

    let mut env: HashMap<String, String> = HashMap::new();

    env.insert(
        "PATH".into(),
        format!("{LIVE_PREFIX}/bin:/usr/bin:/bin:/usr/sbin:/sbin"),
    );

    env.insert("CC".into(), "clang".into());
    env.insert("CXX".into(), "clang++".into());
    env.insert("AR".into(), "ar".into());

    env.insert(
        "ACLOCAL_PATH".into(),
        format!("{LIVE_PREFIX}/share/aclocal"),
    );

    env.insert("CPPFLAGS".into(), format!("-I{LIVE_PREFIX}/include"));
    env.insert("CFLAGS".into(), format!("-I{LIVE_PREFIX}/include"));
    env.insert("CXXFLAGS".into(), format!("-I{LIVE_PREFIX}/include"));
    env.insert(
        "PKG_CONFIG_PATH".into(),
        format!("{LIVE_PREFIX}/lib/pkgconfig:{LIVE_PREFIX}/share/pkgconfig"),
    );
    env.insert("LDFLAGS".into(), format!("-L{LIVE_PREFIX}/lib"));

    #[cfg(target_os = "macos")]
    {
        env.insert("MACOSX_DEPLOYMENT_TARGET".into(), "11.0".into());
        env.insert("DYLD_LIBRARY_PATH".into(), format!("{LIVE_PREFIX}/lib"));
    }

    // allows the recipe to override
    for (k, v) in recipe_env {
        env.insert(k.clone(), v.clone());
    }

    env
}
