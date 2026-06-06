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
        Architecture::Aarch64
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

pub enum Architecture {
    X86,
    X86_64,
    Aarch64,
    Universal,
}

pub enum Platform {
    Linux,
    MacOS,
}
