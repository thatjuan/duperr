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
