//! mdcheck checking its own feature-index contract: rules/feature-index.yaml
//! against the repository fixture.

mod support;

use assert_cmd::Command;
use support::{scratch, write};

const GOOD: &str = include_str!("fixtures/feature-index-good.md");

fn mdcheck(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("mdcheck")
        .expect("binary built")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(args)
        .output()
        .expect("run mdcheck")
}

#[test]
fn dod_feature_index_fixture_passes_its_own_rule_file() {
    let output = mdcheck(&[
        "rules/feature-index.yaml",
        "tests/fixtures/feature-index-good.md",
    ]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"");
}

/// Drops the last cell of every table line — header, delimiter and body rows —
/// so the table still parses, one column short.
fn drop_last_column(markdown: &str) -> String {
    markdown
        .lines()
        .map(|line| {
            if !line.starts_with('|') {
                return line.to_string();
            }
            let mut cells: Vec<&str> = line.split('|').collect();
            cells.remove(cells.len() - 2);
            cells.join("|")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Dropping the `capabilities` column flips the same run to findings.
#[test]
fn dod_dropping_a_column_flips_the_dogfood_run_to_findings() {
    let dir = scratch("dogfood_broken_column");
    let broken = drop_last_column(GOOD);
    assert_ne!(broken, GOOD, "fixture table not found");
    let path = write(&dir, "feature-index.md", &broken);

    // The fixture's wiki-links still resolve against the fixture directory.
    let output = mdcheck(&[
        "rules/feature-index.yaml",
        &path.display().to_string(),
        "--root",
        "tests/fixtures",
    ]);

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert!(stdout.contains("column `capabilities` missing"), "{stdout}");
}
