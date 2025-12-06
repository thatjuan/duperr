use crate::cli::args::{Args, KeepStrategy, OutputFormat};
use anyhow::{anyhow, bail, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::PathBuf;

/// Runtime configuration derived from CLI arguments with validation and defaults
#[derive(Debug, Clone)]
pub struct Config {
    pub paths: Vec<PathBuf>,

    // Filtering
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub extensions: Option<Vec<String>>,
    pub exclude: GlobSet,
    pub hidden: bool,
    pub follow_symlinks: bool,
    pub depth: Option<usize>,

    // Output
    pub output: OutputFormat,
    pub quiet: bool,
    pub verbose: bool,
    pub progress: bool,

    // Actions
    pub delete: bool,
    pub backup_dir: Option<PathBuf>,
    pub dry_run: bool,
    pub keep: KeepStrategy,
    pub yes: bool,

    // Performance
    pub threads: usize,

    // Safety
    pub paranoid: bool,
    pub skip_empty: bool,
}

impl Config {
    /// Create a Config from parsed Args with validation
    pub fn from_args(args: Args) -> Result<Self> {
        // Validate conflicting options
        if args.quiet && args.verbose {
            bail!("Cannot use --quiet and --verbose together");
        }

        // Parse size strings
        let min_size = args
            .min_size
            .as_deref()
            .map(parse_size)
            .transpose()
            .context("Invalid --min-size")?;

        let max_size = args
            .max_size
            .as_deref()
            .map(parse_size)
            .transpose()
            .context("Invalid --max-size")?;

        // Validate size range
        if let (Some(min), Some(max)) = (min_size, max_size) {
            if min > max {
                bail!(
                    "Invalid size range: --min-size ({}) is greater than --max-size ({})",
                    args.min_size.as_deref().unwrap_or("?"),
                    args.max_size.as_deref().unwrap_or("?")
                );
            }
        }

        // Build exclude glob set
        let exclude = build_glob_set(&args.exclude)
            .context("Invalid --exclude pattern")?;

        // Normalize extensions (remove dots if present)
        let extensions = args.extensions.map(|exts| {
            exts.into_iter()
                .map(|e| e.trim_start_matches('.').to_lowercase())
                .collect()
        });

        // Determine progress bar visibility
        let progress = args.progress.unwrap_or_else(|| {
            !args.quiet && atty::is(atty::Stream::Stdout)
        });

        // Determine thread count
        let threads = args.threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
        });

        if threads == 0 {
            bail!("--threads must be at least 1");
        }

        // Validate deletion options
        if args.delete && !args.yes && !args.dry_run {
            // This is OK - interactive mode will prompt
        }

        if args.yes && args.dry_run {
            bail!("Cannot use --yes with --dry-run (dry-run never deletes)");
        }

        if args.backup_dir.is_some() && !args.delete {
            bail!("--backup-dir requires --delete");
        }

        // Validate paths exist and are directories
        for path in &args.paths {
            if !path.exists() {
                bail!("Path does not exist: {}", path.display());
            }
            if !path.is_dir() {
                bail!("Path is not a directory: {}", path.display());
            }
        }

        Ok(Config {
            paths: args.paths,
            min_size,
            max_size,
            extensions,
            exclude,
            hidden: args.hidden,
            follow_symlinks: args.follow_symlinks,
            depth: args.depth,
            output: args.output,
            quiet: args.quiet,
            verbose: args.verbose,
            progress,
            delete: args.delete,
            backup_dir: args.backup_dir,
            dry_run: args.dry_run,
            keep: args.keep,
            yes: args.yes,
            threads,
            paranoid: args.paranoid,
            skip_empty: args.skip_empty,
        })
    }
}

/// Parse a human-readable size string into bytes
///
/// Supports formats like:
/// - "1234" (raw bytes)
/// - "1K" or "1KB" (kilobytes)
/// - "1M" or "1MB" (megabytes)
/// - "1G" or "1GB" (gigabytes)
/// - "1T" or "1TB" (terabytes)
///
/// Case-insensitive, whitespace is ignored.
pub fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim().to_uppercase();

    if s.is_empty() {
        bail!("Size string is empty");
    }

    // Find where the numeric part ends
    let numeric_end = s
        .chars()
        .position(|c| !c.is_ascii_digit())
        .unwrap_or(s.len());

    let (num_part, suffix) = s.split_at(numeric_end);

    let base: u64 = num_part
        .parse()
        .map_err(|_| anyhow!("Invalid number: {}", num_part))?;

    let multiplier = match suffix.trim() {
        "" => 1,
        "K" | "KB" => 1024,
        "M" | "MB" => 1024 * 1024,
        "G" | "GB" => 1024 * 1024 * 1024,
        "T" | "TB" => 1024 * 1024 * 1024 * 1024,
        _ => bail!("Unknown size suffix: '{}'. Use K, M, G, or T", suffix),
    };

    base.checked_mul(multiplier)
        .ok_or_else(|| anyhow!("Size overflow: {} is too large", s))
}

/// Build a GlobSet from a list of patterns
fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let glob = Glob::new(pattern)
            .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
        builder.add(glob);
    }

    builder.build()
        .context("Failed to build glob set")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_bytes() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("0").unwrap(), 0);
    }

    #[test]
    fn test_parse_size_kilobytes() {
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("10k").unwrap(), 10 * 1024);
    }

    #[test]
    fn test_parse_size_megabytes() {
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
    }

    #[test]
    fn test_parse_size_gigabytes() {
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("2GB").unwrap(), 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_size_terabytes() {
        assert_eq!(parse_size("1T").unwrap(), 1024 * 1024 * 1024 * 1024);
        assert_eq!(parse_size("1TB").unwrap(), 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_parse_size_whitespace() {
        assert_eq!(parse_size(" 1M ").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("  100K  ").unwrap(), 100 * 1024);
    }

    #[test]
    fn test_parse_size_invalid() {
        assert!(parse_size("abc").is_err());
        assert!(parse_size("1X").is_err());
        assert!(parse_size("-1").is_err());
        assert!(parse_size("").is_err());
        assert!(parse_size("XYZ").is_err());
    }
}
