//! | Variable         | Expands to                                        |
//! |------------------|---------------------------------------------------|
//! | `DESTDIR`        | Staging destination directory                     |
//! | `PREFIX`         | Hardcoded live prefix (`/ventrica/live`)           |
//! | `SRCDIR`         | Extracted source root                             |
//! | `BUILDDIR`       | Out-of-tree build directory                       |
//! | `NPROC`          | Logical CPU count                                 |
//! | `ARCH`           | Host architecture (`aarch64`/`x86_64`)            |
//! | `NAME`           | Package name                                      |
//! | `MACOS_SDK`      | macOS SDK path from `xcrun --show-sdk-path`       |

use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Environment<'a> {
    pub destdir: &'a str,
    pub prefix: &'a str,
    pub srcdir: &'a str,
    pub builddir: &'a str,
    pub name: &'a str,
}

/// replaces `${VAR}` (and bare `$VAR`) tokens in `s` using our variables.
#[must_use]
pub fn expand(s: &str, vars: &Environment<'_>) -> String {
    if !s.contains('$') {
        return s.to_owned();
    }

    let mut result = String::with_capacity(s.len() + 64);
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'$' {
            result.push(bytes[i] as char);
            i += 1;
            continue;
        }

        // `$` found - peek at next character
        i += 1;
        let (var_name, consumed) = if i < bytes.len() && bytes[i] == b'{' {
            // `${VAR}` form
            i += 1; // skip `{`
            let start = i;
            while i < bytes.len() && bytes[i] != b'}' {
                i += 1;
            }
            let name = &s[start..i];
            if i < bytes.len() {
                i += 1; // skip `}`
            }
            (name, 0usize)
        } else {
            // `$VAR` form - read until non-alphanumeric/underscore
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            (&s[start..i], 0usize)
        };
        let _ = consumed;

        let replacement = resolve(var_name, vars);
        result.push_str(replacement);
    }

    result
}

pub fn expand_all(args: &mut Vec<String>, vars: &Environment<'_>) {
    for arg in args {
        let expanded = expand(arg, vars);
        *arg = expanded;
    }
}

fn resolve<'v>(name: &str, vars: &'v Environment<'_>) -> &'v str {
    match name {
        "DESTDIR" => vars.destdir,
        "PREFIX" => vars.prefix,
        "SRCDIR" => vars.srcdir,
        "BUILDDIR" => vars.builddir,
        "NAME" => vars.name,
        "NPROC" => nproc_str(),
        "ARCH" => host_arch(),
        "MACOS_SDK" => macos_sdk(),
        _ => "",
    }
}

fn nproc_str() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(|| {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .to_string()
    })
}

fn macos_sdk() -> &'static str {
    static S: OnceLock<String> = OnceLock::new();
    S.get_or_init(|| {
        std::process::Command::new("xcrun")
            .args(["--show-sdk-path"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_owned())
            .unwrap_or_default()
    })
}

fn host_arch() -> &'static str {
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64"
    }
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64"
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> Environment<'static> {
        Environment {
            destdir: "/tmp/dest",
            prefix: "/home/user/.ventrica/live",
            srcdir: "/tmp/src",
            builddir: "/tmp/build",
            name: "mypkg",
        }
    }

    #[test]
    fn expand_braced() {
        assert_eq!(expand("${DESTDIR}/usr", &vars()), "/tmp/dest/usr");
    }

    #[test]
    fn expand_bare() {
        assert_eq!(
            expand("$PREFIX/bin", &vars()),
            "/home/user/.ventrica/live/bin"
        );
    }

    #[test]
    fn no_dollar_fast_path() {
        let s = "hello world";
        assert_eq!(expand(s, &vars()), s);
    }
}
