# Rule files

A rule file is the YAML document you write and keep in your repository next to the specs it governs. It says what a document of a given kind must look like: which sections it carries, what shape its lists and tables take, what every repeated block must contain. `mdcheck` reads one rule file and applies it to one or more markdown documents. This page covers how a rule file is put together, how rules nest, how the whole thing reads as a contract for a document, and what happens when the rule file itself is wrong.

## Anatomy

A rule file has a `doc:` header naming the kind of document the rules describe, and a flat `rules:` list. Each entry in that list is one rule: exactly one `select:`, plus one or more assertions.

```yaml
doc: feature-index

rules:
  - select: H1
    count: 1
    max_length: 60

  - select: SECTION("Roles") > UL > LI
    min: 2
    match: '^\*\*.+\*\* — .+'
```

Against this document:

```markdown
# Feature index

## Roles

- **Admin** — manages the vault
- **Editor** — writes specs
```

both rules hold and the run exits 0 with no output.

The list is flat — rules are not grouped, ordered by dependency, or named. Every rule in the list runs, every run, and each collects its own findings; a rule that fails does not stop the rules after it. Rules are also indexed by their position in that list, counting from 0 at the top, which is how rule-file errors point at the rule that caused them.

## Select, then assert

A rule is a sentence. The selector is the subject — *the things this rule is about*. The assertions are the claim — *what I say about those things*.

```yaml
- select: SECTION("Features") > TABLE > TD[col="priority"]
  enum: [p0, p1, p2, p3]
```

Read it as: "of the priority cells in the Features table, every one holds p0, p1, p2 or p3."

That split is the whole language. [Selectors](selectors.md) decide the match set; [assertions](assertions.md) judge it. Some assertions judge the set as a whole (`count`, `min`, `max`, `ordered`) and some judge every member of it (`match`, `max_length`, `enum`) — but they all judge the same set, the one that `select:` produced.

Several assertions on one rule all apply to that same set, and each can report on its own:

```yaml
- select: TABLE > TD[col="capabilities"]
  max_length: 120
  forbid: 'TBD'
```

An over-long cell and a cell containing `TBD` are two separate findings, and a cell that does both produces two.

## `optional: true`

Not every part of a document is present in every document. A rule whose selector matches nothing has nothing to judge, and `optional: true` says that this is intended:

```yaml
- select: SECTION("Story") > P[0]
  optional: true
  starts_with: 'pre:'
```

Against a `Story` section with no paragraphs at all, this passes. The rule reads: "if this section opens with a paragraph, that paragraph is a precondition line."

You need this for anything that legitimately varies: a section only some documents carry, a lead-in line that is a convention rather than a requirement, a block that appears in the long form of a document and not the short one. Without `optional: true`, whether an empty match is silence or a finding depends on the rule's other keys — a rule that also says `min: 1` clearly demands matches, so an empty match set is a finding:

```yaml
- select: SECTION("Features") > TABLE > TD[col="priority"]
  min: 1
```

```
doc.md:1  0 matches, expected ≥ 1
```

Write `optional: true` when absence is acceptable and a cardinality key such as `min:` or `count:` when it is not. Stating the intent explicitly is better than leaving it to be inferred.

## `each:` — a rule list per block

`each:` takes a nested list of rules and runs it once for every match of the outer selector, with that match as the new root scope. It is the difference between "some block has one" and "every block has one".

```markdown
## User Stories

### Login

- I can log in

### Logout

Signing out is described in prose only.
```

A flat rule counting list items under the story sections is satisfied — there is one item in the document, so `min: 1` holds — and Logout goes unnoticed. Nested, the same assertion is made about each block separately:

```yaml
- select: SECTION("User Stories") > SECTION(/^[A-Z]/)
  each:
    - select: UL > LI
      min: 1
      match: '^I can .+'
```

```
doc.md  "Logout" section  inner rule `UL > LI`: 0 matches, expected ≥ 1
```

Three things follow from "the outer match is the new scope", and all three are what you want when checking repeated blocks:

**A finding names the block it happened in.** An inner rule that matched nothing is anchored at its outer match — the story section — and names the inner selector, so you know both which block and which claim.

**One bad block does not spare the others.** Every outer match is evaluated in the same run. Two broken blocks give two findings, and a broken block between two sound ones is the only one reported:

```markdown
## User Stories

### Login

some text

### Logout

more text

### Reset

- I can reset my password
```

```
doc.md  "Login" section   inner rule `UL > LI`: 0 matches, expected ≥ 1
doc.md  "Logout" section  inner rule `UL > LI`: 0 matches, expected ≥ 1
```

Nor does one broken inner rule suppress the rest of the inner list for the same block: a block that has neither a paragraph nor a list, checked by two inner rules, produces two findings for that block.

**A block sees only its own content.** Neighbouring blocks neither satisfy an inner rule nor count towards it. `count: 1` inside `each:` means one per block, not one in the document — a document with two story sections each holding a single item passes.

`optional:` inside an inner rule is judged per block too. A block with no lead paragraph is excused even when its sibling has one.

The outer rule keeps its own assertions. `each:` says what every block contains; a cardinality key on the outer rule says that blocks must exist at all:

```yaml
- select: SECTION("User Stories") > SECTION(/^[A-Z]/)
  min: 1
  each:
    - select: UL > LI
      min: 1
```

