use crate::cli::KeepStrategy;
use crate::scanner::FileEntry;

/// A group of files confirmed to be duplicates
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// The hash shared by all files (set after full hashing)
    pub hash: Option<[u8; 32]>,
    /// File size (same for all files in group)
    pub size: u64,
    /// All duplicate files in this group
    pub files: Vec<FileEntry>,
}

impl DuplicateGroup {
    pub fn new(size: u64, files: Vec<FileEntry>) -> Self {
        Self {
            hash: None,
            size,
            files,
        }
    }

    pub fn with_hash(hash: [u8; 32], size: u64, files: Vec<FileEntry>) -> Self {
        Self {
            hash: Some(hash),
            size,
            files,
        }
    }

    /// Total wasted space (all duplicates except one)
    pub fn wasted_bytes(&self) -> u64 {
        self.size
            .saturating_mul(self.files.len().saturating_sub(1) as u64)
    }

    /// Number of duplicate files (excluding the keeper)
    pub fn duplicate_count(&self) -> usize {
        self.files.len().saturating_sub(1)
    }

    /// Select the file to keep based on strategy
    pub fn select_keeper(&self, strategy: KeepStrategy) -> Option<&FileEntry> {
        if self.files.is_empty() {
            return None;
        }

        match strategy {
            KeepStrategy::First => self.files.first(),
            KeepStrategy::Oldest => self.files.iter().min_by_key(|f| f.modified),
            KeepStrategy::Newest => self.files.iter().max_by_key(|f| f.modified),
            KeepStrategy::ShortestPath => self
                .files
                .iter()
                .min_by(|a, b| a.path.as_os_str().len().cmp(&b.path.as_os_str().len())),
            KeepStrategy::LongestPath => self
                .files
                .iter()
                .max_by(|a, b| a.path.as_os_str().len().cmp(&b.path.as_os_str().len())),
        }
    }

    /// Get files to be deleted (all except keeper)
    pub fn get_duplicates(&self, strategy: KeepStrategy) -> Vec<&FileEntry> {
        let keeper = match self.select_keeper(strategy) {
            Some(k) => k,
            None => return vec![],
        };

        self.files
            .iter()
            .filter(|f| !std::ptr::eq(*f, keeper))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileEntry, FileId};
    use std::time::{Duration, SystemTime};

    fn make_entry(path: &str, modified_secs: u64) -> FileEntry {
        FileEntry {
            path: path.into(),
            size: 100,
            file_id: FileId {
                device: 0,
                inode: 0,
            },
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(modified_secs),
            partial_hash: None,
            full_hash: None,
        }
    }

    #[test]
    fn test_select_keeper_oldest() {
        let group = DuplicateGroup::new(
            100,
            vec![
                make_entry("new.txt", 2000),
                make_entry("old.txt", 1000),
                make_entry("mid.txt", 1500),
            ],
        );

        let keeper = group.select_keeper(KeepStrategy::Oldest).unwrap();
        assert_eq!(keeper.path.to_str().unwrap(), "old.txt");
    }

    #[test]
    fn test_select_keeper_newest() {
        let group = DuplicateGroup::new(
            100,
            vec![make_entry("new.txt", 2000), make_entry("old.txt", 1000)],
        );

        let keeper = group.select_keeper(KeepStrategy::Newest).unwrap();
        assert_eq!(keeper.path.to_str().unwrap(), "new.txt");
    }

    #[test]
    fn test_select_keeper_shortest_path() {
        let group = DuplicateGroup::new(
            100,
            vec![
                make_entry("/very/long/path/file.txt", 1000),
                make_entry("/short.txt", 1000),
            ],
        );

        let keeper = group.select_keeper(KeepStrategy::ShortestPath).unwrap();
        assert_eq!(keeper.path.to_str().unwrap(), "/short.txt");
    }

    #[test]
    fn test_select_keeper_longest_path() {
        let group = DuplicateGroup::new(
            100,
            vec![
                make_entry("/very/long/path/file.txt", 1000),
                make_entry("/short.txt", 1000),
            ],
        );

        let keeper = group.select_keeper(KeepStrategy::LongestPath).unwrap();
        assert_eq!(keeper.path.to_str().unwrap(), "/very/long/path/file.txt");
    }

    #[test]
    fn test_select_keeper_first() {
        let group = DuplicateGroup::new(
            100,
            vec![
                make_entry("first.txt", 1000),
                make_entry("second.txt", 2000),
            ],
        );

        let keeper = group.select_keeper(KeepStrategy::First).unwrap();
        assert_eq!(keeper.path.to_str().unwrap(), "first.txt");
    }

    #[test]
    fn test_wasted_bytes() {
        let group = DuplicateGroup::new(
            100,
            vec![
                make_entry("a.txt", 1000),
                make_entry("b.txt", 1000),
                make_entry("c.txt", 1000),
            ],
        );

        // 3 files of 100 bytes, wasted = 2 * 100 = 200
        assert_eq!(group.wasted_bytes(), 200);
    }

    #[test]
    fn test_duplicate_count() {
        let group = DuplicateGroup::new(
            100,
            vec![
                make_entry("a.txt", 1000),
                make_entry("b.txt", 1000),
                make_entry("c.txt", 1000),
            ],
        );

        // 3 files total, 2 are duplicates (exclude keeper)
        assert_eq!(group.duplicate_count(), 2);
    }

    #[test]
    fn test_get_duplicates() {
        let group = DuplicateGroup::new(
            100,
            vec![
                make_entry("a.txt", 1000),
                make_entry("b.txt", 2000),
                make_entry("c.txt", 3000),
            ],
        );

        let duplicates = group.get_duplicates(KeepStrategy::Oldest);
        assert_eq!(duplicates.len(), 2);

        // The oldest (a.txt at 1000) should be kept, others marked for deletion
        let dup_names: Vec<&str> = duplicates
            .iter()
            .map(|f| f.path.to_str().unwrap())
            .collect();
        assert!(dup_names.contains(&"b.txt"));
        assert!(dup_names.contains(&"c.txt"));
        assert!(!dup_names.contains(&"a.txt"));
    }

    #[test]
    fn test_empty_group() {
        let group = DuplicateGroup::new(100, vec![]);

        assert!(group.select_keeper(KeepStrategy::First).is_none());
        assert_eq!(group.wasted_bytes(), 0);
        assert_eq!(group.duplicate_count(), 0);
        assert_eq!(group.get_duplicates(KeepStrategy::First).len(), 0);
    }
}
