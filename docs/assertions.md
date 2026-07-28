# Assertions

A rule has two halves. The selector picks out elements — that is [Selectors](selectors.md). The assertions say what must be true of them. This page is the assertion vocabulary: how many elements there must be, what their text must say, how long it may run, which values are allowed, what shape a section set or a table must have, what frontmatter must carry, and where links may point.

Two things hold across every family:

- **Assertions combine on one rule.** A rule may carry `max_length` and `forbid` and `enum` at once; each is checked against the same set of matched elements.
- **Every violation is its own finding.** Two oversized paragraphs give two findings. mdcheck never stops at the first one — one run reports everything it found, then exits 1.

Each section below also says **where the finding points**, because that line is what you actually read in the output.

`optional:` (a rule that tolerates matching nothing) and `each:` (nested rules) are rule-level keys, not assertions — see [Rule files](rule-files.md).

## How many — `count`, `min`, `max`

`min` and `max` bound the number of matches; `count` demands an exact number.

```markdown
# T

## Stories

- only one
```

```yaml
- select: SECTION("Stories") > UL > LI
  min: 2
  max: 10
```

```
Stories line: 1 match, expected ≥ 2
```

**Points at:** the scope line — the header of the section the selector was evaluated in, or line 1 for a document-level selector. There is no offending element to point at, so the finding lands on the container that was supposed to hold them.

| written | reported as |
|---|---|
| `min: 2` with 1 match | `1 match, expected ≥ 2` |
| `max: 10` with 11 matches | `11 matches, expected ≤ 10` |
| `count: 3` with 2 matches | `2 matches, expected exactly 3` |
| `count: 1` with 2 matches | `2 matches, expected exactly 1` |

`count: 0` forbids a whole class of element. Combined with a filtering selector it is how you ban something outright — and because each offender exists on the page, these findings point at the offenders themselves:

```yaml
- select: A[href=/^https?:/]
  count: 0
```

```
line 3: link `https://e.com` matched, expected 0 matches
line 5: link `http://f.org` matched, expected 0 matches
```

## What the text says — `match`, `require`, `forbid`, `starts_with`, `ends_with`

Three regexp modes read the same pattern three different ways. `match` demands the **whole** element text match. `require` demands the pattern appear **somewhere** in it. `forbid` demands it appear **nowhere**.

One document, one pattern, three verdicts:

```markdown
# Doc

Status: pending review
```

| rule on `select: P` | verdict | finding at the `Status: pending review` line |
|---|---|---|
| `require: 'pending'` | passes | — |
| `match: 'pending'` | fails | `does not match pattern` |
| `forbid: 'pending'` | fails | `contains forbidden pattern` |

Change the paragraph to `Status: done` and `require: 'pending'` fails in turn: `required pattern not found`.

So `match` is for shapes you fully control — anchor it or not, it is whole-value either way:

```yaml
- select: SECTION("Roles") > UL > LI
  match: '^\*\*.+\*\* — .+'
```

```markdown
## Roles
- **Admin** — manages
- broken item
```

```
broken item line: does not match pattern
```

Quote patterns with single quotes in YAML so backslashes survive.

`starts_with` and `ends_with` compare literal text, not patterns:

```yaml
- select: H2
  ends_with: '?'
```

```
Roles line: does not end with `?`
```

**Points at:** the offending element's own line. Elements that satisfy the assertion produce nothing.

## How long — `min_length`, `max_length`, `min_lines`

`min_length` and `max_length` bound the character length of each matched element's text.

```markdown
# Doc

## Hi

## This header is way too long for the configured limit of twenty
```

```yaml
- select: H2
  max_length: 20
```

```
This header is way too long… line: length 62 > 20
```

With `min_length: 5` instead, the same document reports the other header: `length 2 < 5`.

**Points at:** the offending element's own line, one finding per element. Four paragraphs with two over the cap give exactly two findings in one run.

`min_lines` is different: it asks about the document as a whole, so it goes on `DOCUMENT` and reports once.

```yaml
- select: DOCUMENT
  min_lines: 20
```

```
line 1: document has 3 lines, expected ≥ 20
```

**Points at:** line 1.

## Which values are allowed — `enum`, `no_empty`

`enum` restricts each matched element's text to a set. `no_empty` rejects matches whose value is empty — the usual targets are list items and table cells.

```markdown
# T

