use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::KeepStrategy;
use crate::error::ScanError;
use crate::ScanResult;

/// Statistics about backup operation
#[derive(Debug, Default)]
pub struct BackupStats {
    pub files_backed_up: u64,
    pub bytes_backed_up: u64,
    pub files_failed: u64,
}

/// Backup all duplicate files to the specified directory
///
/// Preserves relative directory structure. For example, if backing up
/// `/home/user/photos/dup.jpg` to `/backup`, it becomes
/// `/backup/home/user/photos/dup.jpg`.
pub fn backup_duplicates(
    result: &ScanResult,
    backup_dir: &Path,
    keep_strategy: KeepStrategy,
) -> Result<BackupStats, ScanError> {
    let mut stats = BackupStats::default();

    // Create backup directory if it doesn't exist
    fs::create_dir_all(backup_dir)
        .map_err(|e| ScanError::BackupDirCreate(backup_dir.to_owned(), e))?;

    println!();
    println!(
        "{} Backing up duplicates to: {}",
        "→".cyan().bold(),
        backup_dir.display()
    );

    for group in &result.groups {
        for file in group.get_duplicates(keep_strategy) {
            match backup_file(&file.path, backup_dir) {
                Ok(_) => {
                    stats.files_backed_up += 1;
                    stats.bytes_backed_up += file.size;
                    println!("  {} {}", "✓".green(), file.path.display());
                }
                Err(e) => {
                    stats.files_failed += 1;
                    eprintln!("  {} {} ({})", "✗".red(), file.path.display(), e);
                }
            }
        }
    }

    print_backup_summary(&stats);
    Ok(stats)
}

/// Backup a single file, preserving directory structure
fn backup_file(source: &Path, backup_dir: &Path) -> Result<PathBuf, ScanError> {
    // Create destination path preserving structure
    // /home/user/file.txt -> backup_dir/home/user/file.txt
    let dest = create_backup_path(source, backup_dir);

    // Create parent directories
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| ScanError::BackupDirCreate(parent.to_owned(), e))?;
    }

    // Copy file
    fs::copy(source, &dest)
        .map_err(|e| ScanError::BackupFailed(source.to_owned(), dest.clone(), e))?;

    Ok(dest)
}

/// Create backup destination path preserving directory structure
fn create_backup_path(source: &Path, backup_dir: &Path) -> PathBuf {
    // Get absolute path and strip root
    let abs_path = source.canonicalize().unwrap_or_else(|_| source.to_owned());

    // Remove the root component (/ on Unix, C:\ on Windows)
    let stripped: PathBuf = abs_path
        .components()
        .skip(1) // Skip root
        .collect();

    backup_dir.join(stripped)
}

fn print_backup_summary(stats: &BackupStats) {
    println!();
    println!("{}", "Backup Summary:".bold());
    println!(
        "  Files backed up: {}",
        stats.files_backed_up.to_string().green()
    );
    println!(
        "  Bytes copied:    {}",
        humansize::format_size(stats.bytes_backed_up, humansize::BINARY).yellow()
    );
    if stats.files_failed > 0 {
        println!(
            "  Files failed:    {}",
            stats.files_failed.to_string().red()
        );
    }
}

/// Verify backup directory is safe to use
///
/// Checks that it's not inside any of the scanned directories
/// (to avoid backing up into a directory we're scanning).
pub fn validate_backup_dir(backup_dir: &Path, scan_paths: &[PathBuf]) -> Result<(), ScanError> {
    let backup_canonical = backup_dir
        .canonicalize()
        .or_else(|_| {
            // Directory might not exist yet, try parent
            if let Some(parent) = backup_dir.parent() {
                parent
                    .canonicalize()
                    .map(|p| p.join(backup_dir.file_name().unwrap_or_default()))
            } else {
                Ok(backup_dir.to_owned())
            }
        })
        .map_err(|e| ScanError::FileRead(backup_dir.to_owned(), e))?;

    for scan_path in scan_paths {
        if let Ok(scan_canonical) = scan_path.canonicalize() {
            if backup_canonical.starts_with(&scan_canonical) {
                return Err(ScanError::BackupInsideScanned(
                    backup_dir.to_owned(),
                    scan_path.to_owned(),
                ));
            }
        }
    }

    Ok(())
}
