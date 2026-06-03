pub mod application;
pub mod shell;

use crate::build::context::BuildContext;
use crate::error::Result;

/// A build-system driver.
pub trait BuildDriver {
    fn run(&self, ctx: &BuildContext<'_>) -> Result<()>;
}

pub fn dispatch(system: Option<&str>, ctx: &BuildContext<'_>) -> Result<()> {
    match system.unwrap_or("shell") {
        "shell" => shell::ShellDriver.run(ctx),
        "application" => application::ApplicationDriver.run(ctx),
        "none" => Ok(()),
        _ => shell::ShellDriver.run(ctx),
    }
}
