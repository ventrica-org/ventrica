use std::process::Stdio;
use std::{ffi::OsStr, path::PathBuf, process::Command};

use crate::SandboxOptions;
use crate::command::SandboxCommand;

const SANDBOX_PROFILE: &str = include_str!("sandbox.sb");

macro_rules! param {
    ($command:expr, $name:expr, $value:expr) => {
        $command.arg("-D").arg(format!("{}={}", $name, $value));
    };

    ($command:expr, $name:expr, bool $value:expr) => {
        $command.arg("-D").arg(format!(
            "{}={}",
            $name,
            if $value { "TRUE" } else { "FALSE" }
        ));
    };
}

pub struct Sandbox {
    path: PathBuf,
    toolchain: Option<PathBuf>,
    prefix: Option<PathBuf>,
    pub options: SandboxOptions,
}

impl Sandbox {
    pub fn new(path: impl Into<PathBuf>, options: SandboxOptions) -> Self {
        Self {
            path: path.into(),
            toolchain: None,
            prefix: None,
            options,
        }
    }

    pub fn toolchain(mut self, path: impl Into<PathBuf>) -> Self {
        self.toolchain = Some(path.into());
        self
    }

    pub fn prefix(mut self, path: impl Into<PathBuf>) -> Self {
        self.prefix = Some(path.into());
        self
    }

    pub fn command<S: AsRef<OsStr>>(&self, program: S) -> SandboxCommand {
        let mut command = Command::new("/usr/bin/sandbox-exec");

        command.arg("-p").arg(SANDBOX_PROFILE);

        param!(command, "SANDBOX_PATH", self.path.display());
        param!(command, "NETWORK_OUTBOUND", bool self.options.network_outbound);
        param!(command, "NETWORK_INBOUND", bool self.options.network_inbound);

        if let Some(toolchain) = &self.toolchain {
            param!(command, "VENTRICA_TOOLCHAIN", toolchain.display());
        }

        if let Some(prefix) = &self.prefix {
            param!(command, "VENTRICA_PREFIX", prefix.display());
        }

        command
            .arg(program)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        SandboxCommand { command }
    }
}
