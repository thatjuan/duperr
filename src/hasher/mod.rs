pub mod partial;
pub mod full;

pub use partial::{partial_hash, PARTIAL_HASH_SIZE};
pub use full::{full_hash, compare_files, MMAP_THRESHOLD};
