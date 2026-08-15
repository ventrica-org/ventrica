use std::process::Command;

pub struct SandboxCommand {
    pub command: Command,
}

impl std::ops::Deref for SandboxCommand {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl std::ops::DerefMut for SandboxCommand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}
