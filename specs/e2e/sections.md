---
confirmed: No
---

# E2E: Sections

Behaviour of the document skeleton assertions — which sections must exist (`ordered:`, `?` optional suffix), in what order, whether anything else may appear (`strict:`), and presence of a section identified by a title regexp.

## Required section set in the declared order

```gherkin
Feature: Required sections present in the declared order

  Rule: A document whose sections appear in the order declared by `ordered:` passes with no findings

    @SEC-1
    Scenario: Every required section present, in the declared order
      Given a document:
        """markdown
        # T
        ## Roles
        x
        ## Features
        y
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features]
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Out-of-order sections

```gherkin
Feature: Sections declared in one order but written in another

  Rule: A member that appears later than declared is reported at its own header

    @SEC-2
    Scenario: Two required sections swapped
      Given a document:
        """markdown
        # T
        ## Features
        y
        ## Roles
        x
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features]
        """
      When mdcheck checks the document
      Then a finding is reported at the "Roles" header: "section order: `Roles` appears after `Features`, expected before"
      And the exit code is 1
```

## Missing required sections

```gherkin
Feature: A declared section that the document never provides

  Rule: A missing member has no header to point at, so it is reported at the line of the scope the rule selects in

    @SEC-3
    Scenario: Required section absent from the document
      Given a document:
        """markdown
        # T
        ## Roles
        x
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features]
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "required section `Features` missing"
      And the exit code is 1
```

## Optional members

```gherkin
Feature: Members marked optional with the `?` suffix

  Rule: An optional member may be absent, may be present, but when present must still sit where it was declared

    @SEC-4
    Scenario: Optional member present in its declared position
      Given a document:
        """markdown
        # T
        ## Roles
        x
        ## Features
        y
        ## Questions
        z
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features, Questions?]
          strict: true
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @SEC-4
    Scenario: Optional member absent
      Given a document:
        """markdown
        # T
        ## Roles
        x
        ## Features
        y
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features, Questions?]
          strict: true
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @SEC-N1
    Scenario: Optional member present but out of the declared order
      Given a document:
        """markdown
        # T
        ## Roles
        x
        ## Questions
        z
        ## Features
        y
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features, Questions?]
          strict: true
        """
      When mdcheck checks the document
      Then a finding is reported at the "Features" header: "section order: `Features` appears after `Questions`, expected before"
      And the exit code is 1
```

## Sections outside the declared set

```gherkin
Feature: Sections the rule never declared

  Rule: `strict: true` forbids members outside the declared list, and reports each one at its own header

    @SEC-5
    Scenario: Stray section under strict
      Given a document:
        """markdown
        # T
        ## Roles
        x
        ## Features
        y
        ## Extra
        z
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features, Questions?]
          strict: true
        """
      When mdcheck checks the document
      Then a finding is reported at the "Extra" header: "section `Extra` not allowed (strict)"
      And the exit code is 1

  Rule: Without `strict:` the declared list constrains only the members it names, and extra sections are tolerated

    @SEC-N2
    Scenario: Extra section allowed when strict is not declared
      Given a document:
        """markdown
        # T
        ## Roles
        x
        ## Features
        y
        ## Extra
        z
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features]
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Sections identified by a title regexp

```gherkin
Feature: Requiring a section whose title is only known by pattern

  Rule: A section is required by selecting it with a title regexp and asserting it matched at least once

    @SEC-6
    Scenario: Section matching the title pattern is present
      Given a document:
        """markdown
        # T
        ## Story: login
        """
      And the rules:
        """yaml
        - select: H2(/^Story: /)
          min: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @SEC-N3 @assumption
    # Open question: scenarios.md fixes the anchor for a missing named section (SEC-3, line 1)
    # but never states the anchor when a title-regexp selector matches nothing; assumed to be
    # the line of the selected scope, which for a document-level selector is line 1.
    Scenario: No section matches the title pattern
      Given a document:
        """markdown
        # T
        ## Login
        """
      And the rules:
        """yaml
        - select: H2(/^Story: /)
          min: 1
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "0 matches, expected ≥ 1"
      And the exit code is 1
```

## Skeleton assertions at a nested level

```gherkin
Feature: `ordered:` declared for a level below the top

  Rule: A skeleton rule constrains only the level its selector picks, leaving every other level unconstrained

    @SEC-N4
    Scenario: Nested order satisfied while the level above is arranged freely
      Given a document:
        """markdown
        # T
        ## Appendix
        note
        ## User Stories
        ### Login
        a
        ### Logout
        b
        """
      And the rules:
        """yaml
        - select: H2 > H3
          ordered: [Login, Logout]
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @SEC-N5
    Scenario: Strict at the nested level does not forbid sections at the level above
      Given a document:
        """markdown
        # T
        ## User Stories
        ### Login
        a
        ### Logout
        b
        ## Appendix
        note
        """
      And the rules:
        """yaml
        - select: H2 > H3
          ordered: [Login, Logout]
          strict: true
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @SEC-N6
    Scenario: Nested member out of order is reported at its own header
      Given a document:
        """markdown
        # T
        ## User Stories
        ### Logout
        b
        ### Login
        a
        """
      And the rules:
        """yaml
        - select: H2 > H3
          ordered: [Login, Logout]
        """
      When mdcheck checks the document
      Then a finding is reported at the "Login" header: "section order: `Login` appears after `Logout`, expected before"
      And the exit code is 1
```

## Several skeleton violations in one run

```gherkin
Feature: A document that breaks the skeleton in more than one way

  Rule: Every skeleton violation produces its own finding in a single run, each at its own anchor

    @SEC-N7
    Scenario: Missing, misplaced and stray members reported together
      Given a document:
        """markdown
        # T
        ## Features
        y
        ## Roles
        x
        ## Extra
        z
        """
      And the rules:
        """yaml
        - select: H1 > H2
          ordered: [Roles, Features, Summary]
          strict: true
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "required section `Summary` missing"
      And a finding is reported at the "Roles" header: "section order: `Roles` appears after `Features`, expected before"
      And a finding is reported at the "Extra" header: "section `Extra` not allowed (strict)"
      And the exit code is 1
```
