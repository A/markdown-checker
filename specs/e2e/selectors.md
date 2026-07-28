---
confirmed: No
---

# E2E: Selectors

How a Rule Author's selector addresses a document — what scoping, combinators, attribute filters, call sugar and positional index pick out, observed through the simplest possible assertion so that the selection itself is what each scenario proves.

Line numbers in `Then` steps count the document docstring's first line as line 1.

## Scoping a rule to one section

```gherkin
Feature: Section scoping

  Rule: A selector rooted at a SECTION sees only that section's content

    @SEL-1
    Scenario: Items of a neighbouring section are never counted
      Given a document:
        """
        # T
        ## A
        - one
        ## B
        - two
        - three
        """
      And the rules:
        """
        - select: SECTION("B") > UL > LI
          min: 2
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N1
    Scenario: A section is not credited with a sibling section's items
      Given a document:
        """
        # T
        ## A
        - one
        ## B
        - two
        - three
        """
      And the rules:
        """
        - select: SECTION("A") > UL > LI
          min: 2
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "1 match, expected ≥ 2"
      And the exit code is 1
```

## What a SECTION spans

```gherkin
Feature: Section extent

  Rule: A SECTION covers its header and everything that follows it until the next header of the same level, so a subsection belongs to its parent section and a sibling section does not

    @SEL-N2
    Scenario: A parent section owns its subsection's content
      Given a document:
        """
        # T
        ## Parent
        - p1
        ### Child
        - c1
        ## Sibling
        - s1
        """
      And the rules:
        """
        - select: SECTION("Parent") UL > LI
          count: 2
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N3
    Scenario: A subsection addressed on its own covers only its own body
      Given a document:
        """
        # T
        ## Parent
        - p1
        ### Child
        - c1
        ## Sibling
        - s1
        """
      And the rules:
        """
        - select: SECTION("Child") UL > LI
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings
```

## Child versus descendant

```gherkin
Feature: Child and descendant combinators

  Rule: `>` matches only direct children, so it never reaches through an intervening element, while a space matches at any depth

    @SEL-N4
    Scenario: The child combinator does not reach into a subsection
      Given a document:
        """
        # T
        ## Parent
        - p1
        ### Child
        - c1
        ## Sibling
        - s1
        """
      And the rules:
        """
        - select: SECTION("Parent") > UL > LI
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N5
    Scenario Outline: Nested list items are reached by descendant but not by child
      Given a document:
        """
        # T
        ## Items
        - top
          - nested
        """
      And the rules:
        """
        - select: <selector>
          count: <count>
        """
      When mdcheck checks the document
      Then the run passes with no findings

      Examples:
        | selector                     | count |
        | SECTION("Items") > UL > LI   | 1     |
        | SECTION("Items") UL LI       | 2     |

    @SEL-N6
    Scenario Outline: Headers nest by level under their parent header
      Given a document:
        """
        # T
        ## A
        ### A1
        ## B
        """
      And the rules:
        """
        - select: <selector>
          count: <count>
        """
      When mdcheck checks the document
      Then the run passes with no findings

      Examples:
        | selector | count |
        | H1 > H2  | 2     |
        | H1 > H3  | 0     |
        | H1 H3    | 1     |
        | H2 > H3  | 1     |
```

## Adjacent sibling

```gherkin
Feature: Adjacent sibling combinator

  Rule: `P + UL` matches the list immediately after that paragraph and no later one

    @SEL-N7
    Scenario: Only the list directly after the paragraph is selected
      Given a document:
        """
        # T
        ## Story
        intro

        - alpha

        later

        - beta
        - gamma
        """
      And the rules:
        """
        - select: SECTION("Story") > P[text="intro"] + UL > LI
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N8
    Scenario: A list separated from the paragraph by another element is not adjacent
      Given a document:
        """
        # T
        ## Story
        intro

        later

        - alpha
        """
      And the rules:
        """
        - select: SECTION("Story") > P[text="intro"] + UL > LI
          min: 1
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "0 matches, expected ≥ 1"
      And the exit code is 1
```

