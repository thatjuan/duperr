use crate::error::ScanError;
use crate::grouper::DuplicateGroup;
use crate::hasher::{full, partial};
use crate::progress::SharedProgress;
use crate::scanner::FileEntry;
use rayon::prelude::*;
use std::collections::HashMap;

/// Group files by partial hash (first 4KB) - PARALLEL
///
/// Takes size-grouped candidates and further filters by partial hash.
/// Uses rayon for parallel hash computation.
pub fn group_by_partial_hash(
    groups: Vec<DuplicateGroup>,
) -> Result<Vec<DuplicateGroup>, ScanError> {
    group_by_partial_hash_with_progress(groups, None)
}

/// Group files by partial hash with optional progress tracking
pub fn group_by_partial_hash_with_progress(
    groups: Vec<DuplicateGroup>,
    progress: Option<SharedProgress>,
) -> Result<Vec<DuplicateGroup>, ScanError> {
    let mut result = Vec::new();

    for group in groups {
        // Compute partial hashes in parallel
        let files_with_hashes: Result<Vec<_>, ScanError> = group
            .files
            .into_par_iter()
            .map(|mut file| {
                let hash = partial::partial_hash(&file.path)?;
                file.partial_hash = Some(hash);
                if let Some(ref pb) = progress {
                    pb.inc(1);
                }
                Ok((hash, file))
            })
            .collect();

        let files_with_hashes = files_with_hashes?;

        // Group by hash (sequential, as HashMap isn't thread-safe)
        let mut hash_map: HashMap<[u8; 32], Vec<FileEntry>> = HashMap::new();
        for (hash, file) in files_with_hashes {
            hash_map.entry(hash).or_default().push(file);
        }

        // Keep only groups with 2+ files
        for (_, files) in hash_map {
            if files.len() >= 2 {
                result.push(DuplicateGroup::new(group.size, files));
            }
        }
    }

    Ok(result)
}

/// Group files by full hash - PARALLEL
///
/// Takes partial-hash-grouped candidates and computes full hashes in parallel.
pub fn group_by_full_hash(groups: Vec<DuplicateGroup>) -> Result<Vec<DuplicateGroup>, ScanError> {
    group_by_full_hash_with_progress(groups, None)
}

/// Group files by full hash with optional progress tracking
pub fn group_by_full_hash_with_progress(
    groups: Vec<DuplicateGroup>,
    progress: Option<SharedProgress>,
) -> Result<Vec<DuplicateGroup>, ScanError> {
    let mut result = Vec::new();

    for group in groups {
        // Compute full hashes in parallel
        let files_with_hashes: Result<Vec<_>, ScanError> = group
            .files
            .into_par_iter()
            .map(|mut file| {
                let hash = full::full_hash(&file.path, file.size)?;
                file.full_hash = Some(hash);
                if let Some(ref pb) = progress {
                    pb.inc(1);
                }
                Ok((hash, file))
            })
            .collect();

        let files_with_hashes = files_with_hashes?;

        // Group by hash
        let mut hash_map: HashMap<[u8; 32], Vec<FileEntry>> = HashMap::new();
        for (hash, file) in files_with_hashes {
            hash_map.entry(hash).or_default().push(file);
        }

        // Keep only groups with 2+ files
        for (hash, files) in hash_map {
            if files.len() >= 2 {
                result.push(DuplicateGroup::with_hash(hash, group.size, files));
            }
        }
    }

    Ok(result)
}

