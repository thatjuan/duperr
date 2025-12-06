// Library root - re-exports modules

pub mod cli;
pub mod scanner;
pub mod hasher;
pub mod grouper;
pub mod output;
pub mod actions;
pub mod progress;
pub mod error;
pub mod pipeline;

use std::time::Duration;

/// Statistics about the scan operation
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ScanStats {
    pub total_files_scanned: u64,
    pub total_bytes_scanned: u64,
    pub files_with_unique_size: u64,
    pub duplicate_groups: u64,
    pub duplicate_files: u64,
    pub wasted_bytes: u64,
    #[serde(skip)]
    pub scan_duration: Duration,
}

/// Complete result of a duplicate scan
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub groups: Vec<grouper::DuplicateGroup>,
    pub stats: ScanStats,
}

impl ScanResult {
    pub fn new(groups: Vec<grouper::DuplicateGroup>) -> Self {
        let stats = ScanStats {
            duplicate_groups: groups.len() as u64,
            duplicate_files: groups.iter().map(|g| g.duplicate_count() as u64).sum(),
            wasted_bytes: groups.iter().map(|g| g.wasted_bytes()).sum(),
            ..Default::default()
        };
        Self { groups, stats }
    }

    pub fn has_duplicates(&self) -> bool {
        !self.groups.is_empty()
    }
}
