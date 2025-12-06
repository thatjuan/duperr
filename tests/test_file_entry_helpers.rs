// Test file_entry_helpers module
mod common;

use common::file_entry_helpers::*;

#[test]
fn test_make_file_entry() {
    let entry = make_file_entry("/test/path.txt", 1024);

    assert_eq!(entry.path.to_str().unwrap(), "/test/path.txt");
    assert_eq!(entry.size, 1024);
    assert_eq!(entry.file_id.device, 1);
    assert!(entry.partial_hash.is_none());
    assert!(entry.full_hash.is_none());
}

#[test]
fn test_make_file_entry_with_time() {
    let entry = make_file_entry_with_time("/test/file.bin", 512, 2000);

    assert_eq!(entry.size, 512);
    assert_eq!(
        entry.modified,
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(2000)
    );
}

#[test]
fn test_make_file_entry_with_inode() {
    let entry = make_file_entry_with_inode("/test/hardlink.txt", 256, 42, 1337);

    assert_eq!(entry.size, 256);
    assert_eq!(entry.file_id.device, 42);
    assert_eq!(entry.file_id.inode, 1337);
}

#[test]
fn test_make_file_entries_same_size() {
    let paths = ["/test/a.txt", "/test/b.txt", "/test/c.txt"];
    let entries = make_file_entries_same_size(&paths, 4096);

    assert_eq!(entries.len(), 3);
    for (i, entry) in entries.iter().enumerate() {
        assert_eq!(entry.path.to_str().unwrap(), paths[i]);
        assert_eq!(entry.size, 4096);
    }
}

#[test]
fn test_entries_have_unique_inodes() {
    let entries = make_file_entries_same_size(&["/a", "/b"], 100);

    // Different files should have different inodes (since we use rand::random)
    // This might fail very rarely if random generates same number, but extremely unlikely
    assert_ne!(entries[0].file_id.inode, entries[1].file_id.inode);
}
