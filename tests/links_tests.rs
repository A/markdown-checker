//! `wiki:` and `resolve:` — the LNK- scenarios of specs/scenarios.md plus the
//! resolution rules of src/links.rs.

mod support;

use std::path::Path;

use mdcheck::finding::Finding;
use mdcheck::model::DocModel;
use support::{reported, rule_set, scratch, write};

/// Runs a bare rule list against markdown with an explicit link root.
fn check_in(root: &Path, rules: &str, markdown: &str) -> Vec<Finding> {
    let doc = DocModel::parse(markdown);
    mdcheck::assert::run_with_root(&rule_set(rules), &doc, "doc.md", root)
}

const WIKI: &str = "- select: A\n  wiki: true";
const RESOLVE: &str = "- select: A[wiki]\n  resolve: true";
const RESOLVE_ALL: &str = "- select: A\n  resolve: true";

#[test]
fn lnk_1_all_links_are_wiki_links() {
    let dir = scratch("lnk_1");
    let findings = check_in(
        &dir,
        WIKI,
        "See [[features/x|X]] and [ext](https://e.com).\n",
    );
    assert_eq!(
        reported(&findings),
        ["1 — `[ext](https://e.com)` is not a wiki-link"]
    );
}

#[test]
fn lnk_2_wiki_link_resolution() {
    let dir = scratch("lnk_2");
    let findings = check_in(&dir, RESOLVE, "See [[features/x/stories|X]].\n");
    assert_eq!(
        reported(&findings),
        ["1 — target `features/x/stories.md` not found under root"]
    );
    assert_eq!(findings[0].actual, "features/x/stories.md");
}

#[test]
fn lnk_3_forbid_external_links() {
    let dir = scratch("lnk_3");
    let findings = check_in(
        &dir,
        "- select: 'A[href=/^https?:/]'\n  count: 0",
        "[a](https://e.com) and [b](http://f.com) and [[c]].\n",
    );
    assert_eq!(
        reported(&findings),
        ["1 — `A[href=/^https?:/]`: 2 matches, expected exactly 0"]
    );
}

#[test]
fn dod_wiki_link_resolves_when_the_target_file_exists_under_root() {
    let dir = scratch("lnk_resolve_hit");
    write(&dir, "features/x/stories.md", "# Stories\n");

    let findings = check_in(&dir, RESOLVE, "See [[features/x/stories|X]].\n");
    assert!(findings.is_empty(), "{:?}", reported(&findings));
}

#[test]
fn dod_wiki_target_keeps_an_explicit_extension() {
    let dir = scratch("lnk_extension");
    write(&dir, "notes/todo.txt", "x\n");

    let hit = check_in(&dir, RESOLVE, "See [[notes/todo.txt]].\n");
    assert!(hit.is_empty(), "{:?}", reported(&hit));

    let miss = check_in(&dir, RESOLVE, "See [[notes/todo.md]].\n");
    assert_eq!(miss[0].actual, "notes/todo.md");
}

#[test]
fn dod_anchor_is_stripped_before_resolution() {
    let dir = scratch("lnk_anchor");
    write(&dir, "guide.md", "# Guide\n");

    let findings = check_in(&dir, RESOLVE, "See [[guide#Setup|the setup]].\n");
    assert!(findings.is_empty(), "{:?}", reported(&findings));
}

#[test]
fn dod_markdown_links_resolve_against_the_root_as_written() {
    let dir = scratch("lnk_markdown");
    write(&dir, "sibling.md", "# Sibling\n");

    let hit = check_in(&dir, RESOLVE_ALL, "See [s](sibling.md).\n");
    assert!(hit.is_empty(), "{:?}", reported(&hit));

    let miss = check_in(&dir, RESOLVE_ALL, "See [s](other.md).\n");
    assert_eq!(
        reported(&miss),
        ["1 — target `other.md` not found under root"]
    );
}

#[test]
fn dod_external_and_fragment_targets_are_not_resolved_on_disk() {
    let dir = scratch("lnk_external");
    let findings = check_in(
        &dir,
        RESOLVE_ALL,
        "[a](https://e.com) [b](mailto:x@e.com) [c](#section)\n",
    );
    assert!(findings.is_empty(), "{:?}", reported(&findings));
}

#[test]
fn dod_wiki_over_a_fixture_with_one_markdown_link_yields_one_finding() {
    let dir = scratch("lnk_wiki_one");
    let markdown = "# T\n\n[[a|A]] [[b]]\n\n[md](link.md)\n\n[[c]]\n";
    let findings = check_in(&dir, WIKI, markdown);
    assert_eq!(findings.len(), 1, "{:?}", reported(&findings));
    assert_eq!(findings[0].line, 5);
    assert_eq!(findings[0].actual, "[md](link.md)");
}

#[test]
fn dod_resolve_reports_every_missing_target_not_just_the_first() {
    let dir = scratch("lnk_all_misses");
    let findings = check_in(&dir, RESOLVE, "[[one]]\n\n[[two]]\n");
    assert_eq!(
        reported(&findings),
        [
            "1 — target `one.md` not found under root",
            "3 — target `two.md` not found under root",
        ]
    );
}

#[test]
fn dod_root_flag_moves_the_resolution_base() {
    let dir = scratch("lnk_root_flag");
    write(&dir, "vault/features/x/stories.md", "# Stories\n");

    let outside = check_in(&dir, RESOLVE, "[[features/x/stories|X]]\n");
    assert_eq!(outside.len(), 1, "{:?}", reported(&outside));

    let inside = check_in(&dir.join("vault"), RESOLVE, "[[features/x/stories|X]]\n");
    assert!(inside.is_empty(), "{:?}", reported(&inside));
}
