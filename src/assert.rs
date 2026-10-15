//! Assertions and the runner.
//!
//! Every assertion key of a rule compiles to one boxed [`Assertion`]; the runner
//! applies all rules and collects all findings — never first-fail.

use std::fmt;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::finding::Finding;
use crate::frontmatter::ListItem;
use crate::model::{DocModel, ElementKind, Node};
use crate::rules::{ListSpec, Rule, RuleSet, ValueSpec};
use crate::selector::{select, select_in, Match};

/// What an assertion needs beyond its match set: the document, the finding
/// identity, and the scope the matches were selected from (the document root, or
/// an outer match inside `each:`).
pub struct Ctx<'a> {
    pub doc: &'a DocModel,
    pub file: &'a str,
    pub check: &'a str,
    /// Selector source, quoted in count-family messages.
    pub selector: &'a str,
    pub scope: &'a Node,
    /// Base directory link targets resolve against.
    pub root: &'a Path,
}

impl Ctx<'_> {
    pub(crate) fn finding(
        &self,
        line: usize,
        expected: impl Into<String>,
        actual: impl Into<String>,
        message: impl Into<String>,
    ) -> Finding {
        Finding::new(self.check, self.file, line, expected, actual, message)
    }
}

pub trait Assertion: fmt::Debug + Send + Sync {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding>;
}

/// Applies every rule of `rules` to `doc`, resolving link targets against
/// `file`'s own directory.
pub fn run(rules: &RuleSet, doc: &DocModel, file: &str) -> Vec<Finding> {
    run_with_root(rules, doc, file, &own_directory(file))
}

fn own_directory(file: &str) -> PathBuf {
    match Path::new(file).parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Applies every rule of `rules` to `doc`, collecting all findings. Load-stage
/// findings come first and never abort the run.
pub fn run_with_root(rules: &RuleSet, doc: &DocModel, file: &str, root: &Path) -> Vec<Finding> {
    let mut findings: Vec<Finding> = doc
        .load_errors
        .iter()
        .map(|error| {
            let crate::frontmatter::FrontmatterError::Unclosed { line } = error;
            Finding::new(
                "load",
                file,
                *line,
                "a parsable document",
                "malformed frontmatter",
                error.to_string(),
            )
        })
        .collect();

    for rule in &rules.rules {
        let matches = select(doc, &rule.selector);
        let ctx = Ctx {
            doc,
            file,
            check: &rule.check,
            selector: &rule.selector_source,
            scope: &doc.root,
            root,
        };
        findings.extend(apply(rule, &matches, &ctx));
    }

    findings
}

fn apply(rule: &Rule, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
    if matches.is_empty() && rule.optional {
        return Vec::new();
    }
    rule.assertions
        .iter()
        .flat_map(|assertion| assertion.evaluate(matches, ctx))
        .collect()
}

fn quantity(n: usize, singular: &str, plural: &str) -> String {
    if n == 1 {
        format!("{n} {singular}")
    } else {
        format!("{n} {plural}")
    }
}

fn matches_word(n: usize) -> String {
    quantity(n, "match", "matches")
}

/// Value as it appears inside a message: single line, bounded length.
fn short(value: &str) -> String {
    let single = value.replace('\n', "⏎");
    let mut out: String = single.chars().take(60).collect();
    if single.chars().count() > 60 {
        out.push('…');
    }
    out
}

fn list(values: &[String]) -> String {
    values
        .iter()
        .map(|v| format!("`{v}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn label(node: &Node) -> &str {
    node.attrs.title.as_deref().unwrap_or(node.text.as_str())
}

#[derive(Debug)]
pub struct Count(pub usize);

impl Assertion for Count {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        if matches.len() == self.0 {
            return Vec::new();
        }
        vec![ctx.finding(
            ctx.scope.start_line,
            matches_word(self.0),
            matches_word(matches.len()),
            format!(
                "`{}`: {}, expected exactly {}",
                ctx.selector,
                matches_word(matches.len()),
                self.0
            ),
        )]
    }
}

#[derive(Debug)]
pub struct Min(pub usize);

impl Assertion for Min {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        if matches.len() >= self.0 {
            return Vec::new();
        }
        vec![ctx.finding(
            ctx.scope.start_line,
            format!("≥ {}", matches_word(self.0)),
            matches_word(matches.len()),
            format!(
                "`{}`: {}, expected ≥ {}",
                ctx.selector,
                matches_word(matches.len()),
                self.0
            ),
        )]
    }
}

#[derive(Debug)]
pub struct Max(pub usize);

impl Assertion for Max {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        if matches.len() <= self.0 {
            return Vec::new();
        }
        vec![ctx.finding(
            ctx.scope.start_line,
            format!("≤ {}", matches_word(self.0)),
            matches_word(matches.len()),
            format!(
                "`{}`: {}, expected ≤ {}",
                ctx.selector,
                matches_word(matches.len()),
                self.0
            ),
        )]
    }
}

