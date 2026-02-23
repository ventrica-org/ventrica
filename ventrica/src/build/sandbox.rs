use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};

const PROFILE_TPL: &str = r#"(version 1)
(allow default)
(deny file-write*)
(allow file-write*
  (subpath "/private/tmp")
  (subpath "/private/var/tmp")
  (subpath "/tmp")
  (subpath "PROCESS_TMPDIR")
  (literal "/dev/null")
  (literal "/dev/zero")
  (literal "/dev/stderr")
  (literal "/dev/stdout")
  (regex "^/dev/fd/[0-9]+$")
  (subpath "SCRATCH")
)
(allow network-outbound)
(allow network-inbound)
(allow process-exec*)
(allow signal)
(allow sysctl*)
(allow ipc-posix-shm)
(allow mach-lookup)
"#;

#[must_use]
pub fn profile_for(scratch: &Path) -> String {
    let real_scratch = std::fs::canonicalize(scratch).unwrap_or_else(|_| scratch.to_owned());

    let tmpdir = std::env::var("TMPDIR").unwrap_or_else(|_| "/var/folders".into());
    let real_tmpdir = std::fs::canonicalize(&tmpdir).unwrap_or_else(|_| tmpdir.into());

    PROFILE_TPL
        .replace("SCRATCH", &real_scratch.display().to_string())
        .replace("PROCESS_TMPDIR", &real_tmpdir.display().to_string())
}

pub fn wrap(cmd: Command, scratch: &Path) -> Result<Command> {
    if !sandbox_exec_available() {
        return Err(Error::Sandbox(
            "sandbox-exec is not available on this system".into(),
        ));
    }

    let profile = profile_for(scratch);
    let (prog, args) = cmd_parts(cmd);

    let mut wrapped = Command::new("/usr/bin/sandbox-exec");
    wrapped.arg("-p").arg(profile).arg(prog).args(args);

    Ok(wrapped)
}

#[must_use]
pub fn sandbox_exec_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        Path::new("/usr/bin/sandbox-exec").exists()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

fn cmd_parts(cmd: Command) -> (std::ffi::OsString, Vec<std::ffi::OsString>) {
    let prog = cmd.get_program().to_owned();
    let args = cmd.get_args().map(ToOwned::to_owned).collect();
    (prog, args)
}
