# Running mdcheck: findings, output and exit codes

This page is about invoking `mdcheck` and reading what comes back. You give it a rule file and one or more markdown documents; it gives you a list of findings — one per violation, each pinned to a file and a line — plus an exit code your shell or CI can branch on. Nothing here is about writing rules; that lives in [Rule files](rule-files.md).

## The command surface

```bash
mdcheck rules.yaml specs/index.md              # one document named by path
mdcheck rules.yaml specs/*.md                  # several documents in one run
cat draft.md | mdcheck rules.yaml -            # "-" reads the document from stdin
mdcheck rules.yaml specs/index.md --format json  # machine-readable output
mdcheck rules.yaml specs/index.md --root ./vault # base for resolving wiki-links
```

The first argument is always the rule file. Everything after it is a document to check. `--root` names the directory that link targets are resolved against; it changes nothing for a run whose rules assert nothing about links.

## What a finding is

Every violation produces one finding with these fields:

| field | meaning |
|---|---|
| `check` | which check produced the finding. Load-stage problems — a malformed frontmatter block, broken link syntax — carry `check: load` |
| `severity` | how the finding is classified |
| `file` | the document the finding came from, so you can go straight to it |
| `line` | the line of the offending element itself |
| `expected` | what the rule demanded |
| `actual` | what the document had instead |
| `message` | the two put together, in a form you can read |

`line` is the element's own source line, not the line of the rule's scope and not the start of the block it sits in. A header that is too long reports at the header. A frontmatter key with a disallowed value reports at that key. Document-wide checks, such as a minimum line count, report at line 1.

## All findings, every run

A run never stops at the first failure. A failing rule does not suppress the rules after it, and a failing match does not suppress the other matches of the same rule.

```markdown
---
confirmed: maybe
---
# Doc
## This header is way too long for the configured limit
```

```yaml
rules:
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }
  - select: DOCUMENT
    min_lines: 20
  - select: H2
    max_length: 20
```

That run reports three findings — at line 2, at line 1 and at line 5 — not one. This is the point of the tool: you read one run, fix everything it names, and run again to confirm. You are not peeling failures off one at a time.

The same holds within a single rule. Two `H2` headers over the limit produce two findings, at their own lines.

## Several documents in one run

Name as many documents as you like. Each finding carries the file it came from, so findings stay attributable:

```bash
$ mdcheck rules.yaml a.md b.md
a.md:2: length 52 > 20
b.md:2: length 53 > 20
```

The run passes only if every document is clean. One clean document and one failing document is a failing run — a clean document simply contributes no findings.

## Reading from stdin

Pass `-` as the path to check a document from a pipe. Its findings carry `file` as the literal string `<stdin>`:

```bash
$ cat draft.md | mdcheck rules.yaml -
<stdin>:2: length 52 > 20
```

## The two output modes

Default output is for a person at a terminal. `--format json` is for a machine. The format chooses the presentation and never the findings: both modes report exactly the same set for the same run.

Take this failing document, `specs/index.md`:

```markdown
---
confirmed: maybe
---
# Doc
## This header is way too long for the configured limit
```

against these rules:

```yaml
rules:
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }
  - select: H2
    max_length: 20
```

Default output — each finding locates itself without further tooling:

```bash
$ mdcheck rules.yaml specs/index.md
specs/index.md:2: `confirmed` is `maybe`, allowed: `yes`, `no`
specs/index.md:5: length 52 > 20
```

`--format json` — one object on stdout and nothing else:

```bash
$ mdcheck rules.yaml specs/index.md --format json
```

```json
{
  "files": ["specs/index.md"],
  "findings": [
    {
      "check": "FRONTMATTER has confirmed",
      "severity": "error",
      "file": "specs/index.md",
      "line": 2,
      "expected": "yes | no",
      "actual": "maybe",
      "message": "`confirmed` is `maybe`, allowed: `yes`, `no`"
    },
    {
      "check": "H2 max_length",
      "severity": "error",
      "file": "specs/index.md",
      "line": 5,
      "expected": "<= 20",
      "actual": "52",
      "message": "length 52 > 20"
    }
  ],
  "summary": { "checked": 1, "failed": 1 }
}
```

