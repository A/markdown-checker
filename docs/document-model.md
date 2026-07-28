# The document model

Before a rule runs, mdcheck reads your markdown into a tree of elements. Every selector you write addresses that tree, and every finding points back at a line in it. This page describes what the tree contains — the element vocabulary, how elements nest, why `SECTION` behaves the way it does, where frontmatter sits, and what happens when a document will not parse. Read it first; the selector syntax on the next page only makes sense against this vocabulary.

## The document is a tree

mdcheck does not treat your file as text with patterns in it. It parses the file into elements — headers, paragraphs, lists, tables, links, code fences — and nests them inside each other. A selector names element types and walks that nesting.

```markdown
# Feature index
## Roles
- **Admin** — manages
## Features
| priority | feature |
|---|---|
| p0 | Parse |
```

That file becomes:

```
DOCUMENT
└─ H1 "Feature index"           ← also the header of SECTION "Feature index"
   ├─ H2 "Roles"                ← also the header of SECTION "Roles"
   │  └─ UL
   │     └─ LI "**Admin** — manages"
   └─ H2 "Features"             ← also the header of SECTION "Features"
      └─ TABLE
         ├─ TR (header row)
         │  ├─ TD "priority"
         │  └─ TD "feature"
         └─ TR
            ├─ TD "p0"
            └─ TD "Parse"
```

Only these element types exist. Inline emphasis, blockquotes and horizontal rules are not addressable as elements — they are part of the text of the element that holds them, which is why `LI "**Admin** — manages"` keeps its asterisks and a rule like `match: '^\*\*.+\*\* — .+'` works on them.

## The element vocabulary

| element | what it picks up | example |
|---|---|---|
| `DOCUMENT` | the whole file, always exactly one match; scoping to it spans everything | `select: DOCUMENT` with `min_lines: 20` |
| `FRONTMATTER` | the key/value block fenced by `---` at the top of the file, always at most one | `confirmed: no` between two `---` lines |
| `H1` … `H6` | one header, by level; its text is the header line without the `#` marks | `## Roles` is an `H2` with text `Roles` |
| `SECTION` | a header **and its body**, up to the next header of the same level | `## Roles` plus everything until the next `##` |
| `P` | one paragraph of body text | `Roles and duties` |
| `UL` | one bullet list | the block of `- ` lines |
| `OL` | one numbered list | the block of `1. ` lines |
| `LI` | one item of a list | `- one` |
| `TABLE` | one pipe table, header row included | the `\| priority \| feature \|` table above |
| `TR` | one row of a table | `\| p0 \| Parse \|` |
| `TD` | one cell; `[col="priority"]` picks the cell under that column header | `p0` |
| `A` | one link, wiki-style or inline | `[[features/x\|X]]`, `[ext](https://e.com)` |
| `CODE` | one fenced code block; `[lang="gherkin"]` narrows by fence language | a ```` ```gherkin ```` fence and its contents |

Two of these are whole-unit selections: `DOCUMENT` matches once whatever the file contains, and `FRONTMATTER` matches the block once if it is there.

## SECTION is a header plus its body

This is the concept people get wrong. A `SECTION` is not the header line. It is the header **and everything that follows it until the next header of the same level** — so a subsection is inside its parent section, and a sibling section is outside it.

```markdown
# T
## Parent
- p1
### Child
- c1
## Sibling
- s1
```

`SECTION("Parent")` runs from `## Parent` down to just before `## Sibling`. `### Child` is inside it; `## Sibling` is not.

| selector | matches | why |
|---|---|---|
| `SECTION("Parent") UL > LI` | 2 — `p1` and `c1` | the descendant combinator reaches into the nested `Child` subsection |
| `SECTION("Parent") > UL > LI` | 1 — `p1` | `>` takes only direct children of the section, so it stops at the subsection boundary |
| `SECTION("Child") UL > LI` | 1 — `c1` | a subsection addressed on its own covers only its own body |
| `SECTION("Sibling") UL > LI` | 1 — `s1` | a sibling section is never credited with `Parent`'s content |

The practical consequence: `SECTION("Features") > TABLE` finds a table that sits directly under `## Features`, but not one buried under a `### Backlog` subsection. Use a space instead of `>` when you mean "anywhere under this section".

