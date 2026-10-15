//! Milestone 4 DoD coverage: `ordered`/`strict`, frontmatter `has`/`not_has`,
//! table assertions and nested `each:` rules.

mod support;

use support::{check, reported};

const ORDERED: &str = r#"- select: H1 > H2
  ordered: [Roles, Priority levels, Features, Questions?]"#;

const GOOD_SECTIONS: &str =
    "# T\n## Roles\na\n## Priority levels\nb\n## Features\nc\n## Questions\nd\n";

/// DoD: the ordered list passes the good fixture.
#[test]
fn dod_ordered_passes_the_good_fixture() {
    assert!(check(ORDERED, GOOD_SECTIONS).is_empty());
    assert!(check(
        ORDERED,
        "# T\n## Roles\na\n## Priority levels\nb\n## Features\nc\n"
    )
    .is_empty());
}

/// DoD: swapped sections produce a finding naming both.
#[test]
fn dod_swapped_sections_produce_a_finding_naming_both() {
    let findings = check(
        ORDERED,
        "# T\n## Roles\na\n## Features\nc\n## Priority levels\nb\n",
    );

    assert_eq!(
        reported(&findings),
        ["6 — section order: `Priority levels` appears after `Features`, expected before"]
    );
    assert_eq!(findings[0].expected, "`Priority levels` before `Features`");
    assert_eq!(findings[0].actual, "`Priority levels` after `Features`");
}

/// DoD: `strict: true` plus a stray section reports it at its own line.
#[test]
fn dod_strict_reports_a_stray_section_with_its_line() {
    let findings = check(
        &format!("{ORDERED}\n  strict: true"),
        &format!("{GOOD_SECTIONS}## Extra\ne\n"),
    );

    assert_eq!(
        reported(&findings),
        ["10 — section `Extra` not allowed (strict)"]
    );
}

/// Without `strict`, extra sections are allowed.
#[test]
fn ordered_without_strict_allows_extra_sections() {
    assert!(check(ORDERED, &format!("{GOOD_SECTIONS}## Extra\ne\n")).is_empty());
}

#[test]
fn ordered_reports_every_missing_required_section() {
    let findings = check(ORDERED, "# T\n## Roles\na\n");
    assert_eq!(
        reported(&findings),
        [
            "1 — required section `Priority levels` missing",
            "1 — required section `Features` missing",
        ]
    );
}

/// `ordered` reads section titles as well as heading text.
#[test]
fn ordered_works_on_section_selectors() {
    assert!(check(
        "- select: SECTION(\"T\") > SECTION\n  ordered: [Roles, Features]",
        "# T\n## Roles\na\n## Features\nb\n"
    )
    .is_empty());
}

/// DoD: `has: {confirmed: {enum: [yes, no]}}` fails value `maybe`, listing allowed.
#[test]
fn dod_frontmatter_enum_lists_allowed_values() {
    let rules = "- select: FRONTMATTER\n  has: { confirmed: { enum: [yes, no] } }";

    assert!(check(rules, "---\nconfirmed: yes\n---\n").is_empty());

    let findings = check(rules, "---\nconfirmed: maybe\n---\n");
    assert_eq!(
        reported(&findings),
        ["2 — `confirmed` is `maybe`, allowed: `yes`, `no`"]
    );
    assert_eq!(findings[0].expected, "one of `yes`, `no`");
    assert_eq!(findings[0].actual, "maybe");
}

#[test]
fn frontmatter_has_reports_a_missing_key() {
    let findings = check(
        "- select: FRONTMATTER\n  has: { confirmed: { enum: [yes, no] } }",
        "---\ntype: x\n---\n",
    );
    assert_eq!(reported(&findings), ["1 — key `confirmed` missing"]);
}

/// A bare string value spec is a regexp the whole value must match.
#[test]
fn frontmatter_has_accepts_a_bare_regexp_spec() {
    let rules = "- select: FRONTMATTER\n  has: { priority: '^P[0-3]$' }";

    assert!(check(rules, "---\npriority: P2\n---\n").is_empty());
    assert_eq!(
        reported(&check(rules, "---\npriority: P7\n---\n")),
        ["2 — `priority` value `P7` does not match `^P[0-3]$`"]
    );
}

/// `has: {key: {}}` — an empty spec only demands the key's presence.
#[test]
fn frontmatter_has_can_demand_presence_only() {
    let rules = "- select: FRONTMATTER\n  has: { type: {} }";

    assert!(check(rules, "---\ntype: feature-index\n---\n").is_empty());
    assert_eq!(
        reported(&check(rules, "---\nother: x\n---\n")),
        ["1 — key `type` missing"]
    );
}

