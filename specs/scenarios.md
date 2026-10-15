# Acceptance scenarios

Each scenario: **input** markdown → **config** (rule yaml) → **expected** result. `expected: pass` means zero findings; otherwise findings are listed as `line — message essence`. These scenarios seed the test suite one-to-one.

## Frontmatter

### FM-1 required key with enum, valid

input:
```markdown
---
confirmed: no
---
# Doc
```
config:
```yaml
- select: FRONTMATTER
  has: { confirmed: { enum: [yes, no] } }
```
expected: pass

### FM-2 enum violation

input: same as FM-1 but `confirmed: maybe`

config: same as FM-1

expected:
- line 2 — `confirmed` is `maybe`, allowed: `yes`, `no`

### FM-3 missing frontmatter

input:
```markdown
# Doc
```
config: same as FM-1

expected:
- line 1 — frontmatter block missing

### FM-4 forbidden key present

input:
```markdown
---
confirmed: no
draft: true
---
```
config:
```yaml
- select: FRONTMATTER
  not_has: [draft]
```
expected:
- line 3 — key `draft` must not be present

### FM-5 key value by regexp

input frontmatter `priority: P7`

config:
```yaml
- select: FRONTMATTER
  has: { priority: '^P[0-3]$' }
```
expected:
- line 2 — `priority` value `P7` does not match `^P[0-3]$`

### FM-6 list length

input:
```markdown
---
tags: []
owners:
  - ann
  - bob
title: Doc
---
# Doc
```
config:
```yaml
- select: FRONTMATTER
  has: { tags: { min: 1 }, owners: { max: 1 }, title: { count: 1 } }
```
expected:
- line 2 — `tags` has 0 items, expected ≥ 1
- line 3 — `owners` has 2 items, expected ≤ 1
- line 6 — `title` value `Doc` is not a list

### FM-7 enum per list item

input:
```markdown
---
tags:
  - index
  - draft
---
# Doc
```
config:
```yaml
- select: FRONTMATTER
  has: { tags: { min: 1, enum: [index, spec] } }
```
expected:
- line 4 — `tags` item `draft` not allowed, allowed: `index`, `spec`

## Document

### DOC-1 min lines

input: 3-line document

config:
```yaml
- select: DOCUMENT
  min_lines: 20
```
expected:
- line 1 — document has 3 lines, expected ≥ 20

## Sections

### SEC-1 required sections present in order

input:
```markdown
# T
## Roles
x
## Features
y
```
config:
```yaml
- select: H1 > H2
  ordered: [Roles, Features]
```
expected: pass

### SEC-2 order swapped

input: SEC-1 with `## Features` before `## Roles`

config: same as SEC-1

expected:
- line of `Roles` — section order: `Roles` appears after `Features`, expected before

### SEC-3 missing required section

input: SEC-1 without `## Features`

config: same as SEC-1

expected:
- line 1 — required section `Features` missing

### SEC-4 optional section

input: SEC-1 + trailing `## Questions`

config:
```yaml
- select: H1 > H2
  ordered: [Roles, Features, Questions?]
  strict: true
```
expected: pass (also pass when `Questions` absent)

### SEC-5 strict: stray section

input: SEC-1 + `## Extra`

config: same as SEC-4

expected:
- line of `Extra` — section `Extra` not allowed (strict)

### SEC-6 required section by title regexp

input:
```markdown
# T
## Story: login
```
config:
```yaml
- select: H2(/^Story: /)
  min: 1
```
expected: pass

## Selector scoping

### SEL-1 checks scoped to a section

input:
```markdown
# T
## A
- one
## B
- two
- three
```
config:
```yaml
- select: SECTION("B") > UL > LI
  min: 2
```
expected: pass (items of A never counted)

### SEL-2 first paragraph of a section

input:
```markdown
# T
## Story
pre: setup line
As a user
```
config:
```yaml
- select: SECTION("Story") > P[0]
  starts_with: 'pre:'
```
expected: pass

### SEL-3 index on empty scope with optional

input: section `Story` with no paragraphs

config: SEL-2 rule + `optional: true`

expected: pass

## Lists

### LST-1 labeled list items match regexp

