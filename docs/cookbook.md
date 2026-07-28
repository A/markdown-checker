# Cookbook

This page is four complete rule files, each next to the document it was written for. The other pages take the primitives one at a time; here you see what a whole-document contract looks like when the primitives are stacked — a frontmatter gate, a header skeleton, the shape of the lists and tables inside it, and a link policy — and what the run prints when a document drifts out of contract. Every rule file below is checked against the document beside it: the good documents produce no findings, the violations produce exactly the findings shown.

## 1. A feature index

A feature index is the top of a spec vault. It is written once by a human, then extended by agents, so it needs the tightest contract: the frontmatter says whether it has been confirmed, the H1 names the product, a fixed set of H2 sections appears in a fixed order, roles are one-line definitions, and the feature table is the payload — every row priced, every capability filled in, every link pointing at a file that exists.

### The document

`specs/feature-index.md`:

```markdown
---
confirmed: no
type: feature-index
---
# Search feature index

## Roles

- **Admin** — manages the index
- **Visitor** — browses features

## Priority levels

1. p0 — must ship
2. p1 — should ship

## Features

| priority | epic | feature | capabilities |
|---|---|---|---|
| p0 | **Core** | Parsing | reads markdown |
| p1 | **Core** | Rendering | writes html |
| p2 | **Api** | Search | queries the index |

### Backlog

Deferred work lives in [[features/backlog|the backlog]].

## Questions

See [[questions/open|open questions]].
```

### The rule file

`docs/example.yaml`:

```yaml
doc: feature-index

rules:
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }
    not_has: [draft]

  - select: DOCUMENT
    min_lines: 20

  - select: H1
    count: 1
    match: '^\S+ feature index$'
    max_length: 60

  - select: H1 > H2
    ordered: [Roles, Priority levels, Features, Questions?]
    strict: true

  - select: SECTION("Roles") > UL > LI
    min: 2
    match: '^\*\*.+\*\* — .+'

  - select: SECTION("Features") > TABLE
    columns: [priority, epic, feature, capabilities]
    rows: { min: 3 }

  - select: SECTION("Features") > TABLE > TD[col="priority"]
    enum: [p0, p1, p2, p3]

  - select: SECTION("Features") > TABLE > TD
    no_empty: true

  - select: SECTION("Features") > TABLE > TD[col="capabilities"]
    max_length: 40

  - select: SECTION("Features") > TABLE > TD[col="capabilities"]
    forbid: 'TBD'

  - select: A
    wiki: true

  - select: A[wiki]
    resolve: true
```

Run it with a vault root, because the last rule opens files:

```bash
mdcheck docs/example.yaml specs/feature-index.md --root specs
```

What each group buys you:

- **Frontmatter** — the document declares its own review state, and cannot ship with the `draft` scaffolding key still attached.
- **Document and H1** — the file is not a stub, and its title follows the vault's naming convention so an index is findable by name.
- **The skeleton** — `ordered:` fixes which H2 sections exist and in what order, `Questions?` marks the one that may be absent, and `strict: true` stops anyone appending a section the contract never planned for. `### Backlog` is untouched: `H1 > H2` reaches H2 headers only, so a subsection is free to appear anywhere.
- **Roles** — at least two roles, each written as `**Name** — description` rather than as prose, so the roles are extractable.
- **The table** — the four columns exist in that order, there are at least three rows, priorities come from the fixed set, no cell is blank, and a capability is a short filled-in phrase rather than a placeholder. These are five rules rather than one, so a bad priority and an empty cell are two findings in two places.
- **Links** — every link is a wiki-link, and every wiki-link resolves to a file under `--root`. Splitting these means an inline link is reported as an inline link, and is not also reported as an unresolvable one.

### What a violation looks like

The same document after a careless edit — the frontmatter half-filled, the title reworded, a role written as prose, a bad priority, a blank cell, a placeholder, and a section nobody declared:

```markdown
---
confirmed: maybe
draft: true
type: feature-index
---
# Search index of features

## Roles

- **Admin** — manages the index
- browses features

## Priority levels

1. p0 — must ship
2. p1 — should ship

## Features

| priority | epic | feature | capabilities |
|---|---|---|---|
| p0 | **Core** | Parsing | reads markdown |
| p9 | **Core** | Rendering | writes html |
| p2 | **Api** |  | TBD |

## Questions

See [[questions/open|open questions]].

## Extra

See [open questions](https://example.com/questions).
```

One run, exit code 1, every rule still evaluated:

