//! Selector string → `Selector` AST.

use std::fmt;

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Element {
    Document,
    Frontmatter,
    Heading(u8),
    Section,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    Child,
    Descendant,
    Adjacent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttrKey {
    Text,
    Title,
    Col,
    Href,
    Lang,
    Wiki,
}

/// A compiled regexp; equality is by source so ASTs stay comparable.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub source: String,
    pub regex: Regex,
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for Pattern {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Test {
    Present,
    Equals(String),
    StartsWith(String),
    EndsWith(String),
    Contains(String),
    Matches(Pattern),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    pub key: AttrKey,
    pub test: Test,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compound {
    pub element: Element,
    pub filters: Vec<Filter>,
    /// Bare-integer filter, applied after the match set is collected.
    pub index: Option<isize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selector {
    pub head: Compound,
    pub tail: Vec<(Combinator, Compound)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    /// 1-based character column of the offending token.
    pub column: usize,
    pub input: String,
}

pub fn parse(input: &str) -> Result<Selector, ParseError> {
    let mut parser = Parser {
        input,
        chars: input.chars().collect(),
        at: 0,
    };
    parser.selector()
}

impl std::str::FromStr for Selector {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Selector, ParseError> {
        parse(input)
    }
}

struct Parser<'a> {
    input: &'a str,
    chars: Vec<char>,
    at: usize,
}

impl Parser<'_> {
    fn selector(&mut self) -> Result<Selector, ParseError> {
        self.skip_spaces();
        let head = self.compound()?;
        let mut tail = Vec::new();

        loop {
            let spaced = self.skip_spaces();
            let Some(c) = self.peek() else { break };
            let combinator = match c {
                '>' => {
                    self.bump();
                    self.skip_spaces();
                    Combinator::Child
                }
                '+' => {
                    self.bump();
                    self.skip_spaces();
                    Combinator::Adjacent
                }
                c if c.is_ascii_alphabetic() && spaced => Combinator::Descendant,
                c => return Err(self.error(format!("unexpected character `{c}`"))),
            };
            tail.push((combinator, self.compound()?));
        }

        Ok(Selector { head, tail })
    }

    fn compound(&mut self) -> Result<Compound, ParseError> {
        let start = self.at;
        let name: String = self.take_while(|c| c.is_ascii_alphanumeric());
        if name.is_empty() {
            return Err(self.error(match self.peek() {
                Some(c) => format!("expected an element name, found `{c}`"),
                None => "expected an element name".to_string(),
            }));
        }

        let element = match name.as_str() {
            "DOCUMENT" => Element::Document,
            "FRONTMATTER" => Element::Frontmatter,
            "SECTION" => Element::Section,
            "P" => Element::Paragraph,
            "UL" => Element::UnorderedList,
            "OL" => Element::OrderedList,
            "LI" => Element::ListItem,
            "TABLE" => Element::Table,
            "TR" => Element::TableRow,
            "TD" => Element::TableCell,
            "A" => Element::Link,
            "CODE" => Element::Code,
            _ => match name.strip_prefix('H').and_then(|l| l.parse::<u8>().ok()) {
                Some(level @ 1..=6) if name.len() == 2 => Element::Heading(level),
                _ => return Err(self.error_at(start, format!("unknown element `{name}`"))),
            },
        };

        let mut compound = Compound {
            element,
            filters: Vec::new(),
            index: None,
        };

        loop {
            match self.peek() {
                Some('(') => {
                    self.bump();
                    let key = match element {
                        Element::Section => AttrKey::Title,
                        _ => AttrKey::Text,
                    };
                    let test = match self.peek() {
                        Some('"') | Some('\'') => Test::Equals(self.string()?),
                        Some('/') => Test::Matches(self.pattern()?),
                        _ => {
                            return Err(self.error("expected a quoted string or /regexp/"));
                        }
                    };
                    self.expect(')')?;
                    compound.filters.push(Filter { key, test });
                }
                Some('[') => {
                    self.bump();
                    match self.peek() {
                        Some(c) if c == '-' || c.is_ascii_digit() => {
                            compound.index = Some(self.integer()?);
                        }
                        _ => compound.filters.push(self.attribute()?),
                    }
                    self.expect(']')?;
                }
                _ => break,
            }
        }

        Ok(compound)
    }

    fn attribute(&mut self) -> Result<Filter, ParseError> {
        let start = self.at;
        let name: String = self.take_while(|c| c.is_ascii_alphabetic());
        let key = match name.as_str() {
            "text" => AttrKey::Text,
            "title" => AttrKey::Title,
            "col" => AttrKey::Col,
            "href" => AttrKey::Href,
            "lang" => AttrKey::Lang,
            "wiki" => AttrKey::Wiki,
            "" => return Err(self.error("expected a filter name")),
            _ => return Err(self.error_at(start, format!("unknown filter `{name}`"))),
        };

        if self.peek() == Some(']') {
            return Ok(Filter {
                key,
                test: Test::Present,
            });
        }

        let op = match (self.peek(), self.peek_at(1)) {
            (Some('='), _) => {
                self.bump();
                '='
            }
            (Some(c @ ('^' | '$' | '*')), Some('=')) => {
                self.bump();
                self.bump();
                c
            }
            _ => {
                return Err(self.error("expected `=`, `^=`, `$=`, `*=` or `]`"));
            }
        };

        let test = match self.peek() {
            Some('/') => {
                if op != '=' {
                    return Err(self.error(format!("`{op}=` does not accept a regexp value")));
                }
                Test::Matches(self.pattern()?)
            }
            Some('"') | Some('\'') => {
                let value = self.string()?;
                match op {
                    '^' => Test::StartsWith(value),
                    '$' => Test::EndsWith(value),
                    '*' => Test::Contains(value),
                    _ => Test::Equals(value),
                }
            }
            _ => return Err(self.error("expected a quoted string or /regexp/")),
        };

        Ok(Filter { key, test })
    }

    fn string(&mut self) -> Result<String, ParseError> {
        let quote = self.bump().expect("caller checked the quote");
        let start = self.at;
        let mut out = String::new();
        loop {
            match self.bump() {
                Some('\\') => match self.bump() {
                    Some(c) => out.push(c),
                    None => return Err(self.error_at(start, "unterminated string")),
                },
                Some(c) if c == quote => return Ok(out),
                Some(c) => out.push(c),
                None => return Err(self.error_at(start, "unterminated string")),
            }
        }
    }

    fn pattern(&mut self) -> Result<Pattern, ParseError> {
        self.bump();
        let start = self.at;
        let mut source = String::new();
        let closed = loop {
            match self.bump() {
                Some('\\') => match self.bump() {
                    Some('/') => source.push('/'),
                    Some(c) => {
                        source.push('\\');
                        source.push(c);
                    }
                    None => break false,
                },
                Some('/') => break true,
                Some(c) => source.push(c),
                None => break false,
            }
        };

        if !closed {
            return Err(self.error_at(start, "unterminated regexp"));
        }

        match Regex::new(&source) {
            Ok(regex) => Ok(Pattern { source, regex }),
            Err(err) => Err(self.error_at(
                start,
                format!("invalid regexp `{source}`: {}", one_line(&err.to_string())),
            )),
        }
    }

    fn integer(&mut self) -> Result<isize, ParseError> {
        let start = self.at;
        let mut raw = String::new();
        if self.peek() == Some('-') {
            raw.push(self.bump().expect("peeked"));
        }
        raw.push_str(&self.take_while(|c| c.is_ascii_digit()));
        raw.parse()
            .map_err(|_| self.error_at(start, format!("invalid index `{raw}`")))
    }

    fn expect(&mut self, expected: char) -> Result<(), ParseError> {
        if self.peek() == Some(expected) {
            self.bump();
            return Ok(());
        }
        Err(self.error(format!("expected `{expected}`")))
    }

    fn peek(&self) -> Option<char> {
        self.peek_at(0)
    }

    fn peek_at(&self, ahead: usize) -> Option<char> {
        self.chars.get(self.at + ahead).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.at += 1;
        Some(c)
    }

    fn take_while(&mut self, accept: impl Fn(char) -> bool) -> String {
        let mut out = String::new();
        while let Some(c) = self.peek() {
            if !accept(c) {
                break;
            }
            out.push(c);
            self.at += 1;
        }
        out
    }

    fn skip_spaces(&mut self) -> bool {
        !self.take_while(char::is_whitespace).is_empty()
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        self.error_at(self.at, message)
    }

    fn error_at(&self, at: usize, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            column: at + 1,
            input: self.input.to_string(),
        }
    }
}

fn one_line(message: &str) -> String {
    message
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Element::Document => write!(f, "DOCUMENT"),
            Element::Frontmatter => write!(f, "FRONTMATTER"),
            Element::Heading(level) => write!(f, "H{level}"),
            Element::Section => write!(f, "SECTION"),
            Element::Paragraph => write!(f, "P"),
            Element::UnorderedList => write!(f, "UL"),
            Element::OrderedList => write!(f, "OL"),
            Element::ListItem => write!(f, "LI"),
            Element::Table => write!(f, "TABLE"),
            Element::TableRow => write!(f, "TR"),
            Element::TableCell => write!(f, "TD"),
            Element::Link => write!(f, "A"),
            Element::Code => write!(f, "CODE"),
        }
    }
}

