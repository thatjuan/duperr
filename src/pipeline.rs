//! Main processing pipeline for finding duplicates
//!
//! Orchestrates: scan → size group → partial hash → full hash → (optional paranoid verify)

use rayon::ThreadPoolBuilder;
use std::time::Instant;

use crate::cli::Config;
use crate::error::ScanError;
use crate::grouper::{
    group_by_full_hash_with_progress, group_by_partial_hash_with_progress,
    group_by_size_with_stats, verify_with_byte_compare,
};
use crate::progress::{ProgressManager, SharedProgress};
use crate::scanner::{FileFilter, Scanner};
use crate::{ScanResult, ScanStats};

/// Run the complete duplicate finding pipeline
pub fn find_duplicates(config: &Config) -> Result<ScanResult, ScanError> {
    let start = Instant::now();

    // Initialize progress manager
    let progress = ProgressManager::new(config.progress && !config.quiet);

    // Configure thread pool
    ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build_global()
        .ok(); // Ignore error if already initialized

    // Phase 1: Scan directories
    let scan_spinner = progress.create_spinner("Scanning directories...");

    // Note: We reconstruct the FileFilter from already-validated config
    // This is a bit redundant but maintains the existing API
    let filter = FileFilter {
        min_size: config.min_size,
        max_size: config.max_size,
        extensions: config.extensions.clone(),
        exclude_patterns: config.exclude.clone(),
        include_hidden: config.hidden,
        skip_empty: config.skip_empty,
    };

    let mut scanner = Scanner::new(filter, config.follow_symlinks, config.depth);
    let files = scanner.scan(&config.paths)?;

    scan_spinner.finish_with_message(format!("Scanned {} files", files.len()));

    let total_files = files.len() as u64;
    let total_bytes: u64 = files.iter().map(|f| f.size).sum();

    // Phase 2: Group by size
    let include_empty = !config.skip_empty;
    let (size_groups, size_stats) = group_by_size_with_stats(files, include_empty);

    if size_groups.is_empty() {
        return Ok(ScanResult {
            groups: vec![],
            stats: ScanStats {
                total_files_scanned: total_files,
                total_bytes_scanned: total_bytes,
                files_with_unique_size: size_stats.unique_sizes as u64,
                scan_duration: start.elapsed(),
                ..Default::default()
            },
        });
    }

    let candidate_count: u64 = size_groups.iter().map(|g| g.files.len() as u64).sum();

    // Phase 3: Group by partial hash (parallel) with progress
    let partial_pb = progress.create_progress_bar(candidate_count, "Partial hashing");
    let partial_progress = SharedProgress::new(partial_pb);

    let partial_groups =
        group_by_partial_hash_with_progress(size_groups, Some(partial_progress.clone()))?;

    partial_progress.finish_with_message("Partial hashing complete");

    if partial_groups.is_empty() {
        return Ok(ScanResult {
            groups: vec![],
            stats: ScanStats {
                total_files_scanned: total_files,
                total_bytes_scanned: total_bytes,
                files_with_unique_size: size_stats.unique_sizes as u64,
                scan_duration: start.elapsed(),
                ..Default::default()
            },
        });
    }

    let full_hash_count: u64 = partial_groups.iter().map(|g| g.files.len() as u64).sum();

    // Phase 4: Group by full hash (parallel) with progress
    let full_pb = progress.create_progress_bar(full_hash_count, "Full hashing");
    let full_progress = SharedProgress::new(full_pb);

    let mut duplicate_groups =
        group_by_full_hash_with_progress(partial_groups, Some(full_progress.clone()))?;

    full_progress.finish_with_message("Full hashing complete");

    // Phase 5: Optional paranoid verification (parallel)
    if config.paranoid {
        duplicate_groups = verify_with_byte_compare(duplicate_groups)?;
    }

    // Sort by wasted space (descending)
    duplicate_groups.sort_by_key(|b| std::cmp::Reverse(b.wasted_bytes()));

    let stats = ScanStats {
        total_files_scanned: total_files,
        total_bytes_scanned: total_bytes,
        files_with_unique_size: size_stats.unique_sizes as u64,
        duplicate_groups: duplicate_groups.len() as u64,
        duplicate_files: duplicate_groups
            .iter()
            .map(|g| g.duplicate_count() as u64)
            .sum(),
        wasted_bytes: duplicate_groups.iter().map(|g| g.wasted_bytes()).sum(),
        scan_duration: start.elapsed(),
    };

    Ok(ScanResult {
        groups: duplicate_groups,
        stats,
    })
}
