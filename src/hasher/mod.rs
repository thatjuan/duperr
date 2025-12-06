pub mod full;
pub mod partial;

pub use full::{compare_files, full_hash, MMAP_THRESHOLD};
pub use partial::{partial_hash, PARTIAL_HASH_SIZE};
