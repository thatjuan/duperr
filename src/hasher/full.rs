use crate::error::ScanError;
use memmap2::Mmap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Threshold for using memory-mapping (128KB)
pub const MMAP_THRESHOLD: u64 = 128 * 1024;

/// Compute full blake3 hash of a file
///
/// Uses memory-mapping for files larger than MMAP_THRESHOLD.
/// For smaller files, uses buffered reading.
pub fn full_hash(path: &Path, size: u64) -> Result<[u8; 32], ScanError> {
    if size > MMAP_THRESHOLD {
        hash_mmap(path)
    } else {
        hash_buffered(path)
    }
}

fn hash_mmap(path: &Path) -> Result<[u8; 32], ScanError> {
    let file = File::open(path).map_err(|e| ScanError::from_io_with_path(path, e))?;

    // SAFETY: File is opened read-only
    let mmap = unsafe { Mmap::map(&file) }.map_err(|e| ScanError::HashError(path.to_owned(), e))?;

    let hash = blake3::hash(&mmap);
    Ok(*hash.as_bytes())
}

fn hash_buffered(path: &Path) -> Result<[u8; 32], ScanError> {
    let file = File::open(path).map_err(|e| ScanError::from_io_with_path(path, e))?;

    let mut hasher = blake3::Hasher::new();
    let mut reader = BufReader::with_capacity(65536, file);
    let mut buffer = [0u8; 65536];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .map_err(|e| ScanError::HashError(path.to_owned(), e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(*hasher.finalize().as_bytes())
}

/// Byte-by-byte comparison of two files (paranoid mode)
///
/// Returns true if files are identical, false otherwise.
/// Uses memory-mapping for efficiency on large files.
pub fn compare_files(path1: &Path, path2: &Path, size: u64) -> Result<bool, ScanError> {
    if size > MMAP_THRESHOLD {
        compare_mmap(path1, path2)
    } else {
        compare_buffered(path1, path2)
    }
}

fn compare_mmap(path1: &Path, path2: &Path) -> Result<bool, ScanError> {
    let file1 = File::open(path1).map_err(|e| ScanError::from_io_with_path(path1, e))?;
    let file2 = File::open(path2).map_err(|e| ScanError::from_io_with_path(path2, e))?;

    let mmap1 =
        unsafe { Mmap::map(&file1) }.map_err(|e| ScanError::FileRead(path1.to_owned(), e))?;
    let mmap2 =
        unsafe { Mmap::map(&file2) }.map_err(|e| ScanError::FileRead(path2.to_owned(), e))?;

    Ok(mmap1[..] == mmap2[..])
}

fn compare_buffered(path1: &Path, path2: &Path) -> Result<bool, ScanError> {
    let file1 = File::open(path1).map_err(|e| ScanError::from_io_with_path(path1, e))?;
    let file2 = File::open(path2).map_err(|e| ScanError::from_io_with_path(path2, e))?;

    let mut reader1 = BufReader::with_capacity(65536, file1);
    let mut reader2 = BufReader::with_capacity(65536, file2);
    let mut buf1 = [0u8; 65536];
    let mut buf2 = [0u8; 65536];

    loop {
        let n1 = reader1
            .read(&mut buf1)
            .map_err(|e| ScanError::FileRead(path1.to_owned(), e))?;
        let n2 = reader2
            .read(&mut buf2)
            .map_err(|e| ScanError::FileRead(path2.to_owned(), e))?;

        if n1 != n2 || buf1[..n1] != buf2[..n2] {
            return Ok(false);
        }

        if n1 == 0 {
            break;
        }
    }

    Ok(true)
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
    fn test_full_hash_small_file_buffered() {
        let dir = tempdir().unwrap();
        let content = b"small content for buffered reading";
        let path = create_test_file(dir.path(), "small.txt", content);

        let hash = full_hash(&path, content.len() as u64).unwrap();
        assert_eq!(hash.len(), 32);

        // Same content = same hash
        let path2 = create_test_file(dir.path(), "small2.txt", content);
        let hash2 = full_hash(&path2, content.len() as u64).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_full_hash_large_file_mmap() {
        let dir = tempdir().unwrap();
        // Create file > MMAP_THRESHOLD (128KB)
        let size = 200 * 1024; // 200KB
        let content: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let path = create_test_file(dir.path(), "large.bin", &content);

        let hash = full_hash(&path, size as u64).unwrap();
        assert_eq!(hash.len(), 32);

        // Same content = same hash
        let path2 = create_test_file(dir.path(), "large2.bin", &content);
        let hash2 = full_hash(&path2, size as u64).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_full_hash_at_mmap_threshold() {
        let dir = tempdir().unwrap();
        // Exactly at threshold
        let size = MMAP_THRESHOLD as usize;
        let content: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let path = create_test_file(dir.path(), "threshold.bin", &content);

        let hash = full_hash(&path, size as u64).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_full_hash_different_content_same_size() {
        let dir = tempdir().unwrap();
        let size = 1000;
        let content1: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let content2: Vec<u8> = (0..size).map(|i| ((i + 1) % 256) as u8).collect();

        let path1 = create_test_file(dir.path(), "file1.bin", &content1);
        let path2 = create_test_file(dir.path(), "file2.bin", &content2);

        let hash1 = full_hash(&path1, size as u64).unwrap();
        let hash2 = full_hash(&path2, size as u64).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compare_files_identical_small() {
        let dir = tempdir().unwrap();
        let content = b"identical content for comparison";
        let path1 = create_test_file(dir.path(), "file1.txt", content);
        let path2 = create_test_file(dir.path(), "file2.txt", content);

        assert!(compare_files(&path1, &path2, content.len() as u64).unwrap());
    }

    #[test]
    fn test_compare_files_identical_large() {
        let dir = tempdir().unwrap();
        let size = 200 * 1024; // 200KB, uses mmap
        let content: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let path1 = create_test_file(dir.path(), "large1.bin", &content);
        let path2 = create_test_file(dir.path(), "large2.bin", &content);

        assert!(compare_files(&path1, &path2, size as u64).unwrap());
    }

    #[test]
    fn test_compare_files_different_small() {
        let dir = tempdir().unwrap();
        let path1 = create_test_file(dir.path(), "file1.txt", b"content one");
        let path2 = create_test_file(dir.path(), "file2.txt", b"content two");

        assert!(!compare_files(&path1, &path2, 11).unwrap());
    }

    #[test]
    fn test_compare_files_different_large() {
        let dir = tempdir().unwrap();
        let size = 200 * 1024;
        let content1: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let mut content2 = content1.clone();
        // Change a byte in the middle to ensure it's different
        content2[size / 2] = !content2[size / 2];

        let path1 = create_test_file(dir.path(), "large1.bin", &content1);
        let path2 = create_test_file(dir.path(), "large2.bin", &content2);

        assert!(!compare_files(&path1, &path2, size as u64).unwrap());
    }

    #[test]
    fn test_full_hash_empty_file() {
        let dir = tempdir().unwrap();
        let path = create_test_file(dir.path(), "empty.txt", b"");

        let hash = full_hash(&path, 0).unwrap();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_full_hash_binary_content() {
        let dir = tempdir().unwrap();
        // Binary content including NUL bytes
        let content: Vec<u8> = vec![0, 1, 2, 0, 255, 254, 0, 128];
        let path = create_test_file(dir.path(), "binary.bin", &content);

        let hash = full_hash(&path, content.len() as u64).unwrap();
        assert_eq!(hash.len(), 32);
    }
}
