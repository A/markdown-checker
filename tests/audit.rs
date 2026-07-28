//! Corpus audit: every acceptance scenario of specs/scenarios.md must exist as a
//! test.
//!
//! The convention is mechanical — a scenario id becomes a test-name token by
//! lowercasing it and turning `-` into `_`, so `TBL-3` is covered by any test
//! whose name contains `tbl_3`. This audit parses the spec at test time, so a
//! scenario added there fails the build until a test carries its id.

use std::path::{Path, PathBuf};

const SCENARIOS: &str = include_str!("../specs/scenarios.md");

/// Scenario ids: the `### FM-1 …` headings of specs/scenarios.md.
fn scenario_ids(markdown: &str) -> Vec<String> {
    markdown
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .filter_map(|heading| heading.split_whitespace().next())
        .filter(|id| is_scenario_id(id))
        .map(str::to_string)
        .collect()
}

/// `FAMILY-N`, family in capitals.
fn is_scenario_id(id: &str) -> bool {
    let Some((family, number)) = id.split_once('-') else {
        return false;
    };
    !family.is_empty()
        && family.chars().all(|c| c.is_ascii_uppercase())
        && !number.is_empty()
        && number.chars().all(|c| c.is_ascii_digit())
}

fn token(id: &str) -> String {
    id.to_lowercase().replace('-', "_")
}

fn missing(ids: &[String], names: &[String]) -> Vec<String> {
    ids.iter()
        .filter(|id| !names.iter().any(|name| name.contains(&token(id))))
        .cloned()
        .collect()
}

/// Every `fn name(` in the integration-test sources.
fn test_names(dir: &Path) -> Vec<String> {
    let mut names = Vec::new();
    for source in sources(dir) {
        let text = std::fs::read_to_string(&source).expect("read test source");
        for line in text.lines() {
            let Some(rest) = line.trim_start().strip_prefix("fn ") else {
                continue;
            };
            let Some(name) = rest.split('(').next() else {
                continue;
            };
            names.push(name.trim().to_string());
        }
    }
    names
}

fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("read tests dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            out.extend(sources(&path));
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    out
}

#[test]
fn dod_every_scenario_id_has_a_test() {
    let ids = scenario_ids(SCENARIOS);
    let names = test_names(&Path::new(env!("CARGO_MANIFEST_DIR")).join("tests"));

    assert!(ids.len() >= 30, "scenario parsing found only {:?}", ids);
    let gaps = missing(&ids, &names);
    assert!(
        gaps.is_empty(),
        "scenarios without a test (name must contain the id, `-` as `_`): {gaps:?}"
    );
}

#[test]
fn dod_every_scenario_family_is_represented() {
    let ids = scenario_ids(SCENARIOS);
    for family in [
        "FM", "DOC", "SEC", "SEL", "LST", "LEN", "SUB", "TBL", "LNK", "LOAD", "CLI",
    ] {
        assert!(
            ids.iter().any(|id| id.starts_with(&format!("{family}-"))),
            "family {family} missing from the parsed corpus: {ids:?}"
        );
    }
}

/// The audit's own red path: an id nothing is named after is reported.
#[test]
fn dod_audit_reports_an_id_without_a_test() {
    let names = vec!["fm_1_required_key".to_string()];
    assert_eq!(
        missing(&["FM-1".to_string(), "ZZZ-9".to_string()], &names),
        ["ZZZ-9"]
    );
}
