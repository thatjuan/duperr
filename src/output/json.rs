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
