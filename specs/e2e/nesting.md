---
confirmed: No
---

# E2E: Nested rule scoping

How a nested rule list under `each:` is applied once per outer selector match, so that a rule says "every block has one" instead of "some block has many".

## A nested rule list is applied to each block on its own

```gherkin
Feature: A nested rule list is evaluated once per outer match

  Rule: Every match of the outer selector becomes the root scope for the inner rules; a match where every inner rule holds contributes nothing, a match that breaks one is reported at that match

    @SUB-1 @assumption
    # Open question: scenarios.md lists only the inner-list finding for the "Broken" block,
    # although that block's first paragraph does not start with `pre:` either. It is unsettled
    # whether `optional: true` merely excuses an inner selector that matches nothing (specs.md
    # section 3) or makes the whole inner rule advisory. This scenario therefore asserts only
    # what SUB-1 states and makes no claim about a second finding on the same block.
    Scenario: A story block with no acceptance list is reported while its well-formed sibling is not
      Given a document:
        """markdown
        ## User Stories
        ### Login
        pre: user exists
        As **Visitor**
        - I can log in
        ### Broken
        some text
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: P[0]
              optional: true
              starts_with: 'pre:'
            - select: P[text=/^As \*\*.+\*\*/] + UL > LI
              min: 1
              match: '^I can .+'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Broken" section: "inner rule `P[text=/^As \*\*.+\*\*/] + UL > LI`: 0 matches, expected ≥ 1"
      And no finding is reported at the "Login" section
      And the exit code is 1

    @SUB-N1
    Scenario: A document where every story block satisfies every inner rule passes
      Given a document:
        """markdown
        ## User Stories

        ### Login

        pre: the user has an account

        As **Visitor**

        - I can log in

        ### Logout

        pre: the user is signed in

        As **Visitor**

        - I can log out
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: P[0]
              optional: true
              starts_with: 'pre:'
            - select: P[text=/^As \*\*.+\*\*/] + UL > LI
              min: 1
              match: '^I can .+'
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## One malformed block does not mask the others

```gherkin
Feature: Every block is checked whatever the outcome of the blocks around it

  Rule: Every outer match is evaluated independently in a single run — a failing match neither stops later matches from being checked nor suppresses further findings on itself

    @SUB-N2
    Scenario: Two malformed blocks each yield their own finding in one run
      Given a document:
        """markdown
        ## User Stories

        ### Login

        some text

        ### Logout

        more text

        ### Reset

        - I can reset my password
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: UL > LI
              min: 1
              match: '^I can .+'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Login" section: "inner rule `UL > LI`: 0 matches, expected ≥ 1"
      And a finding is reported at the "Logout" section: "inner rule `UL > LI`: 0 matches, expected ≥ 1"
      And no finding is reported at the "Reset" section
      And the exit code is 1

    @SUB-N3
    Scenario: A malformed block between two well-formed ones is the only one reported
      Given a document:
        """markdown
        ## User Stories

        ### Login

        - I can log in

        ### Broken

        some text

        ### Logout

        - I can log out
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: UL > LI
              min: 1
              match: '^I can .+'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Broken" section: "inner rule `UL > LI`: 0 matches, expected ≥ 1"
      And no other finding is reported
      And the exit code is 1

    @SUB-N4
    Scenario: Two inner rules broken by the same block yield two findings for that block
      Given a document:
        """markdown
        ## User Stories

        ### Login

        Intro paragraph.

        - I can log in

        ### Broken
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: P
              min: 1
            - select: UL > LI
              min: 1
        """
      When mdcheck checks the document
      Then a finding is reported at the "Broken" section: "inner rule `P`: 0 matches, expected ≥ 1"
      And a finding is reported at the "Broken" section: "inner rule `UL > LI`: 0 matches, expected ≥ 1"
      And no finding is reported at the "Login" section
      And the exit code is 1
