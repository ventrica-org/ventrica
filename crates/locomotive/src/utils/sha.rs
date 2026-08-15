use sha2::{Digest, Sha256};

use crate::Error;

pub fn sha256_file(path: &std::path::Path) -> Result<String, Error> {
    let mut file = std::fs::File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)?;
        if n == 0 {
            break;
        }
        Digest::update(&mut h, &buf[..n]);
    }
    Ok(hex::encode(Digest::finalize(h)))
}
