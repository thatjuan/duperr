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