/// Verify duplicates with byte-by-byte comparison (paranoid mode) - PARALLEL
pub fn verify_with_byte_compare(
    groups: Vec<DuplicateGroup>,
) -> Result<Vec<DuplicateGroup>, ScanError> {
    groups
        .into_par_iter()
        .filter_map(|group| {
            if group.files.len() < 2 {
                return None;
            }

            let reference = &group.files[0];
            let verified: Vec<FileEntry> = std::iter::once(group.files[0].clone())
                .chain(
                    group.files[1..]
                        .par_iter()
                        .filter_map(|file| {
                            match full::compare_files(&reference.path, &file.path, group.size) {
                                Ok(true) => Some(file.clone()),
                                _ => None,
                            }
                        })
                        .collect::<Vec<_>>(),
                )
                .collect();

            if verified.len() >= 2 {
                Some(Ok(DuplicateGroup {
                    hash: group.hash,
                    size: group.size,
                    files: verified,
                }))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::{FileEntry, FileId};
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    fn create_test_file(dir: &std::path::Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    fn make_entry(path: PathBuf, size: u64) -> FileEntry {
        FileEntry {
            path,
            size,
            file_id: FileId { device: 1, inode: rand::random() },
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
            partial_hash: None,
            full_hash: None,
        }
    }

    #[test]
    fn test_group_by_partial_hash_same_content() {
        let dir = tempdir().unwrap();
        let content = b"duplicate content for partial hash test";

        let path1 = create_test_file(dir.path(), "file1.txt", content);
        let path2 = create_test_file(dir.path(), "file2.txt", content);

        let entry1 = make_entry(path1, content.len() as u64);
        let entry2 = make_entry(path2, content.len() as u64);

        let groups = vec![DuplicateGroup::new(content.len() as u64, vec![entry1, entry2])];

        let result = group_by_partial_hash(groups).unwrap();

        assert_eq!(result.len(), 1, "Same content should form one group");
        assert_eq!(result[0].files.len(), 2);
    }

    #[test]
    fn test_group_by_partial_hash_different_content() {
        let dir = tempdir().unwrap();

        let path1 = create_test_file(dir.path(), "file1.txt", b"content one");
        let path2 = create_test_file(dir.path(), "file2.txt", b"content two");

        let entry1 = make_entry(path1, 11);
        let entry2 = make_entry(path2, 11);

        // Same size but different content
        let groups = vec![DuplicateGroup::new(11, vec![entry1, entry2])];

        let result = group_by_partial_hash(groups).unwrap();

        assert_eq!(result.len(), 0, "Different content should not group");
    }

    #[test]
    fn test_group_by_full_hash_same_content() {
        let dir = tempdir().unwrap();
        let content = b"duplicate content for full hash test";

        let path1 = create_test_file(dir.path(), "file1.txt", content);
        let path2 = create_test_file(dir.path(), "file2.txt", content);

        let mut entry1 = make_entry(path1, content.len() as u64);
        let mut entry2 = make_entry(path2, content.len() as u64);

        // Set partial hashes (they would match)
        entry1.partial_hash = Some([0u8; 32]);
        entry2.partial_hash = Some([0u8; 32]);

        let groups = vec![DuplicateGroup::new(content.len() as u64, vec![entry1, entry2])];

        let result = group_by_full_hash(groups).unwrap();

        assert_eq!(result.len(), 1, "Same content should form one group");
        assert_eq!(result[0].files.len(), 2);
        assert!(result[0].hash.is_some(), "Full hash should be set");
    }

    #[test]
    fn test_group_by_full_hash_different_content() {
        let dir = tempdir().unwrap();

        let path1 = create_test_file(dir.path(), "file1.txt", b"different 1");
        let path2 = create_test_file(dir.path(), "file2.txt", b"different 2");

        let entry1 = make_entry(path1, 11);
        let entry2 = make_entry(path2, 11);

        let groups = vec![DuplicateGroup::new(11, vec![entry1, entry2])];

        let result = group_by_full_hash(groups).unwrap();

        assert_eq!(result.len(), 0, "Different content should not group");
    }

    #[test]
    fn test_verify_with_byte_compare_identical() {
        let dir = tempdir().unwrap();
        let content = b"content for paranoid verification";

        let path1 = create_test_file(dir.path(), "file1.txt", content);
        let path2 = create_test_file(dir.path(), "file2.txt", content);

        let entry1 = make_entry(path1, content.len() as u64);
        let entry2 = make_entry(path2, content.len() as u64);

        let groups = vec![DuplicateGroup {
            hash: Some([1u8; 32]),
            size: content.len() as u64,
            files: vec![entry1, entry2],
        }];

        let result = verify_with_byte_compare(groups).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].files.len(), 2);
    }

    #[test]
    fn test_verify_with_byte_compare_different() {
        let dir = tempdir().unwrap();

        let path1 = create_test_file(dir.path(), "file1.txt", b"content A");
        let path2 = create_test_file(dir.path(), "file2.txt", b"content B");

        let entry1 = make_entry(path1, 9);
        let entry2 = make_entry(path2, 9);

        // Pretend they have same hash (simulating collision)
        let groups = vec![DuplicateGroup {
            hash: Some([1u8; 32]),
            size: 9,
            files: vec![entry1, entry2],
        }];

        let result = verify_with_byte_compare(groups).unwrap();

        // Should filter out the false positive
        assert!(result.is_empty() || result[0].files.len() < 2);
    }

    #[test]
    fn test_empty_groups() {
        let result = group_by_partial_hash(vec![]).unwrap();
        assert!(result.is_empty());

        let result = group_by_full_hash(vec![]).unwrap();
        assert!(result.is_empty());

        let result = verify_with_byte_compare(vec![]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_file_groups_filtered() {
        let dir = tempdir().unwrap();
        let content = b"single file";
        let path = create_test_file(dir.path(), "single.txt", content);
        let entry = make_entry(path, content.len() as u64);

        let groups = vec![DuplicateGroup::new(content.len() as u64, vec![entry])];

        let result = group_by_partial_hash(groups).unwrap();
        assert!(result.is_empty(), "Single file should be filtered");
    }
}
