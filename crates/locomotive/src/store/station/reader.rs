use std::path::{Path, PathBuf};

use crate::Error;
use crate::network::NetworkManager;
use crate::store::Repo;
use crate::utils::dir::make_temp_dir_name;

pub struct StationReader {
    dest_path: PathBuf,
}

impl StationReader {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let dest_path = path.as_ref().to_path_buf();

        if !dest_path.is_file() {
            return Err(Error::StationReaderInvalidPath(
                dest_path.display().to_string(),
            ));
        }

        Ok(Self { dest_path })
    }

    pub fn new_with_url(url: String) -> Result<Self, Error> {
        let tmp_path = std::env::temp_dir().join(make_temp_dir_name("ventrica-manifest"));

        let dest_path = NetworkManager::new().download_file(&[url], &tmp_path, None)?;
        Ok(Self { dest_path })
    }

    pub fn read(&self) -> Result<Repo, Error> {
        let contents = std::fs::read_to_string(&self.dest_path)?;
        Ok(kdl::de::from_str::<Repo>(&contents)?)
    }

    pub fn dest_path(&self) -> &Path {
        &self.dest_path
    }
}
