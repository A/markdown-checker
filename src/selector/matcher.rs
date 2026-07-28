//! `Selector` AST × document tree → matched nodes.

use std::collections::HashSet;

use crate::model::{Attrs, DocModel, ElementKind, Node};

use super::parse::{AttrKey, Combinator, Compound, Element, Selector, Test};

/// A matched node with its position; the assertion layer reads text, lines and
/// attributes off it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match<'a> {
    node: &'a Node,
}

impl<'a> Match<'a> {
    pub fn node(&self) -> &'a Node {
        self.node
    }

    pub fn kind(&self) -> ElementKind {
        self.node.kind
    }

    pub fn text(&self) -> &'a str {
        &self.node.text
    }

    pub fn start_line(&self) -> usize {
        self.node.start_line
    }

    pub fn end_line(&self) -> usize {
        self.node.end_line
    }

    pub fn attrs(&self) -> &'a Attrs {
        &self.node.attrs
    }
}

/// Runs `selector` over the whole document. An empty result is not an error.
pub fn select<'a>(doc: &'a DocModel, selector: &Selector) -> Vec<Match<'a>> {
    select_in(&doc.root, selector)
}

/// Runs `selector` over `scope` and its subtree — the scoping primitive behind
/// nested `each:` rules.
pub fn select_in<'a>(scope: &'a Node, selector: &Selector) -> Vec<Match<'a>> {
    let tree = Tree::new(scope);
    let mut current = tree.stage(&selector.head, |_| true);

    for (combinator, compound) in &selector.tail {
        let scoped: HashSet<usize> = current.iter().copied().collect();
        current = tree.stage(compound, |id| match combinator {
            Combinator::Child => tree.parents(id).iter().any(|p| scoped.contains(p)),
            Combinator::Descendant => tree.ancestors(id).iter().any(|a| scoped.contains(a)),
            Combinator::Adjacent => tree.preceding(id).iter().any(|p| scoped.contains(p)),
        });
    }

    current
        .into_iter()
        .map(|id| Match {
            node: tree.nodes[id].node,
        })
        .collect()
}

struct Entry<'a> {
    node: &'a Node,
    parent: Option<usize>,
    children: Vec<usize>,
    /// Position among the parent's children.
    order: usize,
}

/// Flattened document-order view with parent links.
struct Tree<'a> {
    nodes: Vec<Entry<'a>>,
}

