//! Task 2.2 DoD coverage: selector × document → matched nodes.

use mdcheck::model::DocModel;
use mdcheck::selector::{parse, select, select_in, Match};

const GOOD: &str = include_str!("fixtures/feature-index-good.md");
const BAD: &str = include_str!("fixtures/feature-index-bad.md");

fn matches<'a>(doc: &'a DocModel, selector: &str) -> Vec<Match<'a>> {
    let selector = parse(selector).unwrap_or_else(|e| panic!("{e}"));
    select(doc, &selector)
}

fn lines(doc: &DocModel, selector: &str) -> Vec<usize> {
    matches(doc, selector)
        .iter()
        .map(Match::start_line)
        .collect()
}

fn texts(doc: &DocModel, selector: &str) -> Vec<String> {
    matches(doc, selector)
        .iter()
        .map(|m| m.text().to_string())
        .collect()
}

/// Milestone verify: selector string → matched node lines on the M1 fixtures.
#[test]
fn selector_table_maps_to_expected_lines_on_the_good_fixture() {
    let doc = DocModel::parse(GOOD);
    let cases: &[(&str, &[usize])] = &[
        ("DOCUMENT", &[1]),
        ("FRONTMATTER", &[1]),
        ("H1", &[5]),
        ("H2", &[7, 12, 17, 35]),
        ("H3", &[25]),
        ("SECTION", &[5, 7, 12, 17, 25, 35]),
        ("H1 > H2", &[7, 12, 17, 35]),
        ("H1 H3", &[25]),
        ("H1 > H3", &[]),
        ("H2 + H2", &[12, 17, 35]),
        ("H2 + UL", &[9]),
        ("SECTION(\"Roles\") > UL > LI", &[9, 10]),
        ("SECTION(\"Priority levels\") > OL > LI", &[14, 15]),
        ("LI", &[9, 10, 14, 15]),
        ("SECTION(\"Features\") > TABLE", &[19]),
        ("TR", &[19, 21, 22, 23]),
        (
            "SECTION(\"Features\") > TABLE > TD[col=\"priority\"]",
            &[21, 22, 23],
        ),
        ("TABLE > TD[col=\"epic\"]", &[21, 22, 23]),
        ("TR > TD[col=\"feature\"]", &[21, 22, 23]),
        ("TD[col=\"priority\"][0]", &[21]),
        ("TD[col=\"priority\"][-1]", &[23]),
        ("A", &[27, 37]),
        ("A[wiki]", &[27, 37]),
        ("A[href=\"questions/open\"]", &[37]),
        ("A[href=/^https?:/]", &[]),
        ("CODE[lang=\"gherkin\"]", &[29]),
        ("CODE[lang=\"rust\"]", &[]),
        ("H2(/^Question/)", &[35]),
        ("H2(\"Roles\")", &[7]),
        ("SECTION(/^Back/)", &[25]),
        ("H2[text$=\"levels\"]", &[12]),
        ("H2[text*=\"riorit\"]", &[12]),
        ("P[text^=\"See\"]", &[37]),
        ("P[text*=\"backlog\"]", &[27]),
        ("SECTION(\"Backlog\") > P[0]", &[27]),
        ("SECTION(\"Features\") P", &[27]),
        ("SECTION(\"Features\") > P", &[]),
    ];

    for (selector, expected) in cases {
        assert_eq!(lines(&doc, selector), *expected, "selector {selector}");
    }
}

#[test]
fn selector_table_maps_to_expected_lines_on_the_bad_fixture() {
    let doc = DocModel::parse(BAD);
    let cases: &[(&str, &[usize])] = &[
        ("H1 > H2", &[7, 14, 19]),
        (
            "SECTION(\"Features\") > TABLE > TD[col=\"priority\"]",
            &[11, 12],
        ),
        ("A[wiki]", &[]),
        ("A[href=/^https?:/]", &[21]),
        ("SECTION(\"Roles\") > UL > LI", &[16, 17]),
    ];

    for (selector, expected) in cases {
        assert_eq!(lines(&doc, selector), *expected, "selector {selector}");
    }
}

/// DoD: the priority cells and nothing else — the header cell is not a `TD`.
#[test]
fn dod_column_filter_returns_exactly_the_priority_cells() {
    let doc = DocModel::parse(GOOD);
    let found = matches(&doc, "SECTION(\"Features\") > TABLE > TD[col=\"priority\"]");

    let cells: Vec<(&str, usize)> = found.iter().map(|m| (m.text(), m.start_line())).collect();
    assert_eq!(cells, [("p0", 21), ("p1", 22), ("p2", 23)]);
    assert!(found
        .iter()
        .all(|m| m.attrs().column.as_deref() == Some("priority")));
}

