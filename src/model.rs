//! Fully-owned document model built from the comrak AST.
//!
//! The comrak arena is consumed at construction time; nothing in this module
//! borrows from it. Line numbers already carry the frontmatter offset.

use std::fmt;

use comrak::nodes::{AstNode, ListType, NodeValue, Sourcepos};
use comrak::{parse_document, Arena, Options};

use crate::frontmatter::{self, Frontmatter, FrontmatterError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Document,
    Frontmatter,
    /// `H1`..`H6`
    Heading(u8),
    /// A heading plus everything up to the next same-or-higher heading.
    Section(u8),
    Paragraph,
    UnorderedList,
    OrderedList,
    ListItem,
    Table,
    TableRow,
    TableCell,
    Link,
    Code,
}

impl fmt::Display for ElementKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementKind::Document => write!(f, "DOCUMENT"),
            ElementKind::Frontmatter => write!(f, "FRONTMATTER"),
            ElementKind::Heading(level) => write!(f, "H{level}"),
            ElementKind::Section(_) => write!(f, "SECTION"),
            ElementKind::Paragraph => write!(f, "P"),
            ElementKind::UnorderedList => write!(f, "UL"),
            ElementKind::OrderedList => write!(f, "OL"),
            ElementKind::ListItem => write!(f, "LI"),
            ElementKind::Table => write!(f, "TABLE"),
            ElementKind::TableRow => write!(f, "TR"),
            ElementKind::TableCell => write!(f, "TD"),
            ElementKind::Link => write!(f, "A"),
            ElementKind::Code => write!(f, "CODE"),
        }
    }
}

/// Element-specific attributes addressable by selectors.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attrs {
    /// `SECTION[title=…]`
    pub title: Option<String>,
    /// `TD[col=…]` — column name taken from the header row.
    pub column: Option<String>,
    /// Positional index of a `TD` within its row.
    pub column_index: Option<usize>,
    /// `A[href=…]`
    pub href: Option<String>,
    /// `A[wiki]`
    pub wiki: bool,
    /// `CODE[lang=…]`
    pub lang: Option<String>,
    /// True for the header `TR` of a table.
    pub header_row: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub kind: ElementKind,
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
    pub attrs: Attrs,
    pub children: Vec<Node>,
}

impl Node {
    /// Depth-first iterator over all descendants, excluding `self`.
    pub fn descendants(&self) -> Descendants<'_> {
        Descendants {
            stack: self.children.iter().rev().collect(),
        }
    }
}

pub struct Descendants<'a> {
    stack: Vec<&'a Node>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<&'a Node> {
        let node = self.stack.pop()?;
        self.stack.extend(node.children.iter().rev());
        Some(node)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocModel {
    /// `DOCUMENT` pseudo-node; its first child is the `FRONTMATTER` pseudo-node
    /// when a block is present.
    pub root: Node,
    pub frontmatter: Frontmatter,
    /// Line count of the whole source, frontmatter included.
    pub line_count: usize,
    pub load_errors: Vec<FrontmatterError>,
}

impl DocModel {
    pub fn parse(source: &str) -> DocModel {
        let prepass = frontmatter::split(source);
        let line_count = source.lines().count();

        let arena = Arena::new();
        let root = parse_document(&arena, prepass.body, &options());

        let builder = Builder {
            lines: prepass.body.lines().collect(),
            offset: prepass.line_offset,
        };

        let mut children = Vec::new();
        if prepass.frontmatter.present {
            children.push(Node {
                kind: ElementKind::Frontmatter,
                text: source
                    .lines()
                    .take(prepass.frontmatter.end_line)
                    .collect::<Vec<_>>()
                    .join("\n"),
                start_line: prepass.frontmatter.start_line,
                end_line: prepass.frontmatter.end_line,
                attrs: Attrs::default(),
                children: Vec::new(),
            });
        }
        children.extend(fold_sections(builder.blocks(root)));

        DocModel {
            root: Node {
                kind: ElementKind::Document,
                text: source.to_string(),
                start_line: 1,
                end_line: line_count.max(1),
                attrs: Attrs::default(),
                children,
            },
            frontmatter: prepass.frontmatter,
            line_count,
            load_errors: prepass.error.into_iter().collect(),
        }
    }

    /// Human-readable tree rendering, one node per line, for debugging and tests.
    pub fn dump_tree(&self) -> String {
        let mut out = String::new();
        render(&self.root, 0, &mut out);
        out
    }
}

fn options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.wikilinks_title_after_pipe = true;
    options.render.sourcepos = true;
    options
}

