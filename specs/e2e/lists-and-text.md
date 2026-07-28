---
confirmed: No
---

# E2E: Lists, text and cardinality

The element-agnostic value and quantity assertions — regexp modes, prefix/suffix, length and line bounds, allowed sets, emptiness, match counts and rule optionality — demonstrated on headers, paragraphs, list items and the document as a whole.

## Matching element text with a regexp

```gherkin
Feature: Regexp assertions against element text

  Rule: `match` requires the whole value to match the pattern; `require` only that the pattern appears somewhere in it; `forbid` that it appears nowhere — one text and one pattern give three different results

    @LST-1
    Scenario: A list item whose whole text does not match the required shape is reported at its own line
      Given a document:
        """markdown
        ## Roles
        - **Admin** — manages
        - broken item
        """
      And the rules:
        """yaml
        - select: SECTION("Roles") > UL > LI
          match: '^\*\*.+\*\* — .+'
        """
      When mdcheck checks the document
      Then a finding is reported at the "broken item" line: "does not match pattern"
      And no other finding is reported
      And the exit code is 1

    @TXT-N1
    Scenario: `require` passes on a value that merely contains the pattern
      Given a document:
        """markdown
        # Doc

        Status: pending review
        """
      And the rules:
        """yaml
        - select: P
          require: 'pending'
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @TXT-N2
    Scenario: `match` reports the same value and the same pattern that `require` accepted
      Given a document:
        """markdown
        # Doc

        Status: pending review
        """
      And the rules:
        """yaml
        - select: P
          match: 'pending'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Status: pending review" line: "does not match pattern"
      And the exit code is 1

    @TXT-N3
    Scenario: `forbid` reports the same value and the same pattern that `require` accepted
      Given a document:
        """markdown
        # Doc

        Status: pending review
        """
      And the rules:
        """yaml
        - select: P
          forbid: 'pending'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Status: pending review" line: "contains forbidden pattern"
      And the exit code is 1

    @TXT-N4
    Scenario: `require` reports a value the pattern never appears in
      Given a document:
        """markdown
        # Doc

        Status: done
        """
      And the rules:
        """yaml
        - select: P
          require: 'pending'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Status: done" line: "required pattern not found"
      And the exit code is 1
```

## Literal prefix and suffix

```gherkin
Feature: Element text is constrained by a literal prefix or suffix

  Rule: `starts_with` and `ends_with` compare literal text, and report each element that does not carry the prefix or the suffix

    @TXT-N5
    Scenario: A paragraph without the required prefix is reported at its own line
      Given a document:
        """markdown
        # Doc

        pre: setup line

        As a user
        """
      And the rules:
        """yaml
        - select: P
          starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then a finding is reported at the "As a user" line: "does not start with `pre:`"
      And no other finding is reported
      And the exit code is 1

    @TXT-N6
    Scenario: A header without the required suffix is reported at its own line
      Given a document:
        """markdown
        # Doc

        ## Questions?

        ## Roles
        """
      And the rules:
        """yaml
        - select: H2
          ends_with: '?'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Roles" line: "does not end with `?`"
      And no other finding is reported
      And the exit code is 1
```

## Element text length bounds

```gherkin
Feature: Element text length is bounded from below and from above

  Rule: `min_length` and `max_length` bound the length of each matched element's text; only the elements outside the bound are reported, each at its own line

    @LEN-1
    Scenario Outline: A header outside the configured length bound is reported at its own line
      Given a document:
        """markdown
        # Doc

        ## Hi

        ## This header is way too long for the configured limit of twenty
        """
      And the rules:
        """yaml
        - select: H2
          <key>: <bound>
        """
      When mdcheck checks the document
      Then a finding is reported at the "<offending header>" line: "<message>"
      And no other finding is reported
      And the exit code is 1

      Examples:
        | key        | bound | offending header                                             | message       |
        | min_length | 5     | Hi                                                           | length 2 < 5  |
        | max_length | 20    | This header is way too long for the configured limit of twenty | length 62 > 20 |

  Rule: Every element that breaks the bound is reported, one finding per element at its own line, all in a single run

    @LEN-2
    Scenario: Two oversized paragraphs among four produce two findings in one run
      Given a document:
        """markdown
        # Doc

        Short.

        This paragraph is comfortably over the cap.

        Also fine.

        This other paragraph is over the cap as well.
        """
      And the rules:
        """yaml
        - select: P
          max_length: 20
        """
      When mdcheck checks the document
      Then a finding is reported at the "This paragraph is comfortably over the cap." line: "length 43 > 20"
      And a finding is reported at the "This other paragraph is over the cap as well." line: "length 45 > 20"
      And no other finding is reported
      And the exit code is 1
```

## Document line count