/// `{ min: 1 }` demands a non-empty list, flow or block.
#[test]
fn frontmatter_list_min_requires_a_non_empty_list() {
    let rules = "- select: FRONTMATTER\n  has: { tags: { min: 1 } }";

    assert!(check(rules, "---\ntags: [a]\n---\n").is_empty());
    assert!(check(rules, "---\ntags:\n  - a\n---\n").is_empty());

    let findings = check(rules, "---\ntags: []\n---\n");
    assert_eq!(
        reported(&findings),
        ["2 — `tags` has 0 items, expected ≥ 1"]
    );
    assert_eq!(findings[0].expected, "≥ 1 item");
    assert_eq!(findings[0].actual, "0 items");
}

#[test]
fn frontmatter_list_max_count_and_range() {
    let doc = "---\ntags: [a, b, c]\n---\n";
    let has = |spec: &str| {
        check(
            &format!("- select: FRONTMATTER\n  has: {{ tags: {spec} }}"),
            doc,
        )
    };

    assert!(has("{ max: 3 }").is_empty());
    assert!(has("{ count: 3 }").is_empty());
    assert!(has("{ min: 1, max: 3 }").is_empty());
    assert_eq!(
        reported(&has("{ max: 2 }")),
        ["2 — `tags` has 3 items, expected ≤ 2"]
    );
    assert_eq!(
        reported(&has("{ count: 2 }")),
        ["2 — `tags` has 3 items, expected exactly 2"]
    );
    assert_eq!(
        reported(&has("{ min: 4, max: 5 }")),
        ["2 — `tags` has 3 items, expected ≥ 4"]
    );
}

/// A cardinality spec on a scalar or a bare key is a finding, not a pass.
#[test]
fn frontmatter_list_spec_on_a_scalar_reports_not_a_list() {
    let rules = "- select: FRONTMATTER\n  has: { tags: { min: 0 } }";

    assert_eq!(
        reported(&check(rules, "---\ntags: index\n---\n")),
        ["2 — `tags` value `index` is not a list"]
    );
    assert_eq!(
        reported(&check(rules, "---\ntags:\n---\n")),
        ["2 — `tags` value `` is not a list"]
    );
    assert_eq!(
        reported(&check(rules, "---\nother: x\n---\n")),
        ["1 — key `tags` missing"]
    );
}

/// `enum:` on a list value checks every item, each at its own line; on a scalar
/// it stays the whole-value check.
#[test]
fn frontmatter_enum_applies_per_list_item() {
    let rules = "- select: FRONTMATTER\n  has: { tags: { enum: [a, b] } }";

    assert!(check(rules, "---\ntags: [a, b, a]\n---\n").is_empty());
    assert!(check(rules, "---\ntags: b\n---\n").is_empty());
    assert_eq!(
        reported(&check(rules, "---\ntags:\n  - a\n  - x\n  - y\n---\n")),
        [
            "4 — `tags` item `x` not allowed, allowed: `a`, `b`",
            "5 — `tags` item `y` not allowed, allowed: `a`, `b`",
        ]
    );
    assert_eq!(
        reported(&check(
            "- select: FRONTMATTER\n  has: { tags: { count: 1, enum: [a] } }",
            "---\ntags: [x, a]\n---\n"
        )),
        [
            "2 — `tags` has 2 items, expected exactly 1",
            "2 — `tags` item `x` not allowed, allowed: `a`",
        ]
    );
}

#[test]
fn frontmatter_not_has_passes_and_fails() {
    let rules = "- select: FRONTMATTER\n  not_has: [draft, wip]";

    assert!(check(rules, "---\nconfirmed: no\n---\n").is_empty());
    assert_eq!(
        reported(&check(
            rules,
            "---\nconfirmed: no\ndraft: true\nwip: 1\n---\n"
        )),
        [
            "3 — key `draft` must not be present",
            "4 — key `wip` must not be present",
        ]
    );
}

const TABLE_RULES: &str = r#"- select: TABLE
  columns: [priority, epic, feature]
  rows: { min: 1, max: 2 }"#;

/// DoD: missing column, misordered columns and a row over max each give one finding.
#[test]
fn dod_table_missing_column_is_one_finding_at_the_header_line() {
    let findings = check(
        TABLE_RULES,
        "# T\n\n| priority | epic |\n|---|---|\n| p0 | Core |\n",
    );
    assert_eq!(reported(&findings), ["3 — column `feature` missing"]);
    assert_eq!(findings[0].expected, "columns `priority, epic, feature`");
    assert_eq!(findings[0].actual, "columns `priority, epic`");
}