```

## A block sees only its own content

```gherkin
Feature: An inner selector cannot reach into a neighbouring block

  Rule: The outer match is the root scope for its inner rules — content belonging to a sibling match neither satisfies an inner rule nor counts towards it

    @SUB-N5
    Scenario: A block with no acceptance list is reported although a sibling block has several
      Given a document:
        """markdown
        ## User Stories

        ### Login

        - I can log in
        - I can log in with a one-time code
        - I can stay signed in

        ### Logout

        Signing out is described in prose only.
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: UL > LI
              min: 1
              match: '^I can .+'
        """
      When mdcheck checks the document
      Then a finding is reported at the "Logout" section: "inner rule `UL > LI`: 0 matches, expected ≥ 1"
      And no finding is reported at the "Login" section
      And the exit code is 1

    @SUB-N6
    Scenario: Inner matches are counted per block rather than pooled across blocks
      Given a document:
        """markdown
        ## User Stories

        ### Login

        - I can log in

        ### Logout

        - I can log out
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: UL > LI
              count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Where an inner finding points

```gherkin
Feature: An inner finding identifies the block and the inner rule that failed

  Rule: An inner rule that matches nothing is reported at its outer match and names the inner selector; an inner rule broken by a particular element is reported at that element

    @SUB-N7 @assumption
    # Open question: SUB-1 fixes the anchor only for an inner rule with zero matches — the
    # block header. specs.md does not state where an inner rule that matched elements but
    # failed on one of them is anchored. The element's own line is assumed here, consistent
    # with the same assertion applied by a top-level rule (LST-1).
    Scenario: An inner rule broken by one list item is reported at that item
      Given a document:
        """markdown
        ## User Stories

        ### Login

        - I can log in
        - broken item
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: UL > LI
              match: '^I can .+'
        """
      When mdcheck checks the document
      Then a finding is reported at line 6: "does not match pattern"
      And the exit code is 1
```

## An optional inner rule is judged per block

```gherkin
Feature: optional inside a nested rule list applies to one block at a time

  Rule: An inner rule marked optional is excused in a block where its selector matches nothing, regardless of whether that selector matches in other blocks

    @SUB-N8
    Scenario: A block with no lead paragraph passes while a sibling block has one
      Given a document:
        """markdown
        ## User Stories

        ### Login

        - I can log in

        ### Logout

        pre: the user is signed in

        - I can log out
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: P[0]
              optional: true
              starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @SUB-N9 @assumption
    # Open question: specs.md defines `optional: true` as "rule passes when selector matches
    # nothing", which makes a block whose lead paragraph exists but breaks the assertion a
    # finding. SUB-1's expected list can be read as contradicting this. The specs.md reading
    # is assumed here. The finding's line also rests on the element-anchoring assumption above.
    Scenario: A block whose lead paragraph exists but breaks the assertion is reported
      Given a document:
        """markdown
        ## User Stories

        ### Login

        Some prose that is not a precondition.

        - I can log in
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: P[0]
              optional: true
              starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then a finding is reported at line 5: "does not start with `pre:`"
      And the exit code is 1
```

## When nothing matches the outer selector

```gherkin
Feature: A nested rule list over an empty set of blocks

  Rule: With no outer match there is no scope for the inner rules to run in and nothing is reported; requiring blocks to exist is a separate assertion on the outer rule

    @SUB-N10 @assumption
    # Open question: specs.md states that inner rules run per outer match but does not say
    # whether an `each:` rule with zero outer matches is vacuously satisfied or requires
    # `optional: true` to be stated. Vacuous satisfaction is assumed.
    Scenario: A document with no story blocks passes when the rule only nests inner rules
      Given a document:
        """markdown
        ## User Stories

        Nothing here yet.
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          each:
            - select: UL > LI
              min: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @SUB-N11
    Scenario: A document with no story blocks is reported when the outer rule also demands one
      Given a document:
        """markdown
        ## User Stories

        Nothing here yet.
        """
      And the rules:
        """yaml
        - select: SECTION("User Stories") > SECTION(/^[A-Z]/)
          min: 1
          each:
            - select: UL > LI
              min: 1
        """
      When mdcheck checks the document
      Then a finding is reported at the "User Stories" section: "0 matches, expected ≥ 1"
      And no other finding is reported
      And the exit code is 1
```