## Attribute filters on text

```gherkin
Feature: Text attribute operators

  Rule: `[text=…]` compares whole text, `^=` prefix, `$=` suffix, `*=` substring, `=/…/` regexp

    @SEL-N9
    Scenario Outline: Each operator selects exactly the paragraphs it describes
      Given a document:
        """
        # T
        Roles

        Roles and duties

        pre: setup

        is this the end?

        a feature list

        I can log in
        """
      And the rules:
        """
        - select: <selector>
          count: <count>
        """
      When mdcheck checks the document
      Then the run passes with no findings

      Examples:
        | selector             | count |
        | P[text="Roles"]      | 1     |
        | P[text*="Roles"]     | 2     |
        | P[text^="pre:"]      | 1     |
        | P[text$="?"]         | 1     |
        | P[text*="feature"]   | 1     |
        | P[text=/^I can .+/]  | 1     |
```

## Attribute filters on section titles

```gherkin
Feature: Title attribute operators

  Rule: `[title=…]` addresses a SECTION by its header text with the same operators as `[text=…]`

    @SEL-N10
    Scenario Outline: Sections are selected by title with every operator
      Given a document:
        """
        # T
        ## Roles
        x
        ## Story: login
        y
        ## Story: logout
        z
        """
      And the rules:
        """
        - select: <selector>
          count: <count>
        """
      When mdcheck checks the document
      Then the run passes with no findings

      Examples:
        | selector                       | count |
        | SECTION[title="Roles"]         | 1     |
        | SECTION[title^="Story: "]      | 2     |
        | SECTION[title$="login"]        | 1     |
        | SECTION[title*="log"]          | 2     |
        | SECTION[title=/^Story: log/]   | 2     |
```

## Call sugar

```gherkin
Feature: Call-form sugar

  Rule: `H2("Roles")` is the same selector as `H2[text="Roles"]`, `SECTION("Roles")` the same as `SECTION[title="Roles"]`, and both call forms also accept a regexp

    @SEL-N11
    Scenario Outline: The call form and the attribute form select the same elements
      Given a document:
        """
        # T
        ## Roles
        x
        ## Story: login
        y
        ## Story: logout
        z
        """
      And the rules:
        """
        - select: <selector>
          count: <count>
        """
      When mdcheck checks the document
      Then the run passes with no findings

      Examples:
        | selector                      | count |
        | H2("Roles")                   | 1     |
        | H2[text="Roles"]              | 1     |
        | H2(/^Story: /)                | 2     |
        | H2[text=/^Story: /]           | 2     |
        | SECTION("Roles")              | 1     |
        | SECTION[title="Roles"]        | 1     |
        | SECTION(/^Story: /)           | 2     |
        | SECTION[title=/^Story: /]     | 2     |
```

## Positional index

