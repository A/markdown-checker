---
confirmed: No
---

# E2E: CLI, output and exit codes

How documents reach mdcheck (path, several paths, stdin), how the findings of one run come back out (pretty for a reader, json for a machine, findings on stdout and diagnostics on stderr), and what a caller may branch on.

## Documents named by path

```gherkin
Feature: A document named by path is checked and tagged with its path

  A CI Pipeline names the documents to check on the command line; every finding
  says which document it came from, so a Spec Author can go straight to the file.

  Rule: Findings from a document named by path carry that path

    @CLI-N1
    Scenario: a finding names the document it came from
      Given a document "specs/index.md":
        """markdown
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "length 52 > 20"
      And the finding carries file "specs/index.md"
      And the exit code is 1

  Rule: A link-resolution base may be given for the run

    # The link semantics of the resolution base belong to e2e/links.md; here only
    # that the flag is part of the command-line surface and does not disturb a run
    # that asserts nothing about links.
    @CLI-N2
    Scenario: a run that names a link-resolution base behaves as an ordinary run
      Given a document that contains no links
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document with a link-resolution root
      Then the run passes with no findings
      And the exit code is 0
```

## Documents read from stdin

```gherkin
Feature: A document piped in is checked like any other

  A CI Pipeline that has the document in a pipe rather than on disk passes "-" as
  the path; the findings then name the document as "<stdin>".

  Rule: The "-" path argument reads the document from stdin and its findings carry file "<stdin>"

    @CLI-1
    Scenario: a piped document with a violation is reported against "<stdin>"
      Given a document:
        """markdown
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck reads the document from stdin
      Then a finding is reported at line 2: "length 52 > 20"
      And the finding carries file "<stdin>"
      And the exit code is 1

    @CLI-1
    Scenario: a piped document with no violation passes
      Given a document:
        """markdown
        # Doc
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck reads the document from stdin
      Then the run passes with no findings
      And the exit code is 0
```

## Several documents in one run

```gherkin
Feature: One invocation over several documents

  A CI Pipeline checks a whole set of documents in a single call and branches once,
  on one exit code, while each finding stays attributable to its own document.

  Rule: Findings from several documents are tagged with the document they came from

    @CLI-2
    Scenario: two failing documents each get their own findings
      Given a document "a.md":
        """markdown
        # Doc A
        ## This header is way too long for the configured limit
        """
      And a document "b.md":
        """markdown
        # Doc B
        ## Another header that also exceeds the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks both documents
      Then a finding is reported for "a.md" at line 2: "length 52 > 20"
      And a finding is reported for "b.md" at line 2: "length 53 > 20"
      And the exit code is 1

  Rule: A run over several documents passes only when every document is clean

    @CLI-2
    Scenario: one clean and one failing document fail the run
      Given a document "a.md":
        """markdown
        # Doc A
        """
      And a document "b.md":
        """markdown
        # Doc B
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks both documents
      Then a finding is reported for "b.md" at line 2: "length 52 > 20"
      And no finding is reported for "a.md"
      And the exit code is 1

    @CLI-2
    Scenario: every document clean passes the run
      Given a document "a.md":
        """markdown
        # Doc A
        """
      And a document "b.md":
        """markdown
        # Doc B
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks both documents
      Then the run passes with no findings
      And the exit code is 0
```

## Every violation of a run reported at once

```gherkin
Feature: A run collects all findings, never stops at the first

  A Spec Author fixes a document in one pass, so a single run must list everything
  wrong with it — a failing rule never suppresses the rules or matches after it.

  Rule: Independent rules that fail in one run each report

    @CLI-N3
    Scenario: three rules failing over one document produce three findings
      Given a document:
        """markdown
        ---
        confirmed: maybe
        ---
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { confirmed: { enum: [yes, no] } }
        - select: DOCUMENT
          min_lines: 20
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "`confirmed` is `maybe`, allowed: `yes`, `no`"
      And a finding is reported at line 1: "document has 5 lines, expected ≥ 20"
      And a finding is reported at line 5: "length 52 > 20"
      And the exit code is 1

  Rule: Every offending match of one rule reports, not only the first

    @CLI-N4
    Scenario: two matches violating the same rule produce two findings
      Given a document:
        """markdown
        # Doc
        ## This header is way too long for the configured limit
        Body.
        ## Another header that also exceeds the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "length 52 > 20"
      And a finding is reported at line 4: "length 53 > 20"
      And the exit code is 1
```

## Default output for a human reader

```gherkin
Feature: The default output is written for a reader at the terminal

  A Spec Author runs mdcheck by hand and reads the findings directly; each one
  must locate itself in the document without further tooling.

  Rule: Each finding is presented with the document, the line and the message

    @CLI-N5
    Scenario: a finding is legible without parsing
      Given a document "specs/index.md":
        """markdown
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "length 52 > 20"
      And the finding carries file "specs/index.md"
      And the exit code is 1
```

## Machine-readable output

