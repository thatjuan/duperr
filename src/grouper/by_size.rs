use crate::grouper::DuplicateGroup;
use crate::scanner::FileEntry;
use std::collections::HashMap;

/// Group files by size, filtering out unique sizes (no possible duplicates)
///
/// This is the first-pass optimization: files with unique sizes cannot be duplicates.
/// Empty files (size 0) are handled specially - they're grouped together but
/// treated as duplicates only if the caller explicitly wants them.
pub fn group_by_size(files: Vec<FileEntry>, include_empty: bool) -> Vec<DuplicateGroup> {
    let mut size_map: HashMap<u64, Vec<FileEntry>> = HashMap::new();

    for file in files {
        // Optionally skip empty files
        if !include_empty && file.size == 0 {
            continue;
        }

        size_map.entry(file.size).or_default().push(file);
    }

    // Convert to DuplicateGroups, keeping only groups with 2+ files
    size_map
        .into_iter()
        .filter(|(_, files)| files.len() >= 2)
        .map(|(size, files)| DuplicateGroup::new(size, files))
        .collect()
}

/// Statistics about size grouping
#[derive(Debug, Default)]
pub struct SizeGroupStats {
    pub total_files: usize,
    pub unique_sizes: usize,
    pub candidate_groups: usize,
    pub candidate_files: usize,
    pub empty_files_skipped: usize,
}

/// Group files by size with statistics
pub fn group_by_size_with_stats(
    files: Vec<FileEntry>,
    include_empty: bool,
) -> (Vec<DuplicateGroup>, SizeGroupStats) {
    let mut stats = SizeGroupStats {
        total_files: files.len(),
        ..Default::default()
    };

    let mut size_map: HashMap<u64, Vec<FileEntry>> = HashMap::new();

    for file in files {
        if !include_empty && file.size == 0 {
            stats.empty_files_skipped += 1;
            continue;
        }
        size_map.entry(file.size).or_default().push(file);
    }

    stats.unique_sizes = size_map.values().filter(|v| v.len() == 1).count();

    let groups: Vec<DuplicateGroup> = size_map
        .into_iter()
        .filter(|(_, files)| files.len() >= 2)
        .map(|(size, files)| DuplicateGroup::new(size, files))
        .collect();

    stats.candidate_groups = groups.len();
    stats.candidate_files = groups.iter().map(|g| g.files.len()).sum();

    (groups, stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileEntry, FileId};
    use std::time::SystemTime;

    fn make_entry(path: &str, size: u64) -> FileEntry {
        FileEntry {
            path: path.into(),
            size,
            file_id: FileId {
                device: 0,
                inode: 0,
            },
            modified: SystemTime::UNIX_EPOCH,
            partial_hash: None,
            full_hash: None,
        }
    }

    #[test]
    fn test_group_by_size_filters_unique() {
        let files = vec![
            make_entry("a.txt", 100),
            make_entry("b.txt", 100),
            make_entry("c.txt", 200), // unique size
            make_entry("d.txt", 300),
            make_entry("e.txt", 300),
        ];

        let groups = group_by_size(files, true);

        // Should have 2 groups (100 bytes and 300 bytes)
        // 200 bytes is unique, so filtered out
        assert_eq!(groups.len(), 2);

        let sizes: Vec<u64> = groups.iter().map(|g| g.size).collect();
        assert!(sizes.contains(&100));
        assert!(sizes.contains(&300));
        assert!(!sizes.contains(&200));
    }

    #[test]
    fn test_group_by_size_empty_files() {
        let files = vec![make_entry("empty1.txt", 0), make_entry("empty2.txt", 0)];

        let groups_include = group_by_size(files.clone(), true);
        let groups_exclude = group_by_size(files, false);

        assert_eq!(groups_include.len(), 1); // empty files grouped
        assert_eq!(groups_exclude.len(), 0); // empty files excluded
    }

    #[test]
    fn test_group_by_size_single_files() {
        let files = vec![
            make_entry("a.txt", 100),
            make_entry("b.txt", 200),
            make_entry("c.txt", 300),
        ];

        let groups = group_by_size(files, true);

        // All files have unique sizes, so no groups
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_group_by_size_with_stats() {
        let files = vec![
            make_entry("a.txt", 100),
            make_entry("b.txt", 100),
            make_entry("c.txt", 200),
            make_entry("d.txt", 300),
            make_entry("e.txt", 300),
            make_entry("f.txt", 300),
        ];

        let (groups, stats) = group_by_size_with_stats(files, true);

        assert_eq!(stats.total_files, 6);
        assert_eq!(stats.unique_sizes, 1); // 200 bytes
        assert_eq!(stats.candidate_groups, 2); // 100 and 300
        assert_eq!(stats.candidate_files, 5); // 2 + 3
        assert_eq!(groups.len(), 2);
    }
}