The exact line layout of the default output and the exact strings in `check` and `severity` are illustrative — the specification fixes that each finding is presented with its document, its line and its message, not the characters between them. The json keys are fixed: `files`, `findings`, `summary`, and `summary` counts documents, not findings — `checked` is how many documents the run looked at, `failed` is how many of them produced at least one finding.

A clean run in json emits the same object with an empty list:

```json
{ "files": ["specs/index.md"], "findings": [], "summary": { "checked": 1, "failed": 0 } }
```

## Stream discipline

Findings go to stdout. Everything mdcheck says about itself — a missing file, an unloadable rule file, an unparsable selector — goes to stderr. A run that cannot start writes nothing at all to stdout.

That separation is what makes `--format json` pipeable: stdout is one json document or it is empty, and a diagnostic never lands in the middle of it.

```bash
mdcheck rules.yaml specs/*.md --format json | jq '.findings[] | .file + ":" + (.line|tostring)'
```

## Exit codes

| code | meaning |
|---|---|
| `0` | all checks pass |
| `1` | findings emitted |
| `2` | usage, rule-file or selector error |
| `3` | internal error |

The distinction between `1` and `2` is the one that matters to a caller: `1` means the documents are wrong, `2` means the call or the configuration is wrong. A missing document path, no document path at all, a rule file that will not load, a selector that will not parse — all `2`, all reported as a diagnostic on stderr and never as a finding.

## Using it in CI

The exit code is the gate; `--format json` is the artifact. Because everything but findings stays on stderr, one invocation can do both.

A plain shell step:

```bash
#!/usr/bin/env bash
mdcheck rules.yaml specs/*.md --format json > mdcheck.json
status=$?
case $status in
  0) echo "specs clean" ;;
  1) echo "specs have findings; see mdcheck.json" >&2 ;;
  *) echo "mdcheck could not run (exit $status)" >&2 ;;
esac
exit $status
```

Do not run this under `set -e`: it would abort before you could tell exit `1` from exit `2`.

A GitHub Actions step — the run fails the job on findings, and the artifact is uploaded either way:

```yaml
- name: Check specs
  run: mdcheck rules.yaml specs/*.md --format json | tee mdcheck.json

- name: Upload findings
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: mdcheck-findings
    path: mdcheck.json
```

## Using it with AI agents

If an AI agent is filling in a document against a template you wrote, mdcheck is the objective feedback signal it iterates against. The agent writes the document, you run mdcheck, and the findings say exactly which line breaks which rule — `specs/index.md:5: length 52 > 20`, not "the header seems long". The agent fixes that line and the loop repeats until the run exits `0`.

Two properties make this work. All findings come back in one run, so the agent gets the whole remaining gap in a single step instead of rediscovering one violation per attempt. And `--format json` gives a shape an agent harness can read directly — iterate `findings[]`, feed `file`, `line` and `message` back into the next prompt, stop when `summary.failed` is `0`.

promptfoo is one such consumer: point an assertion at a `--format json` run and the assertion passes or fails on the same findings you read at the terminal.

## Not yet settled

- The vocabulary of `severity` — which values it takes and whether any of them is non-fatal.
- How `check` names a finding from a declared rule. Only load-stage findings have a fixed value, `load`.
- Whether `files` lists plain paths or objects carrying per-document detail.
- The order findings are emitted in — by rule, by file, or by line.
- What triggers exit code `3`. It is fixed only as "not a finding and not a bad call"; no input is specified that provokes it.

**Next:** [The document model](document-model.md) · [Selectors](selectors.md) · [Assertions](assertions.md) · [Rule files](rule-files.md) · [Cookbook](cookbook.md) · back to [docs](README.md)
