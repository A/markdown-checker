//! The acceptance corpus of specs/scenarios.md, one test per scenario id, for the
//! families owned by milestones 3 and 4: FM-, DOC-, SEC-, LST-, LEN-, SUB-, TBL-,
//! LOAD-. (SEL- lives in the selector suite, CLI-/LNK- in later milestones.)

mod support;

use support::{check, reported};

const FM_RULES: &str = r#"- select: FRONTMATTER
  has: { confirmed: { enum: [yes, no] } }"#;

#[test]
fn fm_1_required_key_with_enum_valid() {
    let findings = check(FM_RULES, "---\nconfirmed: no\n---\n# Doc\n");
    assert!(findings.is_empty(), "{:?}", reported(&findings));
}

#[test]
fn fm_2_enum_violation() {
    let findings = check(FM_RULES, "---\nconfirmed: maybe\n---\n# Doc\n");
    assert_eq!(
        reported(&findings),
        ["2 — `confirmed` is `maybe`, allowed: `yes`, `no`"]
    );
}

#[test]
fn fm_3_missing_frontmatter() {
    let findings = check(FM_RULES, "# Doc\n");
    assert_eq!(reported(&findings), ["1 — frontmatter block missing"]);
}

#[test]
fn fm_4_forbidden_key_present() {
    let findings = check(
        "- select: FRONTMATTER\n  not_has: [draft]",
        "---\nconfirmed: no\ndraft: true\n---\n",
    );
    assert_eq!(reported(&findings), ["3 — key `draft` must not be present"]);
}

#[test]
fn fm_5_key_value_by_regexp() {
    let findings = check(
        "- select: FRONTMATTER\n  has: { priority: '^P[0-3]$' }",
        "---\npriority: P7\n---\n# Doc\n",
    );
    assert_eq!(
        reported(&findings),
        ["2 — `priority` value `P7` does not match `^P[0-3]$`"]
    );
}

#[test]
fn doc_1_min_lines() {
    let findings = check("- select: DOCUMENT\n  min_lines: 20", "# T\n\nbody\n");
    assert_eq!(
        reported(&findings),
        ["1 — document has 3 lines, expected ≥ 20"]
    );
}

const SEC_RULES: &str = "- select: H1 > H2\n  ordered: [Roles, Features]";
const SEC_STRICT_RULES: &str =
    "- select: H1 > H2\n  ordered: [Roles, Features, Questions?]\n  strict: true";

#[test]
fn sec_1_required_sections_present_in_order() {
    let findings = check(SEC_RULES, "# T\n## Roles\nx\n## Features\ny\n");
    assert!(findings.is_empty(), "{:?}", reported(&findings));
}

#[test]
fn sec_2_order_swapped() {
    let findings = check(SEC_RULES, "# T\n## Features\ny\n## Roles\nx\n");
    assert_eq!(
        reported(&findings),
        ["4 — section order: `Roles` appears after `Features`, expected before"]
    );
}

#[test]
fn sec_3_missing_required_section() {
    let findings = check(SEC_RULES, "# T\n## Roles\nx\n");
    assert_eq!(
        reported(&findings),
        ["1 — required section `Features` missing"]
    );
}

#[test]
fn sec_4_optional_section() {
    let with_questions = check(
        SEC_STRICT_RULES,
        "# T\n## Roles\nx\n## Features\ny\n## Questions\nz\n",
    );
    assert!(with_questions.is_empty(), "{:?}", reported(&with_questions));

    let without = check(SEC_STRICT_RULES, "# T\n## Roles\nx\n## Features\ny\n");
    assert!(without.is_empty(), "{:?}", reported(&without));
}

#[test]
fn sec_5_strict_stray_section() {
    let findings = check(
        SEC_STRICT_RULES,
        "# T\n## Roles\nx\n## Features\ny\n## Extra\nz\n",
    );
    assert_eq!(
        reported(&findings),
        ["6 — section `Extra` not allowed (strict)"]
    );
}

#[test]
fn sec_6_required_section_by_title_regexp() {
    let findings = check(
        "- select: 'H2(/^Story: /)'\n  min: 1",
        "# T\n## Story: login\n",
    );
    assert!(findings.is_empty(), "{:?}", reported(&findings));
}

#[test]
fn lst_1_labeled_list_items_match_regexp() {
    let findings = check(
        r#"- select: SECTION("Roles") > UL > LI
  match: '^\*\*.+\*\* — .+'"#,
        "## Roles\n- **Admin** — manages\n- broken item\n",
    );
    assert_eq!(
        reported(&findings),
        [r"3 — `broken item` does not match `^\*\*.+\*\* — .+`"]
    );
}

