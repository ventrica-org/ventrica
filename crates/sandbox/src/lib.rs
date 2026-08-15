mod command;
mod sandbox;

#[derive(Debug, Clone, Copy)]
pub struct SandboxOptions {
    /// Whether to allow inbound network connections (e.g. listening on a port).
    pub network_inbound: bool,
    /// Whether to allow outbound network connections (e.g. making HTTP requests).
    pub network_outbound: bool,
}

impl Default for SandboxOptions {
    fn default() -> Self {
        Self {
            network_inbound: false,
            network_outbound: false,
        }
    }
}

pub use command::SandboxCommand;
pub use sandbox::Sandbox;
