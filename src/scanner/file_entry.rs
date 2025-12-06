use std::path::PathBuf;
use std::time::SystemTime;

/// Unique identifier for a file (device + inode on Unix)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId {
    pub device: u64,
    pub inode: u64,
}

impl FileId {
    #[cfg(unix)]
    pub fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        use std::os::unix::fs::MetadataExt;
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
        }
    }

    #[cfg(not(unix))]
    pub fn from_metadata(_metadata: &std::fs::Metadata) -> Self {
        // On Windows, use a placeholder - hardlink detection won't work
        Self {
            device: 0,
            inode: 0,
        }
    }
}

/// Represents a scanned file with metadata
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub file_id: FileId,
    pub modified: SystemTime,
    pub partial_hash: Option<[u8; 32]>,
    pub full_hash: Option<[u8; 32]>,
}

impl FileEntry {
    pub fn new(path: PathBuf, metadata: &std::fs::Metadata) -> Self {
        Self {
            path,
            size: metadata.len(),
            file_id: FileId::from_metadata(metadata),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            partial_hash: None,
            full_hash: None,
        }
    }

    /// Check if this is the same physical file (hardlink)
    pub fn is_same_file(&self, other: &FileEntry) -> bool {
        self.file_id == other.file_id && self.file_id.inode != 0
    }
}
