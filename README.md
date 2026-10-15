# mdcheck

**Deterministic specs for markdown documents.** You declare what a document must contain — sections in this order, a table with these columns, every list item shaped like this, every link resolvable — and `mdcheck` tells you exactly which line breaks which rule. No model in the loop, no judgement call, same answer every run.

That determinism is the point when an **AI agent** is the one writing the markdown. Agents are good at producing documents and bad at producing them to a fixed shape. An LLM-as-judge check on that output is slow, costs tokens, and disagrees with itself between runs. A rule file does not:

```text
specs/feature-index.md:14  feature-index.rule[6]  `p9` not in allowed set: p0, p1, p2, p3
```

The agent gets a line number and a reason, fixes that line, and re-runs until the exit code is `0`. The same run is a CI gate for humans, and a [promptfoo](https://promptfoo.dev) assertion for eval suites — `--format json` is the machine-readable form.

Rules are **declarative**: a rule selects part of the document with a CSS-like selector, then asserts something about the selection. There is no scripting, no plugin API, and nothing to run but the binary.

## Install

```bash
cargo install markdown-checker
```

Needs a Rust toolchain, nothing else. The binary is `mdcheck`. Pin a version if you want reproducible installs:

```bash
cargo install markdown-checker --version 0.1.1
```

Or install straight from git for the latest unreleased changes:

```bash
cargo install --git https://github.com/A/markdown-checker
```

From a clone: `cargo install --path .`, or `cargo build --release` for `target/release/mdcheck`.

## Quickstart

```bash
cat > rules.yaml <<'YAML'
doc: notes

rules:
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }

  - select: H1
    count: 1
    max_length: 60

  - select: H1 > H2
    ordered: [Roles, Features]
    strict: true

  - select: SECTION("Roles") > UL > LI
    min: 2
    match: '^\*\*.+\*\* — .+'
YAML

cat > notes.md <<'MD'
---
confirmed: no
---
# Notes

## Roles

- **Admin** — manages the index
- broken item

## Features

Nothing yet.
MD

mdcheck rules.yaml notes.md
```

```text
notes.md:9  notes.rule[3]  `broken item` does not match `^\*\*.+\*\* — .+`
```

One run reports every violation, never just the first.

## What you can check

Each block below is a complete rule — `select:` picks the elements, the rest asserts on them.

**Frontmatter** — required keys, allowed values, list lengths, forbidden keys:

```yaml
- select: FRONTMATTER
  has: { confirmed: { enum: [yes, no] }, priority: '^P[0-3]$', tags: { min: 1 } }
  not_has: [draft]
```

**Document skeleton** — which sections exist, in what order, and nothing else (`?` marks an optional member, `strict:` forbids strays):

```yaml
- select: H1 > H2
  ordered: [Roles, Priority levels, Features, Questions?]
  strict: true
```

**Lists** — scoped to one section, counted, and shaped:

```yaml
- select: SECTION("Roles") > UL > LI
  min: 2
  match: '^\*\*.+\*\* — .+'
```

**Tables** — header set and order, row bounds, then per-column cell rules addressed by name:

```yaml
- select: SECTION("Features") > TABLE
  columns: [priority, epic, feature, capabilities]
  rows: { min: 5 }

- select: SECTION("Features") > TABLE > TD[col="priority"]
  enum: [p0, p1, p2, p3]

- select: SECTION("Features") > TABLE > TD[col="capabilities"]
  max_length: 120
  forbid: 'TBD'
  no_empty: true
```

**Links** — wiki-links only, and every target must exist under `--root`:

```yaml
- select: A
  wiki: true

- select: A[wiki]
  resolve: true
```

**Text and size** — regexps in three modes (`match` whole value, `require` appears, `forbid` must not appear), literal prefixes, length and line bounds:

```yaml
- select: P
  max_length: 200
  forbid: 'TBD'

- select: DOCUMENT
  min_lines: 20
```

**Repeated blocks** — `each:` runs a nested rule list once per outer match, so a failure names the block it happened in:

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

**Code fences** — by language, for documents that carry embedded specs:

```yaml
- select: CODE[lang="gherkin"]
  min: 1
```

Selectors compose the way CSS does: `>` child, space descendant, `+` adjacent sibling; `[text=…]`, `[title=…]`, `[col=…]`, `[href=…]`, `[wiki]`, `[lang=…]` filters with `=` `^=` `$=` `*=` and regexp forms; `P[0]` / `P[-1]` for position; `H2("Roles")` and `SECTION(/^Story: /)` as sugar.

## Usage

```text
mdcheck <RULES> [PATHS]... [--format pretty|json] [--root <DIR>] [--quiet]
```

| flag | meaning |
|---|---|
| `--format pretty` | default; one line per finding, `file:line  check  message`, grouped by file |
| `--format json` | one JSON object: `{files, findings[], summary{checked, failed}}` |
| `--root <DIR>` | base directory for `resolve:` link targets; defaults to each file's own directory (stdin: the working directory) |
| `--quiet` | print nothing, report through the exit code only |

`-` as a path reads the document from stdin, and its findings carry `file: "<stdin>"`. stdout carries findings only; usage errors, rule-file errors and diagnostics go to stderr — so the json is safe to pipe.

```bash
mdcheck rules.yaml notes.md --format json | jq '.summary'
cat notes.md | mdcheck rules.yaml -
```

```json
{
  "files": ["notes.md"],
  "findings": [
    {
      "check": "notes.rule[3]",
      "severity": "error",
      "file": "notes.md",
      "line": 9,
      "expected": "matches `^\\*\\*.+\\*\\* — .+`",
      "actual": "broken item",
      "message": "`broken item` does not match `^\\*\\*.+\\*\\* — .+`"
    }
  ],
  "summary": { "checked": 1, "failed": 1 }
}
```

`summary.checked` counts the files checked, `summary.failed` the ones that produced at least one finding.

## Exit codes

| code | meaning |
|---|---|
| 0 | all checks pass |
| 1 | findings emitted |
| 2 | usage, rule-file or selector error |
| 3 | internal error |

A rule file that will not load is exit `2`, not a finding — a broken rule must never look like a clean document.

## Documentation

Start at [docs/](docs/README.md):

| page | what it covers |
|---|---|
| [The document model](docs/document-model.md) | What mdcheck sees: elements, sections, frontmatter, source lines |
| [Selectors](docs/selectors.md) | Elements, combinators, filters, sugar, indexes, scoping |
| [Assertions](docs/assertions.md) | Every assertion key, grouped by what you are trying to say |
| [Rule files](docs/rule-files.md) | Rule anatomy, `optional:`, `each:`, composing and debugging a rule file |
| [Findings and output](docs/output.md) | Findings, output formats, exit codes, CI and agent loops |
| [Cookbook](docs/cookbook.md) | Complete rule files for real specs, plus patterns and anti-patterns |

[docs/example.yaml](docs/example.yaml) is the worked example from the spec — mdcheck dogfoods it:

```bash
mdcheck docs/example.yaml tests/fixtures/feature-index-good.md   # exit 0
```

The normative definition lives in [specs/specs.md](specs/specs.md) (interface and grammar) and [specs/e2e/](specs/e2e/) (the behavioural contract in Gherkin, indexed by [specs/index.md](specs/index.md)).

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```
