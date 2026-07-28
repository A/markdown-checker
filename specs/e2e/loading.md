---
confirmed: No
---

# E2E: Document loading

How a markdown document is brought into a checkable state, what the load stage reports when it will not parse cleanly, and which failure is severe enough to stop the run.

## Loading a document that parses cleanly

```gherkin
Feature: A well-formed document loads without comment

  Rule: A document that parses cleanly produces no load-stage findings

    @LOAD-N1
    Scenario: A document with a closed frontmatter block and a body loads cleanly
      Given a document:
        """markdown
        ---
        confirmed: no
        ---
        # Doc

        Body text.
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @LOAD-N2
    Scenario: A document with no frontmatter block loads cleanly when no rule asks for one
      Given a document:
        """markdown
        # Doc

        Body text.
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Malformed frontmatter block

```gherkin
Feature: The load stage reports a frontmatter block that never closes

  Rule: A frontmatter block that is opened but never closed is reported at the load stage, attributed to check "load"

    @LOAD-1
    Scenario: An unclosed frontmatter block is reported at the load stage
      Given a document:
        """markdown
        ---
        confirmed: no
        # Doc
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "unclosed frontmatter block"
      And the finding carries check "load"
      And the exit code is 1
```

## Broken link syntax

```gherkin
Feature: The load stage reports link syntax it cannot parse

  Rule: Link markup that never terminates is reported at the load stage, attributed to check "load"

    @LOAD-N3 @assumption
    # Open question: specs.md names "broken link syntax" as a load-stage finding but does
    # not enumerate which malformed forms qualify, nor the wording of the message.
    Scenario: An unterminated wiki-link is reported at the load stage
      Given a document:
        """markdown
        # Doc

        See [[features/x|X and more text.
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then a finding is reported at line 3: "unterminated link syntax"
      And the finding carries check "load"
      And the exit code is 1
```

## A load-stage finding does not stop the declared rules

```gherkin
Feature: Declared rules still run after a load-stage finding

  Rule: A load-stage finding is collected alongside the declared rules' findings; every declared rule still runs and still reports

    @LOAD-N4
    Scenario: A failing declared rule still reports after a load-stage finding
      Given a document:
        """markdown
        ---
        confirmed: no
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "unclosed frontmatter block"
      And the finding carries check "load"
      And a finding is reported at line 4: "length 52 > 20"
      And the exit code is 1

    @LOAD-N5
    Scenario: A load-stage finding fails the run even when every declared rule passes
      Given a document:
        """markdown
        ---
        confirmed: no
        # Doc
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "unclosed frontmatter block"
      And the finding carries check "load"
      And no other finding is reported
      And the exit code is 1
```

## An unreadable document stops the run

```gherkin
Feature: A document that cannot be read aborts instead of reporting findings

  Rule: When the document cannot be read at all, no declared rule runs and nothing is reported as a finding

    @LOAD-N6 @assumption
    # Open question: specs.md states an unreadable document aborts the remaining checks but
    # does not fix the exit code; 2 is assumed as the usage-error code. With several
    # documents named in one invocation it is also unstated whether one unreadable document
    # aborts the whole run or only its own document.
    Scenario: A document path that does not exist aborts the run
      Given a document path that does not exist
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then no findings are reported
      And a diagnostic naming the unreadable path is written to stderr
      And the exit code is 2

    @LOAD-N7 @assumption
    # Open question: same as above — the exit code for an unreadable document is not fixed
    # by specs.md.
    Scenario: A document path that exists but cannot be read aborts the run
      Given a document path that exists but cannot be read
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then no findings are reported
      And a diagnostic naming the unreadable path is written to stderr
      And the exit code is 2
```
