//! Process-level contract of the `mdcheck` binary: input handling, output
//! formats, stream routing and exit codes. Covers the CLI- scenarios of
//! specs/scenarios.md.

mod support;

use std::path::Path;
use std::process::Output;

use assert_cmd::Command;
use serde_json::Value;
use support::{scratch, write};

const RULES: &str = "doc: t\nrules:\n  - select: H1\n    count: 1\n";
const GOOD: &str = "# One\n";
const BAD: &str = "# One\n\n# Two\n";

fn mdcheck() -> Command {
    Command::cargo_bin("mdcheck").expect("binary built")
}

fn run(dir: &Path, args: &[&str]) -> Output {
    mdcheck()
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run mdcheck")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf-8 stderr")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("exit code")
}

#[test]
fn cli_1_stdin() {
    let dir = scratch("cli_1_stdin");
    write(&dir, "rules.yaml", RULES);

    let output = mdcheck()
        .current_dir(&dir)
        .args(["rules.yaml", "-", "--format", "json"])
        .write_stdin(BAD)
        .output()
        .expect("run mdcheck");

    assert_eq!(code(&output), 1, "{}", stderr(&output));
    let json: Value = serde_json::from_str(&stdout(&output)).expect("json object");
    assert_eq!(json["files"], serde_json::json!(["<stdin>"]));
    assert_eq!(json["findings"][0]["file"], "<stdin>");
}

#[test]
fn cli_1_stdin_pretty_labels_findings_as_stdin() {
    let dir = scratch("cli_1_stdin_pretty");
    write(&dir, "rules.yaml", RULES);

    let output = mdcheck()
        .current_dir(&dir)
        .args(["rules.yaml", "-"])
        .write_stdin(BAD)
        .output()
        .expect("run mdcheck");

    assert_eq!(code(&output), 1);
    assert!(
        stdout(&output).starts_with("<stdin>:1  "),
        "{}",
        stdout(&output)
    );
}

#[test]
fn cli_2_multiple_files() {
    let dir = scratch("cli_2_multiple_files");
    write(&dir, "rules.yaml", RULES);
    write(&dir, "a.md", GOOD);
    write(&dir, "b.md", BAD);

    let output = run(&dir, &["rules.yaml", "a.md", "b.md"]);
    assert_eq!(code(&output), 1);
    let printed = stdout(&output);
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines[0].starts_with("b.md:"), "{lines:?}");

    let clean = run(&dir, &["rules.yaml", "a.md", "a.md"]);
    assert_eq!(code(&clean), 0);
    assert_eq!(stdout(&clean), "");
}

#[test]
fn cli_2_pretty_groups_findings_by_file() {
    let dir = scratch("cli_2_grouping");
    write(&dir, "rules.yaml", RULES);
    write(&dir, "a.md", BAD);
    write(&dir, "b.md", BAD);

    let output = run(&dir, &["rules.yaml", "a.md", "b.md", "a.md"]);
    assert_eq!(code(&output), 1);
    let files: Vec<String> = stdout(&output)
        .lines()
        .map(|line| line.split(':').next().unwrap().to_string())
        .collect();
    assert_eq!(files, ["a.md", "a.md", "b.md"]);

    let first = stdout(&output).lines().next().unwrap().to_string();
    assert_eq!(
        first,
        "a.md:1  t.rule[0]  `H1`: 2 matches, expected exactly 1"
    );
}

#[test]
fn cli_3_json_format() {
    let dir = scratch("cli_3_json");
    write(&dir, "rules.yaml", RULES);
    write(&dir, "a.md", GOOD);
    write(&dir, "b.md", BAD);

    let output = run(&dir, &["rules.yaml", "a.md", "b.md", "--format", "json"]);
    assert_eq!(code(&output), 1);
    assert_eq!(stderr(&output), "");

    let json: Value = serde_json::from_str(&stdout(&output)).expect("single json object");
    assert_eq!(json["files"], serde_json::json!(["a.md", "b.md"]));
    assert_eq!(
        json["summary"],
        serde_json::json!({"checked": 2, "failed": 1})
    );

    let finding = &json["findings"][0];
    let keys: Vec<&str> = finding
        .as_object()
        .expect("finding object")
        .keys()
        .map(String::as_str)
        .collect();
    // serde_json sorts object keys when parsing back into a `Value`.
    assert_eq!(
        keys,
        ["actual", "check", "expected", "file", "line", "message", "severity"]
    );
    assert_eq!(finding["file"], "b.md");
    assert_eq!(finding["severity"], "error");
    assert_eq!(finding["line"], 1);
}