## Priorities

- p0
- p9
```

```yaml
- select: SECTION("Priorities") > UL > LI
  enum: [p0, p1, p2, p3]
```

```
p9 line: `p9` not in allowed set
```

```markdown
## Roles

- Admin
-
```

```yaml
- select: SECTION("Roles") > UL > LI
  no_empty: true
```

```
line 6: empty list item
```

**Points at:** the offending element's own line.

## What shape a section set has — `ordered`, `strict`

`ordered:` declares the members of one header level and the order they come in. A `?` suffix marks a member optional: it may be absent, but when present it must still sit where it was declared. `strict: true` additionally forbids anything the list does not name.

```yaml
- select: H1 > H2
  ordered: [Roles, Features, Questions?]
  strict: true
```

The rule constrains only the level its selector picks — `H1 > H2` says nothing about `H3`s, and a `strict:` rule on `H2 > H3` does not forbid extra `H2`s.

Three distinct failures, each with its own anchor:

| failure | example document | finding | points at |
|---|---|---|---|
| **missing member** | `## Roles` only | `` required section `Features` missing `` | line 1 — a section that was never written has no header to point at, so the finding lands on the scope |
| **out of order** | `## Features` then `## Roles` | `` section order: `Roles` appears after `Features`, expected before `` | the `Roles` header — the member that arrived late |
| **stray member** under `strict:` | `## Roles`, `## Features`, `## Extra` | `` section `Extra` not allowed (strict) `` | the `Extra` header |

Without `strict:`, the third case passes: the list constrains only the members it names and extra sections are tolerated.

All three land in one run when a document breaks the skeleton three ways:

```markdown
# T
## Features
## Roles
## Extra
```

```yaml
- select: H1 > H2
  ordered: [Roles, Features, Summary]
  strict: true
```

```
line 1:        required section `Summary` missing
Roles header:  section order: `Roles` appears after `Features`, expected before
Extra header:  section `Extra` not allowed (strict)
```

To require a section whose title you only know by pattern, select it and assert cardinality instead: `select: H2(/^Story: /)` with `min: 1`.

## What shape a table has — `columns`, `rows`, per-cell rules

`columns:` fixes the header row — which columns, in which order. `rows:` bounds how many **body** rows follow; the header row and the delimiter row beneath it are not rows of data.

```markdown
## Features
| priority | epic | feature |
|---|---|---|
| p0 | **Core** | Parse |
| p1 | **Edge** | Render |
```

```yaml
- select: SECTION("Features") > TABLE
  columns: [priority, epic, feature]
  rows: { min: 2, max: 3 }
```

Swap two headers and you get an order mismatch; drop one and you get a missing column — two different findings, not one:

```
table header line: column order `epic, priority, feature`, expected `priority, epic, feature`
table header line: required column `epic` missing
table header line: table has 1 body row, expected ≥ 2
```

**Points at:** the table's header line, for both `columns:` and `rows:`.

Per-cell assertions ride on a `TD[col="…"]` selector: address the column by name, then use any of the text, length or value assertions above. The finding moves to the cell's own row and the message names the column.

```markdown
| priority | capabilities |
|---|---|
| p0 | reads markdown |
| p1 | TBD |
| p2 | renders output |
| p3 | TBD |
```

```yaml
- select: SECTION("Features") > TABLE > TD[col="capabilities"]
  forbid: 'TBD'
```

```
p1 row: column `capabilities` matches forbidden `TBD`
p3 row: column `capabilities` matches forbidden `TBD`
```

A cell rule reaches only the column it names, however similar the neighbouring cells look. Other column messages take the same shape: `` `p9` in column `priority` not in allowed set ``, `` empty cell in column `feature` ``, `` column `capabilities` length 25 > 12 ``.

## Frontmatter keys — `has`, `not_has`

`has:` names keys the document must carry, each with a value spec in one of three forms:

| form | written | means |
|---|---|---|
| exact string | `status: approved` | the value is exactly `approved` |
| `enum:` | `confirmed: { enum: [yes, no] }` | the value is one of the listed values |
| regexp | `priority: '^P[0-3]$'` | the value matches the pattern |

