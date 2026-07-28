# mdcheck documentation

mdcheck checks markdown documents against a rule file you write. The rule file is declarative: each rule **selects** part of a document with a CSS-like selector, then **asserts** something about what it selected. A run reports every violation it finds, with a line number, and exits non-zero if there was one.

These pages explain the whole vocabulary. Read them in order the first time; after that, [Assertions](assertions.md) and the [Cookbook](cookbook.md) are the two you come back to.

| page | what it covers |
|---|---|
| [The document model](document-model.md) | What mdcheck sees in a markdown file: elements, sections, frontmatter, source lines, the load stage |
| [Selectors](selectors.md) | The addressing language — elements, `>` `+` and descendant combinators, attribute filters, call sugar, indexes, scoping |
| [Assertions](assertions.md) | What you claim about the selection — counts, regexp modes, lengths, enums, section order, table shape, frontmatter keys, links |
| [Rule files](rule-files.md) | The artifact you author and maintain: rule anatomy, `optional:`, `each:`, composing a whole-document contract, debugging the rule file itself |
| [Findings and output](output.md) | Running it: the finding record, pretty vs `--format json`, stream discipline, exit codes, CI and AI-agent loops |
| [Cookbook](cookbook.md) | Complete rule files for real specs — a feature index, a user-stories document, a Gherkin document, docs hygiene — plus patterns and anti-patterns |

[example.yaml](example.yaml) beside these pages is a runnable rule file — the feature-index contract from the spec, the one the cookbook walks through. Copy it as a starting point:

```bash
mdcheck docs/example.yaml your-doc.md
```

## The shortest possible version

```yaml
doc: notes

rules:
  - select: SECTION("Roles") > UL > LI     # of the list items in the Roles section…
    min: 2                                  # …there are at least two,
    match: '^\*\*.+\*\* — .+'               # …and each reads "**Name** — description"
```

Everything else is more selectors and more assertions.

## Where the behaviour is defined

The pages here are the explanation. The normative definition lives in the repo:

- [specs/specs.md](../specs/specs.md) — the technical spec: interface, selector grammar, assertion keys
- [specs/index.md](../specs/index.md) — the end-to-end use-case index, one row per test case
- [specs/e2e/](../specs/e2e/) — the behavioural contract in Gherkin, one file per area

Where a page says a behaviour is **not yet settled**, the spec genuinely does not decide it yet — do not rely on it in a rule file you intend to keep.