impl<'a> Tree<'a> {
    fn new(root: &'a Node) -> Tree<'a> {
        let mut tree = Tree { nodes: Vec::new() };
        tree.push(root, None, 0);
        tree
    }

    fn push(&mut self, node: &'a Node, parent: Option<usize>, order: usize) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Entry {
            node,
            parent,
            children: Vec::new(),
            order,
        });
        for (index, child) in node.children.iter().enumerate() {
            let child_id = self.push(child, Some(id), index);
            self.nodes[id].children.push(child_id);
        }
        id
    }

    /// Collects the matches of one compound in document order, keeps those the
    /// combinator accepts, then applies the index filter to the whole set.
    fn stage(&self, compound: &Compound, accept: impl Fn(usize) -> bool) -> Vec<usize> {
        let found: Vec<usize> = (0..self.nodes.len())
            .filter(|&id| self.matches(id, compound) && accept(id))
            .collect();

        let Some(index) = compound.index else {
            return found;
        };
        let position = if index < 0 {
            found.len().checked_sub(index.unsigned_abs())
        } else {
            Some(index as usize)
        };
        position
            .and_then(|p| found.get(p))
            .map(|id| vec![*id])
            .unwrap_or_default()
    }

    fn matches(&self, id: usize, compound: &Compound) -> bool {
        let node = self.nodes[id].node;
        self.is_element(id, compound.element)
            && compound
                .filters
                .iter()
                .all(|f| test_attr(node, f.key, &f.test))
    }

    fn is_element(&self, id: usize, element: Element) -> bool {
        let kind = self.nodes[id].node.kind;
        match element {
            Element::Document => kind == ElementKind::Document,
            Element::Frontmatter => kind == ElementKind::Frontmatter,
            Element::Heading(level) => kind == ElementKind::Heading(level),
            Element::Section => matches!(kind, ElementKind::Section(_)),
            Element::Paragraph => kind == ElementKind::Paragraph,
            Element::UnorderedList => kind == ElementKind::UnorderedList,
            Element::OrderedList => kind == ElementKind::OrderedList,
            Element::ListItem => kind == ElementKind::ListItem,
            Element::Table => kind == ElementKind::Table,
            Element::TableRow => kind == ElementKind::TableRow,
            // Header cells carry column names, not column values: `TD[col=…]`
            // assertions must never see them.
            Element::TableCell => kind == ElementKind::TableCell && !self.in_header_row(id),
            Element::Link => kind == ElementKind::Link,
            Element::Code => kind == ElementKind::Code,
        }
    }

    fn in_header_row(&self, id: usize) -> bool {
        self.nodes[id]
            .parent
            .is_some_and(|p| self.nodes[p].node.attrs.header_row)
    }

    /// A heading stands for its own section, so a heading's relations are its
    /// section's relations.
    fn anchor(&self, id: usize) -> usize {
        let entry = &self.nodes[id];
        match (entry.node.kind, entry.parent) {
            (ElementKind::Heading(_), Some(parent))
                if matches!(self.nodes[parent].node.kind, ElementKind::Section(_))
                    && entry.order == 0 =>
            {
                parent
            }
            _ => id,
        }
    }

    /// The node itself plus the heading it stands for, so both `SECTION("X") > P`
    /// and `H2("X") > P` address the same relation.
    fn with_heading(&self, id: usize) -> Vec<usize> {
        let mut out = vec![id];
        if matches!(self.nodes[id].node.kind, ElementKind::Section(_)) {
            if let Some(&first) = self.nodes[id].children.first() {
                if matches!(self.nodes[first].node.kind, ElementKind::Heading(_)) {
                    out.push(first);
                }
            }
        }
        out
    }

    /// Selector parents: the real parent, its heading proxy, and — since rows are
    /// not addressable between them — the table above a `TD`.
    fn parents(&self, id: usize) -> Vec<usize> {
        let Some(parent) = self.nodes[self.anchor(id)].parent else {
            return Vec::new();
        };
        let mut out = self.with_heading(parent);
        if self.nodes[parent].node.kind == ElementKind::TableRow {
            out.extend(self.nodes[parent].parent);
        }
        out
    }

    fn ancestors(&self, id: usize) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        let mut queue = self.parents(id);
        while let Some(next) = queue.pop() {
            if out.contains(&next) {
                continue;
            }
            out.push(next);
            queue.extend(self.parents(next));
        }
        out
    }

    /// The immediately preceding sibling, plus its heading proxy.
    fn preceding(&self, id: usize) -> Vec<usize> {
        let anchor = self.anchor(id);
        let entry = &self.nodes[anchor];
        let Some(parent) = entry.parent else {
            return Vec::new();
        };
        let Some(previous) = entry.order.checked_sub(1) else {
            return Vec::new();
        };
        match self.nodes[parent].children.get(previous) {
            Some(&sibling) => self.with_heading(sibling),
            None => Vec::new(),
        }
    }
}

fn test_attr(node: &Node, key: AttrKey, test: &Test) -> bool {
    if key == AttrKey::Wiki {
        return matches!(test, Test::Present) && node.attrs.wiki;
    }

    let value = match key {
        AttrKey::Text => Some(node.text.as_str()),
        AttrKey::Title => node.attrs.title.as_deref(),
        AttrKey::Col => node.attrs.column.as_deref(),
        AttrKey::Href => node.attrs.href.as_deref(),
        AttrKey::Lang => node.attrs.lang.as_deref(),
        AttrKey::Wiki => unreachable!("handled above"),
    };

    let Some(value) = value else {
        return false;
    };
    match test {
        Test::Present => true,
        Test::Equals(expected) => value == expected,
        Test::StartsWith(prefix) => value.starts_with(prefix),
        Test::EndsWith(suffix) => value.ends_with(suffix),
        Test::Contains(needle) => value.contains(needle),
        Test::Matches(pattern) => pattern.regex.is_match(value),
    }
}
