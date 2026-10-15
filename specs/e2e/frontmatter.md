---
confirmed: No
---

# E2E: Frontmatter

Contracts over the frontmatter key/value block — `has:` (exact string, `enum:`, regexp, or list-length value spec), `not_has:`, and the line each resulting finding points at.

## Required keys (`has:`)

```gherkin
Feature: Required frontmatter keys

  A Rule Author declares which frontmatter keys a document must carry and what
  values are acceptable, so that a Spec Author is told exactly which key is wrong
  and a CI Pipeline can gate on the exit code.

  Rule: A required key present with a value that satisfies its value spec raises nothing

    @FM-1
    Scenario Outline: a key satisfying its value spec passes in every value-spec form
      Given a document:
        """markdown
        ---
        <key>: <value>
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { <key>: <spec> }
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

      Examples: value spec forms
        | form   | key       | value    | spec                |
        | exact  | status    | approved | approved            |
        | enum   | confirmed | no       | { enum: [yes, no] } |
        | regexp | priority  | P1       | '^P[0-3]$'          |

  Rule: A required key whose value violates its value spec is reported at that key's own line

    # Open question: specs.md fixes no wording for the exact-string mismatch message;
    # only the enum and regexp wordings are stated. The exact row is assumed.
    @FM-2 @FM-5 @assumption
    Scenario Outline: a key violating its value spec is reported at its own line
      Given a document:
        """markdown
        ---
        <key>: <value>
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { <key>: <spec> }
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "<message>"
      And the exit code is 1

      Examples: value spec forms
        | form   | key       | value    | spec                | message                                          |
        | exact  | status    | rejected | approved            | `status` is `rejected`, expected `approved`      |
        | enum   | confirmed | maybe    | { enum: [yes, no] } | `confirmed` is `maybe`, allowed: `yes`, `no`     |
        | regexp | priority  | P7       | '^P[0-3]$'          | `priority` value `P7` does not match `^P[0-3]$`  |

    @FM-N1
    Scenario: the finding points at the offending key, not at the start of the block
      Given a document:
        """markdown
        ---
        title: Doc
        owner: docs-team
        confirmed: maybe
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { confirmed: { enum: [yes, no] } }
        """
      When mdcheck checks the document
      Then a finding is reported at line 4: "`confirmed` is `maybe`, allowed: `yes`, `no`"
      And the exit code is 1

  Rule: A required key absent from a present block is a missing key, not a wrong value

    # Open question: specs.md states no line for a key that has no line of its own;
    # the block-level line 1 is assumed, matching the missing-block finding.
    @FM-N2 @assumption
    Scenario: a required key absent from a present frontmatter block
      Given a document:
        """markdown
        ---
        confirmed: no
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { priority: '^P[0-3]$' }
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "required key `priority` missing"
      And the exit code is 1
```

## List values

```gherkin
Feature: Frontmatter list values

  A Rule Author requires a frontmatter key to carry a list — non-empty, bounded,
  or of an exact length — and limits which items it may hold, so that a Spec
  Author cannot ship an empty `tags:` or an unknown tag.

  Rule: min, max and count bound the number of items; a scalar where a list is required is reported

    @FM-6
    Scenario: list lengths out of bounds and a scalar in place of a list
      Given a document:
        """markdown
        ---
        tags: []
        owners:
          - ann
          - bob
        title: Doc
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { tags: { min: 1 }, owners: { max: 1 }, title: { count: 1 } }
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "`tags` has 0 items, expected ≥ 1"
      And a finding is reported at line 3: "`owners` has 2 items, expected ≤ 1"
      And a finding is reported at line 6: "`title` value `Doc` is not a list"
      And the exit code is 1

  Rule: enum on a list value applies to every item, each reported at its own line

    @FM-7
    Scenario: one list item outside the allowed set
      Given a document:
        """markdown
        ---
        tags:
          - index
          - draft
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { tags: { min: 1, enum: [index, spec] } }
        """
      When mdcheck checks the document
      Then a finding is reported at line 4: "`tags` item `draft` not allowed, allowed: `index`, `spec`"
      And the exit code is 1
```

## Missing frontmatter block

```gherkin
Feature: Frontmatter block absent from the document

  When a Spec Author ships a document with no frontmatter at all, the Rule Author's
  `has:` rule reports the missing block itself rather than each key it asked for.

  Rule: A document with no frontmatter block yields one block-level finding at line 1

    @FM-3
    Scenario: no frontmatter block at all
      Given a document:
        """markdown
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { confirmed: { enum: [yes, no] } }
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "frontmatter block missing"
      And the exit code is 1

    # Open question: specs.md does not say whether an absent block also emits a
    # per-key finding; a single block-level finding is assumed.
    @FM-N3 @assumption
    Scenario: several required keys still yield the single missing-block finding
      Given a document:
        """markdown
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { confirmed: { enum: [yes, no] }, priority: '^P[0-3]$' }
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "frontmatter block missing"
      And the exit code is 1
```

## Forbidden keys (`not_has:`)

```gherkin
Feature: Forbidden frontmatter keys

  A Rule Author lists keys that must never appear, so a Spec Author cannot leave
  scaffolding keys behind in a document that is being published.

  Rule: A listed key that is present is reported at that key's own line

    @FM-4
    Scenario: a forbidden key is present
      Given a document:
        """markdown
        ---
        confirmed: no
        draft: true
        ---
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          not_has: [draft]
        """
      When mdcheck checks the document
      Then a finding is reported at line 3: "key `draft` must not be present"
      And the exit code is 1

  Rule: A listed key that is absent raises nothing

    @FM-N4
    Scenario: no forbidden key in the block
      Given a document:
        """markdown
        ---
        confirmed: no
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          not_has: [draft]
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Keys the rules never name

```gherkin
Feature: Frontmatter keys outside the rule file

  A Spec Author may carry extra frontmatter for other tooling; mdcheck only ever
  speaks about the keys the Rule Author named.

  Rule: Keys named by neither has: nor not_has: are ignored

    @FM-N5
    Scenario: extra keys alongside a satisfied required key
      Given a document:
        """markdown
        ---
        confirmed: no
        owner: docs-team
        tags: [index, draft-tooling]
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { confirmed: { enum: [yes, no] } }
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Every violation in one run

```gherkin
Feature: All frontmatter violations collected in a single run

  A Machine Consumer reads the whole finding list from one invocation, so a block
  with several bad keys must never stop at the first one.

  Rule: Each violating key produces its own finding, all emitted by one run

    @FM-N6
    Scenario: a block violating two value specs and one forbidden key
      Given a document:
        """markdown
        ---
        confirmed: maybe
        priority: P7
        draft: true
        ---
        # Doc
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { confirmed: { enum: [yes, no] }, priority: '^P[0-3]$' }
          not_has: [draft]
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "`confirmed` is `maybe`, allowed: `yes`, `no`"
      And a finding is reported at line 3: "`priority` value `P7` does not match `^P[0-3]$`"
      And a finding is reported at line 4: "key `draft` must not be present"
      And the exit code is 1
```