#[derive(Debug)]
pub struct MinLines(pub usize);

impl Assertion for MinLines {
    fn evaluate(&self, _matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        let lines = ctx.doc.line_count;
        if lines >= self.0 {
            return Vec::new();
        }
        vec![ctx.finding(
            ctx.scope.start_line,
            format!("≥ {}", quantity(self.0, "line", "lines")),
            quantity(lines, "line", "lines"),
            format!("document has {lines} lines, expected ≥ {}", self.0),
        )]
    }
}

/// A compiled regexp together with the source the rule file spelled.
#[derive(Debug)]
pub struct Pattern {
    pub source: String,
    pub regex: Regex,
}

/// `match`: the whole value must match.
#[derive(Debug)]
pub struct MatchAll(pub Pattern);

impl Assertion for MatchAll {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| !self.0.regex.is_match(text))
            .map(|m| {
                ctx.finding(
                    m.start_line(),
                    format!("matches `{}`", self.0.source),
                    m.text(),
                    format!("`{}` does not match `{}`", short(m.text()), self.0.source),
                )
            })
            .collect()
    }
}

/// `require`: the pattern must appear somewhere in the value.
#[derive(Debug)]
pub struct Require(pub Pattern);

impl Assertion for Require {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| !self.0.regex.is_match(text))
            .map(|m| {
                ctx.finding(
                    m.start_line(),
                    format!("contains a match for `{}`", self.0.source),
                    m.text(),
                    format!("required pattern `{}` not found", self.0.source),
                )
            })
            .collect()
    }
}

/// `forbid`: the pattern must not appear.
#[derive(Debug)]
pub struct Forbid(pub Pattern);

