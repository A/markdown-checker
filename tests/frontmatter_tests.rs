//! Task 1.3 DoD coverage. One test per checkbox (plus fixture smoke tests).

use mdcheck::frontmatter::{self, FrontmatterError};

const GOOD: &str = include_str!("fixtures/feature-index-good.md");
const BAD: &str = include_str!("fixtures/feature-index-bad.md");

/// DoD: `---\nconfirmed: no\n---\nbody` → map {confirmed: "no"}, offset 3.
#[test]
fn dod_leading_block_parses_to_map_and_offset_three() {
    let out = frontmatter::split("---\nconfirmed: no\n---\nbody\n");

    assert!(out.frontmatter.present);
    assert_eq!(out.frontmatter.get("confirmed"), Some("no"));
    assert_eq!(out.frontmatter.values.len(), 1);
    assert_eq!(out.line_offset, 3);
    assert_eq!(out.body, "body\n");
    assert!(out.error.is_none());
}

/// DoD: no frontmatter → empty map, offset 0.
#[test]
fn dod_no_frontmatter_yields_empty_map_and_zero_offset() {
    let src = "# Doc\n\ntext\n";
    let out = frontmatter::split(src);

    assert!(!out.frontmatter.present);
    assert!(out.frontmatter.values.is_empty());
    assert_eq!(out.line_offset, 0);
    assert_eq!(out.body, src);
    assert!(out.error.is_none());
}

/// DoD: unclosed `---` → error value carrying line 1, no panic.
#[test]
fn dod_unclosed_block_reports_error_at_line_one_without_panic() {
    let src = "---\nconfirmed: no\n# Doc\n";
    let out = frontmatter::split(src);

    assert_eq!(out.error, Some(FrontmatterError::Unclosed { line: 1 }));
    assert_eq!(out.line_offset, 0);
    assert_eq!(out.body, src);
    assert!(!out.frontmatter.present);
}

/// Values land in an ordered map: iteration order follows the source.
#[test]
fn keys_keep_source_order_and_carry_their_line() {
    let out = frontmatter::split("---\nconfirmed: no\ntype: feature-index\n---\n# Doc\n");

    let keys: Vec<&str> = out.frontmatter.values.keys().map(String::as_str).collect();
    assert_eq!(keys, ["confirmed", "type"]);
    assert_eq!(out.frontmatter.line_of("confirmed"), Some(2));
    assert_eq!(out.frontmatter.line_of("type"), Some(3));
    assert_eq!(out.frontmatter.start_line, 1);
    assert_eq!(out.frontmatter.end_line, 4);
}

#[test]
fn good_fixture_frontmatter_parses() {
    let out = frontmatter::split(GOOD);

    assert_eq!(out.frontmatter.get("confirmed"), Some("no"));
    assert_eq!(out.frontmatter.get("type"), Some("feature-index"));
    assert_eq!(out.line_offset, 4);
    assert!(out.body.starts_with("# Search feature index"));
}

#[test]
fn bad_fixture_frontmatter_parses() {
    let out = frontmatter::split(BAD);

    assert_eq!(out.frontmatter.get("confirmed"), Some("maybe"));
    assert_eq!(out.frontmatter.get("draft"), Some("true"));
    assert_eq!(out.frontmatter.line_of("draft"), Some(3));
    assert_eq!(out.line_offset, 4);
}
