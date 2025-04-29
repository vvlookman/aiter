use sha2::{Digest, Sha256};

pub fn sha256(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);

    format!("{:x}", hasher.finalize())
}
