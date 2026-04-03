pub mod storage;

use sha2::{Sha256, Digest};

pub fn hash_pdf_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let hash = hasher.finalize();

    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    
    #[test]
    fn hash_pdf_bytes_returns_correct_hash() {
        let bytes = fs::read("tests/data/multi-agent.pdf")
            .expect("Failed to read test PDF");
        let expected_hash = "8854515074495a9cdde4cd3b335dd5ce95b4f0349f10f6509ec3af88dd7e85ef";
        let calculated_hash = hash_pdf_bytes(&bytes);
        
        assert_eq!(calculated_hash, expected_hash);
    }
}
