#![allow(dead_code)]

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

// Re-export for convenience
pub use tempfile::tempdir;

pub mod file_entry_helpers;

/// Test fixture for creating duplicate file scenarios
pub struct TestFixture {
    pub dir: TempDir,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            dir: tempdir().expect("Failed to create temp dir"),
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Create a file with specific content
    pub fn create_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.dir.path().join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        let mut file = File::create(&path).expect("Failed to create file");
        file.write_all(content).expect("Failed to write file");
        path
    }

    /// Create a file of specific size with deterministic content
    pub fn create_file_with_size(&self, name: &str, size: usize) -> PathBuf {
        let content: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        self.create_file(name, &content)
    }

    /// Create multiple duplicate files with same content
    pub fn create_duplicates(&self, names: &[&str], content: &[u8]) -> Vec<PathBuf> {
        names.iter().map(|name| self.create_file(name, content)).collect()
    }

    /// Create a pair of duplicate files
    pub fn create_duplicate_pair(&self, name1: &str, name2: &str, content: &[u8]) -> (PathBuf, PathBuf) {
        (self.create_file(name1, content), self.create_file(name2, content))
    }

    /// Create hidden file (starts with .)
    pub fn create_hidden_file(&self, name: &str, content: &[u8]) -> PathBuf {
        let hidden_name = if name.starts_with('.') { name.to_string() } else { format!(".{}", name) };
        self.create_file(&hidden_name, content)
    }

    /// Create a subdirectory
    pub fn create_subdir(&self, name: &str) -> PathBuf {
        let path = self.dir.path().join(name);
        fs::create_dir_all(&path).expect("Failed to create subdir");
        path
    }

    /// Create nested directory structure with files
    pub fn create_nested(&self, depth: usize, file_content: &[u8]) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let mut current = String::new();

        for i in 0..depth {
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(&format!("level{}", i));
            paths.push(self.create_file(&format!("{}/file.txt", current), file_content));
        }

        paths
    }

    /// Create a symlink (Unix only)
    #[cfg(unix)]
    pub fn create_symlink(&self, name: &str, target: &Path) -> PathBuf {
        let path = self.dir.path().join(name);
        std::os::unix::fs::symlink(target, &path).expect("Failed to create symlink");
        path
    }

    /// Create a hardlink (Unix only)
    #[cfg(unix)]
    pub fn create_hardlink(&self, name: &str, target: &Path) -> PathBuf {
        let path = self.dir.path().join(name);
        fs::hard_link(target, &path).expect("Failed to create hardlink");
        path
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Fixture with pre-created duplicate scenarios
pub struct DuplicateScenario {
    pub fixture: TestFixture,
    pub duplicate_groups: Vec<Vec<PathBuf>>,
    pub unique_files: Vec<PathBuf>,
}

impl DuplicateScenario {
    /// Simple scenario: one pair of duplicates
    pub fn simple() -> Self {
        let fixture = TestFixture::new();
        let content = b"duplicate content here";
        let dups = fixture.create_duplicates(&["file1.txt", "file2.txt"], content);
        let unique = vec![fixture.create_file("unique.txt", b"unique content")];

        Self {
            fixture,
            duplicate_groups: vec![dups],
            unique_files: unique,
        }
    }

    /// Multiple duplicate groups
    pub fn multiple_groups() -> Self {
        let fixture = TestFixture::new();

        let group1 = fixture.create_duplicates(&["a1.txt", "a2.txt"], b"content A");
        let group2 = fixture.create_duplicates(&["b1.txt", "b2.txt", "b3.txt"], b"content B");
        let group3 = fixture.create_duplicates(&["c1.txt", "c2.txt"], b"content C");
        let unique = vec![fixture.create_file("unique.txt", b"unique")];

        Self {
            fixture,
            duplicate_groups: vec![group1, group2, group3],
            unique_files: unique,
        }
    }

    /// Duplicates with specific file sizes
    pub fn with_sizes(sizes: &[(usize, usize)]) -> Self {
        let fixture = TestFixture::new();
        let mut groups = Vec::new();

        for (i, (size, count)) in sizes.iter().enumerate() {
            let content: Vec<u8> = (0..*size).map(|j| ((i * 37 + j) % 256) as u8).collect();
            let names: Vec<String> = (0..*count).map(|j| format!("group{}_{}.bin", i, j)).collect();
            let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            groups.push(fixture.create_duplicates(&name_refs, &content));
        }

        Self {
            fixture,
            duplicate_groups: groups,
            unique_files: vec![],
        }
    }

    /// Duplicates across subdirectories
    pub fn nested() -> Self {
        let fixture = TestFixture::new();
        let content = b"nested duplicate content";

        fixture.create_subdir("dir1");
        fixture.create_subdir("dir2/subdir");

        let dups = vec![
            fixture.create_file("dir1/file.txt", content),
            fixture.create_file("dir2/file.txt", content),
            fixture.create_file("dir2/subdir/file.txt", content),
        ];

        Self {
            fixture,
            duplicate_groups: vec![dups],
            unique_files: vec![],
        }
    }

    /// Files of various extensions
    pub fn with_extensions() -> Self {
        let fixture = TestFixture::new();

        let txts = fixture.create_duplicates(&["doc1.txt", "doc2.txt"], b"text content");
        let jpgs = fixture.create_duplicates(&["img1.jpg", "img2.jpg"], b"image content");
        let pngs = fixture.create_duplicates(&["pic1.png", "pic2.png"], b"png content");
        let unique = vec![fixture.create_file("other.doc", b"doc content")];

        Self {
            fixture,
            duplicate_groups: vec![txts, jpgs, pngs],
            unique_files: unique,
        }
    }

    pub fn path(&self) -> &Path {
        self.fixture.path()
    }
}

/// Assert two files have identical content
pub fn assert_files_equal(path1: &Path, path2: &Path) {
    let content1 = fs::read(path1).expect("Failed to read file1");
    let content2 = fs::read(path2).expect("Failed to read file2");
    assert_eq!(content1, content2, "File contents differ");
}

/// Assert file exists and has expected size
pub fn assert_file_size(path: &Path, expected: u64) {
    let metadata = fs::metadata(path).expect("Failed to get metadata");
    assert_eq!(metadata.len(), expected, "File size mismatch");
}

/// Assert file does not exist
pub fn assert_file_not_exists(path: &Path) {
    assert!(!path.exists(), "File should not exist: {:?}", path);
}

/// Parse JSON output and return as Value
pub fn parse_json_output(output: &str) -> serde_json::Value {
    serde_json::from_str(output).expect("Failed to parse JSON output")
}

/// Parse CSV output into records
pub fn parse_csv_output(output: &str) -> Vec<HashMap<String, String>> {
    let mut reader = csv::Reader::from_reader(output.as_bytes());
    let headers: Vec<String> = reader.headers()
        .expect("No headers")
        .iter()
        .map(|s| s.to_string())
        .collect();

    reader.records()
        .map(|r| {
            let record = r.expect("Invalid record");
            headers.iter()
                .zip(record.iter())
                .map(|(h, v)| (h.clone(), v.to_string()))
                .collect()
        })
        .collect()
}

/// Count duplicate groups in JSON output
pub fn count_groups_in_json(output: &str) -> usize {
    let json = parse_json_output(output);
    json["groups"].as_array().map(|a| a.len()).unwrap_or(0)
}

/// Get total wasted bytes from JSON output
pub fn get_wasted_bytes_from_json(output: &str) -> u64 {
    let json = parse_json_output(output);
    json["stats"]["wasted_bytes"].as_u64().unwrap_or(0)
}
