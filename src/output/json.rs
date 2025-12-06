use crate::ScanResult;
use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct JsonOutput<'a> {
    groups: Vec<JsonGroup<'a>>,
    stats: &'a crate::ScanStats,
}

#[derive(Serialize)]
struct JsonGroup<'a> {
    hash: Option<String>,
    size: u64,
    files: Vec<JsonFile<'a>>,
    wasted_bytes: u64,
}

#[derive(Serialize)]
struct JsonFile<'a> {
    path: &'a str,
    size: u64,
    modified: Option<u64>,
}

pub fn print(result: &ScanResult) -> io::Result<()> {
    let output = JsonOutput {
        groups: result
            .groups
            .iter()
            .map(|g| JsonGroup {
                hash: g.hash.map(hex::encode),
                size: g.size,
                wasted_bytes: g.wasted_bytes(),
                files: g
                    .files
                    .iter()
                    .map(|f| JsonFile {
                        path: f.path.to_str().unwrap_or("<invalid utf8>"),
                        size: f.size,
                        modified: f
                            .modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs()),
                    })
                    .collect(),
            })
            .collect(),
        stats: &result.stats,
    };

    let json = serde_json::to_string_pretty(&output).map_err(io::Error::other)?;

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", json)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grouper::DuplicateGroup;
    use crate::scanner::{FileEntry, FileId};
    use crate::{ScanResult, ScanStats};
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

    fn make_result_with_groups(groups: Vec<DuplicateGroup>) -> ScanResult {
        ScanResult {
            groups,
            stats: ScanStats {
                total_files_scanned: 10,
                total_bytes_scanned: 1000,
                files_with_unique_size: 5,
                duplicate_groups: 2,
                duplicate_files: 4,
                wasted_bytes: 200,
                scan_duration: Duration::from_secs(1),
            },
        }
    }

    #[test]
    fn test_json_output_structure() {
        let group = DuplicateGroup::with_hash(
            [1u8; 32],
            100,
            vec![make_entry("/a.txt", 100), make_entry("/b.txt", 100)],
        );
        let result = make_result_with_groups(vec![group]);

        // Just verify the structures are serializable
        let json_output = JsonOutput {
            groups: result.groups.iter().map(|g| JsonGroup {
                hash: g.hash.map(hex::encode),
                size: g.size,
                wasted_bytes: g.wasted_bytes(),
                files: g.files.iter().map(|f| JsonFile {
                    path: f.path.to_str().unwrap_or(""),
                    size: f.size,
                    modified: f.modified.duration_since(SystemTime::UNIX_EPOCH)
                        .ok().map(|d| d.as_secs()),
                }).collect(),
            }).collect(),
            stats: &result.stats,
        };

        let json_str = serde_json::to_string(&json_output).unwrap();
        assert!(json_str.contains("\"groups\""));
        assert!(json_str.contains("\"stats\""));
    }

    #[test]
    fn test_json_empty_result() {
        let result = make_result_with_groups(vec![]);

        let json_output = JsonOutput {
            groups: vec![],
            stats: &result.stats,
        };

        let json_str = serde_json::to_string(&json_output).unwrap();
        assert!(json_str.contains("\"groups\":[]"));
    }

    #[test]
    fn test_json_hash_encoding() {
        let hash = [0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90,
                    0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
                    0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00,
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let hex_str = hex::encode(hash);

        assert_eq!(hex_str.len(), 64);
        assert!(hex_str.starts_with("abcdef"));
    }
}
