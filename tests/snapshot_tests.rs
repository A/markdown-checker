//! Full JSON output of the binary over one deliberately broken fixture per
//! check family, frozen as snapshots.

use assert_cmd::Command;

/// `mdcheck <rules> <doc> --format json` from the crate root, so the file names
/// inside the snapshot are the relative paths of the fixtures.
fn json(family: &str) -> String {
    let output = Command::cargo_bin("mdcheck")
        .expect("binary built")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            &format!("tests/fixtures/bad/{family}.yaml"),
            &format!("tests/fixtures/bad/{family}.md"),
            "--format",
            "json",
        ])
        .output()
        .expect("run mdcheck");

    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf-8 stdout")
}

/// Frontmatter, document, headings, sections, lists, lengths.
#[test]
fn scalar_family_findings() {
    insta::assert_snapshot!(json("scalar"));
}

/// Tables and nested `each:` rules.
#[test]
fn structural_family_findings() {
    insta::assert_snapshot!(json("structural"));
}

/// `wiki:` and `resolve:`.
#[test]
fn links_family_findings() {
    insta::assert_snapshot!(json("links"));
}