input:
```markdown
## Roles
- **Admin** — manages
- broken item
```
config:
```yaml
- select: SECTION("Roles") > UL > LI
  match: '^\*\*.+\*\* — .+'
```
expected:
- line of `broken item` — does not match pattern

### LST-2 min/max items (story count)

input: section with 1 list item

config:
```yaml
- select: SECTION("Stories") > UL > LI
  min: 2
  max: 10
```
expected:
- section line — 1 match, expected ≥ 2

## Element length / starts_with

### LEN-1 header too long

input: `## This header is way too long for the configured limit of twenty`

config:
```yaml
- select: H2
  max_length: 20
```
expected:
- header line — length 62 > 20

### LEN-2 paragraph cap

config:
```yaml
- select: P
  max_length: 200
```
expected: finding per paragraph over 200 chars, line each

## Subsection structure (each:)

### SUB-1 story blocks

input:
```markdown
## User Stories
### Login
pre: user exists
As **Visitor**
- I can log in
### Broken
some text
```
config:
```yaml
- select: SECTION("User Stories") > SECTION(/^[A-Z]/)
  each:
    - select: P[0]
      optional: true
      starts_with: 'pre:'
    - select: P[text=/^As \*\*.+\*\*/] + UL > LI
      min: 1
      match: '^I can .+'
```
expected:
- line of `Broken` section — inner rule `P[text=/^As …/] + UL > LI`: 0 matches, expected ≥ 1

## Tables

### TBL-1 has table with columns in order

input:
```markdown
## Features
| priority | epic | feature |
|---|---|---|
| p0 | **Core** | Parse |
```
config:
```yaml
- select: SECTION("Features") > TABLE
  columns: [priority, epic, feature]
```
expected: pass

### TBL-2 missing / misordered column

input: TBL-1 with header `| epic | priority | feature |`

config: same as TBL-1

expected:
- header line — column order `epic, priority, feature`, expected `priority, epic, feature`

### TBL-3 column enum

input: TBL-1 with a row `| p9 | **X** | Y |`

config:
```yaml
- select: SECTION("Features") > TABLE > TD[col="priority"]
  enum: [p0, p1, p2, p3]
```
expected:
- row line — `p9` not in allowed set

### TBL-4 column no empty values

input: row with empty `feature` cell

config:
```yaml
- select: TABLE > TD[col="feature"]
  no_empty: true
```
expected:
- row line — empty cell in column `feature`

### TBL-5 rows bounds

config:
```yaml
- select: TABLE
  rows: { min: 2, max: 50 }
```
expected: finding when body rows outside bounds

### TBL-6 per-cell length and regexp

config:
```yaml
- select: TABLE > TD[col="capabilities"]
  max_length: 120
  forbid: 'TBD'
```
expected: finding per offending cell, row line each

## Links

### LNK-1 all links are wiki-links

input:
```markdown
See [[features/x|X]] and [ext](https://e.com).
```
config:
```yaml
- select: A
  wiki: true
```
expected:
- line 1 — `[ext](https://e.com)` is not a wiki-link

### LNK-2 wiki-link resolution

input: `[[features/x/stories|X]]` with `--root ./vault`; file `vault/features/x/stories.md` absent

config:
```yaml
- select: A[wiki]
  resolve: true
```
expected:
- line — target `features/x/stories.md` not found under root

### LNK-3 forbid external links

config:
```yaml
- select: A[href=/^https?:/]
  count: 0
```
expected: finding per external link

## Load-stage validation

### LOAD-1 malformed frontmatter

input:
```markdown
---
confirmed: no
# Doc
```
(unclosed block)

expected (any config):
- line 1 — `check: load` — unclosed frontmatter block

## CLI surface

### CLI-1 stdin

`cat doc.md | mdcheck rules.yaml -` → findings carry `file: "<stdin>"`; exit 1 when findings exist.

### CLI-2 multiple files

`mdcheck rules.yaml a.md b.md` → findings tagged per file; exit 0 only if all clean.

### CLI-3 json format

`--format json` → single object `{files, findings[], summary:{checked, failed}}`; nothing else on stdout.

### CLI-4 bad rule file

unknown key in rules yaml → stderr message naming key + rule index; exit 2; no findings output.