```gherkin
Feature: Positional index

  Rule: A bare integer in brackets picks one match by position within the current scope — `[0]` the first, `[-1]` the last

    @SEL-2
    Scenario: The first paragraph of a section is addressed with index 0
      Given a document:
        """
        # T
        ## Story
        pre: setup line
        As a user
        """
      And the rules:
        """
        - select: SECTION("Story") > P[0]
          starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N12
    Scenario: Index 0 selects the first paragraph and not a later one
      Given a document:
        """
        # T
        ## Story
        pre: setup

        As a user

        done: teardown

        ## Other
        first other
        """
      And the rules:
        """
        - select: SECTION("Story") > P[0]
          starts_with: 'As'
        """
      When mdcheck checks the document
      Then a finding is reported at line 3: "does not start with 'As'"
      And the exit code is 1

    @SEL-N13
    Scenario: Index -1 selects the last paragraph in scope
      Given a document:
        """
        # T
        ## Story
        pre: setup

        As a user

        done: teardown

        ## Other
        first other
        """
      And the rules:
        """
        - select: SECTION("Story") > P[-1]
          starts_with: 'done:'
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N14
    Scenario: Index counts from the start of the current scope, not the document
      Given a document:
        """
        # T
        ## Story
        pre: setup

        As a user

        done: teardown

        ## Other
        first other
        """
      And the rules:
        """
        - select: SECTION("Other") > P[0]
          starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then a finding is reported at line 10: "does not start with 'pre:'"
      And the exit code is 1

    @SEL-3
    Scenario: An index over an empty scope is accepted when the rule is optional
      Given a document:
        """
        # T
        ## Story

        ## Other
        text
        """
      And the rules:
        """
        - select: SECTION("Story") > P[0]
          optional: true
          starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N15 @assumption
    # Open question: specs.md states only that `optional: true` makes an empty
    # selection pass. It does not state the finding an empty selection produces
    # without it — neither the message nor whether it is reported at the scope's
    # line, as the min-violation findings in scenarios.md are.
    Scenario: An index over an empty scope is a finding when the rule is not optional
      Given a document:
        """
        # T
        ## Story

        ## Other
        text
        """
      And the rules:
        """
        - select: SECTION("Story") > P[0]
          starts_with: 'pre:'
        """
      When mdcheck checks the document
      Then a finding is reported at line 2: "selector matched no elements"
      And the exit code is 1
```

## Code fences by language

````gherkin
Feature: Fenced code blocks

  Rule: `CODE` selects fenced blocks, and `[lang=…]` narrows the selection to one fence language

    @SEL-N16
    Scenario: Only the fence with the named language is selected
      Given a document:
        """
        # T
        ## Examples

        ```gherkin
        Scenario: x
        ```

        ```yaml
        note: TBD
        ```
        """
      And the rules:
        """
        - select: CODE[lang="gherkin"]
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N17
    Scenario: An unfiltered CODE selector takes every fence
      Given a document:
        """
        # T
        ## Examples

        ```gherkin
        Scenario: x
        ```

        ```yaml
        note: TBD
        ```
        """
      And the rules:
        """
        - select: CODE
          count: 2
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N18
    Scenario: Content of a fence in another language is outside the selection
      Given a document:
        """
        # T
        ## Examples

        ```gherkin
        Scenario: x
        ```

        ```yaml
        note: TBD
        ```
        """
      And the rules:
        """
        - select: CODE[lang="gherkin"]
          forbid: 'TBD'
        """
      When mdcheck checks the document
      Then the run passes with no findings
````

## Whole-unit selections

```gherkin
Feature: DOCUMENT and FRONTMATTER

  Rule: `DOCUMENT` selects the whole document as one element and scopes to everything in it; `FRONTMATTER` selects the frontmatter block, which is not part of the body

    @SEL-N19
    Scenario: DOCUMENT is a single match whatever the document contains
      Given a document:
        """
        ---
        confirmed: no
        ---
        # T
        ## A
        - one
        ## B
        - two
        """
      And the rules:
        """
        - select: DOCUMENT
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N20
    Scenario: A DOCUMENT scope spans every section
      Given a document:
        """
        ---
        confirmed: no
        ---
        # T
        ## A
        - one
        ## B
        - two
        """
      And the rules:
        """
        - select: DOCUMENT LI
          count: 2
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N21
    Scenario: FRONTMATTER is a single match
      Given a document:
        """
        ---
        confirmed: no
        ---
        # T
        ## A
        - one
        ## B
        - two
        """
      And the rules:
        """
        - select: FRONTMATTER
          count: 1
        """
      When mdcheck checks the document
      Then the run passes with no findings

    @SEL-N22
    Scenario: Frontmatter values are not body elements
      Given a document:
        """
        ---
        title: Doc
        summary: a line of text
        ---
        # T
        """
      And the rules:
        """
        - select: P
          count: 0
        """
      When mdcheck checks the document
      Then the run passes with no findings
```
