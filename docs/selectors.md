# Selectors

A selector is how a rule says *which* part of a document it is about. It reads like a CSS selector — element names, attribute filters, combinators — but the elements are markdown constructs (`SECTION`, `P`, `LI`, `TD`) rather than HTML tags. Every rule starts with one `select:`; the assertions on that rule then apply to whatever it matched. This page covers the selector language itself: its shape, its combinators, its filters, and how scoping keeps a rule from reaching into a part of the document you did not mean.

## The shape of a selector

A selector is a sequence of **steps** joined by **combinators**. Each step is:

```
ELEMENT [filter] [index]
```

- the **element name** — `H1`…`H6`, `SECTION`, `P`, `UL`, `OL`, `LI`, `TABLE`, `TR`, `TD`, `A`, `CODE`, `FRONTMATTER`, `DOCUMENT`. See [The document model](document-model.md) for what each one covers.
- an optional **filter** in square brackets, narrowing the step by text, title, column, link target or fence language.
- an optional **index** in square brackets, picking one match by position.

So in:

```yaml
- select: SECTION("Features") > TABLE > TD[col="priority"]
  enum: [p0, p1, p2, p3]
```

there are three steps — `SECTION("Features")`, `TABLE`, `TD[col="priority"]` — joined by two child combinators. The rule's assertions run against the final step's matches: every cell of the `priority` column inside the `Features` section.

## Combinators

| combinator | reads as | example |
|---|---|---|
| `A > B` | a `B` directly inside an `A` | `SECTION("Roles") > UL` |
| `A B` (space) | a `B` anywhere inside an `A`, at any depth | `SECTION("Parent") UL` |
| `A + B` | the `B` immediately after an `A` | `P + UL` |

### Child stops, descendant reaches through

```markdown
# T
## Parent
- p1
### Child
- c1
## Sibling
- s1
```

| selector | matches | why |
|---|---|---|
| `SECTION("Parent") > UL > LI` | 1 — `p1` | the list under `### Child` sits inside an intervening subsection, so `>` does not reach it |
| `SECTION("Parent") UL > LI` | 2 — `p1`, `c1` | the space reaches through the subsection |

`s1` is matched by neither: `SECTION("Parent")` ends where the next `##` begins.

The same contrast applies to nested lists:

```markdown
# T
## Items
- top
  - nested
```

`SECTION("Items") > UL > LI` matches 1 (`top`); `SECTION("Items") UL LI` matches 2 (`top`, `nested`).

Headers nest by level, so the combinators work on them too:

```markdown
# T
## A
### A1
## B
```

`H1 > H2` matches 2, `H1 > H3` matches 0, `H1 H3` matches 1, `H2 > H3` matches 1.

### Adjacent sibling means *right after*

`+` matches only the element immediately following, never a later one.

```markdown
# T
## Story
intro

- alpha

later

- beta
- gamma
```

```yaml
- select: SECTION("Story") > P[text="intro"] + UL > LI
  count: 1
```

This passes: the only list adjacent to `intro` is `- alpha`. The `beta`/`gamma` list follows `later`, not `intro`, so it is out of reach. If you remove the `- alpha` list, so that `later` sits directly after `intro`, the same selector matches nothing — `min: 1` then reports `0 matches, expected ≥ 1`.

## Filters

| filter | matches when |
|---|---|
| `[text="Roles"]` | the element's text equals `Roles` |
| `[text^="pre:"]` | the text starts with `pre:` |
| `[text$="?"]` | the text ends with `?` |
| `[text*="feature"]` | the text contains `feature` |
| `[text=/^I can .+/]` | the text matches the regexp |
| `[title=…]` | a `SECTION`'s header text, with the same five operators |
| `[col="priority"]` | a `TD` sits in the column with that header |
| `[href=/^https?:/]` | a link's target matches the regexp |
| `[wiki]` | the link is written as a wiki-link |
| `[lang="gherkin"]` | a `CODE` fence declares that language |

Against this document:

```markdown
# T
Roles

Roles and duties

pre: setup

is this the end?
```

`P[text="Roles"]` matches 1 — equality is against the whole text, so `Roles and duties` is not a match. `P[text*="Roles"]` matches 2. `P[text^="pre:"]` matches 1, `P[text$="?"]` matches 1.

**`[title=…]` addresses a section, `[text=…]` addresses a header.** `SECTION[title="Roles"]` selects the header *and its body*; `H2[text="Roles"]` selects the header line alone. Both use the same operators: with sections `## Roles`, `## Story: login` and `## Story: logout` in a document, `SECTION[title^="Story: "]` matches 2 and `SECTION[title$="login"]` matches 1.

**`[col=…]` names a column, and applies per cell.** `TD[col="capabilities"]` selects every cell under the `capabilities` header, and each assertion is judged cell by cell — two bad cells produce two findings, each anchored at its own row.

