//! Task 3.3 DoD coverage: one good-pass / bad-fail pair per scalar assertion, with
//! the finding message, `expected` and `actual` asserted.

mod support;

use support::{check, reported};

const DOC: &str = "# Title\n\n## Roles\n\n- **Admin** — manages\n- broken item\n";

#[test]
fn count_passes_and_fails_with_match_counts() {
    assert!(check("- select: H1\n  count: 1", DOC).is_empty());

    let findings = check("- select: H2\n  count: 2", DOC);
    assert_eq!(
        reported(&findings),
        ["1 — `H2`: 1 match, expected exactly 2"]
    );
    assert_eq!(findings[0].expected, "2 matches");
    assert_eq!(findings[0].actual, "1 match");
}

#[test]
fn min_passes_and_fails_with_match_counts() {
    assert!(check("- select: UL > LI\n  min: 2", DOC).is_empty());

    let findings = check("- select: UL > LI\n  min: 3", DOC);
    assert_eq!(
        reported(&findings),
        ["1 — `UL > LI`: 2 matches, expected ≥ 3"]
    );
    assert_eq!(findings[0].expected, "≥ 3 matches");
    assert_eq!(findings[0].actual, "2 matches");
}

#[test]
fn max_passes_and_fails_with_match_counts() {
    assert!(check("- select: UL > LI\n  max: 2", DOC).is_empty());

    let findings = check("- select: UL > LI\n  max: 1", DOC);
    assert_eq!(
        reported(&findings),
        ["1 — `UL > LI`: 2 matches, expected ≤ 1"]
    );
    assert_eq!(findings[0].expected, "≤ 1 match");
    assert_eq!(findings[0].actual, "2 matches");
}

#[test]
fn min_lines_passes_and_fails_on_the_document() {
    assert!(check("- select: DOCUMENT\n  min_lines: 6", DOC).is_empty());

    let findings = check("- select: DOCUMENT\n  min_lines: 20", DOC);
    assert_eq!(
        reported(&findings),
        ["1 — document has 6 lines, expected ≥ 20"]
    );
    assert_eq!(findings[0].expected, "≥ 20 lines");
    assert_eq!(findings[0].actual, "6 lines");
}

/// `match`: the whole value must match.
#[test]
fn match_passes_and_fails_against_the_whole_value() {
    assert!(check("- select: H1\n  match: '^Title$'", DOC).is_empty());

    let findings = check("- select: H1\n  match: 'Titl'", DOC);
    assert_eq!(reported(&findings), ["1 — `Title` does not match `Titl`"]);
    assert_eq!(findings[0].expected, "matches `Titl`");
    assert_eq!(findings[0].actual, "Title");
}

/// `require`: the pattern must appear somewhere in the value.
#[test]
fn require_passes_and_fails_on_appearance() {
    assert!(check("- select: H1\n  require: 'itl'", DOC).is_empty());

    let findings = check("- select: H1\n  require: 'Nope'", DOC);
    assert_eq!(
        reported(&findings),
        ["1 — required pattern `Nope` not found"]
    );
    assert_eq!(findings[0].expected, "contains a match for `Nope`");
    assert_eq!(findings[0].actual, "Title");
}

/// `forbid`: the pattern must not appear.
#[test]
fn forbid_passes_and_fails_on_appearance() {
    assert!(check("- select: H1\n  forbid: 'TBD'", DOC).is_empty());

    let findings = check("- select: H1\n  forbid: 'Tit'", DOC);
    assert_eq!(reported(&findings), ["1 — forbidden pattern `Tit` found"]);
    assert_eq!(findings[0].expected, "no match for `Tit`");
    assert_eq!(findings[0].actual, "Title");
}

#[test]
fn starts_with_passes_and_fails() {
    assert!(check("- select: H1\n  starts_with: 'Ti'", DOC).is_empty());

    let findings = check("- select: H1\n  starts_with: 'pre:'", DOC);
    assert_eq!(
        reported(&findings),
        ["1 — `Title` does not start with `pre:`"]
    );
    assert_eq!(findings[0].expected, "starts with `pre:`");
    assert_eq!(findings[0].actual, "Title");
}