#[test]
fn dod_table_misordered_columns_is_one_finding_at_the_header_line() {
    let findings = check(
        TABLE_RULES,
        "# T\n\n| epic | priority | feature |\n|---|---|---|\n| Core | p0 | Parse |\n",
    );
    assert_eq!(
        reported(&findings),
        ["3 — column order `epic, priority, feature`, expected `priority, epic, feature`"]
    );
}

#[test]
fn dod_table_row_over_max_is_one_finding_at_the_table_line() {
    let findings = check(
        TABLE_RULES,
        "# T\n\n| priority | epic | feature |\n|---|---|---|\n| p0 | Core | a |\n| p1 | Core | b |\n| p2 | Core | c |\n",
    );
    assert_eq!(
        reported(&findings),
        ["3 — table has 3 body rows, expected ≤ 2"]
    );
    assert_eq!(findings[0].expected, "≤ 2 body rows");
    assert_eq!(findings[0].actual, "3 body rows");
}

#[test]
fn table_rows_under_min_is_one_finding() {
    let findings = check(
        TABLE_RULES,
        "# T\n\n| priority | epic | feature |\n|---|---|---|\n",
    );
    assert_eq!(
        reported(&findings),
        ["3 — table has 0 body rows, expected ≥ 1"]
    );
}

/// DoD: a ragged row is a finding, not a panic.
#[test]
fn dod_ragged_row_is_a_finding_not_a_panic() {
    let findings = check(
        "- select: TABLE\n  columns: [priority, epic, feature]",
        "# T\n\n| priority | epic | feature |\n|---|---|---|\n| p0 | Core | a | extra |\n| p1 |\n",
    );
    assert_eq!(
        reported(&findings),
        [
            "5 — row has 4 cells, expected 3",
            "6 — row has 1 cell, expected 3",
        ]
    );
}

#[test]
fn table_assertions_pass_on_a_well_formed_table() {
    assert!(check(
        TABLE_RULES,
        "# T\n\n| priority | epic | feature |\n|---|---|---|\n| p0 | Core | a |\n"
    )
    .is_empty());
}

/// DoD: `each:` re-roots the inner selectors at every outer match and carries the
/// outer context in the check id.
#[test]
fn dod_each_produces_per_story_findings_with_distinct_check_ids() {
    let findings = check(
        r#"- select: SECTION("User Stories") > SECTION(/^[A-Z]/)
  each:
    - select: P[0]
      optional: true
      starts_with: 'pre:'
    - select: P[text=/^As \*\*.+\*\*/] + UL > LI
      min: 1
      match: '^I can .+'"#,
        "## User Stories\n\
         ### Login\n\
         pre: user exists\n\
         As **Visitor**\n\
         - I can log in\n\
         ### Logout\n\
         As **Visitor**\n\
         - log out somehow\n\
         ### Broken\n\
         some text\n",
    );

    assert_eq!(
        findings
            .iter()
            .map(|f| f.check.as_str())
            .collect::<Vec<_>>(),
        [
            "test.rule[0].each[0]@6",
            "test.rule[0].each[1]@6",
            "test.rule[0].each[0]@9",
            "test.rule[0].each[1]@9",
        ]
    );
    assert_eq!(
        reported(&findings),
        [
            "7 — `As **Visitor**` does not start with `pre:`",
            "8 — `log out somehow` does not match `^I can .+`",
            "10 — `some text` does not start with `pre:`",
            r#"9 — `P[text=/^As \*\*.+\*\*/] + UL > LI`: 0 matches, expected ≥ 1"#,
        ]
    );
}

/// Inner rules see only their own outer match — a sibling section never leaks in.
#[test]
fn each_scopes_inner_selectors_to_the_outer_match() {
    let findings = check(
        "- select: SECTION(\"Stories\") > SECTION\n  each:\n    - select: UL > LI\n      count: 1",
        "## Stories\n### A\n- one\n### B\n- two\n- three\n",
    );
    assert_eq!(
        reported(&findings),
        ["4 — `UL > LI`: 2 matches, expected exactly 1"]
    );
}

/// Outer-rule assertions still run next to `each:`.
#[test]
fn each_runs_alongside_the_outer_rules_own_assertions() {
    let findings = check(
        "- select: SECTION(\"Stories\") > SECTION\n  min: 3\n  each:\n    - select: UL > LI\n      min: 1",
        "## Stories\n### A\n- one\n### B\ntext\n",
    );
    assert_eq!(
        reported(&findings),
        [
            r#"1 — `SECTION("Stories") > SECTION`: 2 matches, expected ≥ 3"#,
            "4 — `UL > LI`: 0 matches, expected ≥ 1",
        ]
    );
}
