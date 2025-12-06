use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
    Csv,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq, Eq)]
pub enum KeepStrategy {
    #[default]
    First,
    Oldest,
    Newest,
    ShortestPath,
    LongestPath,
}

/// Fast and safe duplicate file finder
#[derive(Parser, Debug)]
#[command(name = "duperr")]
#[command(author, version, about, long_about = None)]
#[command(after_help = "EXAMPLES:
    # Scan a directory and show duplicates
    duperr ~/Documents

    # Scan multiple directories, only images
    duperr ~/Photos ~/Backup -e jpg,png,gif

    # Output as JSON, exclude temp files
    duperr . --output json --exclude '*.tmp'

    # Delete duplicates interactively (dry-run first)
    duperr ~/Downloads --delete --keep newest

    # Delete with backup (requires explicit --yes)
    duperr ~/Downloads --delete --backup-dir ~/dup-backup --yes
")]
pub struct Args {
    /// Directories to scan for duplicates
    #[arg(required = true)]
    pub paths: Vec<PathBuf>,

    // === Filtering Options ===

    /// Minimum file size (e.g., 1K, 1M, 1G). Files smaller are skipped.
    #[arg(long, value_name = "SIZE")]
    pub min_size: Option<String>,

    /// Maximum file size (e.g., 1K, 1M, 1G). Files larger are skipped.
    #[arg(long, value_name = "SIZE")]
    pub max_size: Option<String>,

    /// Only include files with these extensions (comma-separated, without dots)
    #[arg(short = 'e', long, value_delimiter = ',', value_name = "EXT")]
    pub extensions: Option<Vec<String>>,

    /// Exclude files matching these glob patterns (can be repeated)
    #[arg(long, action = clap::ArgAction::Append, value_name = "PATTERN")]
    pub exclude: Vec<String>,

    /// Include hidden files and directories (starting with '.')
    #[arg(long, default_value = "false")]
    pub hidden: bool,

    /// Follow symbolic links (WARNING: may cause infinite loops)
    #[arg(long, default_value = "false")]
    pub follow_symlinks: bool,

    /// Maximum directory depth to scan (0 = only specified dirs)
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,

    // === Output Options ===

    /// Output format for results
    #[arg(short = 'o', long, value_enum, default_value = "human")]
    pub output: OutputFormat,

    /// Suppress all output except errors
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Enable verbose/debug output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Show progress bar (default: auto-detect TTY)
    #[arg(long)]
    pub progress: Option<bool>,

    // === Action Options ===

    /// Enable deletion mode (dry-run by default unless --yes)
    #[arg(long)]
    pub delete: bool,

    /// Backup duplicates to this directory before deletion
    #[arg(long, value_name = "DIR")]
    pub backup_dir: Option<PathBuf>,

    /// Show what would be deleted without making changes
    #[arg(long)]
    pub dry_run: bool,

    /// Strategy for selecting which file to keep
    #[arg(long, value_enum, default_value = "first")]
    pub keep: KeepStrategy,

    /// Skip interactive confirmation (DANGEROUS with --delete)
    #[arg(long, short = 'y')]
    pub yes: bool,

    // === Performance Options ===

    /// Number of threads (default: number of CPU cores)
    #[arg(short = 'j', long, value_name = "N")]
    pub threads: Option<usize>,

    // === Safety Options ===

    /// Byte-by-byte comparison after hash match (slower but paranoid-safe)
    #[arg(long)]
    pub paranoid: bool,

    /// Skip empty files (0 bytes)
    #[arg(long, default_value = "true")]
    pub skip_empty: bool,
}
