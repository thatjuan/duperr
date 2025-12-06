use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("duplicate file finder"))
        .stdout(predicate::str::contains("--min-size"))
        .stdout(predicate::str::contains("--extensions"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_scan_empty_directory() {
    let dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn test_scan_with_duplicates() {
    let dir = tempdir().unwrap();

    // Create duplicate files
    fs::write(dir.path().join("file1.txt"), "duplicate content").unwrap();
    fs::write(dir.path().join("file2.txt"), "duplicate content").unwrap();
    fs::write(dir.path().join("unique.txt"), "unique content").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Group 1"))
        .stdout(predicate::str::contains("file1.txt").or(predicate::str::contains("file2.txt")));
}

#[test]
fn test_json_output() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("b.txt"), "same").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"groups\""))
        .stdout(predicate::str::contains("\"stats\""));
}

#[test]
fn test_extension_filter() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("b.txt"), "same").unwrap();
    fs::write(dir.path().join("c.jpg"), "same").unwrap();

    // Only txt files
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("-e")
        .arg("txt")
        .assert()
        .success()
        .stdout(predicate::str::contains("a.txt").or(predicate::str::contains("b.txt")))
        .stdout(predicate::str::contains("c.jpg").not());
}

#[test]
fn test_dry_run() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("b.txt"), "same").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--delete")
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN"))
        .stdout(predicate::str::contains("Would delete"));

    // Files should still exist
    assert!(dir.path().join("a.txt").exists());
    assert!(dir.path().join("b.txt").exists());
}

#[test]
fn test_invalid_path() {
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg("/nonexistent/path/that/does/not/exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_size_filter_min() {
    let dir = tempdir().unwrap();

    // Create files with different sizes
    fs::write(dir.path().join("small1.txt"), "x".repeat(50)).unwrap();
    fs::write(dir.path().join("small2.txt"), "x".repeat(50)).unwrap();
    fs::write(dir.path().join("large1.txt"), "x".repeat(200)).unwrap();
    fs::write(dir.path().join("large2.txt"), "x".repeat(200)).unwrap();

    // Only files >= 100 bytes
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--min-size")
        .arg("100")
        .assert()
        .success()
        .stdout(predicate::str::contains("large1.txt").or(predicate::str::contains("large2.txt")))
        .stdout(predicate::str::contains("small1.txt").not());
}

#[test]
fn test_hidden_files_excluded_by_default() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join(".hidden1"), "content").unwrap();
    fs::write(dir.path().join(".hidden2"), "content").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn test_hidden_files_with_flag() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join(".hidden1"), "content").unwrap();
    fs::write(dir.path().join(".hidden2"), "content").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--hidden")
        .assert()
        .success()
        .stdout(predicate::str::contains(".hidden"));
}

#[test]
fn test_quiet_mode() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("b.txt"), "same").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path()).arg("--quiet").assert().success();
    // In quiet mode, there should be minimal output
}

#[test]
fn test_no_duplicates_found() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("unique1.txt"), "content1").unwrap();
    fs::write(dir.path().join("unique2.txt"), "content2").unwrap();
    fs::write(dir.path().join("unique3.txt"), "content3").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates found"));
}

#[test]
fn test_multiple_duplicate_groups() {
    let dir = tempdir().unwrap();

    // Group 1: duplicate content A
    fs::write(dir.path().join("a1.txt"), "content A").unwrap();
    fs::write(dir.path().join("a2.txt"), "content A").unwrap();

    // Group 2: duplicate content B
    fs::write(dir.path().join("b1.txt"), "content B").unwrap();
    fs::write(dir.path().join("b2.txt"), "content B").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Group 1"))
        .stdout(predicate::str::contains("Group 2"));
}

#[test]
fn test_exclude_pattern() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("keep1.txt"), "same").unwrap();
    fs::write(dir.path().join("keep2.txt"), "same").unwrap();
    fs::write(dir.path().join("exclude1.tmp"), "same").unwrap();
    fs::write(dir.path().join("exclude2.tmp"), "same").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--exclude")
        .arg("*.tmp")
        .assert()
        .success()
        .stdout(predicate::str::contains("keep1.txt").or(predicate::str::contains("keep2.txt")))
        .stdout(predicate::str::contains("exclude1.tmp").not())
        .stdout(predicate::str::contains("exclude2.tmp").not());
}

// Additional integration tests for duperr

#[test]
fn test_csv_output_format() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same content").unwrap();
    fs::write(dir.path().join("b.txt"), "same content").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    let output = cmd
        .arg(dir.path())
        .arg("--output")
        .arg("csv")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check CSV header
    assert!(stdout.contains("group,hash,size,path,modified"));
}

#[test]
fn test_multiple_directories() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    // Create duplicate across directories
    fs::write(dir1.path().join("file.txt"), "cross-directory duplicate").unwrap();
    fs::write(dir2.path().join("file.txt"), "cross-directory duplicate").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir1.path())
        .arg(dir2.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Group 1"));
}

#[test]
fn test_keep_oldest() {
    let dir = tempdir().unwrap();

    let old_file = dir.path().join("old.txt");
    let new_file = dir.path().join("new.txt");

    fs::write(&old_file, "same content").unwrap();
    fs::write(&new_file, "same content").unwrap();

    // Set old file to earlier time
    let old_time = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1000);
    filetime::set_file_mtime(&old_file, filetime::FileTime::from_system_time(old_time)).ok();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--keep")
        .arg("oldest")
        .assert()
        .success();
}

