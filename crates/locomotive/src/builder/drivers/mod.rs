mod dmg;
mod shell;

pub use dmg::MacOSApplicationDriver;
pub use shell::ShellDriver;

use std::path::Path;

use crate::Error;
use crate::builder::PackageBuilderOptions;

pub trait BuildDriver {
    fn extract(&self, _archive_path: &Path, _options: &PackageBuilderOptions) -> Result<(), Error> {
        Ok(())
    }

    fn run(&self, _options: &PackageBuilderOptions) -> Result<(), Error> {
        Ok(())
    }
}

pub(crate) fn make_sandbox(options: &PackageBuilderOptions) -> sandbox::Sandbox {
    let sandbox_options = sandbox::SandboxOptions::default();

    let mut sandbox = sandbox::Sandbox::new(options.build_dir(), sandbox_options);
    if let Some(prefix) = options.prefix() {
        sandbox = sandbox.prefix(prefix);
    }

    sandbox
}
