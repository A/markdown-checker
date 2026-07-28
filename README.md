# mdcheck

Deterministic structural checks over markdown documents. A YAML rule file
declares checks as CSS-like selectors plus assertions; `mdcheck` applies them to
files or stdin and emits uniform findings.

The full spec lives in [specs/specs.md](specs/specs.md), the behavioural
contract in [specs/scenarios.md](specs/scenarios.md).

## Install

```bash
cargo install --path .        # installs the `mdcheck` binary
cargo build --release         # or just build it: target/release/mdcheck
```

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

Machine-readable output, and stdin:

```bash
mdcheck rules.yaml notes.md --format json
cat notes.md | mdcheck rules.yaml -          # findings carry file "<stdin>"
```

```json
{
  "files": [
    "notes.md"
  ],
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
  "summary": {
    "checked": 1,
    "failed": 1
  }
}
```

`summary.checked` counts the files checked, `summary.failed` the ones that
produced at least one finding.

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

`-` as a path reads the document from stdin. stdout carries findings only;
usage errors, rule-file errors and diagnostics go to stderr.

## Exit codes

| code | meaning |
|---|---|
| 0 | all checks pass |
| 1 | findings emitted |
| 2 | usage, rule-file or selector error |
| 3 | internal error |

## Rule files

A rule file names a doc type and lists rules; every rule **selects** elements
and then **asserts** on them. [rules/feature-index.yaml](rules/feature-index.yaml)
is the worked example from the spec — mdcheck dogfoods it:

```bash
mdcheck rules/feature-index.yaml tests/fixtures/feature-index-good.md   # exit 0
```

Selectors: `H1`..`H6`, `SECTION`, `P`, `UL`/`OL`/`LI`, `TABLE`/`TR`/`TD`, `A`,
`CODE`, `FRONTMATTER`, `DOCUMENT`; combined with `>` (child), space
(descendant) and `+` (adjacent sibling); filtered by attribute
(`[text=/^I can /]`, `[col="priority"]`, `[href=/^https?:/]`, `[wiki]`,
`[lang="gherkin"]`) or by index (`P[0]`, `P[-1]`). `H2("Roles")` and
`SECTION("Roles")` are sugar for the `text` / `title` filters and accept
regexps: `SECTION(/^Story: /)`.

Assertions: `count`, `min`, `max`, `min_lines`, `match`, `require`, `forbid`,
`starts_with`, `ends_with`, `min_length`, `max_length`, `enum`, `no_empty`,
`ordered` (+ `strict`), `columns`, `rows`, `has` / `not_has`, `wiki`,
`resolve`, `optional`, and `each:` for rules nested per outer match.

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```
