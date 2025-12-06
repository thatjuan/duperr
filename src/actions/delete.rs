use std::path::Path;
use std::fs;
use dialoguer::Select;
use colored::Colorize;

use crate::ScanResult;
use crate::cli::KeepStrategy;
use crate::error::ScanError;

/// Statistics about deletion operation
#[derive(Debug, Default)]
pub struct DeleteStats {
    pub files_deleted: u64,
    pub bytes_freed: u64,
    pub files_skipped: u64,
    pub files_failed: u64,
}

/// Prompt choices for deletion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteChoice {
    Yes,
    No,
    All,
    Quit,
}

/// Delete duplicates with interactive confirmation
///
/// Prompts for each file unless auto_confirm is true.
/// Returns statistics about the deletion operation.
pub fn interactive_delete(
    result: &ScanResult,
    keep_strategy: KeepStrategy,
    auto_confirm: bool,
) -> Result<DeleteStats, ScanError> {
    let mut stats = DeleteStats::default();
    let mut confirm_all = auto_confirm;

    for group in &result.groups {
        let keeper = match group.select_keeper(keep_strategy) {
            Some(k) => k,
            None => continue,
        };

        println!();
        println!(
            "{} Keeping: {}",
            "→".green().bold(),
            keeper.path.display().to_string().green()
        );

        for file in group.get_duplicates(keep_strategy) {
            if confirm_all {
                match delete_file(&file.path) {
                    Ok(()) => {
                        stats.files_deleted += 1;
                        stats.bytes_freed += file.size;
                        println!(
                            "  {} Deleted: {}",
                            "✓".green(),
                            file.path.display()
                        );
                    }
                    Err(e) => {
                        stats.files_failed += 1;
                        println!(
                            "  {} Failed: {} ({})",
                            "✗".red(),
                            file.path.display(),
                            e
                        );
                    }
                }
                continue;
            }

            let choice = prompt_delete_choice(&file.path)?;

            match choice {
                DeleteChoice::Yes => {
                    match delete_file(&file.path) {
                        Ok(()) => {
                            stats.files_deleted += 1;
                            stats.bytes_freed += file.size;
                        }
                        Err(e) => {
                            stats.files_failed += 1;
                            eprintln!("  {} Failed to delete: {}", "✗".red(), e);
                        }
                    }
                }
                DeleteChoice::No => {
                    stats.files_skipped += 1;
                }
                DeleteChoice::All => {
                    confirm_all = true;
                    match delete_file(&file.path) {
                        Ok(()) => {
                            stats.files_deleted += 1;
                            stats.bytes_freed += file.size;
                        }
                        Err(e) => {
                            stats.files_failed += 1;
                            eprintln!("  {} Failed to delete: {}", "✗".red(), e);
                        }
                    }
                }
                DeleteChoice::Quit => {
                    println!("Deletion cancelled by user.");
                    return Ok(stats);
                }
            }
        }
    }

    print_delete_summary(&stats);
    Ok(stats)
}

fn prompt_delete_choice(path: &Path) -> Result<DeleteChoice, ScanError> {
    let items = &["No", "Yes", "All remaining", "Quit"];

    let selection = Select::new()
        .with_prompt(format!("Delete {}?", path.display()))
        .items(items)
        .default(0)  // Default to "No" for safety
        .interact()
        .map_err(|_| ScanError::Cancelled)?;

    Ok(match selection {
        0 => DeleteChoice::No,
        1 => DeleteChoice::Yes,
        2 => DeleteChoice::All,
        _ => DeleteChoice::Quit,
    })
}

fn delete_file(path: &Path) -> Result<(), ScanError> {
    fs::remove_file(path)
        .map_err(|e| ScanError::DeleteFailed(path.to_owned(), e))
}

fn print_delete_summary(stats: &DeleteStats) {
    println!();
    println!("{}", "Deletion Summary:".bold());
    println!("  Files deleted: {}", stats.files_deleted.to_string().green());
    println!(
        "  Space freed:   {}",
        humansize::format_size(stats.bytes_freed, humansize::BINARY).yellow()
    );
    if stats.files_skipped > 0 {
        println!("  Files skipped: {}", stats.files_skipped);
    }
    if stats.files_failed > 0 {
        println!("  Files failed:  {}", stats.files_failed.to_string().red());
    }
}

/// Show what would be deleted (dry-run mode)
pub fn dry_run(result: &ScanResult, keep_strategy: KeepStrategy) {
    println!();
    println!("{}", "DRY RUN - No files will be deleted".yellow().bold());
    println!();

    let mut total_files = 0u64;
    let mut total_bytes = 0u64;

    for group in &result.groups {
        let keeper = match group.select_keeper(keep_strategy) {
            Some(k) => k,
            None => continue,
        };

        println!(
            "{} Would keep: {}",
            "→".green(),
            keeper.path.display()
        );

        for file in group.get_duplicates(keep_strategy) {
            println!(
                "  {} Would delete: {}",
                "×".red(),
                file.path.display()
            );
            total_files += 1;
            total_bytes += file.size;
        }
        println!();
    }

    println!("{}", "=".repeat(50));
    println!(
        "Would delete {} files, freeing {}",
        total_files.to_string().cyan(),
        humansize::format_size(total_bytes, humansize::BINARY).yellow()
    );
    println!();
    println!("Run with {} to actually delete files.", "--yes".cyan());
}
