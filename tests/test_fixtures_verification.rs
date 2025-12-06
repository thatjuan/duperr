// Simple test to verify test fixtures work correctly
mod common;

use common::{TestFixture, DuplicateScenario};

#[test]
fn test_fixture_create_file() {
    let fixture = TestFixture::new();
    let path = fixture.create_file("test.txt", b"hello world");

    assert!(path.exists());
    let content = std::fs::read(&path).unwrap();
    assert_eq!(content, b"hello world");
}

#[test]
fn test_fixture_create_duplicates() {
    let fixture = TestFixture::new();
    let content = b"duplicate content";
    let paths = fixture.create_duplicates(&["file1.txt", "file2.txt", "file3.txt"], content);

    assert_eq!(paths.len(), 3);
    for path in &paths {
        assert!(path.exists());
        let file_content = std::fs::read(path).unwrap();
        assert_eq!(file_content, content);
    }
}

#[test]
fn test_scenario_simple() {
    let scenario = DuplicateScenario::simple();

    assert_eq!(scenario.duplicate_groups.len(), 1);
    assert_eq!(scenario.duplicate_groups[0].len(), 2);
    assert_eq!(scenario.unique_files.len(), 1);

    // Verify all files exist
    for group in &scenario.duplicate_groups {
        for path in group {
            assert!(path.exists());
        }
    }
    for path in &scenario.unique_files {
        assert!(path.exists());
    }
}

#[test]
fn test_scenario_multiple_groups() {
    let scenario = DuplicateScenario::multiple_groups();

    assert_eq!(scenario.duplicate_groups.len(), 3);
    assert_eq!(scenario.duplicate_groups[0].len(), 2);  // group1: 2 files
    assert_eq!(scenario.duplicate_groups[1].len(), 3);  // group2: 3 files
    assert_eq!(scenario.duplicate_groups[2].len(), 2);  // group3: 2 files
}

#[test]
fn test_scenario_nested() {
    let scenario = DuplicateScenario::nested();

    assert_eq!(scenario.duplicate_groups.len(), 1);
    assert_eq!(scenario.duplicate_groups[0].len(), 3);

    // All files should have the same content
    let content1 = std::fs::read(&scenario.duplicate_groups[0][0]).unwrap();
    let content2 = std::fs::read(&scenario.duplicate_groups[0][1]).unwrap();
    let content3 = std::fs::read(&scenario.duplicate_groups[0][2]).unwrap();

    assert_eq!(content1, content2);
    assert_eq!(content2, content3);
}