## What contains what

Containment is what `>` (direct child) and a space (any depth) traverse.

- `DOCUMENT` contains everything in the body.
- Headers nest by level: an `H2` is a child of the `H1` above it, an `H3` a child of that `H2`. So `H1 > H2` matches the top-level sections' headers, `H1 > H3` matches nothing, and `H1 H3` matches every `H3` in the document.
- A `SECTION` contains its subsections and all the block elements of its body: `P`, `UL`, `OL`, `TABLE`, `CODE`.
- `UL` and `OL` contain `LI`. A nested list hangs off its parent `LI`, so `UL > LI` counts only top-level items while `UL LI` counts nested ones too.
- `TABLE` contains `TR`, and `TR` contains `TD`.
- `A` lives inside whatever text element holds it — a `P`, an `LI`, a `TD`.

```markdown
## Items
- top
  - nested
```

`SECTION("Items") > UL > LI` matches 1 (`top`). `SECTION("Items") UL LI` matches 2.

## Frontmatter is a separate namespace

The frontmatter block is a key/value map, not markdown structure. Its lines never become body elements, and body selectors never reach into it.

```markdown
---
title: Doc
summary: a line of text
---
# T
```

`select: P` matches **0** elements here — `summary: a line of text` is a frontmatter value, not a paragraph. Frontmatter gets its own assertions, `has:` and `not_has:`, which name keys rather than select elements:

```yaml
- select: FRONTMATTER
  has: { confirmed: { enum: [yes, no] } }
  not_has: [draft]
```

Keys that neither `has:` nor `not_has:` names are ignored, so a document may carry extra frontmatter for other tooling without mdcheck objecting.

## Every element carries a line

Each element remembers the source line it starts at, and that is the line a finding points to. Which element the finding lands on depends on the assertion:

- An assertion about a particular element is reported at that element's own line. In a `Roles` list whose second item is `- broken item` on line 4, `select: SECTION("Roles") > UL > LI` with `match: '^\*\*.+\*\* — .+'` reports `does not match pattern` at line 4 and says nothing about the item above it.
- An assertion about how many elements matched has no offending element to point at, so it is reported at the line of the scope the selector was evaluated in. `SECTION("A") > UL > LI` with `min: 2` over a section holding one item reports `1 match, expected ≥ 2` at the `## A` header line.
- A frontmatter finding points at the offending key's own line; a missing key or a missing block points at line 1.
- A table cell finding points at that cell's row; a `columns:` or `rows:` finding points at the table's header line.

## The load stage

A document must parse before your rules run. Problems found while parsing are emitted as findings with `check: load`, and they do not stop anything — every rule you declared still runs and still reports in the same invocation.

```markdown
---
confirmed: no
# Doc
## This header is way too long for the configured limit
```

The frontmatter block is opened and never closed. With rules:

```yaml
- select: H2
  max_length: 20
```

you get two findings from one run: `unclosed frontmatter block` at line 1, carrying `check: load`, and `length 52 > 20` at line 4 from the declared rule. Exit code 1.

A load finding fails the run on its own. A clean document whose rules all pass but whose frontmatter block never closes still exits 1, with the load finding as the only entry.

Link markup that never terminates is reported the same way — a line like `See [[features/x|X and more text.` produces `unterminated link syntax` at its line under `check: load`.

The one failure that stops the run is a document that cannot be read at all: a path that does not exist, or a file mdcheck has no access to. No rule runs, nothing is reported as a finding, and a diagnostic naming the path goes to stderr.

## Not yet settled

- Which malformed link forms count as broken link syntax at the load stage, and the exact wording of that message.
- The exit code for an unreadable document, and whether one unreadable path among several named in a single invocation aborts the whole run or only its own document.
- Whether `TABLE > TD` reaches cells through the intervening `TR` — the rule examples in the spec are written that way, but no scenario fixes it. `TABLE TD` (descendant) is unambiguous.
- What the text of an element that contains nested structure is — whether an `LI` holding a nested list contributes the nested items' text to its own for `match` and `max_length`.

**Next:** [Selectors](selectors.md) · [Assertions](assertions.md) · [Rule files](rule-files.md) · [Findings and output](output.md) · [Cookbook](cookbook.md) · back to [docs](README.md)
