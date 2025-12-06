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
