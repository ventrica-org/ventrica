use std::{
    fs,
    path::{Path, PathBuf},
};

use bincode_next::config;

use crate::{
    Error,
    store::{Package, Repo},
    utils::sha::sha256_file,
};

pub struct StationWriter {
    path: PathBuf,
}

impl StationWriter {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();

        if !path.join("repo.kdl").is_file() {
            return Err(Error::BuilderRepoInvalid(path.display().to_string()));
        }

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub fn write(&self) -> Result<(), Error> {
        let encoded = self.build()?;
        let cache_path = self.path.join("cache");
        let manifest_path = cache_path.join(super::MANIFEST_FILE);
        let manifest_hash_path = cache_path.join(super::MANIFEST_HASH_FILE);
        fs::write(&manifest_path, encoded)?;
        fs::write(&manifest_hash_path, sha256_file(&manifest_path)?)?;
        Ok(())
    }

    fn build(&self) -> Result<Vec<u8>, Error> {
        let mut repo = self.read_repo()?;
        repo.packages = Some(self.collect_packages()?);

        let encoded = bincode_next::serde::encode_to_vec(&repo, config::standard())?;
        Ok(encoded)
    }

    fn read_repo(&self) -> Result<Repo, Error> {
        let contents = fs::read_to_string(self.path.join("repo.kdl"))?;
        Ok(kdl::de::from_str::<Repo>(&contents)?)
    }

    fn collect_packages(&self) -> Result<Vec<Package>, Error> {
        let cache = self.path.join("cache");
        let mut packages = Vec::new();

        for entry in fs::read_dir(cache)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if path.extension().is_none_or(|extension| extension != "var") {
                continue;
            }

            if let Some(package) = varchive::read_metadata::<Package>(&path)? {
                packages.push(package);
            }
        }

        Ok(packages)
    }
}