fn render(node: &Node, depth: usize, out: &mut String) {
    use std::fmt::Write;

    let indent = "  ".repeat(depth);
    let mut attrs = String::new();
    if let Some(title) = &node.attrs.title {
        let _ = write!(attrs, " title={title:?}");
    }
    if let Some(col) = &node.attrs.column {
        let _ = write!(attrs, " col={col:?}");
    }
    if let Some(href) = &node.attrs.href {
        let _ = write!(attrs, " href={href:?}");
    }
    if node.attrs.wiki {
        attrs.push_str(" wiki");
    }
    if let Some(lang) = &node.attrs.lang {
        let _ = write!(attrs, " lang={lang:?}");
    }
    if node.attrs.header_row {
        attrs.push_str(" header");
    }

    let _ = writeln!(
        out,
        "{indent}{} [{}-{}]{attrs} {:?}",
        node.kind,
        node.start_line,
        node.end_line,
        preview(&node.text)
    );
    for child in &node.children {
        render(child, depth + 1, out);
    }
}

fn preview(text: &str) -> String {
    let single = text.replace('\n', "⏎");
    let mut out: String = single.chars().take(48).collect();
    if single.chars().count() > 48 {
        out.push('…');
    }
    out
}

/// Groups a flat block sequence into nested `SECTION` nodes: a heading owns every
/// following sibling until the next same-or-higher heading.
fn fold_sections(blocks: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    let mut blocks = blocks.into_iter().peekable();

    while let Some(block) = blocks.next() {
        let ElementKind::Heading(level) = block.kind else {
            out.push(block);
            continue;
        };

        let mut body = Vec::new();
        while let Some(next) = blocks.peek() {
            if matches!(next.kind, ElementKind::Heading(l) if l <= level) {
                break;
            }
            body.push(blocks.next().expect("peeked"));
        }

        let title = block.text.clone();
        let start_line = block.start_line;
        let mut children = vec![block];
        children.extend(fold_sections(body));
        let end_line = children.last().map_or(start_line, |c| c.end_line);

        out.push(Node {
            kind: ElementKind::Section(level),
            text: join_text(&children),
            start_line,
            end_line,
            attrs: Attrs {
                title: Some(title),
                ..Attrs::default()
            },
            children,
        });
    }

    out
}

fn join_text(children: &[Node]) -> String {
    children
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

struct Builder<'a> {
    lines: Vec<&'a str>,
    offset: usize,
}