```gherkin
Feature: json output for a machine consumer

  A Machine Consumer — promptfoo, or a CI Pipeline step that inspects findings
  rather than just the exit code — parses stdout as one json document.

  Rule: json format writes one object with files, findings and summary, and nothing else on stdout

    @CLI-3
    Scenario: a failing run emits a single json object
      Given a document "specs/index.md":
        """markdown
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document in json format
      Then stdout holds a single json object with keys "files", "findings" and "summary"
      And a finding is reported at line 2: "length 52 > 20"
      And nothing else is written to stdout
      And the exit code is 1

    @CLI-3
    Scenario: a clean run emits the same object with an empty findings list
      Given a document:
        """markdown
        # Doc
        """
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document in json format
      Then stdout holds a single json object with keys "files", "findings" and "summary"
      And the findings list is empty
      And nothing else is written to stdout
      And the exit code is 0

  Rule: The summary counts the documents checked and the documents that produced findings

    @CLI-3
    Scenario: a two-document run summarises two checked and one failed
      Given a document "a.md":
        """markdown
        # Doc A
        """
      And a document "b.md":
        """markdown
        # Doc B
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks both documents in json format
      Then the summary reports 2 checked and 1 failed
      And a finding is reported for "b.md" at line 2: "length 52 > 20"
      And nothing else is written to stdout
      And the exit code is 1

  Rule: The format chooses the presentation, never the findings

    @CLI-N6
    Scenario: json reports the same findings the default output reports
      Given a document:
        """markdown
        ---
        confirmed: maybe
        ---
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: FRONTMATTER
          has: { confirmed: { enum: [yes, no] } }
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document in json format
      Then the reported findings are the same findings the default output reports for this run
      And the exit code is 1
```

## Stream separation

```gherkin
Feature: Findings on stdout, diagnostics on stderr

  A CI Pipeline pipes stdout into another tool; anything mdcheck wants to say
  about itself must stay out of that pipe.

  Rule: A run that produces findings writes only findings to stdout

    @CLI-N7
    Scenario: nothing but the findings reaches stdout
      Given a document:
        """markdown
        # Doc
        ## This header is way too long for the configured limit
        """
      And the rules:
        """yaml
        - select: H2
          max_length: 20
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "length 52 > 20"
      And nothing else is written to stdout
      And the exit code is 1

  Rule: A diagnostic never reaches stdout

    @CLI-N8
    Scenario: a run that cannot start leaves stdout empty
      Given a document path that does not exist
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then nothing is written to stdout
      And the diagnostic is written to stderr
      And the exit code is 2
```

## Usage errors

```gherkin
Feature: An invocation mdcheck cannot act on

  A CI Pipeline must be able to tell "the documents are wrong" from "the call was
  wrong"; the second is never reported as a finding.

  Rule: An invocation that cannot be acted on yields a diagnostic on stderr, no findings, and exit code 2

    # Open question: specs.md shows the document path always given explicitly, "-"
    # included, but does not say whether an invocation naming no path at all is a
    # usage error or falls back to stdin. A usage error is assumed.
    @CLI-N9 @assumption
    Scenario: no document to check is named
      Given the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck is invoked with no document path
      Then no findings are reported
      And the diagnostic is written to stderr
      And the exit code is 2

    @CLI-N10
    Scenario: a named document does not exist
      Given a document path that does not exist
      And the rules:
        """yaml
        - select: H1
          count: 1
        """
      When mdcheck checks the document
      Then no findings are reported
      And the diagnostic naming the missing path is written to stderr
      And the exit code is 2
```

## The exit code as the branch point

```gherkin
Feature: The exit code classifies the outcome of a run

  A CI Pipeline branches on the exit code alone: green, document findings, a call
  or configuration it must fix, or a defect in mdcheck itself.

  Rule: 0 all checks pass · 1 findings emitted · 2 usage, rule-file or selector error · 3 internal error

    # Rule-file and selector errors are specified in e2e/rules.md; they appear here
    # only as members of the exit code 2 class.
    @CLI-1 @CLI-2 @CLI-3
    Scenario Outline: the outcome class determines the exit code
      Given <situation>
      When mdcheck is invoked
      Then the exit code is <code>

      Examples: outcome classes
        | situation                                                         | code |
        | a document and rules under which every check passes               | 0    |
        | several documents and rules under which every document is clean   | 0    |
        | a document and rules under which at least one check fails         | 1    |
        | several documents of which one produces a finding                 | 1    |
        | an invocation naming no document to check                         | 2    |
        | an invocation naming a document path that does not exist          | 2    |
        | an invocation whose rule file cannot be loaded                    | 2    |
        | an invocation whose rule file holds an unparsable selector        | 2    |

  Rule: A failure inside mdcheck is reported as an internal error, distinct from findings and from a bad call

    # Open question: specs.md names exit code 3 "internal error" but states no
    # triggerable condition, so no input in this suite can provoke it; the scenario
    # fixes only the contract that such a failure is neither a finding nor exit 2.
    @CLI-N11 @assumption
    Scenario: an internal failure is not reported as a finding
      Given a run that fails for a reason internal to mdcheck
      When mdcheck is invoked
      Then no findings are reported
      And the diagnostic is written to stderr
      And the exit code is 3
```
