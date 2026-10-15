//! Task 3.1 DoD coverage: rule-file schema, loader and load-stage validation.

mod support;

use mdcheck::rules::RuleSet;

/// The rule file from specs.md §1, verbatim — every key the schema must accept.
const SPEC_RULE_FILE: &str = r#"
doc: feature-index

rules:
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }
    not_has: [draft]

  - select: DOCUMENT
    min_lines: 20

  - select: H1
    count: 1
    match: '^\S+ feature index$'
    max_length: 60

  - select: H1 > H2
    ordered: [Roles, Priority levels, Features, Questions?]
    strict: true

  - select: SECTION("Roles") > UL > LI
    min: 2
    match: '^\*\*.+\*\* — .+'

  - select: SECTION("Features") > TABLE
    columns: [priority, epic, feature, capabilities]
    rows: { min: 5 }

  - select: SECTION("Features") > TABLE > TD[col="priority"]
    enum: [p0, p1, p2, p3]

  - select: A
    wiki: true
    resolve: true

  - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
    each:
      - select: P[0]
        optional: true
        starts_with: 'pre:'
      - select: P[text=/^As \*\*.+\*\*/] + UL > LI
        min: 1
        match: '^I can .+'
"#;

#[test]
fn dod_valid_rule_file_loads_with_pre_parsed_selectors() {
    let set = RuleSet::parse(SPEC_RULE_FILE).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(set.doc, "feature-index");
    assert_eq!(set.rules.len(), 9);
    assert_eq!(
        set.rules
            .iter()
            .map(|r| r.selector.to_string())
            .collect::<Vec<_>>(),
        [
            "FRONTMATTER",
            "DOCUMENT",
            "H1",
            "H1 > H2",
            "SECTION[title=\"Roles\"] > UL > LI",
            "SECTION[title=\"Features\"] > TABLE",
            "SECTION[title=\"Features\"] > TABLE > TD[col=\"priority\"]",
            "A",
            "SECTION[title=\"User Stories\"] > SECTION[title=/^[A-Z]/]",
        ]
    );
}

/// Nested `each:` selectors are pre-parsed too — a broken one fails the load.
#[test]
fn nested_selectors_are_pre_parsed() {
    let error =
        RuleSet::parse("doc: d\nrules:\n  - select: SECTION\n    each:\n      - select: 'P >'\n")
            .unwrap_err();
    let message = error.to_string();

    assert!(
        message.starts_with("rule 0.each[0]: invalid selector"),
        "{message}"
    );
}

/// Every rule carries a check id: `<doc>.rule[i]`, or the explicit `id:`.
#[test]
fn check_ids_default_to_the_rule_index_and_honour_explicit_ids() {
    let set = RuleSet::parse(SPEC_RULE_FILE).unwrap();
    assert_eq!(set.rules[0].check, "feature-index.rule[0]");
    assert_eq!(set.rules[8].check, "feature-index.rule[8]");

    let set = RuleSet::parse("doc: d\nrules:\n  - select: H1\n    id: title\n    count: 1\n")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(set.rules[0].check, "title");
}

/// DoD: unknown assertion key names the key and the rule index (0-based, the same
/// index that appears in check ids).
#[test]
fn dod_unknown_key_names_key_and_rule_index() {
    let error = RuleSet::parse(
        "doc: d\nrules:\n  - select: H1\n  - select: H2\n  - select: H3\n  - select: H4\n    foo: 1\n",
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "unknown key 'foo' in rule 3");
}

#[test]
fn unknown_key_inside_each_names_the_nested_position() {
    let error = RuleSet::parse(
        "doc: d\nrules:\n  - select: H1\n  - select: SECTION\n    each:\n      - select: P\n        bar: 2\n",
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "unknown key 'bar' in rule 1.each[0]");
}

/// DoD: duplicate explicit ids are a load error.
#[test]
fn dod_duplicate_ids_are_a_load_error() {
    let error = RuleSet::parse(
        "doc: d\nrules:\n  - select: H1\n    id: title\n  - select: H2\n    id: title\n",
    )
    .unwrap_err();

    assert_eq!(error.to_string(), "duplicate rule id 'title' in rule 1");
}

#[test]
fn invalid_selector_is_a_load_error_naming_the_rule() {
    let error = RuleSet::parse("doc: d\nrules:\n  - select: 'H1 >> H2'\n").unwrap_err();
    let message = error.to_string();

    assert!(message.starts_with("rule 0: invalid selector"), "{message}");
    assert!(message.contains("H1 >> H2"), "{message}");
}

#[test]
fn invalid_assertion_regex_is_a_load_error_naming_the_rule() {
    let error = RuleSet::parse("doc: d\nrules:\n  - select: H1\n    match: '('\n").unwrap_err();
    let message = error.to_string();

    assert!(message.starts_with("rule 0: invalid regexp"), "{message}");
}

#[test]
fn missing_select_is_a_load_error() {
    let error = RuleSet::parse("doc: d\nrules:\n  - count: 1\n").unwrap_err();
    assert!(error.to_string().contains("select"), "{error}");
}

#[test]
fn frontmatter_list_spec_loads() {
    RuleSet::parse(
        "doc: d\nrules:\n  - select: FRONTMATTER\n    has: { tags: { min: 1, max: 3, enum: [a, b] }, ids: { count: 2 } }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn frontmatter_list_count_with_min_or_max_is_a_load_error() {
    let error = RuleSet::parse(
        "doc: d\nrules:\n  - select: FRONTMATTER\n    has: { tags: { count: 1, max: 2 } }\n",
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "rule 0: has `tags`: `count` cannot be combined with `min` or `max`"
    );
}

#[test]
fn frontmatter_list_min_above_max_is_a_load_error() {
    let error = RuleSet::parse(
        "doc: d\nrules:\n  - select: FRONTMATTER\n    has: { tags: { min: 3, max: 1 } }\n",
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "rule 0: has `tags`: `min` 3 is greater than `max` 1"
    );
}

/// A misspelled key in a value-spec mapping stays a load error.
#[test]
fn frontmatter_misspelled_value_spec_key_is_a_load_error() {
    assert!(RuleSet::parse(
        "doc: d\nrules:\n  - select: FRONTMATTER\n    has: { tags: { minimum: 1 } }\n",
    )
    .is_err());
}

#[test]
fn malformed_yaml_is_a_load_error() {
    let error = RuleSet::parse("doc: d\nrules: [").unwrap_err();
    assert!(!error.to_string().is_empty());
}

#[test]
fn yaml_1_1_booleans_stay_strings_in_enum_sets() {
    let set = RuleSet::parse(
        "doc: d\nrules:\n  - select: FRONTMATTER\n    has: { confirmed: { enum: [yes, no] } }\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));

    let findings = support::check(
        "- select: FRONTMATTER\n  has: { confirmed: { enum: [yes, no] } }",
        "---\nconfirmed: yes\n---\n",
    );
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(set.rules.len(), 1);
}