`not_has:` is a plain list of keys that must never appear. Keys named by neither are ignored — carry whatever else your other tooling needs.

```markdown
---
confirmed: maybe
priority: P7
draft: true
---
# Doc
```

```yaml
- select: FRONTMATTER
  has: { confirmed: { enum: [yes, no] }, priority: '^P[0-3]$' }
  not_has: [draft]
```

```
line 2: `confirmed` is `maybe`, allowed: `yes`, `no`
line 3: `priority` value `P7` does not match `^P[0-3]$`
line 4: key `draft` must not be present
```

**Points at:** the offending key's own line, not the start of the block. Two exceptions, both because there is no key line to point at: a required key that is absent from a present block reports `` required key `priority` missing `` at line 1, and a document with no frontmatter block at all reports `frontmatter block missing` once at line 1 — one finding for the block, however many keys the rule asked for.

## Links — `wiki`, `resolve`

`wiki: true` requires every link the selector matches to be written as a wiki-link.

```markdown
See [[features/x|X]] and [ext](https://e.com).
```

```yaml
- select: A
  wiki: true
```

```
line 1: `[ext](https://e.com)` is not a wiki-link
```

`resolve: true` requires each selected wiki-link's target to exist as a file under the vault root you pass with `--root`:

```bash
mdcheck rules.yaml doc.md --root ./vault
```

```yaml
- select: A[wiki]
  resolve: true
```

```
line 1: target `features/x/stories.md` not found under root
```

An aliased link resolves on its path, not its label: `[[features/x|Overview]]` looks for `features/x.md`.

**Points at:** the offending link's own line, one finding per link, each naming its own target. Scope narrows the judgement — `SECTION("Refs") A` judges the links in that section and never mentions the ones outside it.

## Reference

| key | asserts | finding points at |
|---|---|---|
| `count` | exact number of selector matches | scope line |
| `min` | at least this many matches | scope line |
| `max` | at most this many matches | scope line |
| `match` | element text as a whole matches the regexp | the element |
| `require` | the regexp appears somewhere in the element text | the element |
| `forbid` | the regexp appears nowhere in the element text | the element |
| `starts_with` | element text begins with the literal string | the element |
| `ends_with` | element text ends with the literal string | the element |
| `min_length` | element text is at least this many characters | the element |
| `max_length` | element text is at most this many characters | the element |
| `min_lines` | the document has at least this many lines (`DOCUMENT`) | line 1 |
| `enum` | element text is one of the listed values | the element |
| `no_empty` | no matched element has an empty value | the element |
| `ordered` | named members present in the declared order; `?` = optional | scope line if missing, the member's header if misordered |
| `strict` | no members outside the `ordered` list | the stray member's header |
| `columns` | table header set and its order | table header line |
| `rows` | body row count within `{min, max}` | table header line |
| `has` | frontmatter keys present, values satisfying exact / `enum:` / regexp | the key's line (line 1 if absent) |
| `not_has` | frontmatter keys absent | the key's line |
| `wiki` | selected links are wiki-links | the link |
| `resolve` | wiki-link targets exist under `--root` | the link |

## Not yet settled

- Whether a rule whose selector matches nothing is itself a finding when the rule carries no cardinality key and no `optional: true`. The two readings — an empty match set is reported as `` selector `P` matched no elements ``, or it passes vacuously and presence must be demanded with `min`/`count` — are both live in the spec.
- Whether `columns:` fixes an exhaustive set or only a subset: a header column the rule does not name is currently read as an error (`` column `notes` not allowed ``), but no `strict:` flag is defined for tables the way it is for sections.
- What `resolve: true` does when `--root` is omitted. Reading it as a usage error (exit 2, nothing on stdout) is the current assumption; silently skipping the check, or defaulting the root to the document's own directory, are the alternatives.
- Whether a `TD[col="…"]` naming a column no table carries is silence (it selects no cells, so a per-cell assertion has nothing to say) or an authoring mistake in the rule file.

**Next:** [Rule files](rule-files.md) for `optional:`, `each:` and the shape of the file itself, then [Findings and output](output.md) for how these findings are formatted, and the [Cookbook](cookbook.md) for whole rule files. Back to [docs](README.md).
