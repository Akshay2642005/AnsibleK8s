//! e01s02 t3/t4: the convert → validate CLI workflow against the oracle.

use std::fs;
use std::path::{Path, PathBuf};

use ansiblek8s_rs::cli;
use ansiblek8s_rs::config::Role;

const SAMPLE_DIR: &str = "reference/inventory/sample";

fn tmp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

/// Recursively hash every file under `dir` (stdlib only) so any write into
/// the oracle changes the value (story §17).
fn snapshot(dir: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut files: Vec<PathBuf> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).expect("oracle readable") {
            let path = entry.expect("oracle entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    let mut hasher = DefaultHasher::new();
    for file in &files {
        file.hash(&mut hasher);
        fs::read(file).expect("oracle file readable").hash(&mut hasher);
    }
    hasher.finish()
}

#[test]
fn convert_then_validate_workflow_writes_valid_yaml_and_leaves_oracle_untouched() {
    let before = snapshot(Path::new("reference/inventory"));
    let out = tmp_file("ansiblek8s_rs_cli_out.yaml");
    let _ = fs::remove_file(&out);

    // convert → exit 0 (t3)
    let code = cli::run(&["convert", SAMPLE_DIR, "-o", out.to_str().unwrap()]);
    assert_eq!(code, 0, "convert must succeed on the oracle sample");

    // t4: 3 servers + 2 agents, and the written file passes the REAL loader (§6)
    let cfg = ansiblek8s_rs::config::load(&out).expect("convert output must pass config::load");
    assert_eq!(
        cfg.hosts.iter().filter(|h| h.role == Role::Server).count(),
        3,
        "converted file carries 3 servers"
    );
    assert_eq!(
        cfg.hosts.iter().filter(|h| h.role == Role::Agent).count(),
        2,
        "converted file carries 2 agents"
    );

    // validate accepts the converted file...
    assert_eq!(
        cli::run(&["validate", out.to_str().unwrap()]),
        0,
        "validate accepts converted output"
    );
    // ...and rejects broken input
    let bad = tmp_file("ansiblek8s_rs_cli_bad.yaml");
    fs::write(&bad, "name: [broken\n").expect("tmp write");
    assert_ne!(
        cli::run(&["validate", bad.to_str().unwrap()]),
        0,
        "validate must reject invalid input"
    );

    // §17: conversion never touches reference/
    let after = snapshot(Path::new("reference/inventory"));
    assert_eq!(before, after, "conversion must never modify reference/");

    let _ = fs::remove_file(out);
    let _ = fs::remove_file(bad);
}
