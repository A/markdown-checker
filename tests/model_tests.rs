//! Task 1.4 DoD coverage. One test per checkbox (plus element-kind coverage).

use mdcheck::model::{DocModel, ElementKind, Node};

const GOOD: &str = include_str!("fixtures/feature-index-good.md");
const BAD: &str = include_str!("fixtures/feature-index-bad.md");

fn all_of(node: &Node, kind: ElementKind) -> Vec<&Node> {
    node.descendants().filter(|n| n.kind == kind).collect()
}

fn first_of(node: &Node, kind: ElementKind) -> &Node {
    all_of(node, kind)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no {kind:?} node found"))
}

fn section<'a>(node: &'a Node, title: &str) -> &'a Node {
    node.descendants()
        .find(|n| {
            matches!(n.kind, ElementKind::Section(_)) && n.attrs.title.as_deref() == Some(title)
        })
        .unwrap_or_else(|| panic!("no SECTION titled {title:?}"))
}

/// DoD: content between `## A` and the next `##` lands inside A's SECTION node.
#[test]
fn dod_section_owns_content_until_next_same_or_higher_heading() {
    let doc = DocModel::parse("# T\n\n## A\n\n- one\n\n### A1\n\ntext\n\n## B\n\n- two\n");

    let a = section(&doc.root, "A");
    let b = section(&doc.root, "B");

    assert_eq!(a.kind, ElementKind::Section(2));
    assert_eq!(all_of(a, ElementKind::ListItem).len(), 1);
    assert_eq!(all_of(a, ElementKind::ListItem)[0].text, "one");

    // The `###` subsection nests inside A, and B is not part of A.
    let a1 = section(a, "A1");
    assert_eq!(a1.kind, ElementKind::Section(3));
    assert!(a
        .descendants()
        .all(|n| n.attrs.title.as_deref() != Some("B")));
    assert_eq!(all_of(b, ElementKind::ListItem)[0].text, "two");
}

/// DoD: section tree shape of the good fixture.
#[test]
fn dod_section_tree_shape_matches_snapshot() {
    let doc = DocModel::parse(GOOD);
    insta::assert_snapshot!(doc.dump_tree());
}

/// DoD: TD nodes report the column name from the header row.
#[test]
fn dod_table_cell_reports_column_name_from_header_row() {
    let doc = DocModel::parse(GOOD);
    let table = first_of(section(&doc.root, "Features"), ElementKind::Table);
    let rows = all_of(table, ElementKind::TableRow);

    assert!(rows[0].attrs.header_row);
    assert!(!rows[1].attrs.header_row);

    let cells = all_of(rows[1], ElementKind::TableCell);
    let cols: Vec<Option<&str>> = cells.iter().map(|c| c.attrs.column.as_deref()).collect();
    assert_eq!(
        cols,
        [
            Some("priority"),
            Some("epic"),
            Some("feature"),
            Some("capabilities")
        ]
    );
    assert_eq!(cells[0].text, "p0");
    assert_eq!(cells[1].text, "**Core**");
    assert_eq!(cells[0].attrs.column_index, Some(0));
    assert_eq!(cells[3].attrs.column_index, Some(3));
}

/// DoD: ragged row → cell keeps its positional index, no panic.
#[test]
fn dod_ragged_row_cells_keep_positional_index() {
    let doc = DocModel::parse(BAD);
    let table = first_of(&doc.root, ElementKind::Table);
    let rows = all_of(table, ElementKind::TableRow);
    let short = all_of(rows[2], ElementKind::TableCell);

    assert_eq!(short[0].text, "**Core**");
    assert_eq!(short[1].text, "p1");
    let indices: Vec<Option<usize>> = short.iter().map(|c| c.attrs.column_index).collect();
    assert_eq!(indices, [Some(0), Some(1), Some(2)]);
    assert_eq!(short[2].text, "");
    assert_eq!(short[2].attrs.column.as_deref(), Some("feature"));
}

/// DoD: overlong row keeps positional index with no column name, no panic.
#[test]
fn dod_row_longer_than_header_keeps_index_without_column_name() {
    let doc = DocModel::parse("| a | b |\n|---|---|\n| 1 | 2 | 3 |\n");
    let table = first_of(&doc.root, ElementKind::Table);
    let rows = all_of(table, ElementKind::TableRow);
    let cells = all_of(rows[1], ElementKind::TableCell);

    assert!(cells.len() >= 2);
    let last = cells.last().unwrap();
    assert_eq!(last.attrs.column_index, Some(cells.len() - 1));
    if cells.len() > 2 {
        assert_eq!(last.attrs.column, None);
    }
}

/// DoD: `[[target|label]]` parses as A with wiki=true and text `label`.
#[test]
fn dod_wikilink_is_link_with_wiki_flag_and_label_text() {
    let doc = DocModel::parse(GOOD);
    let link = first_of(section(&doc.root, "Backlog"), ElementKind::Link);

    assert!(link.attrs.wiki);
    assert_eq!(link.attrs.href.as_deref(), Some("features/backlog"));
    assert_eq!(link.text, "the backlog");
}

