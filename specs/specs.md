# mdcheck — technical spec

Deterministic structural checks over markdown documents. A YAML rule file declares checks as CSS-like selectors + assertions; `mdcheck` applies them to files or stdin and emits uniform findings.

Every failed check emits a finding `{check, severity, file, line, expected, actual, message}`. All findings are collected per run — never first-fail. Machine consumers (promptfoo asserts, CI) read `--format json`; humans read the default pretty output.

## 1. Desired interface

CLI:

```bash
mdcheck feature-index.yaml specs/feature-index.md      # file(s)
cat doc.md | mdcheck feature-index.yaml -              # stdin
mdcheck rules.yaml doc.md --format json                # machine output
mdcheck rules.yaml doc.md --root ./vault               # link-resolution base
```

Exit codes: `0` all checks pass · `1` findings emitted · `2` usage / rule-file / selector error · `3` internal error. stdout carries findings only; diagnostics go to stderr.

Rule file — flat list of rules; each rule **selects** elements, then **asserts** on them:

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
    ordered: [Roles, Priority levels, Features, Questions?]   # ? = optional
    strict: true      # any H2 outside this list = error

  - select: SECTION("Roles") > UL > LI
    min: 2
    match: '^\*\*.+\*\* — .+'

  - select: SECTION("Features") > TABLE
    columns: [priority, epic, feature, capabilities]
    rows: { min: 5 }

  - select: SECTION("Features") > TABLE > TD[col="priority"]
    enum: [p0, p1, p2, p3]

  - select: A
    wiki: true        # all links must be wiki-links
    resolve: true     # and must point at existing files
```

Nested scoping with `each:` — inner rules run per outer match:

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

## 2. Selector grammar

Elements:

| selector | selects |
|---|---|
| `H1` … `H6` | headers by level |
| `SECTION` | header + its body (until next same-level header) |
| `P` | paragraph |
| `UL`, `OL`, `LI` | lists and items |
| `TABLE`, `TR`, `TD` | tables, rows, cells |
| `A` | links |
| `CODE` | fenced code block |
| `FRONTMATTER` | frontmatter block |
| `DOCUMENT` | whole document |

Combinators: `>` child · ` ` (space) descendant · `+` adjacent sibling (`P + UL` — list right after paragraph).

Attribute filters — CSS attribute-selector syntax, plus regex:

| filter | meaning |
|---|---|
| `[text="Roles"]` | text equals |
| `[text^="pre:"]` | starts with |
| `[text$="?"]` | ends with |
| `[text*="feature"]` | contains |
| `[text=/^I can .+/]` | matches regexp |
| `[title=…]` | SECTION title (same operators) |
| `[col="priority"]` | TD by column name |
| `[href=/…/]`, `[wiki]` | link target / wiki-style link |
| `[lang="gherkin"]` | CODE fence language |

Sugar: `H2("Roles")` ≡ `H2[text="Roles"]`; `SECTION("Roles")` ≡ `SECTION[title="Roles"]`; both accept regex: `H2(/^Story: /)`.

Index: bare integer in brackets — `P[0]` first match in scope, `[-1]` last.

## 3. Assertions

Regexp assertions come in three modes everywhere: `match` (whole value must match), `require` (must appear), `forbid` (must not appear).

| key | meaning |
|---|---|
| `count`, `min`, `max` | number of selector matches |
| `match` / `require` / `forbid` | regexp modes against element text |
| `starts_with`, `ends_with` | literal prefix/suffix |
| `min_length`, `max_length` | element text length bounds |
| `min_lines` | DOCUMENT line count |
| `enum` | value from allowed set |
| `no_empty` | no empty matches (cells, items) |
| `ordered: [...]` | header/section set order; `?` suffix = optional member |
| `strict: true` | no members outside the `ordered`/required list |
| `columns: [...]` | table header set + order |
| `rows: {min, max}` | table row count bounds |
| `has:` / `not_has:` | frontmatter keys; value spec = exact string, `enum:`, or regexp |
| `wiki: true` | selected links are wiki-links |
| `resolve: true` | link targets exist under `--root` |
| `optional: true` | rule passes when selector matches nothing |
| `each:` | nested rules run per outer match |

## 4. Load-stage validation

Before any declared rule runs, the document must parse cleanly. Load-stage findings (malformed frontmatter block, broken link syntax) are emitted with `check: load` and do not abort the remaining checks unless the document is unreadable.

## 5. Acceptance scenarios

The behavioral contract lives in [scenarios.md](scenarios.md) — each scenario is a triplet: input markdown → rule config → expected result. Scenarios are the seed corpus for the test suite: every scenario must exist as a test case.
