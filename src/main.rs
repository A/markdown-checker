//! The `mdcheck` binary: rule file + inputs in, findings out.
//!
//! stdout carries findings only — pretty lines or one JSON object; everything
//! else goes to stderr. Exit codes are the contract of specs/specs.md §1.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use mdcheck::finding::Finding;
use mdcheck::model::DocModel;
use mdcheck::rules::RuleSet;
use serde::Serialize;

const STDIN: &str = "<stdin>";

/// Deterministic structural checks over markdown documents.
#[derive(Parser, Debug)]
#[command(
    name = "mdcheck",
    version,
    about,
    after_help = "Exit codes:\n  0  all checks pass\n  1  findings emitted\n  2  usage, rule-file or selector error\n  3  internal error"
)]
struct Cli {
    /// Rule file (YAML)
    rules: PathBuf,

    /// Markdown files to check; `-` reads stdin
    paths: Vec<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value_t = Format::Pretty)]
    format: Format,

    /// Base directory for link resolution; defaults to each file's own directory
    #[arg(long)]
    root: Option<PathBuf>,

    /// Print nothing; report through the exit code only
    #[arg(long)]
    quiet: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Pretty,
    Json,
}

/// Failure classes behind exit codes 2 and 3.
enum Fail {
    Usage(String),
    Internal(String),
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(Fail::Usage(message)) => {
            eprintln!("mdcheck: {message}");
            ExitCode::from(2)
        }
        Err(Fail::Internal(message)) => {
            eprintln!("mdcheck: internal error: {message}");
            ExitCode::from(3)
        }
    }
}

fn run() -> Result<ExitCode, Fail> {
    let cli = Cli::parse();
    if cli.paths.is_empty() {
        return Err(Fail::Usage(
            "no input files: pass markdown paths, or `-` to read stdin".to_string(),
        ));
    }

    let source = std::fs::read_to_string(&cli.rules)
        .map_err(|e| Fail::Usage(format!("{}: {e}", cli.rules.display())))?;
    let rules = RuleSet::parse(&source).map_err(|e| Fail::Usage(e.to_string()))?;

    let mut files: Vec<String> = Vec::new();
    let mut findings: Vec<Finding> = Vec::new();
    let mut failed = 0;
    for path in &cli.paths {
        let input = read(path, cli.root.as_deref())?;
        let doc = DocModel::parse(&input.source);
        let found = mdcheck::assert::run_with_root(&rules, &doc, &input.name, &input.root);
        if !found.is_empty() {
            failed += 1;
        }
        files.push(input.name);
        findings.extend(found);
    }

    if !cli.quiet {
        let report = match cli.format {
            Format::Pretty => pretty(&files, &findings),
            Format::Json => json(&files, &findings, failed)?,
        };
        if !report.is_empty() {
            println!("{report}");
        }
    }

    Ok(if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

struct Input {
    name: String,
    source: String,
    root: PathBuf,
}

fn read(path: &Path, root: Option<&Path>) -> Result<Input, Fail> {
    if path.as_os_str() == "-" {
        let mut source = String::new();
        std::io::stdin()
            .read_to_string(&mut source)
            .map_err(|e| Fail::Internal(format!("stdin: {e}")))?;
        let root = match root {
            Some(root) => root.to_path_buf(),
            None => std::env::current_dir()
                .map_err(|e| Fail::Internal(format!("current directory: {e}")))?,
        };
        return Ok(Input {
            name: STDIN.to_string(),
            source,
            root,
        });
    }

    if !path.exists() {
        return Err(Fail::Usage(format!("{}: no such file", path.display())));
    }
    let source = std::fs::read_to_string(path)
        .map_err(|e| Fail::Internal(format!("{}: {e}", path.display())))?;
    let root = match root {
        Some(root) => root.to_path_buf(),
        None => own_directory(path),
    };
    Ok(Input {
        name: path.display().to_string(),
        source,
        root,
    })
}

fn own_directory(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// One line per finding, files in the order they were first checked.
fn pretty(files: &[String], findings: &[Finding]) -> String {
    let mut seen: Vec<&String> = Vec::new();
    for file in files {
        if !seen.contains(&file) {
            seen.push(file);
        }
    }
    seen.iter()
        .flat_map(|file| findings.iter().filter(move |f| &&f.file == file))
        .map(|f| format!("{}:{}  {}  {}", f.file, f.line, f.check, f.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Serialize)]
struct Report<'a> {
    files: &'a [String],
    findings: &'a [Finding],
    summary: Summary,
}

#[derive(Serialize)]
struct Summary {
    /// Files checked.
    checked: usize,
    /// Files that produced at least one finding.
    failed: usize,
}

fn json(files: &[String], findings: &[Finding], failed: usize) -> Result<String, Fail> {
    let report = Report {
        files,
        findings,
        summary: Summary {
            checked: files.len(),
            failed,
        },
    };
    serde_json::to_string_pretty(&report).map_err(|e| Fail::Internal(e.to_string()))
}