#[test]
fn test_keep_newest() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("old.txt"), "same content").unwrap();
    fs::write(dir.path().join("new.txt"), "same content").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--keep")
        .arg("newest")
        .assert()
        .success();
}

#[test]
fn test_keep_shortest_path() {
    let dir = tempdir().unwrap();

    fs::create_dir_all(dir.path().join("very/long/path")).unwrap();
    fs::write(dir.path().join("short.txt"), "same").unwrap();
    fs::write(dir.path().join("very/long/path/file.txt"), "same").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--keep")
        .arg("shortest-path")
        .assert()
        .success();
}

#[test]
fn test_keep_longest_path() {
    let dir = tempdir().unwrap();

    fs::create_dir_all(dir.path().join("deep/nested")).unwrap();
    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("deep/nested/b.txt"), "same").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--keep")
        .arg("longest-path")
        .assert()
        .success();
}

#[test]
fn test_depth_limit_zero() {
    let dir = tempdir().unwrap();

    fs::create_dir_all(dir.path().join("subdir")).unwrap();
    fs::write(dir.path().join("root.txt"), "content").unwrap();
    fs::write(dir.path().join("subdir/nested.txt"), "content").unwrap();

    // Depth 1 should only include root directory
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--depth")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates"));
}

#[test]
fn test_verbose_mode() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "content").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("-v")
        .assert()
        .success();
}

#[test]
fn test_thread_count() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("b.txt"), "same").unwrap();

    // Single thread
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("-j")
        .arg("1")
        .assert()
        .success();

    // Multiple threads
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("-j")
        .arg("4")
        .assert()
        .success();
}

#[test]
fn test_paranoid_mode() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same content").unwrap();
    fs::write(dir.path().join("b.txt"), "same content").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--paranoid")
        .assert()
        .success()
        .stdout(predicate::str::contains("Group 1"));
}

#[test]
fn test_include_empty_files() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("empty1.txt"), "").unwrap();
    fs::write(dir.path().join("empty2.txt"), "").unwrap();

    // By default, empty files are skipped
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No duplicates"));

    // With --skip-empty=false, they should be found (if supported)
    // Note: This depends on the CLI supporting the flag
}

#[test]
fn test_large_duplicate_group() {
    let dir = tempdir().unwrap();

    // Create many duplicates
    let content = b"content for large group test";
    for i in 0..10 {
        fs::write(dir.path().join(format!("file{}.txt", i)), content).unwrap();
    }

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("10 files"));
}

#[test]
fn test_multiple_exclude_patterns() {
    let dir = tempdir().unwrap();

    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("b.txt"), "same").unwrap();
    fs::write(dir.path().join("a.tmp"), "same").unwrap();
    fs::write(dir.path().join("b.log"), "same").unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--exclude")
        .arg("*.tmp")
        .arg("--exclude")
        .arg("*.log")
        .assert()
        .success();
}

#[test]
fn test_nested_duplicates() {
    let dir = tempdir().unwrap();

    fs::create_dir_all(dir.path().join("level1/level2/level3")).unwrap();

    let content = b"deeply nested content";
    fs::write(dir.path().join("root.txt"), content).unwrap();
    fs::write(dir.path().join("level1/file.txt"), content).unwrap();
    fs::write(dir.path().join("level1/level2/file.txt"), content).unwrap();
    fs::write(dir.path().join("level1/level2/level3/file.txt"), content).unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("4 files"));
}

#[test]
fn test_binary_files() {
    let dir = tempdir().unwrap();

    // Binary content with various byte values
    let content: Vec<u8> = (0..256).map(|i| i as u8).collect();
    fs::write(dir.path().join("binary1.bin"), &content).unwrap();
    fs::write(dir.path().join("binary2.bin"), &content).unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Group 1"));
}

#[test]
fn test_unicode_filenames() {
    let dir = tempdir().unwrap();

    let content = b"unicode test content";
    fs::write(dir.path().join("日本語.txt"), content).unwrap();
    fs::write(dir.path().join("中文.txt"), content).unwrap();
    fs::write(dir.path().join("emoji_🎉.txt"), content).unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("3 files"));
}

#[test]
fn test_conflicting_options() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file.txt"), "content").unwrap();

    // quiet + verbose should fail
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--quiet")
        .arg("--verbose")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot use --quiet and --verbose"));
}

#[test]
fn test_size_range() {
    let dir = tempdir().unwrap();

    // Create files of different sizes
    fs::write(dir.path().join("small.txt"), "x").unwrap();  // 1 byte
    fs::write(dir.path().join("medium1.txt"), "x".repeat(500)).unwrap();
    fs::write(dir.path().join("medium2.txt"), "x".repeat(500)).unwrap();
    fs::write(dir.path().join("large.txt"), "x".repeat(10000)).unwrap();

    // Only medium files (100-1000 bytes)
    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--min-size")
        .arg("100")
        .arg("--max-size")
        .arg("1000")
        .assert()
        .success()
        .stdout(predicate::str::contains("medium"));
}

#[test]
fn test_invalid_size_format() {
    let dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--min-size")
        .arg("invalid")
        .assert()
        .failure();
}

#[test]
fn test_backup_requires_delete() {
    let dir = tempdir().unwrap();
    let backup = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("duperr").unwrap();
    cmd.arg(dir.path())
        .arg("--backup-dir")
        .arg(backup.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("--backup-dir requires --delete"));
}