impl fmt::Display for AttrKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            AttrKey::Text => "text",
            AttrKey::Title => "title",
            AttrKey::Col => "col",
            AttrKey::Href => "href",
            AttrKey::Lang => "lang",
            AttrKey::Wiki => "wiki",
        };
        f.write_str(name)
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.test {
            Test::Present => write!(f, "[{}]", self.key),
            Test::Equals(v) => write!(f, "[{}=\"{}\"]", self.key, escape(v)),
            Test::StartsWith(v) => write!(f, "[{}^=\"{}\"]", self.key, escape(v)),
            Test::EndsWith(v) => write!(f, "[{}$=\"{}\"]", self.key, escape(v)),
            Test::Contains(v) => write!(f, "[{}*=\"{}\"]", self.key, escape(v)),
            Test::Matches(p) => write!(f, "[{}=/{}/]", self.key, p.source),
        }
    }
}

impl fmt::Display for Compound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.element)?;
        for filter in &self.filters {
            write!(f, "{filter}")?;
        }
        if let Some(index) = self.index {
            write!(f, "[{index}]")?;
        }
        Ok(())
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.head)?;
        for (combinator, compound) in &self.tail {
            let sep = match combinator {
                Combinator::Child => " > ",
                Combinator::Descendant => " ",
                Combinator::Adjacent => " + ",
            };
            write!(f, "{sep}{compound}")?;
        }
        Ok(())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pad = " ".repeat(self.column.saturating_sub(1));
        write!(
            f,
            "invalid selector: {} (column {})\n  {}\n  {pad}^",
            self.message, self.column, self.input
        )
    }
}

impl std::error::Error for ParseError {}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
