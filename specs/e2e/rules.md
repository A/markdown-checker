---
confirmed: No
---

# E2E: Rule file loading

How a Rule Author's YAML rule file is accepted, and how a mistake inside it surfaces — as an error on stderr with exit code 2 and an empty stdout, never as a finding against the document.

Two docstring steps are used in this area: `And the rules:` carries the flat rule list as written in scenarios.md, while `And the rule file:` carries the whole file (`doc:` header + `rules:` list) for the cases where the file's own shape is what is under test.

## Accepting a well-formed rule file

```gherkin
Feature: A well-formed rule file is accepted and its rules run

  The Rule Author writes a `doc:` header and a flat `rules:` list; every rule in
  that list is applied to the document the Spec Author wrote.

  Rule: A rule file with a `doc:` header and a flat `rules:` list is accepted, and every rule in the list is applied

    @RULE-N1
    Scenario: A well-formed rule file whose every rule holds passes the run
      Given a document:
        """markdown
        # Feature index
        ## Roles
        - **Admin** — manages
        """
      And the rule file:
        """yaml
        doc: feature-index

        rules:
          - select: H1
            count: 1
          - select: SECTION("Roles") > UL > LI
            min: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @RULE-N2
    Scenario: Every rule in the list runs, not only the first
      Given a document:
        """markdown
        # Feature index

        ## Roles
        """
      And the rule file:
        """yaml
        doc: feature-index

        rules:
          - select: H1
            max_length: 5
          - select: SECTION("Roles") > UL > LI
            min: 1
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "length 13 > 5"
      And a finding is reported at line 3: "0 matches, expected ≥ 1"
      And the exit code is 1
```

## Unknown assertion key

```gherkin
Feature: An unknown assertion key is a rule-file error

  A key the rule language does not define is the Rule Author's mistake, not the
  Spec Author's. The CI Pipeline must be able to tell the two apart by exit code
  alone: 2 means "fix the rule file", 1 means "fix the document".

  Rule: An assertion key the rule language does not define aborts the run before any finding is emitted, naming the key and the rule it appears in

    # Rule indices count rules from the top of the flat `rules:` list, first rule = index 0.
    @CLI-4
    Scenario: An unknown assertion key names the key and the rule index
      Given a document that satisfies every well-formed rule in the file
      And the rules:
        """yaml
        - select: H1
          count: 1
        - select: H2
          max_lenght: 60
        """
      When mdcheck checks the document
      Then the run reports an error naming the key "max_lenght" and rule index 1
      And nothing is written to stdout
      And the exit code is 2

    @RULE-N3
    Scenario: A rule-file error replaces the findings the document would have produced
      Given a document:
        """markdown
        ## Roles
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        - select: H2
          max_lenght: 60
        """
      When mdcheck checks the document
      Then the run reports an error naming the key "max_lenght" and rule index 1
      And nothing is written to stdout
      And the exit code is 2

    @RULE-N4 @assumption
    Scenario: An unknown key inside a nested rule is a rule-file error too
      # open question: how a nested rule is identified in the message — by the
      # index of the outer rule only, or by a path down to the inner rule
      Given a document that contains a section with subsections
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: P[0]
              starts_wtih: 'pre:'
        """
      When mdcheck checks the document
      Then the run reports an error naming the key "starts_wtih" and rule index 0
      And nothing is written to stdout
      And the exit code is 2
```

## Malformed selector

```gherkin
Feature: A malformed selector is a rule-file error

  A `select:` value that cannot be read as the selector grammar is a defect in
  the rule file. It never becomes a finding against the document, however badly
  the document happens to be written.

  Rule: A selector that cannot be parsed aborts the run with a rule-file error, never a document finding

    @RULE-N5
    Scenario: An unparseable selector is reported instead of running the rule
      Given a document:
        """markdown
        # T
        ## Roles
        """
      And the rules:
        """yaml
        - select: SECTION("Roles" > UL > LI
          min: 1
        """
      When mdcheck checks the document
      Then the run reports an error naming the selector "SECTION("Roles" > UL > LI" and rule index 0
      And nothing is written to stdout
      And the exit code is 2

    @RULE-N6
    Scenario Outline: Selectors that break the grammar are rejected whatever the breakage
      Given a document that contains a heading and a list
      And the rules:
        """yaml
        - select: <selector>
          min: 1
        """
      When mdcheck checks the document
      Then the run reports an error naming the selector "<selector>" and rule index 0
      And the exit code is 2

      Examples:
        | selector    | breakage                         |
        | H1 >        | combinator with nothing after it |
        | P[text=     | unterminated attribute filter    |
        | H2(/^Story  | unterminated regexp sugar        |

    @RULE-N7 @assumption
    Scenario: An element name outside the selector grammar
      # open question: specs.md lists the element names but does not say whether an
      # unlisted name is rejected when the rule file loads or simply matches nothing
      Given a document that contains a block quote
      And the rules:
        """yaml
        - select: BLOCKQUOTE
          min: 1
        """
      When mdcheck checks the document
      Then the run reports an error naming the selector "BLOCKQUOTE" and rule index 0
      And nothing is written to stdout
      And the exit code is 2
```

