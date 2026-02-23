use std::path::{Path, PathBuf};

use super::file_types;
use crate::error::{Error, Result};
use crate::store::verify_sha256;

const USER_AGENT: &str = concat!("Ventrica/", "0");

pub fn download(url: &str, dest: &Path, mirrors: &[String]) -> Result<PathBuf> {
    if dest.exists() {
        return Ok(dest.to_owned());
    }
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut last_err = String::new();
    for u in std::iter::once(url).chain(mirrors.iter().map(String::as_str)) {
        match download_one(u, dest) {
            Ok(()) => return Ok(dest.to_owned()),
            Err(e) => {
                last_err = e.to_string();
                let _ = std::fs::remove_file(dest);
            }
        }
    }

    Err(Error::DownloadFailed {
        url: url.into(),
        reason: last_err,
    })
}

fn download_one(url: &str, dest: &Path) -> Result<()> {
    use std::io::Write;

    macro_rules! curl {
        ($expr:expr) => {
            $expr.map_err(|e| Error::DownloadFailed {
                url: url.into(),
                reason: e.to_string(),
            })?
        };
    }

    let mut easy = curl::easy::Easy::new();
    curl!(easy.url(url));
    curl!(easy.follow_location(true));
    curl!(easy.fail_on_error(true));
    curl!(easy.progress(false));
    curl!(easy.connect_timeout(std::time::Duration::from_secs(30)));
    curl!(easy.timeout(std::time::Duration::from_secs(600)));

    let mut headers = curl::easy::List::new();
    curl!(headers.append(&format!("User-Agent: {}", USER_AGENT)));
    curl!(
        headers.append(
            "Accept: application/octet-stream, application/x-tar, application/zip, */*;q=0.9"
        )
    );
    curl!(easy.http_headers(headers));

    let mut file = std::fs::File::create(dest)?;
    let mut transfer = easy.transfer();
    curl!(transfer.write_function(move |data| {
        file.write_all(data)
            .map(|()| data.len())
            .map_err(|_| curl::easy::WriteError::Pause)
    }));
    curl!(transfer.perform());

    Ok(())
}

pub fn extract(archive: &Path, dest_dir: &Path, kind_hint: Option<&str>) -> Result<PathBuf> {
    std::fs::create_dir_all(dest_dir)?;
    let extractor = match kind_hint {
        Some(hint) => file_types::from_hint(hint)
            .ok_or_else(|| Error::ExtractionFailed(format!("unknown archive kind: {hint}")))?,
        None => file_types::detect(archive)?,
    };
    extractor.extract(archive, dest_dir)
}

pub fn fetch_and_verify(
    url: &str,
    sha256: &str,
    mirrors: &[String],
    cache_dir: &Path,
) -> Result<PathBuf> {
    let cached = cache_dir.join(sha256);
    let path = download(url, &cached, mirrors)?;
    verify_sha256(&path, sha256)?;
    Ok(path)
}
