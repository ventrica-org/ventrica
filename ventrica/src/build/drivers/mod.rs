pub mod application;
pub mod shell;

use crate::build::context::BuildContext;
use crate::error::Result;
use crate::schema::package::BuildSystem;

/// A build-system driver.
pub trait BuildDriver {
    fn run(&self, ctx: &BuildContext<'_>) -> Result<()>;
}

pub fn dispatch(system: BuildSystem, ctx: &BuildContext<'_>) -> Result<()> {
    match system {
        BuildSystem::Shell => shell::ShellDriver.run(ctx),
        BuildSystem::Application => application::ApplicationDriver.run(ctx),
        BuildSystem::None => Ok(()),
    }
}
