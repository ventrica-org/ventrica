use std::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Architecture {
    X86,
    X86_64,
    Arm64,
}

impl Display for Architecture {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Architecture::X86 => write!(f, "x86"),
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Arm64 => write!(f, "arm64"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Platform {
    Linux,
    MacOS,
}

impl Display for Platform {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Platform::Linux => write!(f, "linux"),
            Platform::MacOS => write!(f, "mac"),
        }
    }
}

pub const fn host() -> Architecture {
    #[cfg(target_arch = "x86")]
    {
        Architecture::X86
    }
    #[cfg(target_arch = "x86_64")]
    {
        Architecture::X86_64
    }
    #[cfg(target_arch = "aarch64")]
    {
        Architecture::Arm64
    }
}

pub fn platform() -> Platform {
    #[cfg(target_os = "linux")]
    {
        Platform::Linux
    }
    #[cfg(target_os = "macos")]
    {
        Platform::MacOS
    }
}

#[must_use]
pub fn package_supports_host(platforms: &[String]) -> bool {
    if platforms.is_empty() {
        return true;
    }

    let host_os = platform().to_string();
    let host_arch = host().to_string();

    platforms.iter().any(|entry| {
        let normalized = entry.to_ascii_lowercase();
        normalized == format!("{host_os}_{host_arch}")
            || normalized == format!("{host_os}_{host_arch}")
            || normalized == format!("{host_os}_{host_arch}")
            || normalized == format!("{host_os}_{host_arch}")
    })
}
