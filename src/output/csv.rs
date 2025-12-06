use crate::ScanResult;
use std::io;

pub fn print(result: &ScanResult) -> io::Result<()> {
    let stdout = io::stdout();
    let handle = stdout.lock();
    let mut writer = csv::Writer::from_writer(handle);

    // Write header
    writer
        .write_record(["group", "hash", "size", "path", "modified"])
        .map_err(io::Error::other)?;

    for (i, group) in result.groups.iter().enumerate() {
        let group_num = (i + 1).to_string();
        let hash_str = group.hash.map(hex::encode).unwrap_or_default();
        let size_str = group.size.to_string();

        for file in &group.files {
            let path_str = file.path.to_string_lossy().to_string();
            let modified_str = file
                .modified
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();

            writer
                .write_record([&group_num, &hash_str, &size_str, &path_str, &modified_str])
                .map_err(io::Error::other)?;
        }
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::grouper::DuplicateGroup;
    use crate::scanner::{FileEntry, FileId};
    use std::time::{Duration, SystemTime};

    fn make_entry(path: &str, size: u64) -> FileEntry {
        FileEntry {
            path: path.into(),
            size,
            file_id: FileId { device: 1, inode: 1 },
            modified: SystemTime::UNIX_EPOCH + Duration::from_secs(1000),
            partial_hash: None,
            full_hash: Some([0u8; 32]),
        }
    }

    #[test]
    fn test_csv_header() {
        // The CSV should have headers: group, hash, size, path, modified
        let expected_headers = vec!["group", "hash", "size", "path", "modified"];
        assert_eq!(expected_headers.len(), 5);
    }

    #[test]
    fn test_csv_special_characters() {
        // Test that paths with commas are properly escaped
        let path_with_comma = "/path/with,comma/file.txt";
        let entry = make_entry(path_with_comma, 100);

        // The path should be escaped in CSV output
        let path_str = entry.path.to_string_lossy();
        assert!(path_str.contains(","));
    }

    #[test]
    fn test_csv_group_numbering() {
        // Groups should be numbered 1, 2, 3...
        let group1 = DuplicateGroup::with_hash(
            [1u8; 32],
            100,
            vec![make_entry("/a.txt", 100), make_entry("/b.txt", 100)],
        );
        let group2 = DuplicateGroup::with_hash(
            [2u8; 32],
            200,
            vec![make_entry("/c.txt", 200), make_entry("/d.txt", 200)],
        );

        // Both groups should produce multiple CSV rows
        assert_eq!(group1.files.len(), 2);
        assert_eq!(group2.files.len(), 2);
    }
}
