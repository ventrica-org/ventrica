use std::fs;
use std::path::Path;

use crate::Error;
use crate::builder::PackageBuilderOptions;

// hdiutil comes with the system, not open source.
const HDIUTIL: &str = "/usr/bin/hdiutil";

pub struct MacOSApplicationDriver;
impl super::BuildDriver for MacOSApplicationDriver {
    fn extract(&self, archive_path: &Path, options: &PackageBuilderOptions) -> Result<(), Error> {
        let sandbox = super::make_sandbox(options);

        let dest_dir = options.build_dest_dir().join("Applications");
        fs::create_dir_all(&dest_dir)?;

        let mount_dir = options.build_dir().join("dmg");
        fs::create_dir_all(&mount_dir)?;

        let mut status = sandbox
            .command(HDIUTIL)
            .args([
                "attach",
                archive_path.to_str().unwrap(),
                "-mountpoint",
                mount_dir.to_str().unwrap(),
                "-nobrowse",
            ])
            .spawn()?
            .wait()?;

        if !status.success() {
            return Err(Error::BuilderCommandFailed {
                name: "hdiutil attach".into(),
                status,
            });
        }

        let app = fs::read_dir(&mount_dir)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .find(|path| path.extension().is_some_and(|ext| ext == "app"))
            .ok_or(Error::BuilderMacOSAppNotFound)?;

        status = sandbox
            .command("cp")
            .args(["-R"])
            .arg(app)
            .arg(&dest_dir)
            .envs(options.env())
            .spawn()?
            .wait()?;

        if !status.success() {
            return Err(Error::BuilderCommandFailed {
                name: "cp".into(),
                status,
            });
        }

        status = sandbox
            .command(HDIUTIL)
            .args(["detach", mount_dir.to_str().unwrap(), "-quiet"])
            .spawn()?
            .wait()?;

        if !status.success() {
            return Err(Error::BuilderCommandFailed {
                name: "hdiutil detach".into(),
                status,
            });
        }

        Ok(())
    }
}