impl Assertion for Forbid {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| self.0.regex.is_match(text))
            .map(|m| {
                ctx.finding(
                    m.start_line(),
                    format!("no match for `{}`", self.0.source),
                    m.text(),
                    format!("forbidden pattern `{}` found", self.0.source),
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct StartsWith(pub String);

impl Assertion for StartsWith {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| !text.starts_with(&self.0))
            .map(|m| {
                ctx.finding(
                    m.start_line(),
                    format!("starts with `{}`", self.0),
                    m.text(),
                    format!("`{}` does not start with `{}`", short(m.text()), self.0),
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct EndsWith(pub String);

impl Assertion for EndsWith {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| !text.ends_with(&self.0))
            .map(|m| {
                ctx.finding(
                    m.start_line(),
                    format!("ends with `{}`", self.0),
                    m.text(),
                    format!("`{}` does not end with `{}`", short(m.text()), self.0),
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct MinLength(pub usize);

impl Assertion for MinLength {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| text.chars().count() < self.0)
            .map(|m| {
                let length = m.text().chars().count();
                ctx.finding(
                    m.start_line(),
                    format!("≥ {}", quantity(self.0, "character", "characters")),
                    quantity(length, "character", "characters"),
                    format!("length {length}, expected ≥ {}", self.0),
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct MaxLength(pub usize);

impl Assertion for MaxLength {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| text.chars().count() > self.0)
            .map(|m| {
                let length = m.text().chars().count();
                ctx.finding(
                    m.start_line(),
                    format!("≤ {}", quantity(self.0, "character", "characters")),
                    quantity(length, "character", "characters"),
                    format!("length {length}, expected ≤ {}", self.0),
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct EnumSet(pub Vec<String>);

impl Assertion for EnumSet {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| !self.0.iter().any(|v| v == text))
            .map(|m| {
                ctx.finding(
                    m.start_line(),
                    format!("one of {}", list(&self.0)),
                    m.text(),
                    format!(
                        "`{}` is not allowed, allowed: {}",
                        short(m.text()),
                        list(&self.0)
                    ),
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct NoEmpty;

impl Assertion for NoEmpty {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        offenders(matches, |text| text.trim().is_empty())
            .map(|m| {
                let message = match (m.kind(), m.attrs().column.as_deref()) {
                    (ElementKind::TableCell, Some(column)) => {
                        format!("empty cell in column `{column}`")
                    }
                    _ => "empty value".to_string(),
                };
                ctx.finding(m.start_line(), "a non-empty value", m.text(), message)
            })
            .collect()
    }
}

fn offenders<'m, 'a>(
    matches: &'m [Match<'a>],
    failing: impl Fn(&str) -> bool + 'm,
) -> impl Iterator<Item = &'m Match<'a>> {
    matches.iter().filter(move |m| failing(m.text()))
}

#[derive(Debug)]
pub struct OrderedEntry {
    pub name: String,
    pub optional: bool,
}

/// `ordered:` plus its `strict:` companion.
#[derive(Debug)]
pub struct Ordered {
    pub entries: Vec<OrderedEntry>,
    pub strict: bool,
}

impl Assertion for Ordered {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        let labels: Vec<&str> = matches.iter().map(|m| label(m.node())).collect();
        let expected = self
            .entries
            .iter()
            .map(|e| e.name.clone())
            .collect::<Vec<_>>();
        let mut findings = Vec::new();

        for entry in &self.entries {
            if entry.optional || labels.contains(&entry.name.as_str()) {
                continue;
            }
            findings.push(ctx.finding(
                ctx.scope.start_line,
                format!("section `{}`", entry.name),
                "missing",
                format!("required section `{}` missing", entry.name),
            ));
        }

        let mut previous: Option<(&str, usize)> = None;
        for entry in &self.entries {
            let Some(position) = labels.iter().position(|l| *l == entry.name) else {
                continue;
            };
            if let Some((previous_name, previous_position)) = previous {
                if position < previous_position {
                    findings.push(ctx.finding(
                        matches[previous_position].start_line(),
                        format!("`{previous_name}` before `{}`", entry.name),
                        format!("`{previous_name}` after `{}`", entry.name),
                        format!(
                            "section order: `{previous_name}` appears after `{}`, expected before",
                            entry.name
                        ),
                    ));
                }
            }
            previous = Some((entry.name.as_str(), position));
        }

        if self.strict {
            for m in matches {
                let name = label(m.node());
                if self.entries.iter().any(|e| e.name == name) {
                    continue;
                }
                findings.push(ctx.finding(
                    m.start_line(),
                    format!("one of {}", list(&expected)),
                    name,
                    format!("section `{name}` not allowed (strict)"),
                ));
            }
        }

        findings
    }
}

#[derive(Debug)]
pub struct FrontmatterHas(pub Vec<(String, ValueSpec)>);

impl Assertion for FrontmatterHas {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        if matches.is_empty() {
            return vec![ctx.finding(
                ctx.scope.start_line,
                "frontmatter block",
                "missing",
                "frontmatter block missing",
            )];
        }

        let frontmatter = &ctx.doc.frontmatter;
        let mut findings = Vec::new();
        for (key, spec) in &self.0 {
            let Some(value) = frontmatter.get(key) else {
                findings.push(ctx.finding(
                    frontmatter.start_line.max(1),
                    format!("key `{key}`"),
                    "missing",
                    format!("key `{key}` missing"),
                ));
                continue;
            };
            let line = frontmatter.line_of(key).unwrap_or(frontmatter.start_line);
            match spec {
                ValueSpec::Present => {}
                ValueSpec::Enum(allowed) if frontmatter.list(key).is_some() => {
                    let items = frontmatter.list(key).unwrap_or_default();
                    findings.extend(list_items_outside(key, items, allowed, ctx));
                }
                ValueSpec::List(spec) => {
                    let Some(items) = frontmatter.list(key) else {
                        findings.push(ctx.finding(
                            line,
                            "a list",
                            value,
                            format!("`{key}` value `{}` is not a list", short(value)),
                        ));
                        continue;
                    };
                    if let Some(finding) = list_length(key, items.len(), spec, line, ctx) {
                        findings.push(finding);
                    }
                    if let Some(allowed) = &spec.allowed {
                        findings.extend(list_items_outside(key, items, allowed, ctx));
                    }
                }
                ValueSpec::Enum(allowed) => {
                    if !allowed.iter().any(|a| a == value) {
                        findings.push(ctx.finding(
                            line,
                            format!("one of {}", list(allowed)),
                            value,
                            format!("`{key}` is `{value}`, allowed: {}", list(allowed)),
                        ));
                    }
                }
                ValueSpec::Matches(pattern) => {
                    if !pattern.regex.is_match(value) {
                        findings.push(ctx.finding(
                            line,
                            format!("`{key}` matches `{}`", pattern.source),
                            value,
                            format!(
                                "`{key}` value `{value}` does not match `{}`",
                                pattern.source
                            ),
                        ));
                    }
                }
            }
        }
        findings
    }
}

/// Length of a frontmatter list against `min` / `max` (`count` arrives as both).
fn list_length(
    key: &str,
    len: usize,
    spec: &ListSpec,
    line: usize,
    ctx: &Ctx<'_>,
) -> Option<Finding> {
    let items = |n| quantity(n, "item", "items");
    let (op, bound) = match (spec.min, spec.max) {
        (Some(min), Some(max)) if min == max && len != min => ("exactly ", min),
        (Some(min), _) if len < min => ("≥ ", min),
        (_, Some(max)) if len > max => ("≤ ", max),
        _ => return None,
    };
    Some(ctx.finding(
        line,
        format!("{op}{}", items(bound)),
        items(len),
        format!("`{key}` has {}, expected {op}{bound}", items(len)),
    ))
}

/// One finding per list item outside the allowed set, at the item's own line.
fn list_items_outside(
    key: &str,
    items: &[ListItem],
    allowed: &[String],
    ctx: &Ctx<'_>,
) -> Vec<Finding> {
    items
        .iter()
        .filter(|item| !allowed.contains(&item.value))
        .map(|item| {
            ctx.finding(
                item.line,
                format!("one of {}", list(allowed)),
                item.value.as_str(),
                format!(
                    "`{key}` item `{}` not allowed, allowed: {}",
                    short(&item.value),
                    list(allowed)
                ),
            )
        })
        .collect()
}

#[derive(Debug)]
pub struct FrontmatterNotHas(pub Vec<String>);

impl Assertion for FrontmatterNotHas {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        if matches.is_empty() {
            return Vec::new();
        }
        let frontmatter = &ctx.doc.frontmatter;
        self.0
            .iter()
            .filter(|key| frontmatter.get(key).is_some())
            .map(|key| {
                ctx.finding(
                    frontmatter.line_of(key).unwrap_or(frontmatter.start_line),
                    format!("no `{key}` key"),
                    format!("`{key}` present"),
                    format!("key `{key}` must not be present"),
                )
            })
            .collect()
    }
}

/// `columns:` — header set and order, plus per-row cell-count consistency.
#[derive(Debug)]
pub struct Columns(pub Vec<String>);

impl Assertion for Columns {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        let mut findings = Vec::new();
        for table in matches.iter().filter(|m| m.kind() == ElementKind::Table) {
            let Some(header) = header_row(table.node()) else {
                continue;
            };
            let actual: Vec<String> = header.children.iter().map(|c| c.text.clone()).collect();
            let expected_line = format!("columns `{}`", self.0.join(", "));
            let actual_line = format!("columns `{}`", actual.join(", "));

            let missing: Vec<&String> = self
                .0
                .iter()
                .filter(|column| !actual.contains(column))
                .collect();

            if missing.is_empty() {
                if actual != self.0 {
                    findings.push(ctx.finding(
                        header.start_line,
                        expected_line.clone(),
                        actual_line.clone(),
                        format!(
                            "column order `{}`, expected `{}`",
                            actual.join(", "),
                            self.0.join(", ")
                        ),
                    ));
                }
            } else {
                for column in missing {
                    findings.push(ctx.finding(
                        header.start_line,
                        expected_line.clone(),
                        actual_line.clone(),
                        format!("column `{column}` missing"),
                    ));
                }
            }

            let width = actual.len();
            for row in body_rows(table.node()) {
                let Some(cells) = source_cell_count(ctx.doc, row.start_line) else {
                    continue;
                };
                if cells != width {
                    findings.push(ctx.finding(
                        row.start_line,
                        quantity(width, "cell", "cells"),
                        quantity(cells, "cell", "cells"),
                        format!(
                            "row has {}, expected {width}",
                            quantity(cells, "cell", "cells")
                        ),
                    ));
                }
            }
        }
        findings
    }
}

#[derive(Debug)]
pub struct Rows {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl Assertion for Rows {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        let mut findings = Vec::new();
        for table in matches.iter().filter(|m| m.kind() == ElementKind::Table) {
            let rows = body_rows(table.node()).count();
            let actual = quantity(rows, "body row", "body rows");
            if let Some(min) = self.min.filter(|min| rows < *min) {
                findings.push(ctx.finding(
                    table.start_line(),
                    format!("≥ {}", quantity(min, "body row", "body rows")),
                    actual.clone(),
                    format!("table has {actual}, expected ≥ {min}"),
                ));
            }
            if let Some(max) = self.max.filter(|max| rows > *max) {
                findings.push(ctx.finding(
                    table.start_line(),
                    format!("≤ {}", quantity(max, "body row", "body rows")),
                    actual.clone(),
                    format!("table has {actual}, expected ≤ {max}"),
                ));
            }
        }
        findings
    }
}

fn header_row(table: &Node) -> Option<&Node> {
    table.children.iter().find(|row| row.attrs.header_row)
}

fn body_rows(table: &Node) -> impl Iterator<Item = &Node> {
    table
        .children
        .iter()
        .filter(|row| row.kind == ElementKind::TableRow && !row.attrs.header_row)
}

/// Cells as the source line spells them — comrak pads and truncates rows to the
/// header width, so raggedness is only visible in the source.
fn source_cell_count(doc: &DocModel, line: usize) -> Option<usize> {
    let raw = doc.root.text.lines().nth(line.checked_sub(1)?)?.trim();
    if !raw.contains('|') {
        return None;
    }
    let inner = raw.strip_prefix('|').unwrap_or(raw);
    let inner = inner.strip_suffix('|').unwrap_or(inner);

    let mut cells = 1;
    let mut escaped = false;
    for c in inner.chars() {
        match c {
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            '|' => cells += 1,
            _ => {}
        }
    }
    Some(cells)
}

/// `each:` — inner rules re-rooted at every outer match.
#[derive(Debug)]
pub struct Each(pub Vec<Rule>);

impl Assertion for Each {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        let mut findings = Vec::new();
        for outer in matches {
            for rule in &self.0 {
                let inner = select_in(outer.node(), &rule.selector);
                let check = format!("{}.{}@{}", ctx.check, rule.check, outer.start_line());
                let inner_ctx = Ctx {
                    doc: ctx.doc,
                    file: ctx.file,
                    check: &check,
                    selector: &rule.selector_source,
                    scope: outer.node(),
                    root: ctx.root,
                };
                findings.extend(apply(rule, &inner, &inner_ctx));
            }
        }
        findings
    }
}