impl Builder<'_> {
    /// Converts the block-level children of `node`, flattening containers that the
    /// model does not model explicitly.
    fn blocks<'a>(&self, node: &'a AstNode<'a>) -> Vec<Node> {
        node.children().flat_map(|c| self.block(c)).collect()
    }

    fn block<'a>(&self, node: &'a AstNode<'a>) -> Vec<Node> {
        let value = node.data.borrow().value.clone();
        let pos = node.data.borrow().sourcepos;

        let built = match value {
            NodeValue::Heading(heading) => Node {
                kind: ElementKind::Heading(heading.level),
                text: self.inline_text(node),
                start_line: self.line(pos.start.line),
                end_line: self.line(pos.end.line),
                attrs: Attrs::default(),
                children: self.links(node),
            },
            NodeValue::Paragraph => Node {
                kind: ElementKind::Paragraph,
                text: self.inline_text(node),
                start_line: self.line(pos.start.line),
                end_line: self.line(pos.end.line),
                attrs: Attrs::default(),
                children: self.links(node),
            },
            NodeValue::List(list) => {
                let children = self.blocks(node);
                Node {
                    kind: match list.list_type {
                        ListType::Bullet => ElementKind::UnorderedList,
                        ListType::Ordered => ElementKind::OrderedList,
                    },
                    text: join_text(&children),
                    start_line: self.line(pos.start.line),
                    end_line: self.line(pos.end.line),
                    attrs: Attrs::default(),
                    children,
                }
            }
            NodeValue::Item(_) => {
                let children = self.blocks(node);
                Node {
                    kind: ElementKind::ListItem,
                    text: join_text(&children),
                    start_line: self.line(pos.start.line),
                    end_line: self.line(pos.end.line),
                    attrs: Attrs::default(),
                    children,
                }
            }
            NodeValue::Table(_) => self.table(node, pos),
            NodeValue::CodeBlock(code) => Node {
                kind: ElementKind::Code,
                text: code.literal.trim_end_matches('\n').to_string(),
                start_line: self.line(pos.start.line),
                end_line: self.line(pos.end.line),
                attrs: Attrs {
                    lang: (!code.info.is_empty()).then(|| code.info.clone()),
                    ..Attrs::default()
                },
                children: Vec::new(),
            },
            _ => return self.blocks(node),
        };

        vec![built]
    }

    fn table<'a>(&self, node: &'a AstNode<'a>, pos: Sourcepos) -> Node {
        let mut columns: Vec<String> = Vec::new();
        let mut rows: Vec<Node> = Vec::new();

        for row in node.children() {
            let header = matches!(row.data.borrow().value, NodeValue::TableRow(true));
            let row_pos = row.data.borrow().sourcepos;
            let mut cells: Vec<Node> = Vec::new();

            for (index, cell) in row.children().enumerate() {
                let cell_pos = cell.data.borrow().sourcepos;
                let text = self.inline_text(cell);
                if header {
                    columns.push(text.clone());
                }
                cells.push(Node {
                    kind: ElementKind::TableCell,
                    text,
                    start_line: self.line(cell_pos.start.line),
                    end_line: self.line(cell_pos.end.line),
                    attrs: Attrs {
                        column: columns.get(index).cloned(),
                        column_index: Some(index),
                        ..Attrs::default()
                    },
                    children: self.links(cell),
                });
            }

            rows.push(Node {
                kind: ElementKind::TableRow,
                text: join_text(&cells),
                start_line: self.line(row_pos.start.line),
                end_line: self.line(row_pos.end.line),
                attrs: Attrs {
                    header_row: header,
                    ..Attrs::default()
                },
                children: cells,
            });
        }

        Node {
            kind: ElementKind::Table,
            text: join_text(&rows),
            start_line: self.line(pos.start.line),
            end_line: self.line(pos.end.line),
            attrs: Attrs::default(),
            children: rows,
        }
    }

    /// Collects `A` nodes from an inline subtree.
    fn links<'a>(&self, node: &'a AstNode<'a>) -> Vec<Node> {
        let mut out = Vec::new();
        for child in node.children() {
            let value = child.data.borrow().value.clone();
            let pos = child.data.borrow().sourcepos;
            match value {
                NodeValue::Link(link) => out.push(self.link(child, pos, link.url, false)),
                NodeValue::WikiLink(link) => out.push(self.link(child, pos, link.url, true)),
                _ => out.extend(self.links(child)),
            }
        }
        out
    }

    fn link<'a>(&self, node: &'a AstNode<'a>, pos: Sourcepos, url: String, wiki: bool) -> Node {
        Node {
            kind: ElementKind::Link,
            text: self.inline_text(node),
            start_line: self.line(pos.start.line),
            end_line: self.line(pos.end.line),
            attrs: Attrs {
                href: Some(url),
                wiki,
                ..Attrs::default()
            },
            children: Vec::new(),
        }
    }

    /// Source text of a node's inline content, markdown markers included: the span
    /// runs from the first to the last inline child, so container markers (`## `,
    /// `- `, `| `) stay out.
    fn inline_text<'a>(&self, node: &'a AstNode<'a>) -> String {
        let mut span: Option<Sourcepos> = None;
        for child in node.children() {
            let pos = child.data.borrow().sourcepos;
            span = Some(match span {
                None => pos,
                Some(mut acc) => {
                    if pos.end.line > acc.end.line
                        || (pos.end.line == acc.end.line && pos.end.column > acc.end.column)
                    {
                        acc.end = pos.end;
                    }
                    acc
                }
            });
        }

        let Some(span) = span else {
            return String::new();
        };
        self.slice(span)
    }

    fn slice(&self, span: Sourcepos) -> String {
        let mut out: Vec<String> = Vec::new();
        for line_no in span.start.line..=span.end.line {
            let Some(line) = self.lines.get(line_no - 1) else {
                continue;
            };
            let from = if line_no == span.start.line {
                span.start.column.saturating_sub(1)
            } else {
                0
            };
            let to = if line_no == span.end.line {
                span.end.column
            } else {
                line.len()
            };
            out.push(clamp_slice(line, from, to).to_string());
        }
        out.join("\n")
    }

    fn line(&self, line: usize) -> usize {
        line + self.offset
    }
}

/// Byte-range slice clamped to the line and to char boundaries.
fn clamp_slice(line: &str, from: usize, to: usize) -> &str {
    let mut from = from.min(line.len());
    let mut to = to.clamp(from, line.len());
    while !line.is_char_boundary(from) {
        from += 1;
    }
    while !line.is_char_boundary(to) {
        to += 1;
    }
    &line[from..to]
}