## Wrong-typed assertion value

```gherkin
Feature: A known assertion key with a wrong-typed value is a rule-file error

  The key exists, so the Rule Author gets no "unknown key" hint; the value is
  the wrong shape for it, and the run must say so rather than guess.

  Rule: A defined assertion key whose value is not the shape the key takes is reported like an unknown key — named, indexed, exit 2

    @RULE-N8
    Scenario: A numeric bound written as a word
      Given a document:
        """markdown
        ## Roles
        - **Admin** — manages
        - **Editor** — edits
        """
      And the rules:
        """yaml
        - select: SECTION("Roles") > UL > LI
          min: "two"
        """
      When mdcheck checks the document
      Then the run reports an error naming the key "min" and rule index 0
      And nothing is written to stdout
      And the exit code is 2

    @RULE-N9 @assumption
    Scenario Outline: Assertion values of the wrong shape
      # open question: whether every assertion key validates its value shape when the
      # rule file loads, or only the keys that take a number
      Given a document that contains a section with a table
      And the rules:
        """yaml
        - select: <selector>
          <key>: <value>
        """
      When mdcheck checks the document
      Then the run reports an error naming the key "<key>" and rule index 0
      And the exit code is 2

      Examples:
        | selector                    | key     | value    | expected shape     |
        | SECTION("Roles") > UL > LI  | max     | "ten"    | a number           |
        | SECTION("Features") > TABLE | columns | priority | a list of names    |
        | SECTION("Features") > TABLE | rows    | 5        | a min/max mapping  |
        | H1 > H2                     | ordered | Roles    | a list of members  |
```

## Rule file that is not a rule file

```gherkin
Feature: A rule file that cannot be read as a rule file

  Before any document is checked, the file itself must parse as YAML and carry a
  flat `rules:` list. Anything else stops the run at exit code 2 so the CI
  Pipeline never mistakes it for a clean or a failing document.

  Rule: A rule file that is not valid YAML stops the run before any document is checked

    @RULE-N10
    Scenario: The rule file is not valid YAML at all
      Given a document that contains a single heading
      And the rule file:
        """yaml
        doc: feature-index

        rules:
          - select: H1
           count: 1
        """
      When mdcheck checks the document
      Then the run reports an error naming the rule file as unreadable YAML
      And nothing is written to stdout
      And the exit code is 2

  Rule: A rule file that parses as YAML but carries no flat `rules:` list is a rule-file error

    @RULE-N11 @assumption
    Scenario: The rule file has no rules list
      # open question: specs.md shows `doc:` + `rules:` together but does not state
      # whether a file with a `doc:` header and no `rules:` key is an error or a no-op
      Given a document that contains a single heading
      And the rule file:
        """yaml
        doc: feature-index
        """
      When mdcheck checks the document
      Then the run reports an error naming the missing `rules:` list
      And nothing is written to stdout
      And the exit code is 2

    @RULE-N12 @assumption
    Scenario: The rules key holds a single rule instead of a list
      # open question: specs.md calls `rules:` a flat list; whether a lone mapping is
      # accepted as a one-rule shorthand is unstated
      Given a document that contains a single heading
      And the rule file:
        """yaml
        doc: feature-index

        rules:
          select: H1
          count: 1
        """
      When mdcheck checks the document
      Then the run reports an error naming the `rules:` list as not a list of rules
      And nothing is written to stdout
      And the exit code is 2
```

## A selector that matches nothing is not a rule-file error

```gherkin
Feature: Selecting nothing is not a defect in the rule file

  A rule may legitimately point at elements a given document does not contain.
  Whether that is silence or a finding is decided by the rule's own cardinality
  and `optional:` keys — it is never an exit code 2.

  Rule: A well-formed selector that matches no element is not a rule-file error

    @RULE-N13
    Scenario: A rule whose selector matches nothing and asserts no cardinality passes
      Given a document:
        """markdown
        # T
        ## Roles
        - **Admin** — manages
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE > TD[col="priority"]
          enum: [p0, p1, p2, p3]
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @RULE-N14
    Scenario: The same empty match is a document finding when the rule requires matches
      Given a document:
        """markdown
        # T
        ## Roles
        - **Admin** — manages
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE > TD[col="priority"]
          min: 1
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "0 matches, expected ≥ 1"
      And the exit code is 1
```
