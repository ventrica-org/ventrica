mod reader;
#[cfg(feature = "builder")]
mod writer;

pub use reader::StationReader;
#[cfg(feature = "builder")]
pub use writer::StationWriter;

pub(crate) const MANIFEST_FILE: &str = "manifest";
pub(crate) const MANIFEST_HASH_FILE: &str = "manifest.sha256";