```text
specs/feature-index.md:2   `confirmed` is `maybe`, allowed: `yes`, `no`
specs/feature-index.md:3   key `draft` must not be present
specs/feature-index.md:6   does not match pattern
specs/feature-index.md:30  section `Extra` not allowed (strict)
specs/feature-index.md:11  does not match pattern
specs/feature-index.md:23  `p9` in column `priority` not in allowed set
specs/feature-index.md:24  empty cell in column `feature`
specs/feature-index.md:24  column `capabilities` matches forbidden `TBD`
specs/feature-index.md:32  `[open questions](https://example.com/questions)` is not a wiki-link
```

Note what is *not* reported. `Questions` is optional and present, so it passes; the document is 32 lines, so `min_lines` passes; `TBD` is three characters, so the capability length cap passes while the placeholder rule catches it. Each finding names one thing to fix.

## 2. A user-stories document

A stories document is a list of blocks that all have the same shape: a heading, an optional precondition, a role line, and the acceptance list. You are not checking the prose — you are checking that no block was left half-written. This is what `each:` is for: the inner rules run once per block, so a block that satisfies them contributes nothing and a block that does not is named individually.

### The document

`specs/checkout/stories.md`:

```markdown
---
confirmed: no
---
# Checkout user stories

## User Stories

### Sign in

pre: the visitor has an account

As **Visitor**

- I can sign in with my email
- I can stay signed in on this device

### Pay

As **Customer**

- I can pay with a saved card
- I can pay with a new card

### Refund

pre: the order is delivered

As **Support agent**

- I can refund an order in full
```

### The rule file

```yaml
doc: user-stories

rules:
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }

  - select: H1
    count: 1
    match: '^\S+ user stories$'

  - select: H1 > H2
    ordered: [User Stories]
    strict: true

  - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
    min: 3
    each:
      - select: P[text^="pre:"]
        optional: true
        max: 1

      - select: P[text=/^As \*\*.+\*\*/]
        count: 1

      - select: P[text=/^As \*\*.+\*\*/] + UL > LI
        min: 1
        match: '^I can .+'
```

What each group buys you:

- **The outer rule** — story blocks are the H3 sections inside `## User Stories`, addressed by a title pattern rather than by name because you do not know what they will be called. `min: 3` demands that blocks exist at all; without a cardinality assertion, a document with no blocks would satisfy the `each:` list vacuously.
- **The precondition** — at most one `pre:` paragraph per block, and none is acceptable. Selecting it by its own prefix is what makes it genuinely optional. Writing it as `P[0]` with `starts_with: 'pre:'` — the form in the spec's own example — says something stricter: *if the block has any paragraph at all, its first one must be a precondition*, which fails on the `Pay` block above, whose first paragraph is the role line.
- **The role line** — exactly one `As **Role**` paragraph, so a block cannot be assigned to two roles or to none.
- **The acceptance list** — the list *immediately after* the role line, not any list in the block, and every item in it phrased as a capability. `+` is doing real work here: a list further down the block, under a nested heading or after commentary, is not the acceptance list.

### What a violation looks like

An agent adds a fourth block and writes it as prose:

```markdown
### Cancel

Cancelling is described in the refunds policy.
```

with `### Cancel` on line 32:

```text
specs/checkout/stories.md:32  inner rule `P[text=/^As \*\*.+\*\*/]`: 0 matches, expected exactly 1
specs/checkout/stories.md:32  inner rule `P[text=/^As \*\*.+\*\*/] + UL > LI`: 0 matches, expected ≥ 1
```

Both findings point at the block that is wrong and name the inner rule that failed. The three well-formed blocks report nothing, and the optional precondition rule is excused in `Cancel` exactly as it is in `Pay` — optionality is judged per block, not per document.

## 3. A Gherkin spec document

A behavioural spec is prose plus executable examples, and the failure mode is examples that drift into the prose — a scenario written as a paragraph, or dropped into an unlabelled fence where no Gherkin tooling will find it. The contract says: every section carries exactly one fence, that fence is `gherkin`, and it contains scenarios.

### The document

`specs/e2e/tables.md`:

````markdown
---
confirmed: no
---
# E2E: Tables

How a rule pins down a table's column header set and order, and bounds how many
body rows it carries.

## Declaring a table's columns

```gherkin
Feature: A rule fixes the column header set and its order

  Rule: A header carrying exactly the named columns, in order, satisfies columns:

    @TBL-1
    Scenario: A matching header passes
      Given a document with a Features table
      When mdcheck checks the document
      Then the run passes with no findings
```

## Bounding the number of body rows

```gherkin
Feature: A rule bounds how many rows of data a table carries

  Rule: A body row count outside the bounds is reported at the table header line

    @TBL-5
    Scenario: A table with one body row is reported
      Given a Features table with 1 body row
      When mdcheck checks the document
      Then a finding is reported at the table header line
```
````

### The rule file

```yaml
doc: e2e-spec

rules:
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }

  - select: H1
    count: 1
    match: '^E2E: \S.*'

  - select: H1 > H2
    min: 2
    match: '^[A-Z].+'
    max_length: 70

  - select: P
    forbid: 'Scenario:'

  - select: SECTION(/^E2E: /) > SECTION(/^[A-Z]/)
    min: 2
    each:
      - select: CODE
        count: 1

      - select: CODE[lang="gherkin"]
        count: 1
        require: 'Scenario:'
```

What each group buys you:

- **H1 and H2** — the file announces itself as an E2E spec, and every section title is a capitalised phrase short enough to read in a list.
- **`P` forbids `Scenario:`** — a scenario written as a paragraph is the exact drift you are guarding against, and this catches it wherever in the document it happens.
- **The `each:` pair** — inside every section, `CODE count: 1` says there is one fence and `CODE[lang="gherkin"] count: 1` says that fence is Gherkin. Together they forbid an unlabelled fence without forbidding fences. `require:` then asserts the fence has at least one scenario in it.
- **`min: 2` on the sections** — the document carries at least two example blocks. mdcheck counts fences, not `Scenario:` lines inside them, so this contract deliberately puts one `Feature:` block per section and counts sections.

### What a violation looks like

A third section, added with the examples in a bare fence and a scenario title left in the prose:

````markdown
## Naming a column the table does not have

Scenario: a cell rule whose column is absent

```
Given a Features table with no owner column
Then nothing is reported
```
````

with `## Naming a column…` on line 37:

```text
specs/e2e/tables.md:39  contains forbidden pattern
specs/e2e/tables.md:37  inner rule `CODE[lang="gherkin"]`: 0 matches, expected exactly 1
```

`CODE count: 1` passes — there *is* one fence — and the language rule is the one that fails, which is the finding that tells you what to change.

## 4. Docs hygiene

This is the rule file you drop on every markdown file in a repo, not on one document type. It asserts nothing about structure, because it does not know the structure; it asserts only the things that are wrong in any document. Every rule whose element may legitimately be absent carries `optional: true`, which is what makes the file portable.

### The rule file

`hygiene.yaml` — put it wherever your repo keeps tooling config:

```yaml
doc: any-markdown

rules:
  - select: DOCUMENT
    min_lines: 15

  - select: H1
    count: 1
    max_length: 60

  - select: H2
    max_length: 60
    optional: true

  - select: H3
    max_length: 60
    optional: true

  - select: P
    forbid: 'TBD'
    optional: true

  - select: LI
    forbid: 'TBD'
    optional: true

  - select: TABLE > TD
    forbid: 'TBD'
    optional: true

  - select: A[href=/^https?:/]
    count: 0
```

Run it over everything:

```bash
mdcheck hygiene.yaml docs/*.md specs/**/*.md
```

What each group buys you:

- **`min_lines`** — a page that is a heading and a sentence is a stub, and a stub in a docs tree is worse than a missing file.
- **One H1, capped headers** — one title per document, and no header long enough to be a paragraph wearing a `##`.
- **`forbid: 'TBD'`** — three rules rather than one, because placeholders hide in prose, in lists and in table cells, and each element type reports at its own line. Each is optional, so a document with no table is not a finding.
- **`count: 0` on external links** — an internal docs tree links internally; anything with a scheme is either a citation that belongs in a references section or a link that will rot. Zero matches passes, so a document with no links at all is silent.

### What a violation looks like

`docs/notes.md`:

```markdown
# Notes

Status: TBD

See [the spec](https://example.com/spec).
```

```text
docs/notes.md:1  document has 5 lines, expected ≥ 15
docs/notes.md:3  contains forbidden pattern
docs/notes.md:5  link `https://example.com/spec` matched, expected 0 matches
```

The H2, H3, list and table rules are all excused — their selectors match nothing and they are marked optional. Drop the `optional: true` from any of them and this document starts reporting the absence of things it never claimed to have.

## Patterns

**Scope a rule to a section before asserting.** `SECTION("Roles") > UL > LI` says "the roles list", where `UL > LI` says "any list anywhere"; a rule that judges the wrong list reports a finding the author cannot act on.

**Use `each:` when the document has repeated blocks.** Without it a rule says "somewhere in this document there are at least three acceptance items"; with it the rule says "every story block has at least one", and names the block that does not.

**Use `optional: true` to say "this may be absent" out loud.** It is the difference between a rule that tolerates absence and a rule that happens not to have been tested against an empty document — and inside `each:` it is judged one block at a time.

**Assert the skeleton with `ordered:` + `strict:` before asserting anything inside it.** A missing section otherwise surfaces as a puzzling "0 matches" from some rule deep inside it, instead of as "required section `Features` missing".

**Keep one claim per rule, so one document mistake produces one finding.** The feature-index table takes five rules for this reason: a bad priority and a `TBD` capability are separate mistakes, made by different people, fixed in different edits.

## Anti-patterns

**Rules that break on presentation rather than content.** Asserting a section's exact prose, or its line count where a section boundary would do, turns a rewording into a failure. Assert the shape you need to read out — a role line, a filled cell, a resolvable link — and let the wording move.

**One giant rule that reports everything at once.** Piling every assertion about a section onto a single selector means the author fixes one thing, reruns, and is told about the next one. Findings are collected per run precisely so a whole document can be fixed in one pass; splitting rules is what makes that work.

**Regexps that encode formatting instead of meaning.** `'^\*\*.+\*\* — .+'` is a contract about roles — a bolded name, a separator, a definition. A pattern that also pins the number of spaces, the dash character or the capitalisation of the description is a contract about typing, and it will fail on documents that are perfectly correct.

**Next** — back to [docs](README.md). The pieces these recipes are built from are in [The document model](document-model.md), [Selectors](selectors.md), [Assertions](assertions.md) and [Rule files](rule-files.md); the finding lines quoted above are explained in [Findings and output](output.md).