#[test]
fn plain_link_is_link_with_wiki_false() {
    let doc = DocModel::parse(BAD);
    let link = first_of(&doc.root, ElementKind::Link);

    assert!(!link.attrs.wiki);
    assert_eq!(
        link.attrs.href.as_deref(),
        Some("https://example.com/questions")
    );
    assert_eq!(link.text, "open questions");
}

/// DoD: node line numbers match source file lines (frontmatter offset applied).
#[test]
fn dod_line_numbers_carry_frontmatter_offset() {
    let doc = DocModel::parse("---\nconfirmed: no\n---\n# T\n\npara\n");
    let h1 = first_of(&doc.root, ElementKind::Heading(1));
    let p = first_of(&doc.root, ElementKind::Paragraph);

    assert_eq!((h1.start_line, h1.end_line), (4, 4));
    assert_eq!((p.start_line, p.end_line), (6, 6));
}

/// DoD: exact line numbers for heading, cell and link in the good fixture.
#[test]
fn dod_exact_line_numbers_for_heading_cell_and_link() {
    let doc = DocModel::parse(GOOD);

    let h1 = first_of(&doc.root, ElementKind::Heading(1));
    assert_eq!(h1.text, "Search feature index");
    assert_eq!(h1.start_line, 5);

    let features = section(&doc.root, "Features");
    assert_eq!(features.start_line, 17);
    let heading = first_of(features, ElementKind::Heading(2));
    assert_eq!(heading.start_line, 17);

    let table = first_of(features, ElementKind::Table);
    let rows = all_of(table, ElementKind::TableRow);
    assert_eq!(rows[0].start_line, 19);
    let cell = &all_of(rows[3], ElementKind::TableCell)[0];
    assert_eq!(cell.text, "p2");
    assert_eq!(cell.start_line, 23);

    let link = first_of(section(&doc.root, "Questions"), ElementKind::Link);
    assert_eq!(link.start_line, 37);
}

/// DoD: frontmatter + DOCUMENT pseudo-nodes.
#[test]
fn dod_document_and_frontmatter_pseudo_nodes_exist() {
    let doc = DocModel::parse(GOOD);

    assert_eq!(doc.root.kind, ElementKind::Document);
    assert_eq!(doc.root.start_line, 1);
    assert_eq!(doc.line_count, 37);

    let fm = &doc.root.children[0];
    assert_eq!(fm.kind, ElementKind::Frontmatter);
    assert_eq!((fm.start_line, fm.end_line), (1, 4));
    assert_eq!(doc.frontmatter.get("confirmed"), Some("no"));
    assert!(doc.load_errors.is_empty());
}

#[test]
fn document_without_frontmatter_has_no_pseudo_node() {
    let doc = DocModel::parse("# T\n");

    assert!(all_of(&doc.root, ElementKind::Frontmatter).is_empty());
    assert!(!doc.frontmatter.present);
}

#[test]
fn unclosed_frontmatter_is_reported_as_load_error_without_panic() {
    let doc = DocModel::parse("---\nconfirmed: no\n# T\n");

    assert_eq!(doc.load_errors.len(), 1);
    assert!(all_of(&doc.root, ElementKind::Frontmatter).is_empty());
}

/// DoD: typed wrappers exist for every element kind in the spec list.
#[test]
fn dod_all_element_kinds_are_exposed() {
    let doc = DocModel::parse(GOOD);
    let root = &doc.root;

    assert_eq!(all_of(root, ElementKind::Heading(1)).len(), 1);
    assert_eq!(all_of(root, ElementKind::Heading(2)).len(), 4);
    assert_eq!(all_of(root, ElementKind::Heading(3)).len(), 1);
    assert_eq!(all_of(root, ElementKind::UnorderedList).len(), 1);
    assert_eq!(all_of(root, ElementKind::OrderedList).len(), 1);
    assert_eq!(all_of(root, ElementKind::ListItem).len(), 4);
    assert_eq!(all_of(root, ElementKind::Table).len(), 1);
    assert_eq!(all_of(root, ElementKind::Link).len(), 2);
    assert!(!all_of(root, ElementKind::Paragraph).is_empty());

    let code = first_of(root, ElementKind::Code);
    assert_eq!(code.attrs.lang.as_deref(), Some("gherkin"));
    assert_eq!((code.start_line, code.end_line), (29, 33));
    assert!(code.text.starts_with("Given a feature index"));
}

/// List item text keeps its markdown markers so regexp assertions can match them.
#[test]
fn list_item_text_keeps_markdown_markers() {
    let doc = DocModel::parse(GOOD);
    let items = all_of(section(&doc.root, "Roles"), ElementKind::ListItem);

    assert_eq!(items[0].text, "**Admin** — manages the index");
    assert_eq!(items[0].start_line, 9);
    assert_eq!(items[1].text, "**Visitor** — browses features");
}
