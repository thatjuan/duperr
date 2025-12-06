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