```gherkin
Feature: The document as a whole is required to reach a line count

  Rule: `min_lines` on DOCUMENT compares the document's line count against the bound and reports once, at the top of the document

    @DOC-1
    Scenario: A document shorter than the required line count is reported at line 1
      Given a document:
        """markdown
        # Doc

        Body.
        """
      And the rules:
        """yaml
        - select: DOCUMENT
          min_lines: 20
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "document has 3 lines, expected ≥ 20"
      And the exit code is 1

    @TXT-N7
    Scenario: A document with exactly the required number of lines passes
      Given a document that is exactly 20 lines long
      And the rules:
        """yaml
        - select: DOCUMENT
          min_lines: 20
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Cardinality of the selector matches

```gherkin
Feature: The number of elements a selector matches is bounded

  Rule: `min` and `max` bound how many elements the selector matches; the finding is reported once, at the line of the scope the selector was evaluated in

    @LST-2
    Scenario: Fewer matches than the minimum are reported at the scope line
      Given a document:
        """markdown
        # T

        ## Stories

        - only one
        """
      And the rules:
        """yaml
        - select: SECTION("Stories") > UL > LI
          min: 2
          max: 10
        """
      When mdcheck checks the document
      Then a finding is reported at the "Stories" line: "1 match, expected ≥ 2"
      And no other finding is reported
      And the exit code is 1

    @TXT-N8
    Scenario: More matches than the maximum are reported at the scope line
      Given a document with a "Stories" section holding a list of 11 items
      And the rules:
        """yaml
        - select: SECTION("Stories") > UL > LI
          min: 2
          max: 10
        """
      When mdcheck checks the document
      Then a finding is reported at the "Stories" line: "11 matches, expected ≤ 10"
      And no other finding is reported
      And the exit code is 1

  Rule: `count` demands an exact number of matches and reports whenever the actual number differs, whether it is below or above

    @TXT-N9
    Scenario Outline: An exact match count that is missed in either direction is reported at the scope line
      Given a document:
        """markdown
        # T

        ## Stories

        - one
        - two
        """
      And the rules:
        """yaml
        - select: SECTION("Stories") > UL > LI
          count: <count>
        """
      When mdcheck checks the document
      Then a finding is reported at the "Stories" line: "<message>"
      And the exit code is 1

      Examples:
        | count | message                          |
        | 3     | 2 matches, expected exactly 3    |
        | 1     | 2 matches, expected exactly 1    |
```

## Allowed values and empty values

```gherkin
Feature: Element values are drawn from an allowed set and are never empty

  Rule: `enum` reports every matched element whose value is outside the allowed set, at that element's line

    @TXT-N10
    Scenario: A list item outside the allowed set is reported at its own line
      Given a document:
        """markdown
        # T

        ## Priorities

        - p0
        - p9
        """
      And the rules:
        """yaml
        - select: SECTION("Priorities") > UL > LI
          enum: [p0, p1, p2, p3]
        """
      When mdcheck checks the document
      Then a finding is reported at the "p9" line: "`p9` not in allowed set"
      And no other finding is reported
      And the exit code is 1

  Rule: `no_empty` reports every matched element whose value is empty, at that element's line

    @TXT-N11
    Scenario: An empty list item is reported at its own line
      Given a document:
        """markdown
        # T

        ## Roles

        - Admin
        -
        """
      And the rules:
        """yaml
        - select: SECTION("Roles") > UL > LI
          no_empty: true
        """
      When mdcheck checks the document
      Then a finding is reported at line 6: "empty list item"
      And no other finding is reported
      And the exit code is 1
```

## A rule whose selector matches nothing

```gherkin
Feature: Optionality of a whole rule

  Rule: A rule marked `optional: true` passes when its selector matches nothing, whatever assertions it carries

    @SEL-3
    # The indexing half of SEL-3 — an index such as P[0] applied to an empty scope —
    # is covered in e2e/selectors.md and is deliberately not repeated here.
    Scenario: A rule whose selector matches nothing passes when it is marked optional
      Given a document:
        """markdown
        # T

        ## Stories

        - one
        """
      And the rules:
        """yaml
        - select: SECTION("Roles") > UL > LI
          min: 2
          optional: true
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @TXT-N12
    Scenario: An optional rule still asserts on the elements it does match
      Given a document:
        """markdown
        # T

        ## Roles

        - broken item
        """
      And the rules:
        """yaml
        - select: SECTION("Roles") > UL > LI
          match: '^\*\*.+\*\* — .+'
          optional: true
        """
      When mdcheck checks the document
      Then a finding is reported at the "broken item" line: "does not match pattern"
      And the exit code is 1

  Rule: A rule that is not marked `optional: true` reports when its selector matches nothing

    @TXT-N13 @assumption
    # Open question: specs.md defines `optional: true` as "rule passes when selector matches
    # nothing" but never states the behaviour without it. Assumed here: an empty match set is
    # itself a finding, since otherwise `optional: true` would assert nothing at all. The
    # competing reading — a non-optional rule passes vacuously and presence must be demanded
    # with `min`/`count` — would make this scenario expect a clean pass instead. The line the
    # finding anchors to for a document-scoped selector is likewise unstated; line 1 assumed.
    Scenario: A rule with no elements to assert on and no optional marker is reported
      Given a document:
        """markdown
        # Doc

        ## Roles
        """
      And the rules:
        """yaml
        - select: P
          starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "selector `P` matched no elements"
      And the exit code is 1
```
