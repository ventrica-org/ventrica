mod models;
mod profile;
mod station;

use std::path::PathBuf;

use crate::Error;
use crate::db::ValidEntry;

pub use models::{
    package::{Dependency, Package},
    profile::Profile,
    repo::Repo,
};
pub use profile::GenerationInfo;

// Store layout:
//
//   <prefix>/store/
//       pkg-<hash>-<name>-<version>/      package files
//       pkg-<hash>-<name>-<version>.kdl   package metadata
//       stn-<hash>-<name>/                repository/station files
//       stn-<hash>-<name>.kdl             repository metadata
//       gen-<hash>/                       merged generation tree (hardlinks)
//       gen-<hash>.kdl                    generation metadata
//
//   <prefix>/var/lib/ventrica/profiles/
//       current            -> <prefix>/store/gen-<hash>   (active generation)
//       profile-<ts>       -> <prefix>/store/gen-<hash>   (archived generations)
//
//   <prefix>/usr           -> <prefix>/var/lib/ventrica/profiles/current

pub enum StoreEntryType {
    /// `<prefix>/store/stn-<hash>-<name>`
    RepositoryDir,
    /// `<prefix>/store/stn-<hash>-<name>.kdl`
    RepositoryMeta,
    /// `<prefix>/store/pkg-<hash>-<name>-<version>`
    PackageDir,
    /// `<prefix>/store/pkg-<hash>-<name>-<version>.kdl`
    PackageMeta,
    /// `<prefix>/store/gen-<hash>`
    GenerationDir,
    /// `<prefix>/store/gen-<hash>.kdl`
    GenerationMeta,
}

impl std::fmt::Display for StoreEntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreEntryType::RepositoryDir
            | StoreEntryType::PackageDir
            | StoreEntryType::GenerationDir => write!(f, "dir"),
            StoreEntryType::RepositoryMeta => write!(f, "repo"),
            StoreEntryType::PackageMeta => write!(f, "package"),
            StoreEntryType::GenerationMeta => write!(f, "profile"),
        }
    }
}

pub struct Store {
    prefix: PathBuf,
}

impl Store {
    pub fn new(prefix: PathBuf) -> Result<Self, Error> {
        Ok(Self { prefix })
    }

    pub fn activate(&self, entries: &[&ValidEntry]) -> Result<(), Error> {
        for entry in entries {
            if entry.r#type == "profile" {
                return Err(Error::StoreInvalidPath(entry.path.clone()));
            }
        }

        let packages: Vec<PathBuf> = entries.iter().map(|e| PathBuf::from(&e.path)).collect();
        let gen_path = profile::build(&self.prefix, &packages)?;
        profile::activate(&self.prefix, &gen_path)?;
        Ok(())
    }

    pub fn rollback(&self, number: Option<u32>) -> Result<(), Error> {
        profile::rollback(&self.prefix, number)
    }
}
