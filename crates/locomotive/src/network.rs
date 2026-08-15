use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

use crate::Error;

const USER_AGENT: &str = "Ventrica/1.0";

pub struct NetworkManager {
    client: Client,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn download_file(
        &self,
        urls: &[String],
        dest_dir: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<PathBuf, Error> {
        let mut last_error: Option<Error> = None;

        for url in urls {
            match self.download_single_file(url, dest_dir, expected_sha256) {
                Ok(file_path) => return Ok(file_path),
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::NetworkInvalidUrl("No URLs provided or all mirror downloads failed".to_string())
        }))
    }

    fn download_single_file(
        &self,
        url: &str,
        dest_dir: &Path,
        expected_sha256: Option<&str>,
    ) -> Result<PathBuf, Error> {
        let file_name = url
            .rsplit('/')
            .next()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error::NetworkInvalidUrl(url.to_string()))?;

        let dest_path = dest_dir.join(file_name);

        let mut response = self.client.get(url).send()?.error_for_status()?;
        let mut file = File::create(&dest_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let bytes_read = match response.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    let _ = std::fs::remove_file(&dest_path);
                    return Err(Error::Io(e));
                }
            };

            let chunk = &buffer[..bytes_read];

            if let Err(e) = file.write_all(chunk) {
                let _ = std::fs::remove_file(&dest_path);
                return Err(Error::Io(e));
            }

            if expected_sha256.is_some() {
                hasher.update(chunk);
            }
        }

        if let Some(expected) = expected_sha256 {
            let actual = hex::encode(hasher.finalize());
            if !actual.eq_ignore_ascii_case(expected) {
                let _ = std::fs::remove_file(&dest_path);
                return Err(Error::NetworkHashMismatch {
                    expected: expected.to_string(),
                    actual,
                });
            }
        }

        Ok(dest_path)
    }
}
