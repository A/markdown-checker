//! Frontmatter pre-pass: strip a leading `---` block and parse flat `key: value`
//! pairs before comrak sees the document body.

use indexmap::IndexMap;
use thiserror::Error;

/// Parsed frontmatter block. Values are plain strings by design — nested YAML is
/// not supported in frontmatter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frontmatter {
    pub present: bool,
    pub values: IndexMap<String, String>,
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
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            if !key.is_empty() {
                frontmatter
                    .values
                    .insert(key.clone(), value.trim().to_string());
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
