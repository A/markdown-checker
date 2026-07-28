//! Milestone 4 verify: the whole rule file against the M1 fixtures.

use mdcheck::finding::Finding;
use mdcheck::model::DocModel;
use mdcheck::rules::RuleSet;

const RULES: &str = include_str!("fixtures/feature-index.yaml");
const GOOD: &str = include_str!("fixtures/feature-index-good.md");
const BAD: &str = include_str!("fixtures/feature-index-bad.md");

fn run(markdown: &str, file: &str) -> Vec<Finding> {
    let set = RuleSet::parse(RULES).unwrap_or_else(|e| panic!("{e}"));
    mdcheck::run(&set, &DocModel::parse(markdown), file)
}

fn render(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|f| format!("{} {}:{} — {}", f.check, f.file, f.line, f.message))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn good_fixture_produces_no_findings() {
    let findings = run(GOOD, "feature-index-good.md");
    assert!(findings.is_empty(), "{}", render(&findings));
}

#[test]
fn bad_fixture_produces_the_full_findings_set() {
    let findings = run(BAD, "feature-index-bad.md");
    insta::assert_snapshot!(render(&findings));
}
