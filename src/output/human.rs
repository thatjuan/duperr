use crate::cli::KeepStrategy;
use crate::grouper::DuplicateGroup;
use crate::{ScanResult, ScanStats};
use colored::Colorize;
use humansize::{format_size, BINARY};
use std::io::{self, Write};

pub fn print(result: &ScanResult, keep_strategy: KeepStrategy, quiet: bool) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if quiet {
        return print_quiet(result, &mut handle);
    }

    if !result.has_duplicates() {
        writeln!(handle, "{}", "No duplicates found.".green())?;
        return Ok(());
    }

    writeln!(handle)?;
    writeln!(handle, "{}", "Duplicate files found:".bold())?;
    writeln!(handle, "{}", "=".repeat(60))?;

    for (i, group) in result.groups.iter().enumerate() {
        print_group(&mut handle, group, i + 1, keep_strategy)?;
    }

    print_summary(&mut handle, &result.stats)?;

    Ok(())
}

fn print_group(
    handle: &mut impl Write,
    group: &DuplicateGroup,
    index: usize,
    keep_strategy: KeepStrategy,
) -> io::Result<()> {
    let size_str = format_size(group.size, BINARY);

    writeln!(handle)?;
    writeln!(
        handle,
        "{} {} {} ({} files, {})",
        "Group".cyan().bold(),
        index.to_string().cyan().bold(),
        "-".dimmed(),
        group.files.len(),
        size_str.yellow()
    )?;

    let keeper = group.select_keeper(keep_strategy);

    for file in &group.files {
        let is_keeper = keeper.map(|k| k.path == file.path).unwrap_or(false);

        if is_keeper {
            writeln!(
                handle,
                "  {} {} {}",
                "→".green(),
                "[keep]".green().bold(),
                file.path.display()
            )?;
        } else {
            writeln!(
                handle,
                "  {} {}",
                "×".red(),
                file.path.display().to_string().dimmed()
            )?;
        }
    }

    Ok(())
}

fn print_summary(handle: &mut impl Write, stats: &ScanStats) -> io::Result<()> {
    writeln!(handle)?;
    writeln!(handle, "{}", "=".repeat(60))?;
    writeln!(handle, "{}", "Summary:".bold())?;
    writeln!(
        handle,
        "  Duplicate groups: {}",
        stats.duplicate_groups.to_string().cyan()
    )?;
    writeln!(
        handle,
        "  Duplicate files:  {}",
        stats.duplicate_files.to_string().cyan()
    )?;
    writeln!(
        handle,
        "  Wasted space:     {}",
        format_size(stats.wasted_bytes, BINARY).yellow()
    )?;

    if !stats.scan_duration.is_zero() {
        writeln!(
            handle,
            "  Scan time:        {:.2}s",
            stats.scan_duration.as_secs_f64()
        )?;
    }

    Ok(())
}

fn print_quiet(result: &ScanResult, handle: &mut impl Write) -> io::Result<()> {
    // Just print file paths, one per line, for scripting
    for group in &result.groups {
        for file in &group.files {
            writeln!(handle, "{}", file.path.display())?;
        }
    }
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

    fn make_result() -> ScanResult {
        let group = DuplicateGroup::with_hash(
            [1u8; 32],
            100,
            vec![make_entry("/a.txt", 100), make_entry("/b.txt", 100)],
        );
        ScanResult {
            groups: vec![group],
            stats: ScanStats {
                total_files_scanned: 10,
                total_bytes_scanned: 1000,
                files_with_unique_size: 5,
                duplicate_groups: 1,
                duplicate_files: 1,
                wasted_bytes: 100,
                scan_duration: Duration::from_secs(1),
            },
        }
    }

    #[test]
    fn test_quiet_mode_output() {
        let result = make_result();
        let mut buffer = Vec::new();

        print_quiet(&result, &mut buffer).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        // In quiet mode, should just print paths
        assert!(output.contains("/a.txt") || output.contains("/b.txt"));
    }

    #[test]
    fn test_no_duplicates() {
        let result = ScanResult {
            groups: vec![],
            stats: ScanStats::default(),
        };

        assert!(!result.has_duplicates());
    }

    #[test]
    fn test_has_duplicates() {
        let result = make_result();
        assert!(result.has_duplicates());
    }
}