/// DoD: `+` matches only the list adjacent to a matching paragraph.
#[test]
fn dod_adjacent_sibling_matches_only_the_adjacent_list() {
    let doc = DocModel::parse(
        "# T\n\n## Login\n\npre: setup\n\n- setup step\n\nAs a user\n\n- I can log in\n- I can log out\n\n## Other\n\n- not counted\n",
    );

    assert_eq!(lines(&doc, "P[text=/^As /] + UL > LI"), [11, 12]);
    assert_eq!(
        texts(&doc, "P[text=/^As /] + UL > LI"),
        ["I can log in", "I can log out"]
    );
    assert_eq!(lines(&doc, "UL > LI").len(), 4);
}

/// DoD: `H1 > H2` (child) vs `H1 H3` (descendant) through the section tree.
#[test]
fn dod_child_and_descendant_combinators_resolve_through_sections() {
    let doc = DocModel::parse("# T\n\n## A\n\n### A1\n\n#### A2\n\n## B\n");

    assert_eq!(texts(&doc, "H1 > H2"), ["A", "B"]);
    assert_eq!(texts(&doc, "H1 H3"), ["A1"]);
    assert_eq!(texts(&doc, "H1 H4"), ["A2"]);
    assert!(texts(&doc, "H1 > H3").is_empty());
    assert_eq!(texts(&doc, "H2 > H3"), ["A1"]);
    assert_eq!(texts(&doc, "H2 H4"), ["A2"]);
    assert_eq!(texts(&doc, "H2(\"A\") > H3"), ["A1"]);
    assert_eq!(texts(&doc, "SECTION(\"A\") > SECTION"), ["A1\nA2"]);
}

/// DoD: no match is an empty set, never an error.
#[test]
fn dod_empty_match_returns_an_empty_set() {
    let doc = DocModel::parse(GOOD);

    assert!(matches(&doc, "SECTION(\"Nope\") > UL > LI").is_empty());
    assert!(matches(&doc, "SECTION(\"Nope\") > P[0]").is_empty());
    assert!(matches(&doc, "H6").is_empty());
    assert!(matches(&DocModel::parse(""), "DOCUMENT > P").is_empty());
}

/// SEL-1: checks scoped to a section — items of `A` are never counted.
#[test]
fn sel_1_selector_scoped_to_a_section() {
    let doc = DocModel::parse("# T\n## A\n- one\n## B\n- two\n- three\n");
    let found = matches(&doc, "SECTION(\"B\") > UL > LI");

    assert_eq!(found.len(), 2);
    assert_eq!(
        found.iter().map(Match::text).collect::<Vec<_>>(),
        ["two", "three"]
    );
}

/// SEL-2: first paragraph of a section.
#[test]
fn sel_2_first_paragraph_of_a_section() {
    let doc = DocModel::parse("# T\n## Story\npre: setup line\nAs a user\n");
    let found = matches(&doc, "SECTION(\"Story\") > P[0]");

    assert_eq!(found.len(), 1);
    assert!(found[0].text().starts_with("pre:"));
    assert_eq!(found[0].start_line(), 3);
}

/// SEL-3: index filter on an empty scope yields no match and no error.
#[test]
fn sel_3_index_on_empty_scope_matches_nothing() {
    let doc = DocModel::parse("# T\n## Story\n### Sub\n");

    assert!(matches(&doc, "SECTION(\"Story\") > P[0]").is_empty());
    assert!(matches(&doc, "SECTION(\"Story\") > P[-1]").is_empty());
}

/// The index filter applies after the match set is collected, in document order.
#[test]
fn index_filter_applies_after_the_match_set_is_collected() {
    let doc = DocModel::parse(GOOD);

    assert_eq!(texts(&doc, "H2[0]"), ["Roles"]);
    assert_eq!(texts(&doc, "H2[-1]"), ["Questions"]);
    assert_eq!(texts(&doc, "H2[3]"), ["Questions"]);
    assert!(texts(&doc, "H2[4]").is_empty());
    assert!(texts(&doc, "H2[-5]").is_empty());
    assert_eq!(texts(&doc, "H1 > H2[1]"), ["Priority levels"]);
}

/// `select_in` scopes a run to a subtree — the primitive behind nested `each:`.
#[test]
fn select_in_scopes_matching_to_a_subtree() {
    let doc = DocModel::parse(GOOD);
    let roles = matches(&doc, "SECTION(\"Roles\")");
    let selector = parse("UL > LI").unwrap();

    let found = select_in(roles[0].node(), &selector);
    assert_eq!(
        found.iter().map(Match::text).collect::<Vec<_>>(),
        [
            "**Admin** — manages the index",
            "**Visitor** — browses features"
        ]
    );
}

/// Matches carry everything the assertion layer needs.
#[test]
fn match_exposes_text_lines_and_attrs() {
    let doc = DocModel::parse(GOOD);
    let code = matches(&doc, "CODE")[0];

    assert_eq!(code.kind().to_string(), "CODE");
    assert_eq!((code.start_line(), code.end_line()), (29, 33));
    assert_eq!(code.attrs().lang.as_deref(), Some("gherkin"));
    assert!(code.text().starts_with("Given a feature index"));

    let link = matches(&doc, "A[wiki]")[0];
    assert_eq!(link.attrs().href.as_deref(), Some("features/backlog"));
    assert_eq!(link.text(), "the backlog");
}
