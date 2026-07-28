//! Link assertions: `wiki:` (every selected link is a wiki-link) and `resolve:`
//! (every target exists under the root).
//!
//! Targets resolve against `Ctx::root` — the `--root` flag, or the checked
//! file's own directory. Wiki targets name documents, so an extension-less one
//! gets `.md`; markdown targets are paths and resolve as written. Anything the
//! filesystem cannot answer for — schemes, in-document fragments — is skipped.

use std::path::{Path, PathBuf};

use crate::assert::{Assertion, Ctx};
use crate::finding::Finding;
use crate::model::ElementKind;
use crate::selector::Match;

#[derive(Debug)]
pub struct Wiki;

impl Assertion for Wiki {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        links(matches)
            .filter(|m| !m.attrs().wiki)
            .map(|m| {
                let source = markdown_source(&m);
                ctx.finding(
                    m.start_line(),
                    "a wiki-link",
                    source.clone(),
                    format!("`{source}` is not a wiki-link"),
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct Resolve;

impl Assertion for Resolve {
    fn evaluate(&self, matches: &[Match<'_>], ctx: &Ctx<'_>) -> Vec<Finding> {
        links(matches)
            .filter_map(|m| {
                let target = target(m.attrs().href.as_deref()?, m.attrs().wiki)?;
                if ctx.root.join(&target).exists() {
                    return None;
                }
                Some(ctx.finding(
                    m.start_line(),
                    "an existing file under the root",
                    display(&target),
                    format!("target `{}` not found under root", display(&target)),
                ))
            })
            .collect()
    }
}

fn links<'m, 'a>(matches: &'m [Match<'a>]) -> impl Iterator<Item = Match<'a>> + 'm {
    matches
        .iter()
        .copied()
        .filter(|m| m.kind() == ElementKind::Link)
}

/// A non-wiki link as the source spells it, for messages.
fn markdown_source(m: &Match<'_>) -> String {
    match &m.attrs().href {
        Some(href) => format!("[{}]({href})", m.text()),
        None => m.text().to_string(),
    }
}

/// The filesystem path a link target names, or `None` when it names nothing on
/// disk.
fn target(href: &str, wiki: bool) -> Option<PathBuf> {
    let href = href.split('#').next().unwrap_or_default().trim();
    if href.is_empty() || href.contains("://") || scheme(href) {
        return None;
    }
    let path = Path::new(href);
    if wiki && path.extension().is_none() {
        return Some(path.with_extension("md"));
    }
    Some(path.to_path_buf())
}

/// `mailto:`, `tel:` and friends — a scheme before any path separator.
fn scheme(href: &str) -> bool {
    match href.find(':') {
        Some(colon) => !href[..colon].contains('/'),
        None => false,
    }
}

fn display(path: &Path) -> String {
    path.display().to_string()
}
