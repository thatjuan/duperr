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
