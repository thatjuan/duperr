use crate::error::ScanError;
use crate::scanner::{FileEntry, FileFilter, FileId};
use std::collections::HashSet;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub struct Scanner {
    filter: FileFilter,
    follow_symlinks: bool,
    max_depth: Option<usize>,
}

impl Scanner {
    pub fn new(filter: FileFilter, follow_symlinks: bool, max_depth: Option<usize>) -> Self {
        Self {
            filter,
            follow_symlinks,
            max_depth,
        }
    }

    /// Scan directories and return all matching files
    pub fn scan(&mut self, paths: &[impl AsRef<Path>]) -> Result<Vec<FileEntry>, ScanError> {
        let mut files = Vec::new();
        let mut seen_files: HashSet<FileId> = HashSet::new();

        for path in paths {
            self.scan_path(path.as_ref(), &mut files, &mut seen_files)?;
        }

        Ok(files)
    }

    fn scan_path(
        &mut self,
        path: &Path,
        files: &mut Vec<FileEntry>,
        seen_files: &mut HashSet<FileId>,
    ) -> Result<(), ScanError> {
        let mut walker = WalkDir::new(path).follow_links(self.follow_symlinks);

        if let Some(depth) = self.max_depth {
            walker = walker.max_depth(depth);
        }

        for entry in walker {
            match entry {
                Ok(entry) => {
                    if let Err(e) = self.process_entry(entry, files, seen_files) {
                        // Log warning but continue scanning
                        log::warn!("Error processing entry: {}", e);
                    }
                }
                Err(e) => {
                    log::warn!("Walk error: {}", e);
                }
            }
        }

        Ok(())
    }

    fn process_entry(
        &mut self,
        entry: DirEntry,
        files: &mut Vec<FileEntry>,
        seen_files: &mut HashSet<FileId>,
    ) -> Result<(), ScanError> {
        let metadata = entry
            .metadata()
            .map_err(|e| ScanError::Metadata(entry.path().to_owned(), e.into()))?;

        // Skip non-regular files (directories, symlinks, devices, sockets, etc.)
        if !FileFilter::is_regular_file(&metadata) {
            return Ok(());
        }

        let path = entry.path();

        // Apply filters
        if !self.filter.should_include(path, &metadata) {
            return Ok(());
        }

        // Create file entry
        let file_entry = FileEntry::new(path.to_owned(), &metadata);

        // Skip if we've seen this inode (hardlink to already-seen file)
        if !seen_files.insert(file_entry.file_id) {
            log::debug!("Skipping hardlink: {}", path.display());
            return Ok(());
        }

        files.push(file_entry);
        Ok(())
    }
}