#[test]
fn cli_3_json_on_a_clean_run_is_still_a_full_object() {
    let dir = scratch("cli_3_json_clean");
    write(&dir, "rules.yaml", RULES);
    write(&dir, "a.md", GOOD);

    let output = run(&dir, &["rules.yaml", "a.md", "--format", "json"]);
    assert_eq!(code(&output), 0);
    let json: Value = serde_json::from_str(&stdout(&output)).expect("json object");
    assert_eq!(json["findings"], serde_json::json!([]));
    assert_eq!(
        json["summary"],
        serde_json::json!({"checked": 1, "failed": 0})
    );
}

#[test]
fn cli_4_bad_rule_file() {
    let dir = scratch("cli_4_bad_rules");
    write(
        &dir,
        "rules.yaml",
        "doc: t\nrules:\n  - select: H1\n    nonsense: 1\n",
    );
    write(&dir, "a.md", BAD);

    let output = run(&dir, &["rules.yaml", "a.md"]);
    assert_eq!(code(&output), 2);
    assert_eq!(stdout(&output), "");
    let message = stderr(&output);
    assert!(message.contains("nonsense"), "{message}");
    assert!(message.contains("rule 0"), "{message}");
}

#[test]
fn dod_selector_error_in_the_rule_file_exits_2() {
    let dir = scratch("cli_selector_error");
    write(&dir, "rules.yaml", "doc: t\nrules:\n  - select: 'H1 >'\n");
    write(&dir, "a.md", GOOD);

    let output = run(&dir, &["rules.yaml", "a.md"]);
    assert_eq!(code(&output), 2);
    assert_eq!(stdout(&output), "");
    assert!(!stderr(&output).is_empty());
}

#[test]
fn dod_missing_rule_file_exits_2() {
    let dir = scratch("cli_missing_rules");
    write(&dir, "a.md", GOOD);

    let output = run(&dir, &["nope.yaml", "a.md"]);
    assert_eq!(code(&output), 2);
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("nope.yaml"), "{}", stderr(&output));
}

#[test]
fn dod_missing_input_file_exits_2() {
    let dir = scratch("cli_missing_input");
    write(&dir, "rules.yaml", RULES);

    let output = run(&dir, &["rules.yaml", "nope.md"]);
    assert_eq!(code(&output), 2);
    assert_eq!(stdout(&output), "");
    assert!(stderr(&output).contains("nope.md"), "{}", stderr(&output));
}

#[test]
fn dod_no_input_paths_is_a_usage_error() {
    let dir = scratch("cli_no_paths");
    write(&dir, "rules.yaml", RULES);

    let output = run(&dir, &["rules.yaml"]);
    assert_eq!(code(&output), 2);
    assert_eq!(stdout(&output), "");
    assert!(!stderr(&output).is_empty());
}

#[test]
fn dod_unreadable_input_is_an_internal_error() {
    let dir = scratch("cli_internal");
    write(&dir, "rules.yaml", RULES);
    std::fs::write(dir.join("bytes.md"), [0xff, 0xfe, 0x00]).expect("write bytes");

    let output = run(&dir, &["rules.yaml", "bytes.md"]);
    assert_eq!(code(&output), 3);
    assert_eq!(stdout(&output), "");
    assert!(!stderr(&output).is_empty());
}

#[test]
fn dod_unknown_format_is_a_usage_error() {
    let dir = scratch("cli_bad_format");
    write(&dir, "rules.yaml", RULES);
    write(&dir, "a.md", GOOD);

    let output = run(&dir, &["rules.yaml", "a.md", "--format", "yaml"]);
    assert_eq!(code(&output), 2);
    assert_eq!(stdout(&output), "");
}

#[test]
fn dod_quiet_prints_nothing_and_keeps_the_exit_code() {
    let dir = scratch("cli_quiet");
    write(&dir, "rules.yaml", RULES);
    write(&dir, "a.md", BAD);
    write(&dir, "b.md", GOOD);

    let failing = run(&dir, &["rules.yaml", "a.md", "--quiet"]);
    assert_eq!(code(&failing), 1);
    assert_eq!(stdout(&failing), "");

    let clean = run(&dir, &["rules.yaml", "b.md", "--quiet"]);
    assert_eq!(code(&clean), 0);
    assert_eq!(stdout(&clean), "");

    let json = run(&dir, &["rules.yaml", "a.md", "--quiet", "--format", "json"]);
    assert_eq!(code(&json), 1);
    assert_eq!(stdout(&json), "");
}

#[test]
fn dod_help_documents_flags_and_exit_codes() {
    let dir = scratch("cli_help");
    let output = run(&dir, &["--help"]);
    assert_eq!(code(&output), 0);
    let help = stdout(&output);
    for expected in [
        "--format",
        "--root",
        "--quiet",
        "pretty",
        "json",
        "Exit codes",
        "0  all checks pass",
        "1  findings emitted",
        "2  usage",
        "3  internal error",
    ] {
        assert!(
            help.contains(expected),
            "`{expected}` missing from:\n{help}"
        );
    }
}