#[test]
fn ends_with_passes_and_fails() {
    assert!(check("- select: H1\n  ends_with: 'le'", DOC).is_empty());

    let findings = check("- select: H1\n  ends_with: '?'", DOC);
    assert_eq!(reported(&findings), ["1 — `Title` does not end with `?`"]);
    assert_eq!(findings[0].expected, "ends with `?`");
    assert_eq!(findings[0].actual, "Title");
}

#[test]
fn min_length_passes_and_fails() {
    assert!(check("- select: H1\n  min_length: 5", DOC).is_empty());

    let findings = check("- select: H1\n  min_length: 10", DOC);
    assert_eq!(reported(&findings), ["1 — length 5, expected ≥ 10"]);
    assert_eq!(findings[0].expected, "≥ 10 characters");
    assert_eq!(findings[0].actual, "5 characters");
}

#[test]
fn max_length_passes_and_fails() {
    assert!(check("- select: H1\n  max_length: 5", DOC).is_empty());

    let findings = check("- select: H1\n  max_length: 4", DOC);
    assert_eq!(reported(&findings), ["1 — length 5, expected ≤ 4"]);
    assert_eq!(findings[0].expected, "≤ 4 characters");
    assert_eq!(findings[0].actual, "5 characters");
}

/// DoD: the `enum` message lists the allowed values and the offending value verbatim.
#[test]
fn enum_passes_and_fails_listing_allowed_values() {
    let table = "| priority |\n|---|\n| p0 |\n| p9 |\n";
    assert!(check(
        "- select: TD[col=\"priority\"]\n  enum: [p0, p1, p9]",
        table
    )
    .is_empty());

    let findings = check("- select: TD[col=\"priority\"]\n  enum: [p0, p1]", table);
    assert_eq!(
        reported(&findings),
        ["4 — `p9` is not allowed, allowed: `p0`, `p1`"]
    );
    assert_eq!(findings[0].expected, "one of `p0`, `p1`");
    assert_eq!(findings[0].actual, "p9");
}

#[test]
fn no_empty_passes_and_fails() {
    assert!(check("- select: UL > LI\n  no_empty: true", "- one\n- two\n").is_empty());

    let findings = check(
        "- select: TD[col=\"feature\"]\n  no_empty: true",
        "| feature |\n|---|\n|  |\n",
    );
    assert_eq!(reported(&findings), ["3 — empty cell in column `feature`"]);
    assert_eq!(findings[0].expected, "a non-empty value");
    assert_eq!(findings[0].actual, "");
}

/// Text regexps anchor per line: the document model merges consecutive source
/// lines into one paragraph, so `^`/`$` would otherwise be unusable.
#[test]
fn text_regexps_are_multi_line() {
    let doc = "# T\n\npre: user exists\nAs **Visitor**\n";
    assert!(check("- select: P\n  require: '^As \\*\\*.+\\*\\*'", doc).is_empty());
    assert!(check("- select: P[text=/^As \\*\\*.+\\*\\*/]\n  count: 1", doc).is_empty());
}

/// Every match is reported, not just the first.
#[test]
fn per_match_assertions_report_every_offender() {
    let findings = check(
        "- select: UL > LI\n  starts_with: '**'",
        "- **a** ok\n- bad\n- worse\n",
    );
    assert_eq!(support::lines(&findings), [2, 3]);
}

/// Long values are truncated in messages but the assertion still sees the whole text.
#[test]
fn long_values_are_truncated_in_messages() {
    let long = "y".repeat(90);
    let findings = check("- select: P\n  starts_with: 'x'", &format!("{long}\n"));
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains('…'), "{}", findings[0].message);
    assert_eq!(findings[0].actual, long);
}
