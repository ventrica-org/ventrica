//! Application driver: mounts a `.dmg` and copies `.app` bundles.

use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};

use super::BuildDriver;
use crate::build::context::BuildContext;
use crate::error::{Error, Result};

pub struct ApplicationDriver;

impl BuildDriver for ApplicationDriver {
    fn run(&self, ctx: &BuildContext<'_>) -> Result<()> {
        let dmg = find_dmg(ctx)?;

        let mount_point = ctx.scratch.join("dmg-mount");
        std::fs::create_dir_all(&mount_point)?;

        mount_dmg(&dmg, &mount_point, ctx.name)?;
        let result = copy_apps(&mount_point, ctx.dest, ctx.name);
        detach_dmg(&mount_point);
        result
    }
}

fn find_dmg(ctx: &BuildContext<'_>) -> Result<std::path::PathBuf> {
    std::fs::read_dir(ctx.src)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension() == Some(OsStr::new("dmg")))
        .map(|e| e.path())
        .ok_or_else(|| Error::BuildFailed {
            name: ctx.name.into(),
            reason: "no .dmg file found in source directory".into(),
        })
}

fn mount_dmg(dmg: &Path, mount_point: &Path, pkg_name: &str) -> Result<()> {
    log::info!("mounting {}", dmg.display());
    let status = Command::new("/usr/bin/hdiutil")
        .args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
        .arg(mount_point)
        .arg(dmg)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| Error::BuildFailed {
            name: pkg_name.into(),
            reason: format!("hdiutil attach: {e}"),
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(Error::BuildFailed {
            name: pkg_name.into(),
            reason: format!("hdiutil attach exited with {status}"),
        })
    }
}

fn detach_dmg(mount_point: &Path) {
    let result = Command::new("/usr/bin/hdiutil")
        .args(["detach", "-quiet"])
        .arg(mount_point)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .status();
    if let Ok(s) = result {
        if !s.success() {
            log::warn!("hdiutil detach exited with {s}");
        }
    }
}

fn copy_apps(mount: &Path, dest: &Path, pkg_name: &str) -> Result<()> {
    let apps_dest = dest.join("Applications");
    std::fs::create_dir_all(&apps_dest)?;

    let mut found = false;
    for entry in std::fs::read_dir(mount)? {
        let entry = entry?;
        let src = entry.path();
        if src.extension() != Some(OsStr::new("app")) {
            continue;
        }
        let target = apps_dest.join(entry.file_name());
        log::info!("{} -> {}", src.display(), target.display());
        let status = Command::new("/usr/bin/ditto")
            .arg(&src)
            .arg(&target)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|e| Error::BuildFailed {
                name: pkg_name.into(),
                reason: format!("ditto failed: {e}"),
            })?;
        if !status.success() {
            return Err(Error::BuildFailed {
                name: pkg_name.into(),
                reason: format!("ditto exited with {status}"),
            });
        }
        found = true;
    }

    if found {
        Ok(())
    } else {
        Err(Error::BuildFailed {
            name: pkg_name.into(),
            reason: "no .app bundle found in DMG".into(),
        })
    }
}
