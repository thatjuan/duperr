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
