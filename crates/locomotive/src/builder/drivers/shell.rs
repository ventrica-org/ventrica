use std::path::Path;

use crate::Error;
use crate::builder::PackageBuilderOptions;
use crate::builder::extract::extract_archive;

pub struct ShellDriver;
impl super::BuildDriver for ShellDriver {
    fn extract(&self, archive_path: &Path, options: &PackageBuilderOptions) -> Result<(), Error> {
        extract_archive(archive_path, &options.build_src_dir(), 1)?;
        Ok(())
    }

    fn run(&self, options: &PackageBuilderOptions) -> Result<(), Error> {
        let Some(build) = &options.package().build else {
            return Ok(());
        };

        let sandbox = super::make_sandbox(options);

        let status = sandbox
            .command("sh")
            .arg("-e")
            .arg("-c")
            .arg(build)
            .envs(options.env())
            .current_dir(&options.build_src_dir())
            .spawn()?
            .wait()?;

        if !status.success() {
            return Err(Error::BuilderCommandFailed {
                name: "sh".into(),
                status,
            });
        }

        Ok(())
    }
}
