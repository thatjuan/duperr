use clap::Parser;
use colored::Colorize;
use anyhow::Context;
use duperr::cli::{Args, Config, OutputFormat};
use duperr::pipeline::find_duplicates;
use duperr::{output, actions};

fn main() {
    // Initialize logger
    env_logger::init();

    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);

        // Print cause chain for debugging
        let mut chain = e.chain().skip(1).peekable();
        if chain.peek().is_some() {
            eprintln!();
            eprintln!("{}", "Caused by:".yellow());
            for (i, cause) in chain.enumerate() {
                eprintln!("  {}. {}", i + 1, cause);
            }
        }

        // Hint about backtrace
        if std::env::var("RUST_BACKTRACE").is_err() {
            eprintln!();
            eprintln!("{}", "Hint: Set RUST_BACKTRACE=1 for more details".dimmed());
        }

        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::from_args(args)
        .context("Failed to parse configuration")?;

    // Run the duplicate finding pipeline
    let result = find_duplicates(&config)
        .context("Failed to scan for duplicates")?;

    // Output results
    match config.output {
        OutputFormat::Human => output::human::print(&result, config.keep, config.quiet)
            .context("Failed to print results")?,
        OutputFormat::Json => output::json::print(&result)
            .context("Failed to generate JSON output")?,
        OutputFormat::Csv => output::csv::print(&result)
            .context("Failed to generate CSV output")?,
    }

    // Handle deletion if requested
    if config.delete && result.has_duplicates() {
        // Backup first if backup_dir is specified
        if let Some(ref backup_dir) = config.backup_dir {
            actions::validate_backup_dir(backup_dir, &config.paths)
                .context("Backup directory validation failed")?;
            actions::backup_duplicates(&result, backup_dir, config.keep)
                .context("Failed to backup files")?;
        }

        // Then delete (or dry-run)
        if config.dry_run {
            actions::dry_run(&result, config.keep);
        } else {
            actions::interactive_delete(&result, config.keep, config.yes)
                .context("Failed to delete files")?;
        }
    }

    Ok(())
}
