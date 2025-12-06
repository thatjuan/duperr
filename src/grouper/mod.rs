mod by_hash;
mod by_size;
mod duplicate_group;

pub use by_hash::{
    group_by_partial_hash,
    group_by_partial_hash_with_progress,
    group_by_full_hash,
    group_by_full_hash_with_progress,
    verify_with_byte_compare
};
pub use by_size::{group_by_size, group_by_size_with_stats, SizeGroupStats};
pub use duplicate_group::DuplicateGroup;
