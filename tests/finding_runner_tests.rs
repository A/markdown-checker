//! Task 3.2 DoD coverage: finding model and the all-rules runner.

mod support;

use mdcheck::finding::{Finding, Severity};
use mdcheck::model::DocModel;
use support::{check, reported, rule_set};

/// DoD: three violations of three different rules produce three findings in one run.
#[test]
fn dod_three_violated_rules_yield_three_findings_in_one_run() {
    let findings = check(
        r#"- select: H1
  count: 1
- select: H2
  max_length: 5
- select: UL > LI
  min: 3"#,
        "# One\n\n# Two\n\n## A far too long header\n\n- only one\n",
    );

    assert_eq!(
        findings
            .iter()
            .map(|f| f.check.as_str())
            .collect::<Vec<_>>(),
        ["test.rule[0]", "test.rule[1]", "test.rule[2]"]
    );
    assert_eq!(reported(&findings).len(), 3);
}

/// DoD: a rule-level failure with no node is reported at the document start with
/// `actual: "0 matches"`.
#[test]
fn dod_rule_level_failure_without_a_node_reports_document_start() {
    let findings = check("- select: H1\n  count: 1", "no headers here\n");

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 1);
    assert_eq!(findings[0].actual, "0 matches");
    assert_eq!(findings[0].expected, "1 match");
}

/// DoD: findings carry the matched node's line.
#[test]
fn finding_line_is_the_matched_nodes_line() {
    let findings = check("- select: H2\n  max_length: 3", "# T\n\n## Long header\n");

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 3);
}

/// DoD: `optional: true` with zero matches emits nothing.
#[test]
fn dod_optional_rule_with_zero_matches_is_skipped() {
    let findings = check(
        "- select: SECTION(\"Nope\") > P[0]\n  optional: true\n  starts_with: 'pre:'",
        "# T\n## Story\n### Sub\n",
    );
    assert!(findings.is_empty(), "{:?}", reported(&findings));

    let required = check(
        "- select: SECTION(\"Nope\") > P[0]\n  count: 1",
        "# T\n## Story\n### Sub\n",
    );
    assert_eq!(required.len(), 1);
}

/// An optional rule that does match is still asserted on.
#[test]
fn optional_rule_still_asserts_when_it_matches() {
    let findings = check(
        "- select: SECTION(\"Story\") > P[0]\n  optional: true\n  starts_with: 'pre:'",
        "# T\n## Story\nnot a pre line\n",
    );
    assert_eq!(findings.len(), 1);
}

/// The runner applies every rule — no first-fail short circuit.
#[test]
fn runner_never_stops_at_the_first_failing_rule() {
    let findings = check(
        "- select: H1\n  count: 5\n- select: H1\n  max_length: 1\n- select: H1\n  starts_with: 'x'",
        "# Title\n",
    );
    assert_eq!(findings.len(), 3);
}

/// The caller sets `file`; severity is `error` in v1.
#[test]
fn findings_carry_the_callers_file_name_and_error_severity() {
    let doc = DocModel::parse("# T\n");
    let findings = mdcheck::run(&rule_set("- select: H2\n  count: 1"), &doc, "<stdin>");

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "<stdin>");
    assert_eq!(findings[0].severity, Severity::Error);
}

/// Findings are serde-serializable with the uniform shape.
#[test]
fn findings_serialize_to_the_uniform_json_shape() {
    let doc = DocModel::parse("# T\n");
    let findings = mdcheck::run(&rule_set("- select: H2\n  count: 1"), &doc, "doc.md");
    let json = serde_json::to_value(&findings[0]).unwrap();

    assert_eq!(json["check"], "test.rule[0]");
    assert_eq!(json["severity"], "error");
    assert_eq!(json["file"], "doc.md");
    assert_eq!(json["line"], 1);
    assert_eq!(json["expected"], "1 match");
    assert_eq!(json["actual"], "0 matches");
    assert!(json["message"].is_string());
    assert_eq!(json.as_object().unwrap().len(), 7);
}

/// Load-stage findings are emitted alongside rule findings and never abort the run.
#[test]
fn load_findings_are_emitted_before_rule_findings() {
    let findings = check("- select: H6\n  count: 1", "---\nkey: v\n# Doc\n");
    assert_eq!(findings[0].check, "load");
}

#[test]
fn finding_is_constructible_by_callers() {
    let finding = Finding::new("c", "f.md", 3, "a", "b", "m");
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(finding.line, 3);
}
