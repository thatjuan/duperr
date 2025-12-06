use crate::ScanResult;
use std::io;

pub fn print(result: &ScanResult) -> io::Result<()> {
    let stdout = io::stdout();
    let handle = stdout.lock();
    let mut writer = csv::Writer::from_writer(handle);

    // Write header
    writer.write_record(["group", "hash", "size", "path", "modified"])
        .map_err(io::Error::other)?;

    for (i, group) in result.groups.iter().enumerate() {
        let group_num = (i + 1).to_string();
        let hash_str = group.hash
            .map(hex::encode)
            .unwrap_or_default();
        let size_str = group.size.to_string();

        for file in &group.files {
            let path_str = file.path.to_string_lossy().to_string();
            let modified_str = file.modified
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();

            writer.write_record([
                &group_num,
                &hash_str,
                &size_str,
                &path_str,
                &modified_str,
            ]).map_err(io::Error::other)?;
        }
    }

    writer.flush()?;
    Ok(())
}
