//! `ansiblek8s-rs` command-line surface (migration half of ADR-002).

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use crate::{config, convert};

#[derive(Parser)]
#[command(
    name = "ansiblek8s-rs",
    about = "HA k3s cluster orchestrator — the Rust successor to AnsibleK8s"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert a legacy Ansible inventory directory into cluster.yaml
    Convert {
        /// Legacy inventory directory (hosts.ini + group_vars/)
        dir: PathBuf,
        /// Output path for the generated cluster.yaml
        #[arg(short = 'o', long = "output")]
        output: PathBuf,
    },
    /// Validate a cluster.yaml file
    Validate {
        /// Path to cluster.yaml
        file: PathBuf,
    },
}

/// Run the CLI with `argv` **excluding** the program name and return the
/// process exit code: 0 success, 2 usage error, 1 command failure.
///
/// Warnings go to stderr (story §10), never stdout.
pub fn run(args: &[&str]) -> i32 {
    let mut argv: Vec<&str> = vec!["ansiblek8s-rs"];
    argv.extend_from_slice(args);
    let cli = match Cli::try_parse_from(&argv) {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return if err.use_stderr() { 2 } else { 0 };
        }
    };
    match cli.command {
        Command::Convert { dir, output } => convert_cmd(&dir, &output),
        Command::Validate { file } => validate_cmd(&file),
    }
}

fn convert_cmd(dir: &Path, output: &Path) -> i32 {
    let (cfg, warnings) = match convert::inventory(dir) {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("error: {err}");
            return 1;
        }
    };
    for warning in &warnings {
        eprintln!("warning: {}: {}", warning.key, warning.reason);
    }
    let yaml = match serde_yml::to_string(&cfg) {
        Ok(yaml) => yaml,
        Err(err) => {
            eprintln!("error: cannot serialize cluster config: {err}");
            return 1;
        }
    };
    // §6: never write a file the real loader would reject — round-trip first.
    if let Err(err) = config::load_from_str(&yaml) {
        eprintln!("error: conversion produced an invalid cluster.yaml: {err}");
        return 1;
    }
    if let Err(err) = std::fs::write(output, yaml) {
        eprintln!("error: cannot write {}: {err}", output.display());
        return 1;
    }
    0
}

fn validate_cmd(file: &Path) -> i32 {
    match config::load(file) {
        Ok(_) => {
            println!("{}: OK", file.display());
            0
        }
        Err(err) => {
            eprintln!("{}: {err}", file.display());
            1
        }
    }
}
