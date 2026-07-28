---
confirmed: No
---

# E2E: Tables

How a rule pins down a table's column header set and order, bounds how many body rows it carries, and asserts on the cells of a column addressed by name.

## Declaring a table's columns

```gherkin
Feature: A rule fixes the column header set and its order

  Rule: A table whose header row carries exactly the named columns, in the named order, satisfies `columns:`

    @TBL-1
    Scenario: A table whose header matches the declared columns in order passes
      Given a document:
        """markdown
        ## Features
        | priority | epic | feature |
        |---|---|---|
        | p0 | **Core** | Parse |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE
          columns: [priority, epic, feature]
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

  Rule: A column that is present but sits in the wrong position is reported as an order mismatch, at the table header line

    @TBL-2
    Scenario: Two declared columns swapped in the header are reported as a single order mismatch
      Given a document:
        """markdown
        ## Features
        | epic | priority | feature |
        |---|---|---|
        | **Core** | p0 | Parse |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE
          columns: [priority, epic, feature]
        """
      When mdcheck checks the document
      Then a finding is reported at the table header line: "column order `epic, priority, feature`, expected `priority, epic, feature`"
      And no other finding is reported
      And the exit code is 1

  Rule: A declared column that the header does not carry at all is reported as missing — a different finding from an order mismatch

    @TBL-N1
    Scenario: A declared column absent from the header is reported as missing, not as misordered
      Given a document:
        """markdown
        ## Features
        | priority | feature |
        |---|---|
        | p0 | Parse |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE
          columns: [priority, epic, feature]
        """
      When mdcheck checks the document
      Then a finding is reported at the table header line: "required column `epic` missing"
      And no other finding is reported
      And the exit code is 1

  Rule: A column the rule does not name is reported, because `columns:` fixes the header set and not only its order

    @TBL-N2 @assumption
    # Open question: specs.md calls `columns:` a "table header set + order" but does not say
    # whether an unnamed extra column is an error on its own, nor whether a `strict:` flag
    # governs it the way it governs `ordered:` for sections. Exhaustive set is assumed here.
    Scenario: A header column the rule does not name is reported
      Given a document:
        """markdown
        ## Features
        | priority | epic | feature | notes |
        |---|---|---|---|
        | p0 | **Core** | Parse | none |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE
          columns: [priority, epic, feature]
        """
      When mdcheck checks the document
      Then a finding is reported at the table header line: "column `notes` not allowed"
      And no other finding is reported
      And the exit code is 1
```

## Bounding the number of body rows

```gherkin
Feature: A rule bounds how many rows of data a table carries

  Rule: A body row count outside the `rows:` bounds is reported at the table header line

    @TBL-5
    Scenario Outline: A table with a body row count outside the bounds is reported
      Given a document that holds a table with <body rows> body rows under "## Features"
      And the rules:
        """yaml
        - select: TABLE
          rows: { min: 2, max: 3 }
        """
      When mdcheck checks the document
      Then a finding is reported at the table header line: "<message>"
      And the exit code is 1

      Examples:
        | body rows | message                              |
        | 1         | table has 1 body row, expected ≥ 2   |
        | 4         | table has 4 body rows, expected ≤ 3  |

  Rule: `rows:` counts body rows only — the header row and the delimiter row beneath it are not rows of data

    @TBL-N3
    Scenario: A table with exactly the bounded number of body rows passes despite its header and delimiter lines
      Given a document:
        """markdown
        ## Features
        | priority | feature |
        |---|---|
        | p0 | Parse |
        | p1 | Render |
        """
      And the rules:
        """yaml
        - select: TABLE
          rows: { min: 2, max: 2 }
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Asserting on the cells of a named column

```gherkin
Feature: A rule addresses a column by name and asserts on every cell in it

  Rule: A column whose cells all satisfy the assertion produces no findings

    @TBL-N4
    Scenario: Every cell of the addressed column satisfying its assertion passes
      Given a document:
        """markdown
        ## Features
        | priority | epic | feature |
        |---|---|---|
        | p0 | **Core** | Parse |
        | p2 | **Edge** | Render |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE > TD[col="priority"]
          enum: [p0, p1, p2, p3]
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

  Rule: A cell that violates its column's assertion is reported at that cell's own row, and the message names the column

    @TBL-3 @TBL-4 @TBL-6
    Scenario Outline: A cell violating the assertion on its column is reported at its row
      Given a document:
        """markdown
        ## Features
        | priority | epic | feature | capabilities |
        |---|---|---|---|
        | p0 | **Core** | Parse | reads markdown |
        | <priority> | **Edge** | <feature> | <capabilities> |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE > TD[col="<column>"]
          <assertion>
        """
      When mdcheck checks the document
      Then a finding is reported at the "<priority>" row: "<message>"
      And the exit code is 1

      Examples:
        | column       | assertion               | priority | feature | capabilities              | message                                          |
        | priority     | enum: [p0, p1, p2, p3]  | p9       | Parse   | reads markdown            | `p9` in column `priority` not in allowed set     |
        | feature      | no_empty: true          | p1       |         | reads markdown            | empty cell in column `feature`                   |
        | capabilities | max_length: 12          | p2       | Parse   | this text is far too long | column `capabilities` length 25 > 12             |
        | capabilities | forbid: 'TBD'           | p3       | Parse   | TBD                       | column `capabilities` matches forbidden `TBD`    |
```

## Where a cell finding points

```gherkin
Feature: Every offending cell answers for itself

  Rule: Each offending cell in a column produces its own finding in one run, anchored at its own row; compliant rows produce none

    @TBL-6
    Scenario: Two offending cells in one column produce two findings in a single run
      Given a document:
        """markdown
        ## Features
        | priority | capabilities |
        |---|---|
        | p0 | reads markdown |
        | p1 | TBD |
        | p2 | renders output |
        | p3 | TBD |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE > TD[col="capabilities"]
          forbid: 'TBD'
        """
      When mdcheck checks the document
      Then a finding is reported at the "p1" row: "column `capabilities` matches forbidden `TBD`"
      And a finding is reported at the "p3" row: "column `capabilities` matches forbidden `TBD`"
      And no other finding is reported
      And the exit code is 1

  Rule: A cell assertion reaches only the column it names, however similar the neighbouring cells are

    @TBL-N5
    Scenario: An identical value in an unnamed column is left alone
      Given a document:
        """markdown
        ## Features
        | priority | epic | capabilities |
        |---|---|---|
        | p0 | TBD | reads markdown |
        | p1 | **Edge** | TBD |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE > TD[col="capabilities"]
          forbid: 'TBD'
        """
      When mdcheck checks the document
      Then a finding is reported at the "p1" row: "column `capabilities` matches forbidden `TBD`"
      And no other finding is reported
      And the exit code is 1
```

## Naming a column the table does not have

```gherkin
Feature: A cell rule whose column is absent from the table

  Rule: A column name that no header carries selects no cells, so a per-cell assertion has nothing to report

    @TBL-N6 @assumption
    # Open question: specs.md defines `optional: true` as "rule passes when selector matches
    # nothing", which implies an empty match set is otherwise unremarkable for per-cell
    # assertions; it does not say whether a `TD[col="…"]` naming a column no table carries is
    # instead reported as an authoring mistake in the rule file. Silence is assumed here.
    Scenario: A cell assertion on a column the table does not carry reports nothing
      Given a document:
        """markdown
        ## Features
        | priority | feature |
        |---|---|
        | p0 | Parse |
        """
      And the rules:
        """yaml
        - select: SECTION("Features") > TABLE > TD[col="owner"]
          no_empty: true
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```
