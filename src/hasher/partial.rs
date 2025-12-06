use crate::error::ScanError;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Size of partial hash read (4KB)
pub const PARTIAL_HASH_SIZE: usize = 4096;

/// Compute partial hash of a file (first PARTIAL_HASH_SIZE bytes)
///
/// For files smaller than PARTIAL_HASH_SIZE, hashes the entire file.
/// Returns blake3 hash as 32-byte array.
pub fn partial_hash(path: &Path) -> Result<[u8; 32], ScanError> {
    let file = File::open(path).map_err(|e| ScanError::from_io_with_path(path, e))?;

    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; PARTIAL_HASH_SIZE];

    let bytes_read = reader
        .read(&mut buffer)
        .map_err(|e| ScanError::HashError(path.to_owned(), e))?;

    let hash = blake3::hash(&buffer[..bytes_read]);
    Ok(*hash.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_partial_hash_small_file() {
        let dir = tempdir().unwrap();
        let content = b"small file content";
        let path = create_test_file(dir.path(), "small.txt", content);

        let hash = partial_hash(&path).unwrap();

        // Verify it's a valid 32-byte hash
        assert_eq!(hash.len(), 32);

        // Same content should produce same hash
        let path2 = create_test_file(dir.path(), "small2.txt", content);
        let hash2 = partial_hash(&path2).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_partial_hash_exactly_4kb() {
        let dir = tempdir().unwrap();
        let content: Vec<u8> = (0..PARTIAL_HASH_SIZE).map(|i| (i % 256) as u8).collect();
        let path = create_test_file(dir.path(), "exact4k.bin", &content);

        let hash = partial_hash(&path).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_partial_hash_large_file() {
        let dir = tempdir().unwrap();
        // Create 10KB file
        let content: Vec<u8> = (0..10240).map(|i| (i % 256) as u8).collect();
        let path = create_test_file(dir.path(), "large.bin", &content);

        let hash = partial_hash(&path).unwrap();

        // File with same first 4KB should have same partial hash
        let mut different_tail: Vec<u8> = content[..PARTIAL_HASH_SIZE].to_vec();
        different_tail.extend(vec![0u8; 6144]); // Different tail
        let path2 = create_test_file(dir.path(), "same_prefix.bin", &different_tail);

        let hash2 = partial_hash(&path2).unwrap();
        assert_eq!(hash, hash2, "Same prefix should produce same partial hash");
    }

    #[test]
    fn test_partial_hash_different_content() {
        let dir = tempdir().unwrap();
        let path1 = create_test_file(dir.path(), "file1.txt", b"content one");
        let path2 = create_test_file(dir.path(), "file2.txt", b"content two");

        let hash1 = partial_hash(&path1).unwrap();
        let hash2 = partial_hash(&path2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_partial_hash_empty_file() {
        let dir = tempdir().unwrap();
        let path = create_test_file(dir.path(), "empty.txt", b"");

        let hash = partial_hash(&path).unwrap();
        // Empty file should still produce a valid hash
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_partial_hash_nonexistent_file() {
        let result = partial_hash(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_partial_hash_deterministic() {
        let dir = tempdir().unwrap();
        let content = b"test content for determinism";
        let path = create_test_file(dir.path(), "det.txt", content);

        // Hash same file multiple times
        let hash1 = partial_hash(&path).unwrap();
        let hash2 = partial_hash(&path).unwrap();
        let hash3 = partial_hash(&path).unwrap();

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }
}
