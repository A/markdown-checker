//! Rule files: YAML schema, loader and compile pass.
//!
//! Loading runs in two phases. Phase one deserializes the file into raw structs
//! with `deny_unknown_fields`, preceded by a key scan that reports an unknown key
//! together with the rule it sits in. Phase two compiles: selectors are parsed,
//! regexps compiled and enum sets built, so a rule that loads always runs.

use std::fmt;

use indexmap::IndexMap;
use regex::Regex;
use serde::de::IgnoredAny;
use serde::Deserialize;

use crate::assert::{self, Assertion, Pattern};
use crate::links;
use crate::selector::{self, Selector};

/// Every key a rule may carry — the key scan checks against this list, and the
/// raw schema below must stay in sync with it.
const KEYS: &[&str] = &[
    "select",
    "id",
    "optional",
    "count",
    "min",
    "max",
    "min_lines",
    "match",
    "require",
    "forbid",
    "starts_with",
    "ends_with",
    "min_length",
    "max_length",
    "enum",
    "no_empty",
    "ordered",
    "strict",
    "columns",
    "rows",
    "has",
    "not_has",
    "wiki",
    "resolve",
    "each",
];

#[derive(Debug)]
pub struct LoadError {
    message: String,
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for LoadError {}

impl LoadError {
    fn new(message: impl Into<String>) -> LoadError {
        LoadError {
            message: message.into(),
        }
    }
}

/// A frontmatter value spec: a bare string is a regexp the whole value must match.
#[derive(Debug)]
pub enum ValueSpec {
    Present,
    /// The scalar value is one of these; on a list value, every item is.
    Enum(Vec<String>),
    Matches(Pattern),
    /// The value is a sequence whose length lies within the bounds, and whose
    /// items are all in `allowed` when given.
    List(ListSpec),
}

#[derive(Debug)]
pub struct ListSpec {
    pub min: Option<usize>,
    pub max: Option<usize>,
    pub allowed: Option<Vec<String>>,
}

/// A rule ready to run: selector parsed, assertions compiled.
#[derive(Debug)]
pub struct Rule {
    /// Check id of findings from this rule. Nested `each:` rules carry only their
    /// slot (`each[0]`); the runner prefixes the outer context.
    pub check: String,
    pub selector: Selector,
    pub selector_source: String,
    pub optional: bool,
    pub assertions: Vec<Box<dyn Assertion>>,
}

#[derive(Debug)]
pub struct RuleSet {
    pub doc: String,
    pub rules: Vec<Rule>,
}

impl RuleSet {
    pub fn parse(yaml: &str) -> Result<RuleSet, LoadError> {
        let scan: ScanFile = from_yaml(yaml)?;
        for (index, rule) in scan.rules.iter().enumerate() {
            check_keys(rule, &index.to_string())?;
        }

        let raw: RawFile = from_yaml(yaml)?;
        let doc = raw.doc.unwrap_or_else(|| "doc".to_string());
        let mut ids: Vec<String> = Vec::new();
        let mut rules = Vec::new();
        for (index, rule) in raw.rules.iter().enumerate() {
            let path = index.to_string();
            let check = match &rule.id {
                Some(id) => id.clone(),
                None => format!("{doc}.rule[{index}]"),
            };
            rules.push(compile(rule, check, &path, &mut ids)?);
        }

        Ok(RuleSet { doc, rules })
    }
}

/// YAML 1.1 booleans stay strings: rule files spell frontmatter values such as
/// `yes` / `no` unquoted, and frontmatter values are plain strings.
fn from_yaml<T: serde::de::DeserializeOwned>(yaml: &str) -> Result<T, LoadError> {
    let options = serde_saphyr::options! { strict_booleans: true, };
    serde_saphyr::from_str_with_options(yaml, options).map_err(|e| LoadError::new(e.to_string()))
}

/// Key scan: the typed schema rejects unknown keys too, but only this pass knows
/// which rule they sit in. `path` is the 0-based rule index — the same index the
/// check id carries — with `each:` positions appended (`1.each[0]`).
fn check_keys(rule: &ScanRule, path: &str) -> Result<(), LoadError> {
    for key in rule.keys.keys() {
        if !KEYS.contains(&key.as_str()) {
            return Err(LoadError::new(format!(
                "unknown key '{key}' in rule {path}"
            )));
        }
    }
    for (index, inner) in rule.each.iter().flatten().enumerate() {
        check_keys(inner, &format!("{path}.each[{index}]"))?;
    }
    Ok(())
}

fn compile(
    raw: &RawRule,
    check: String,
    path: &str,
    ids: &mut Vec<String>,
) -> Result<Rule, LoadError> {
    if let Some(id) = &raw.id {
        if ids.contains(id) {
            return Err(LoadError::new(format!(
                "duplicate rule id '{id}' in rule {path}"
            )));
        }
        ids.push(id.clone());
    }

    let selector =
        selector::parse(&raw.select).map_err(|e| LoadError::new(format!("rule {path}: {e}")))?;

    let mut assertions: Vec<Box<dyn Assertion>> = Vec::new();
    let regex =
        |source: &String| -> Result<Pattern, LoadError> { compile_regex(source, path, false) };

    if let Some(count) = raw.count {
        assertions.push(Box::new(assert::Count(count)));
    }
    if let Some(min) = raw.min {
        assertions.push(Box::new(assert::Min(min)));
    }
    if let Some(max) = raw.max {
        assertions.push(Box::new(assert::Max(max)));
    }
    if let Some(min_lines) = raw.min_lines {
        assertions.push(Box::new(assert::MinLines(min_lines)));
    }
    if let Some(source) = &raw.r#match {
        assertions.push(Box::new(assert::MatchAll(compile_regex(
            source, path, true,
        )?)));
    }
    if let Some(source) = &raw.require {
        assertions.push(Box::new(assert::Require(regex(source)?)));
    }
    if let Some(source) = &raw.forbid {
        assertions.push(Box::new(assert::Forbid(regex(source)?)));
    }
    if let Some(prefix) = &raw.starts_with {
        assertions.push(Box::new(assert::StartsWith(prefix.clone())));
    }
    if let Some(suffix) = &raw.ends_with {
        assertions.push(Box::new(assert::EndsWith(suffix.clone())));
    }
    if let Some(min_length) = raw.min_length {
        assertions.push(Box::new(assert::MinLength(min_length)));
    }
    if let Some(max_length) = raw.max_length {
        assertions.push(Box::new(assert::MaxLength(max_length)));
    }
    if let Some(values) = &raw.r#enum {
        assertions.push(Box::new(assert::EnumSet(values.clone())));
    }
    if raw.no_empty.unwrap_or(false) {
        assertions.push(Box::new(assert::NoEmpty));
    }
    if let Some(ordered) = &raw.ordered {
        assertions.push(Box::new(assert::Ordered {
            entries: ordered
                .iter()
                .map(String::as_str)
                .map(ordered_entry)
                .collect(),
            strict: raw.strict.unwrap_or(false),
        }));
    }
    if let Some(has) = &raw.has {
        let mut specs = Vec::new();
        for (key, spec) in has {
            specs.push((key.clone(), value_spec(key, spec, path)?));
        }
        assertions.push(Box::new(assert::FrontmatterHas(specs)));
    }
    if let Some(not_has) = &raw.not_has {
        assertions.push(Box::new(assert::FrontmatterNotHas(not_has.clone())));
    }
    if let Some(columns) = &raw.columns {
        assertions.push(Box::new(assert::Columns(columns.clone())));
    }
    if let Some(rows) = &raw.rows {
        assertions.push(Box::new(assert::Rows {
            min: rows.min,
            max: rows.max,
        }));
    }
    if raw.wiki.unwrap_or(false) {
        assertions.push(Box::new(links::Wiki));
    }
    if raw.resolve.unwrap_or(false) {
        assertions.push(Box::new(links::Resolve));
    }
    if let Some(each) = &raw.each {
        let mut inner = Vec::new();
        for (index, rule) in each.iter().enumerate() {
            let slot = match &rule.id {
                Some(id) => id.clone(),
                None => format!("each[{index}]"),
            };
            inner.push(compile(rule, slot, &format!("{path}.each[{index}]"), ids)?);
        }
        assertions.push(Box::new(assert::Each(inner)));
    }

    Ok(Rule {
        check,
        selector,
        selector_source: raw.select.clone(),
        optional: raw.optional.unwrap_or(false),
        assertions,
    })
}

