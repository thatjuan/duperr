use duperr::scanner::{FileEntry, FileId};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

/// Create a mock FileEntry for testing without real files
pub fn make_file_entry(path: &str, size: u64) -> FileEntry {
    FileEntry {
        path: PathBuf::from(path),
        size,
        file_id: FileId { device: 1, inode: rand::random() },
        modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
        partial_hash: None,
        full_hash: None,
    }
}

/// Create FileEntry with specific modification time
pub fn make_file_entry_with_time(path: &str, size: u64, modified_secs: u64) -> FileEntry {
    FileEntry {
        path: PathBuf::from(path),
        size,
        file_id: FileId { device: 1, inode: rand::random() },
        modified: SystemTime::UNIX_EPOCH + Duration::from_secs(modified_secs),
        partial_hash: None,
        full_hash: None,
    }
}

/// Create FileEntry with specific inode (for hardlink testing)
pub fn make_file_entry_with_inode(path: &str, size: u64, device: u64, inode: u64) -> FileEntry {
    FileEntry {
        path: PathBuf::from(path),
        size,
        file_id: FileId { device, inode },
        modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
        partial_hash: None,
        full_hash: None,
    }
}

/// Create multiple FileEntries with same size
pub fn make_file_entries_same_size(paths: &[&str], size: u64) -> Vec<FileEntry> {
    paths.iter().map(|p| make_file_entry(p, size)).collect()
}
