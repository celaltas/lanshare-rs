use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

pub fn compute_file_sha256(path: impl AsRef<Path>) -> io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    let mut sha = [0u8; 32];
    sha.copy_from_slice(&result);
    Ok(sha)
}

pub fn sha_to_hex(sha: &[u8; 32]) -> String {
    sha.iter().map(|b| format!("{:02x}", b)).collect()
}