#[test]
fn lst_2_min_max_items() {
    let findings = check(
        r#"- select: SECTION("Stories") > UL > LI
  min: 2
  max: 10"#,
        "## Stories\n- only one\n",
    );
    assert_eq!(
        reported(&findings),
        [r#"1 — `SECTION("Stories") > UL > LI`: 1 match, expected ≥ 2"#]
    );
}

#[test]
fn len_1_header_too_long() {
    let findings = check(
        "- select: H2\n  max_length: 20",
        "## This header is way too long for the configured limit of twenty\n",
    );
    assert_eq!(reported(&findings), ["1 — length 62, expected ≤ 20"]);
}

#[test]
fn len_2_paragraph_cap() {
    let long = "x".repeat(240);
    let markdown = format!("# T\n\nshort\n\n{long}\n\n{long}\n");
    let findings = check("- select: P\n  max_length: 200", &markdown);
    assert_eq!(
        reported(&findings),
        [
            "5 — length 240, expected ≤ 200",
            "7 — length 240, expected ≤ 200"
        ]
    );
}

#[test]
fn sub_1_story_blocks() {
    let findings = check(
        r#"- select: SECTION("User Stories") > SECTION(/^[A-Z]/)
  each:
    - select: P[0]
      optional: true
      starts_with: 'pre:'
    - select: P[text=/^As \*\*.+\*\*/] + UL > LI
      min: 1
      match: '^I can .+'"#,
        "## User Stories\n### Login\npre: user exists\nAs **Visitor**\n- I can log in\n### Broken\nsome text\n",
    );

    // The scenario's finding.
    assert!(
        reported(&findings).contains(
            &r#"6 — `P[text=/^As \*\*.+\*\*/] + UL > LI`: 0 matches, expected ≥ 1"#.to_string()
        ),
        "{:?}",
        reported(&findings)
    );
    // `optional: true` means "passes when the selector matches nothing"
    // (specs.md §3, SEL-3). `Broken` *has* a first paragraph, so the optional
    // inner rule still judges it — a second finding scenarios.md does not list.
    assert_eq!(
        reported(&findings),
        [
            "7 — `some text` does not start with `pre:`",
            r#"6 — `P[text=/^As \*\*.+\*\*/] + UL > LI`: 0 matches, expected ≥ 1"#,
        ]
    );
}

const TBL_GOOD: &str =
    "## Features\n| priority | epic | feature |\n|---|---|---|\n| p0 | **Core** | Parse |\n";
const TBL_COLUMNS: &str = r#"- select: SECTION("Features") > TABLE
  columns: [priority, epic, feature]"#;

#[test]
fn tbl_1_has_table_with_columns_in_order() {
    let findings = check(TBL_COLUMNS, TBL_GOOD);
    assert!(findings.is_empty(), "{:?}", reported(&findings));
}

#[test]
fn tbl_2_missing_or_misordered_column() {
    let findings = check(
        TBL_COLUMNS,
        "## Features\n| epic | priority | feature |\n|---|---|---|\n| p0 | **Core** | Parse |\n",
    );
    assert_eq!(
        reported(&findings),
        ["2 — column order `epic, priority, feature`, expected `priority, epic, feature`"]
    );
}

#[test]
fn tbl_3_column_enum() {
    let findings = check(
        "- select: SECTION(\"Features\") > TABLE > TD[col=\"priority\"]\n  enum: [p0, p1, p2, p3]",
        &format!("{TBL_GOOD}| p9 | **X** | Y |\n"),
    );
    assert_eq!(
        reported(&findings),
        ["5 — `p9` is not allowed, allowed: `p0`, `p1`, `p2`, `p3`"]
    );
}

#[test]
fn tbl_4_column_no_empty_values() {
    let findings = check(
        "- select: TABLE > TD[col=\"feature\"]\n  no_empty: true",
        "## Features\n| priority | epic | feature |\n|---|---|---|\n| p0 | **Core** |  |\n",
    );
    assert_eq!(reported(&findings), ["4 — empty cell in column `feature`"]);
}

#[test]
fn tbl_5_rows_bounds() {
    let findings = check("- select: TABLE\n  rows: { min: 2, max: 50 }", TBL_GOOD);
    assert_eq!(
        reported(&findings),
        ["2 — table has 1 body row, expected ≥ 2"]
    );
}

#[test]
fn tbl_6_per_cell_length_and_regexp() {
    let long = "x".repeat(130);
    let markdown =
        format!("## F\n| feature | capabilities |\n|---|---|\n| A | TBD |\n| B | {long} |\n");
    let findings = check(
        "- select: TABLE > TD[col=\"capabilities\"]\n  max_length: 120\n  forbid: 'TBD'",
        &markdown,
    );
    assert_eq!(
        reported(&findings),
        // Assertion order inside a rule is fixed: regexp modes before length bounds.
        [
            "4 — forbidden pattern `TBD` found",
            "5 — length 130, expected ≤ 120",
        ]
    );
}

#[test]
fn load_1_malformed_frontmatter() {
    let findings = check("- select: H1\n  count: 1", "---\nconfirmed: no\n# Doc\n");
    let load: Vec<_> = findings.iter().filter(|f| f.check == "load").collect();
    assert_eq!(load.len(), 1, "{:?}", reported(&findings));
    assert_eq!(load[0].line, 1);
    assert_eq!(load[0].message, "unclosed frontmatter block");
}
