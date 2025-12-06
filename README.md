# duperr

A fast and safe duplicate file finder written in Rust.

## Features

- **Fast**: Progressive hashing (size → partial → full) with parallel processing
- **Safe**: Interactive deletion with backup support, dry-run mode by default
- **Flexible**: Multiple output formats (human, JSON, CSV)
- **Smart**: Hardlink detection, empty file handling, symlink awareness

## Installation

### Quick Install (Linux/macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/thatjuan/duperr/main/install.sh | sh
```

### From Source

```bash
git clone https://github.com/thatjuan/duperr.git
cd duperr
make install
```

### Using Cargo

```bash
cargo install --git https://github.com/thatjuan/duperr.git
```

## Usage

```bash
# Scan a directory for duplicates
duperr ~/Documents

# Scan multiple directories
duperr ~/Photos ~/Backup

# Only check specific file types
duperr ~/Downloads -e jpg,png,gif

# Output as JSON
duperr ~/Documents --output json

# Delete duplicates (shows dry-run first)
duperr ~/Downloads --delete --keep newest

# Delete with backup
duperr ~/Downloads --delete --backup-dir ~/dup-backup --yes
```

## Options

### Filtering
| Option | Description |
|--------|-------------|
| `--min-size SIZE` | Minimum file size (e.g., 1K, 1M, 1G) |
| `--max-size SIZE` | Maximum file size |
| `-e, --extensions EXT` | Only include these extensions (comma-separated) |
| `--exclude PATTERN` | Exclude files matching glob pattern |
| `--hidden` | Include hidden files |
| `--follow-symlinks` | Follow symbolic links |
| `--depth N` | Maximum directory depth |

### Output
| Option | Description |
|--------|-------------|
| `-o, --output FORMAT` | Output format: human, json, csv |
| `-q, --quiet` | Suppress output except errors |
| `-v, --verbose` | Verbose output |
| `--progress` | Show progress bar |

### Actions
| Option | Description |
|--------|-------------|
| `--delete` | Enable deletion mode |
| `--backup-dir DIR` | Backup duplicates before deletion |
| `--dry-run` | Show what would be deleted |
| `--keep STRATEGY` | Keep strategy: first, oldest, newest, shortest-path, longest-path |
| `-y, --yes` | Skip confirmation prompts |

### Performance & Safety
| Option | Description |
|--------|-------------|
| `-j, --threads N` | Number of threads (default: CPU cores) |
| `--paranoid` | Byte-by-byte verification after hash match |
| `--skip-empty` | Skip empty files (default: true) |

## How It Works

1. **Scan**: Recursively walk directories, filtering by criteria
2. **Group by Size**: Files with unique sizes can't be duplicates
3. **Partial Hash**: Hash first 4KB to eliminate more candidates
4. **Full Hash**: BLAKE3 hash of entire file for confirmation
5. **Report/Act**: Display results or delete with backup

## License

MIT

## Contributing

Contributions welcome! Please run `make check` before submitting PRs.
