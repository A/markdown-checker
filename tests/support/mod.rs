//! Shared helpers for the rule-file / assertion test suites.

// Each test binary compiles the module and uses part of it.
#![allow(dead_code)]

use mdcheck::finding::Finding;
use mdcheck::model::DocModel;
use mdcheck::rules::RuleSet;

/// Wraps a bare rule list — the form scenarios.md uses — into a full rule file.
pub fn rule_file(rules: &str) -> String {
    if rules.trim_start().starts_with("doc:") {
        return rules.to_string();
    }
    let body: String = rules
        .lines()
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("  {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("doc: test\nrules:\n{body}\n")
}

pub fn rule_set(rules: &str) -> RuleSet {
    RuleSet::parse(&rule_file(rules)).unwrap_or_else(|e| panic!("{e}"))
}

/// Runs a bare rule list against markdown source.
pub fn check(rules: &str, markdown: &str) -> Vec<Finding> {
    let doc = DocModel::parse(markdown);
    mdcheck::run(&rule_set(rules), &doc, "test.md")
}

pub fn lines(findings: &[Finding]) -> Vec<usize> {
    findings.iter().map(|f| f.line).collect()
}

/// `line — message` pairs, the shape scenarios.md states expectations in.
pub fn reported(findings: &[Finding]) -> Vec<String> {
    findings
        .iter()
        .map(|f| format!("{} — {}", f.line, f.message))
        .collect()
}
