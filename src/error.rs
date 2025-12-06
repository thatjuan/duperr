use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during scanning
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("Failed to read metadata for '{0}': {1}")]
    Metadata(PathBuf, #[source] std::io::Error),

    #[error("Permission denied: '{0}'")]
    PermissionDenied(PathBuf),

    #[error("File not found: '{0}'")]
    NotFound(PathBuf),

    #[error("Path does not exist: '{0}'")]
    PathNotExists(PathBuf),

    #[error("Path is not a directory: '{0}'")]
    NotADirectory(PathBuf),

    #[error("Failed to read file '{0}': {1}")]
    FileRead(PathBuf, #[source] std::io::Error),

    #[error("Failed to hash file '{0}': {1}")]
    HashError(PathBuf, #[source] std::io::Error),

    #[error("Invalid glob pattern '{0}': {1}")]
    GlobPattern(String, #[source] globset::Error),

    #[error("Failed to create backup directory '{0}': {1}")]
    BackupDirCreate(PathBuf, #[source] std::io::Error),

    #[error("Failed to backup file '{0}' to '{1}': {2}")]
    BackupFailed(PathBuf, PathBuf, #[source] std::io::Error),

    #[error("Failed to delete file '{0}': {1}")]
    DeleteFailed(PathBuf, #[source] std::io::Error),

    #[error("Backup directory '{0}' is inside scanned directory '{1}'")]
    BackupInsideScanned(PathBuf, PathBuf),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("User cancelled operation")]
    Cancelled,
}

impl ScanError {
    /// Create error from IO error with path context
    pub fn from_io_with_path(path: &std::path::Path, err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::PermissionDenied => ScanError::PermissionDenied(path.to_owned()),
            std::io::ErrorKind::NotFound => ScanError::NotFound(path.to_owned()),
            _ => ScanError::FileRead(path.to_owned(), err),
        }
    }

    /// Check if this error should stop the entire operation
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            ScanError::Config(_) | ScanError::BackupInsideScanned(_, _) | ScanError::Cancelled
        )
    }

    /// Check if this error can be safely ignored (logged as warning)
    pub fn is_ignorable(&self) -> bool {
        matches!(
            self,
            ScanError::PermissionDenied(_) | ScanError::NotFound(_) | ScanError::FileRead(_, _)
        )
    }
}

/// Result type alias for scan operations
pub type ScanResult<T> = std::result::Result<T, ScanError>;

/// Collection of non-fatal errors encountered during scanning
#[derive(Debug, Default)]
pub struct ErrorLog {
    pub warnings: Vec<ScanError>,
}

impl ErrorLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: ScanError) {
        self.warnings.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.warnings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.warnings.len()
    }

    /// Print warnings in a user-friendly format
    pub fn print_warnings(&self) {
        if self.is_empty() {
            return;
        }

        use colored::Colorize;

        eprintln!();
        eprintln!(
            "{} {} warning(s) encountered during scan:",
            "Warning:".yellow().bold(),
            self.len()
        );

        for (i, err) in self.warnings.iter().take(10).enumerate() {
            eprintln!("  {}. {}", i + 1, err);
        }

        if self.len() > 10 {
            eprintln!("  ... and {} more", self.len() - 10);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_from_io_with_path_permission_denied() {
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let scan_err = ScanError::from_io_with_path(Path::new("/test/path"), err);

        match scan_err {
            ScanError::PermissionDenied(p) => assert_eq!(p, PathBuf::from("/test/path")),
            _ => panic!("Expected PermissionDenied"),
        }
    }

    #[test]
    fn test_from_io_with_path_not_found() {
        let err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let scan_err = ScanError::from_io_with_path(Path::new("/missing/file"), err);

        match scan_err {
            ScanError::NotFound(p) => assert_eq!(p, PathBuf::from("/missing/file")),
            _ => panic!("Expected NotFound"),
        }
    }

    #[test]
    fn test_from_io_with_path_other_error() {
        let err = io::Error::new(io::ErrorKind::Other, "other error");
        let scan_err = ScanError::from_io_with_path(Path::new("/some/file"), err);

        match scan_err {
            ScanError::FileRead(p, _) => assert_eq!(p, PathBuf::from("/some/file")),
            _ => panic!("Expected FileRead"),
        }
    }

    #[test]
    fn test_is_fatal() {
        assert!(ScanError::Config("error".to_string()).is_fatal());
        assert!(ScanError::BackupInsideScanned(
            PathBuf::from("/a"),
            PathBuf::from("/b")
        ).is_fatal());
        assert!(ScanError::Cancelled.is_fatal());

        assert!(!ScanError::PermissionDenied(PathBuf::new()).is_fatal());
        assert!(!ScanError::NotFound(PathBuf::new()).is_fatal());
    }

    #[test]
    fn test_is_ignorable() {
        assert!(ScanError::PermissionDenied(PathBuf::new()).is_ignorable());
        assert!(ScanError::NotFound(PathBuf::new()).is_ignorable());
        assert!(ScanError::FileRead(PathBuf::new(),
            io::Error::new(io::ErrorKind::Other, "")
        ).is_ignorable());

        assert!(!ScanError::Config("error".to_string()).is_ignorable());
        assert!(!ScanError::Cancelled.is_ignorable());
    }

    #[test]
    fn test_error_log() {
        let mut log = ErrorLog::new();
        assert!(log.is_empty());
        assert_eq!(log.len(), 0);

        log.add(ScanError::PermissionDenied(PathBuf::from("/test")));
        assert!(!log.is_empty());
        assert_eq!(log.len(), 1);

        log.add(ScanError::NotFound(PathBuf::from("/missing")));
        assert_eq!(log.len(), 2);
    }

    #[test]
    fn test_error_display() {
        let err = ScanError::PermissionDenied(PathBuf::from("/secret/file"));
        assert!(err.to_string().contains("Permission denied"));
        assert!(err.to_string().contains("/secret/file"));

        let err = ScanError::Config("invalid setting".to_string());
        assert!(err.to_string().contains("Configuration error"));
        assert!(err.to_string().contains("invalid setting"));
    }
}