**`[lang=…]` narrows to one fence, and its content is what gets read.** With a `gherkin` fence and a `yaml` fence in the same section, `CODE[lang="gherkin"]` matches 1, bare `CODE` matches 2, and a `forbid: 'TBD'` on `CODE[lang="gherkin"]` passes even when the `yaml` fence contains `TBD` — text inside a fence the selector did not match is outside the selection.

**`[wiki]` is a bare filter** — no operator, no value. `A[wiki]` selects the wiki-links and leaves inline links alone; `A[href=/^https?:/]` selects the external ones.

## Call sugar

A quoted string in parentheses after an element name is shorthand for the filter you almost always want:

| sugar | equivalent to |
|---|---|
| `H2("Roles")` | `H2[text="Roles"]` |
| `SECTION("Roles")` | `SECTION[title="Roles"]` |
| `H2(/^Story: /)` | `H2[text=/^Story: /]` |
| `SECTION(/^Story: /)` | `SECTION[title=/^Story: /]` |

The call form takes a string for equality or a `/…/` regexp; the two forms select identically, so pick whichever reads better.

## Index

A bare integer in brackets picks a single match by position: `[0]` is the first, `[-1]` is the last.

```markdown
# T
## Story
pre: setup

As a user

done: teardown

## Other
first other
```

```yaml
- select: SECTION("Story") > P[0]
  starts_with: 'pre:'
```

This passes — `[0]` is `pre: setup`. `SECTION("Story") > P[-1]` is `done: teardown`.

**The index counts within the current scope, not within the document.** `SECTION("Other") > P[0]` is `first other` — the third paragraph in the document, but the first one in `Other`'s scope. Asserting `starts_with: 'pre:'` on it fails and reports at `first other`'s own line, not at `pre: setup`.

## Scoping

Everything left of the last combinator is scope. This is what makes selectors usable for specs: a rule aimed at one section can never be satisfied — or broken — by a neighbouring section.

```markdown
# T
## A
- one
## B
- two
- three
```

```yaml
- select: SECTION("B") > UL > LI
  min: 2
```

Passes: `B` has two items. Change the selector to `SECTION("A") > UL > LI` with the same `min: 2` and you get `1 match, expected ≥ 2` — `A` is not credited with `B`'s items, even though the document has three list items in total.

Scope follows the section's extent, not the file order: a `SECTION` runs from its header to the next header *of the same level*, so `SECTION("Parent")` includes `### Child` and everything under it, while `SECTION("Child")` covers only the subsection's own body.

`DOCUMENT` is the widest scope — `DOCUMENT LI` reaches every list item in the file. Frontmatter is not part of the body, so `FRONTMATTER` is the only way to address it, and a `summary:` key in frontmatter is never matched by `P`.

When you want a scoped rule applied once per match rather than once overall — "every story block has an acceptance list", not "the document has some acceptance lists" — that is `each:`, on the [Rule files](rule-files.md) page.

## Reading a selector out loud

```yaml
- select: SECTION("User Stories") > SECTION(/^[A-Z]/)
  each:
    - select: P[text=/^As \*\*.+\*\*/] + UL > LI
      min: 1
      match: '^I can .+'
```

The outer selector: *find the section titled `User Stories`; inside it, take each direct subsection whose title starts with a capital letter.* Each of those becomes a scope for the inner rules.

The inner selector, read right to left as a scope chain and left to right as a walk: *within this story block, find the paragraph whose text starts with `As **…**`; take the list immediately after it; take that list's direct items.* Then `min: 1` requires at least one, and `match` requires each one to read `I can …`.

Applied to:

```markdown
## User Stories
### Login
As **Visitor**
- I can log in
### Broken
some text
```

`Login` satisfies the inner rule and reports nothing. `Broken` has no `As **…**` paragraph, so the inner selector matches nothing there and the finding lands on the `Broken` section: `inner rule ...: 0 matches, expected ≥ 1`.

## When a selector matches nothing

An empty match set is not automatically an error — it is what a `count: 0` rule is built on, and what `min: 1` is built to catch. Which of the two you get depends on the assertions you attach and on `optional: true`, which makes a rule pass when its selector matches nothing. Both are settled on the [Rule files](rule-files.md) page; write your selector for what you mean to address, and decide there whether absence is a failure.

## Not yet settled

- The finding produced when a selector matches nothing and the rule is *not* `optional:` — its message, and whether it is anchored at the scope's line.
- Whether `TD[col="…"]` naming a column the table does not carry is silently an empty selection or is reported as a mistake in the rule file.
- Whether `[href=…]` accepts the equality, prefix, suffix and substring operators, or only the regexp form.
- Whether several filters, or a filter and an index, can be stacked on one step (`CODE[lang="yaml"][0]`).

**Next:** [Assertions](assertions.md) · [Rule files](rule-files.md) · back to [docs](README.md)
