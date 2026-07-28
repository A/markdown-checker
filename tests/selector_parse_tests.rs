//! Task 2.1 DoD coverage: selector string → AST / parse error.

use mdcheck::selector::{parse, AttrKey, Element, Test};

/// Every selector that appears anywhere in specs.md or scenarios.md.
const CORPUS: &[&str] = &[
    // specs.md — rule file examples
    "FRONTMATTER",
    "DOCUMENT",
    "H1",
    "H1 > H2",
    "SECTION(\"Roles\") > UL > LI",
    "SECTION(\"Features\") > TABLE",
    "SECTION(\"Features\") > TABLE > TD[col=\"priority\"]",
    "A",
    "SECTION(\"User Stories\") > SECTION(/^[A-Z]/)",
    "P[0]",
    "P[text=/^As \\*\\*.+\\*\\*/] + UL > LI",
    // specs.md — grammar tables
    "H2",
    "H3",
    "H4",
    "H5",
    "H6",
    "SECTION",
    "P",
    "UL",
    "OL",
    "LI",
    "TABLE",
    "TR",
    "TD",
    "CODE",
    "P[text=\"Roles\"]",
    "P[text^=\"pre:\"]",
    "H2[text$=\"?\"]",
    "P[text*=\"feature\"]",
    "LI[text=/^I can .+/]",
    "SECTION[title=\"Roles\"]",
    "SECTION[title=/^Story: /]",
    "TD[col=\"priority\"]",
    "A[href=/^https?:/]",
    "A[href=\"features/x\"]",
    "A[wiki]",
    "CODE[lang=\"gherkin\"]",
    "H2(\"Roles\")",
    "H2(/^Story: /)",
    "P[-1]",
    // scenarios.md
    "SECTION(\"B\") > UL > LI",
    "SECTION(\"Story\") > P[0]",
    "SECTION(\"Stories\") > UL > LI",
    "TABLE > TD[col=\"feature\"]",
    "TABLE > TD[col=\"capabilities\"]",
];

/// DoD: every selector in the specs parses.
#[test]
fn dod_every_selector_in_the_specs_parses() {
    let failures: Vec<String> = CORPUS
        .iter()
        .filter_map(|input| parse(input).err().map(|e| format!("{input} → {e}")))
        .collect();

    assert!(failures.is_empty(), "unparsed selectors:\n{failures:#?}");
}

/// Round-trip through the canonical form pins the AST shape for the whole grammar.
#[test]
fn parses_to_canonical_ast() {
    let cases: &[(&str, &str)] = &[
        ("H1", "H1"),
        ("H1 > H2", "H1 > H2"),
        ("H1 H3", "H1 H3"),
        ("P + UL", "P + UL"),
        ("H1>H2", "H1 > H2"),
        ("P+UL>LI", "P + UL > LI"),
        ("  H1   >   H2  ", "H1 > H2"),
        ("H2(\"Roles\")", "H2[text=\"Roles\"]"),
        ("H2(/^Story: /)", "H2[text=/^Story: /]"),
        ("SECTION(\"Roles\")", "SECTION[title=\"Roles\"]"),
        ("SECTION(/^[A-Z]/)", "SECTION[title=/^[A-Z]/]"),
        ("P[text^=\"pre:\"]", "P[text^=\"pre:\"]"),
        ("P[text$=\"?\"]", "P[text$=\"?\"]"),
        ("P[text*=\"feature\"]", "P[text*=\"feature\"]"),
        ("A[wiki]", "A[wiki]"),
        ("TD[col=\"priority\"][0]", "TD[col=\"priority\"][0]"),
        ("P[-1]", "P[-1]"),
        (
            "SECTION(\"Features\") > TABLE > TD[col=\"priority\"]",
            "SECTION[title=\"Features\"] > TABLE > TD[col=\"priority\"]",
        ),
    ];

    for (input, canonical) in cases {
        let selector = parse(input).unwrap_or_else(|e| panic!("{input}: {e}"));
        assert_eq!(selector.to_string(), *canonical, "input {input}");
    }
}

/// DoD: `[text=/^I can/]` yields a regex filter.
#[test]
fn dod_regex_value_yields_regex_filter() {
    let selector = parse("LI[text=/^I can/]").unwrap();

    assert_eq!(selector.head.element, Element::ListItem);
    assert_eq!(selector.head.filters.len(), 1);
    assert_eq!(selector.head.filters[0].key, AttrKey::Text);
    let Test::Matches(pattern) = &selector.head.filters[0].test else {
        panic!("expected a regex test, got {:?}", selector.head.filters[0]);
    };
    assert_eq!(pattern.source, "^I can");
    assert!(pattern.regex.is_match("I can log in"));
    assert!(!pattern.regex.is_match("As a user I can log in"));
}

/// DoD: invalid regex → parse error naming the pattern.
#[test]
fn dod_invalid_regex_is_a_parse_error_naming_the_pattern() {
    let error = parse("P[text=/^(I can/]").unwrap_err();

    assert!(
        error.message.contains("^(I can"),
        "message should name the pattern: {}",
        error.message
    );
    assert_eq!(error.column, 9, "column should point at the pattern");
}

/// DoD: integer vs key disambiguation inside brackets.
#[test]
fn dod_bare_integer_is_index_and_key_is_attribute_filter() {
    let index = parse("P[0]").unwrap();
    assert_eq!(index.head.index, Some(0));
    assert!(index.head.filters.is_empty());

    let last = parse("P[-1]").unwrap();
    assert_eq!(last.head.index, Some(-1));
    assert!(last.head.filters.is_empty());

    let attr = parse("P[text=\"x\"]").unwrap();
    assert_eq!(attr.head.index, None);
    assert_eq!(attr.head.filters.len(), 1);
    assert_eq!(attr.head.filters[0].test, Test::Equals("x".to_string()));
}

/// DoD: garbage never panics; the error carries a position and renders a pointer.
#[test]
fn dod_garbage_selectors_report_position_without_panicking() {
    let cases: &[(&str, usize, &str)] = &[
        ("", 1, "expected an element"),
        ("???", 1, "expected an element"),
        ("DIV", 1, "unknown element"),
        ("H7", 1, "unknown element"),
        ("H1 >", 5, "expected an element"),
        ("H1 > > H2", 6, "expected an element"),
        ("P[bogus=\"x\"]", 3, "unknown filter"),
        ("P[text~=\"x\"]", 7, "expected"),
        ("P[text=\"x\"", 11, "expected `]`"),
        ("P[text=\"x]", 9, "unterminated string"),
        ("P[text=/^x]", 9, "unterminated regexp"),
        ("H2(\"Roles\"", 11, "expected `)`"),
        ("H2(Roles)", 4, "expected"),
        ("P[text=x]", 8, "expected"),
        ("H1 H2!", 6, "unexpected"),
    ];

    for (input, column, needle) in cases {
        let error = parse(input).unwrap_err();
        assert!(
            error.message.contains(needle),
            "input {input:?}: message {:?} should contain {needle:?}",
            error.message
        );
        assert_eq!(error.column, *column, "input {input:?}");
        let rendered = error.to_string();
        assert!(rendered.contains('^'), "input {input:?}: {rendered}");
    }
}
