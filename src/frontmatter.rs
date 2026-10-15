//! Frontmatter pre-pass: strip a leading `---` block and parse flat `key: value`
//! pairs before comrak sees the document body.

use indexmap::IndexMap;
use thiserror::Error;

/// Parsed frontmatter block. Values are plain strings by design — nested YAML is
/// not supported in frontmatter. The one structure recognised is a sequence of
/// scalars, flow (`tags: [a, b]`) or block (`tags:` followed by `- a` lines); its
/// items land in `lists` while `values` keeps the raw text (empty for a block).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub present: bool,
    pub values: IndexMap<String, String>,
    /// Keys whose value is a sequence, with their items in source order.
    pub lists: IndexMap<String, Vec<ListItem>>,
    /// Source line of each key, for findings.
    pub key_lines: IndexMap<String, usize>,
    /// Line of the opening `---` (0 when absent).
    pub start_line: usize,
    /// Line of the closing `---` (0 when absent).
    pub end_line: usize,
}

impl Frontmatter {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn line_of(&self, key: &str) -> Option<usize> {
        self.key_lines.get(key).copied()
    }

    /// Items of a sequence value; `None` when the key is absent or not a list.
    pub fn list(&self, key: &str) -> Option<&[ListItem]> {
        self.lists.get(key).map(Vec::as_slice)
    }
}

/// One item of a frontmatter sequence and the line it sits on (the key's line
/// for a flow sequence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FrontmatterError {
    #[error("unclosed frontmatter block")]
    Unclosed { line: usize },
}

/// Result of the pre-pass: the frontmatter, the remaining body, and the number of
/// lines stripped ahead of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepass<'a> {
    pub frontmatter: Frontmatter,
    pub body: &'a str,
    /// Lines removed from the front of the source; body line 1 is source line
    /// `line_offset + 1`.
    pub line_offset: usize,
    pub error: Option<FrontmatterError>,
}

/// Split a source document into frontmatter and body. Never panics: a malformed
/// block yields an error value and the whole source as body.
pub fn split(source: &str) -> Prepass<'_> {
    let no_frontmatter = || Prepass {
        frontmatter: Frontmatter::default(),
        body: source,
        line_offset: 0,
        error: None,
    };

    let mut lines = LineScanner::new(source);
    match lines.next() {
        Some((line, _)) if line.trim_end() == "---" => {}
        _ => return no_frontmatter(),
    }

    let mut frontmatter = Frontmatter {
        present: true,
        start_line: 1,
        ..Frontmatter::default()
    };
    let mut line_no = 1;
    // Key whose empty value may open a block sequence on the following lines.
    let mut open_block: Option<String> = None;

    while let Some((line, rest)) = lines.next() {
        line_no += 1;
        if line.trim_end() == "---" {
            frontmatter.end_line = line_no;
            return Prepass {
                frontmatter,
                body: rest,
                line_offset: line_no,
                error: None,
            };
        }
        if let Some(key) = &open_block {
            if let Some(item) = block_item(line) {
                frontmatter
                    .lists
                    .entry(key.clone())
                    .or_default()
                    .push(ListItem {
                        value: unquote(item).to_string(),
                        line: line_no,
                    });
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            open_block = None;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            if !key.is_empty() {
                let value = value.trim();
                if value.is_empty() {
                    open_block = Some(key.clone());
                } else if let Some(items) = flow_sequence(value) {
                    let items = items
                        .into_iter()
                        .map(|value| ListItem {
                            value,
                            line: line_no,
                        })
                        .collect();
                    frontmatter.lists.insert(key.clone(), items);
                }
                frontmatter.values.insert(key.clone(), value.to_string());
                frontmatter.key_lines.insert(key, line_no);
            }
        }
    }

    Prepass {
        frontmatter: Frontmatter::default(),
        body: source,
        line_offset: 0,
        error: Some(FrontmatterError::Unclosed { line: 1 }),
    }
}

/// `- item` (or a bare `-`) of a block sequence, at any indentation.
fn block_item(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed == "-" {
        return Some("");
    }
    trimmed.strip_prefix("- ").map(str::trim)
}

/// Items of a flow sequence `[a, "b, c", d]`; `None` when the value is not one.
/// Commas inside quotes do not split; a trailing comma adds no empty item.
fn flow_sequence(value: &str) -> Option<Vec<String>> {
    let inner = value.strip_prefix('[')?.strip_suffix(']')?;
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for c in inner.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => {
                quote = None;
                current.push(c);
            }
            (None, '"' | '\'') => {
                quote = Some(c);
                current.push(c);
            }
            (None, ',') => items.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    items.push(current);
    let mut items: Vec<String> = items
        .iter()
        .map(|item| unquote(item.trim()).to_string())
        .collect();
    if items.last().is_some_and(String::is_empty) {
        items.pop();
    }
    Some(items)
}

/// Strips one pair of matching surrounding quotes.
fn unquote(value: &str) -> &str {
    for q in ['"', '\''] {
        if value.len() >= 2 && value.starts_with(q) && value.ends_with(q) {
            return &value[1..value.len() - 1];
        }
    }
    value
}

/// Yields each line together with the remainder of the source after it.
struct LineScanner<'a> {
    rest: &'a str,
}

impl<'a> LineScanner<'a> {
    fn new(source: &'a str) -> Self {
        LineScanner { rest: source }
    }

    fn next(&mut self) -> Option<(&'a str, &'a str)> {
        if self.rest.is_empty() {
            return None;
        }
        let (line, rest) = match self.rest.find('\n') {
            Some(idx) => (&self.rest[..idx], &self.rest[idx + 1..]),
            None => (self.rest, ""),
        };
        self.rest = rest;
        Some((line, rest))
    }
}