Against a `User Stories` section with no subsections, the outer `min: 1` reports `0 matches, expected ≥ 1` at the section, and the inner list has no scope to run in.

## A whole document contract

Rules compose by accumulation: each one pins down one property, and the file as a whole is the shape of the document. Here is a complete rule file for a feature spec, and the document it accepts.

```markdown
---
confirmed: no
---

# Feature: Password reset

## Roles

- **Visitor** — has no session
- **Member** — signed in

## User Stories

### Reset by email

pre: the member has a verified address

As **Member**

- I can request a reset link

### Reset by code

pre: the member has no access to email

As **Member**

- I can request a one-time code
```

```yaml
doc: feature-spec

rules:
  # The document declares its confirmation state, and is not left marked draft.
  - select: FRONTMATTER
    has: { confirmed: { enum: [yes, no] } }
    not_has: [draft]

  # Exactly one title, named as a feature, short enough to read in a list.
  - select: H1
    count: 1
    match: '^Feature: .+'
    max_length: 60

  # These top-level sections, in this order, and no others.
  # `Open questions?` may be absent; `strict` bans anything unlisted.
  - select: H1 > H2
    ordered: [Roles, User Stories, Open questions?]
    strict: true

  # At least two roles, each a bold name and a gloss.
  - select: SECTION("Roles") > UL > LI
    min: 2
    match: '^\*\*.+\*\* — .+'

  # At least one story block — and every block complete on its own terms.
  - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
    min: 1
    each:
      # A precondition line if there is one, but blocks may open with the actor.
      - select: P[0]
        optional: true
        starts_with: 'pre:'
      # The list right after the actor line holds capability statements.
      - select: P[text=/^As \*\*.+\*\*/] + UL > LI
        min: 1
        match: '^I can .+'
```

Read top to bottom, the file says: this document declares whether it is confirmed; it is titled as a feature; it has a roles section and a stories section in that order and nothing else beyond an optional open-questions section; roles are labelled; and every story block names an actor and lists at least one capability.

## When the rule file itself is wrong

A rule file that will not load is a different class of failure from a document that fails its checks, and `mdcheck` keeps the two apart by exit code so a CI job can tell "fix the rule file" from "fix the document" without reading any output.

| exit | meaning |
|---|---|
| 0 | every rule held |
| 1 | the document produced findings |
| 2 | the rule file could not be loaded, or a selector in it could not be parsed |

On exit 2 the run stops before any rule is applied. The error goes to stderr, names what is wrong and the index of the rule it is in, and **nothing is written to stdout** — not even the findings the document would otherwise have produced. A document with real problems and a rule file with a typo gives you the typo, alone.

```yaml
- select: H1
  count: 1
- select: H2
  max_lenght: 60
```

```
error: rule 1: unknown assertion key `max_lenght`
```

Common mistakes, all of them exit 2:

| mistake | example | what you get |
|---|---|---|
| unknown assertion key | `max_lenght: 60` | the key and the rule index |
| a selector that will not parse | `SECTION("Roles" > UL > LI` | the selector text and the rule index |
| a bound written as a word | `min: "two"` | the key `min` and the rule index |
| a value of the wrong shape | `columns: priority` where a list is expected | the key and the rule index |
| the file is not valid YAML | a misindented `count:` | the rule file named as unreadable |
| no `rules:` list | a file with only `doc:` | the missing `rules:` list named |

An unknown key inside an `each:` list is a rule-file error on the same terms.

One thing that is *not* a rule-file error: a well-formed selector that matches nothing. Pointing a rule at a section a particular document does not have is legitimate, and the outcome is decided by the rule's own `optional:` and cardinality keys — never by exit 2.

## Keeping rules maintainable

The rule file is read most often when something fails, so optimise for that moment.

**Keep each rule narrow.** One rule, one claim. A rule that checks a table's columns and its row count and its cell contents can only tell you that "the Features table is wrong". Three rules tell you which of the three things changed.

**Keep rules independent.** Rules do not depend on one another, and you should not write them as if they did. A rule that only makes sense given the outcome of an earlier rule produces a second finding for a problem the first already reported.

**Use `each:` for anything repeated.** A flat rule over repeated blocks aggregates: one document change in one block can shift a document-wide count and move a finding onto a block that did not change. Nested, the finding lands where the edit was.

**Say what varies.** A rule without `optional: true` over a section that is genuinely sometimes absent produces noise on every document that omits it — and noise is what makes people stop reading the output.

Taken together, these keep the mapping honest: one document change should produce one finding, not twenty.

## Not yet settled

- Whether a rule whose selector matches nothing, with neither `optional: true` nor a cardinality key, passes silently or reports a finding. Write `optional: true` when you mean the match may legitimately be absent.
- Whether `optional: true` only excuses a selector that matched nothing, or also softens the assertion for a match that does exist but breaks it.
- Whether an `each:` rule with zero outer matches is satisfied on its own, or needs `optional: true` stated. Add `min:` to the outer rule when blocks must exist.
- How a nested rule is identified in a rule-file error message — by the index of the outer rule alone, or by a path down to the inner rule.

**Next:** [Selectors](selectors.md) for what `select:` can address, [Assertions](assertions.md) for the full key vocabulary, [Findings and output](output.md) for how failures are reported, and the [Cookbook](cookbook.md) for ready-made rules. Back to [docs](README.md).