/// Text regexps anchor per line (`(?m)`): the document model merges consecutive
/// source lines into a single paragraph, so `^` / `$` would otherwise only ever
/// see the whole block. `full` additionally demands that the whole value match —
/// the `match` mode.
fn compile_regex(source: &str, path: &str, full: bool) -> Result<Pattern, LoadError> {
    let expanded = if full {
        format!(r"(?m)\A(?:{source})\z")
    } else {
        format!("(?m){source}")
    };
    match Regex::new(&expanded) {
        Ok(regex) => Ok(Pattern {
            source: source.to_string(),
            regex,
        }),
        Err(error) => Err(LoadError::new(format!(
            "rule {path}: invalid regexp `{source}`: {}",
            one_line(&error.to_string())
        ))),
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

/// `Questions?` — a trailing `?` marks an optional member.
fn ordered_entry(entry: &str) -> assert::OrderedEntry {
    match entry.strip_suffix('?') {
        Some(name) => assert::OrderedEntry {
            name: name.to_string(),
            optional: true,
        },
        None => assert::OrderedEntry {
            name: entry.to_string(),
            optional: false,
        },
    }
}

fn value_spec(key: &str, raw: &RawValueSpec, path: &str) -> Result<ValueSpec, LoadError> {
    let spec = match raw {
        RawValueSpec::Pattern(source) => {
            return Ok(ValueSpec::Matches(compile_regex(source, path, true)?))
        }
        RawValueSpec::Map(spec) => spec,
    };
    let invalid = |why: &str| LoadError::new(format!("rule {path}: has `{key}`: {why}"));
    let (min, max) = match (spec.count, spec.min, spec.max) {
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
            return Err(invalid("`count` cannot be combined with `min` or `max`"));
        }
        (Some(count), None, None) => (Some(count), Some(count)),
        (None, Some(min), Some(max)) if min > max => {
            return Err(invalid(&format!("`min` {min} is greater than `max` {max}")));
        }
        (None, min, max) => (min, max),
    };
    Ok(match (min, max, &spec.r#enum) {
        (None, None, None) => ValueSpec::Present,
        (None, None, Some(allowed)) => ValueSpec::Enum(allowed.clone()),
        (min, max, allowed) => ValueSpec::List(ListSpec {
            min,
            max,
            allowed: allowed.clone(),
        }),
    })
}

/// Key-scan schema: `each:` by name so the scan can recurse, everything else
/// collected by name only.
#[derive(Debug, Deserialize)]
struct ScanFile {
    rules: Vec<ScanRule>,
}

#[derive(Debug, Deserialize)]
struct ScanRule {
    #[serde(default)]
    each: Option<Vec<ScanRule>>,
    #[serde(flatten)]
    keys: IndexMap<String, IgnoredAny>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFile {
    #[serde(default)]
    doc: Option<String>,
    rules: Vec<RawRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    select: String,
    id: Option<String>,
    optional: Option<bool>,
    count: Option<usize>,
    min: Option<usize>,
    max: Option<usize>,
    min_lines: Option<usize>,
    r#match: Option<String>,
    require: Option<String>,
    forbid: Option<String>,
    starts_with: Option<String>,
    ends_with: Option<String>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    r#enum: Option<Vec<String>>,
    no_empty: Option<bool>,
    ordered: Option<Vec<String>>,
    strict: Option<bool>,
    columns: Option<Vec<String>>,
    rows: Option<RawRows>,
    has: Option<IndexMap<String, RawValueSpec>>,
    not_has: Option<Vec<String>>,
    wiki: Option<bool>,
    resolve: Option<bool>,
    each: Option<Vec<RawRule>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRows {
    min: Option<usize>,
    max: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawValueSpec {
    Pattern(String),
    Map(RawSpecMap),
}

/// The mapping form: `{}` is presence only, `enum:` an allowed set, and
/// `min` / `max` / `count` bound a list's length. Rejects any other key so a
/// misspelled value spec is a load error instead of a silent presence check.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSpecMap {
    r#enum: Option<Vec<String>>,
    min: Option<usize>,
    max: Option<usize>,
    count: Option<usize>,
}
