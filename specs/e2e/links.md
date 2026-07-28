---
confirmed: No
---

# E2E: Links

The three link checks that look beyond the document itself — requiring links to be wiki-links, resolving wiki targets against the vault root supplied with `--root`, and forbidding a whole class of link by matching its target.

## Requiring every selected link to be a wiki-link

```gherkin
Feature: Selected links must be wiki-links

  Rule: With `wiki: true`, every link the selector matches must be written as a wiki-link; each link that is not produces its own finding naming that link

    @LNK-1
    Scenario: An inline link among wiki-links is reported
      Given a document:
        """markdown
        See [[features/x|X]] and [ext](https://e.com).
        """
      And the rules:
        """yaml
        - select: A
          wiki: true
        """
      When mdcheck checks the document
      Then a finding is reported at line 1: "`[ext](https://e.com)` is not a wiki-link"
      And no other finding is reported
      And the exit code is 1

    @LNK-N1
    Scenario: A document whose links are all wiki-links passes
      Given a document:
        """markdown
        # Doc

        See [[features/x|X]] and [[features/y]].
        """
      And the rules:
        """yaml
        - select: A
          wiki: true
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @LNK-N2
    Scenario: Each inline link in one document is reported separately, naming its own target
      Given a document:
        """markdown
        # Doc

        See [ext](https://e.com).

        Also [notes](./notes.md).
        """
      And the rules:
        """yaml
        - select: A
          wiki: true
        """
      When mdcheck checks the document
      Then a finding is reported at line 3: "`[ext](https://e.com)` is not a wiki-link"
      And a finding is reported at line 5: "`[notes](./notes.md)` is not a wiki-link"
      And the exit code is 1
```

## Resolving wiki targets against the vault root

```gherkin
Feature: Wiki-link targets must exist under the vault root

  Rule: With `resolve: true`, each selected wiki-link target must exist as a file under the root supplied with `--root`; a target with no file produces a finding naming that target

    @LNK-2
    Scenario: A wiki-link whose target file is absent under the root is reported
      Given a document:
        """markdown
        See [[features/x/stories|X]].
        """
      And the rules:
        """yaml
        - select: A[wiki]
          resolve: true
        """
      And a vault root with no "features/x/stories.md"
      When mdcheck checks the document against the vault root
      Then a finding is reported at line 1: "target `features/x/stories.md` not found under root"
      And the exit code is 1

    @LNK-N3
    Scenario: A wiki-link whose target file exists under the root passes
      Given a document:
        """markdown
        See [[features/x/stories|X]].
        """
      And the rules:
        """yaml
        - select: A[wiki]
          resolve: true
        """
      And a vault root containing "features/x/stories.md"
      When mdcheck checks the document against the vault root
      Then the run passes with no findings
      And the exit code is 0

    @LNK-N4
    Scenario: An aliased wiki-link resolves on its path, not on its display label
      Given a document:
        """markdown
        See [[features/x|Overview]].
        """
      And the rules:
        """yaml
        - select: A[wiki]
          resolve: true
        """
      And a vault root containing "features/x.md"
      And a vault root with no "Overview.md"
      When mdcheck checks the document against the vault root
      Then the run passes with no findings
      And the exit code is 0

    @LNK-N5
    Scenario: Each unresolvable wiki-link is reported separately, naming its own target
      Given a document:
        """markdown
        # Doc

        See [[features/x/stories|X]].

        And [[features/y/stories|Y]].
        """
      And the rules:
        """yaml
        - select: A[wiki]
          resolve: true
        """
      And a vault root with no "features/x/stories.md"
      And a vault root with no "features/y/stories.md"
      When mdcheck checks the document against the vault root
      Then a finding is reported at line 3: "target `features/x/stories.md` not found under root"
      And a finding is reported at line 5: "target `features/y/stories.md` not found under root"
      And the exit code is 1
```

## Asking for resolution without a vault root

```gherkin
Feature: Resolution needs a root to resolve against

  Rule: A rule that asks for `resolve: true` when no root was supplied cannot be evaluated and is rejected as a usage error, before any finding is emitted

    @LNK-N6 @assumption
    # Open question: specs.md introduces --root as the "link-resolution base" but does not
    # say what `resolve: true` does when it is omitted. Three readings are open: reject the
    # invocation as a usage error (assumed here, exit 2), silently skip the resolve check,
    # or default the root to the checked document's own directory.
    Scenario: Resolution requested with no root supplied is a usage error
      Given a document:
        """markdown
        See [[features/x/stories|X]].
        """
      And the rules:
        """yaml
        - select: A[wiki]
          resolve: true
        """
      When mdcheck checks the document
      Then no findings are reported
      And a diagnostic stating that `resolve: true` requires a root is written to stderr
      And the exit code is 2
```

## Forbidding a class of link by its target

```gherkin
Feature: A class of link can be forbidden outright

  Rule: A selector that filters links by target combined with `count: 0` forbids that class; each matching link produces its own finding naming it

    @LNK-3
    Scenario: External links are forbidden and each one is reported
      Given a document:
        """markdown
        # Doc

        See [ext](https://e.com).

        And [more](http://f.org).
        """
      And the rules:
        """yaml
        - select: A[href=/^https?:/]
          count: 0
        """
      When mdcheck checks the document
      Then a finding is reported at line 3: "link `https://e.com` matched, expected 0 matches"
      And a finding is reported at line 5: "link `http://f.org` matched, expected 0 matches"
      And the exit code is 1

    @LNK-N7
    Scenario: A document with no link of the forbidden class passes
      Given a document:
        """markdown
        # Doc

        See [[features/x|X]] and [notes](./notes.md).
        """
      And the rules:
        """yaml
        - select: A[href=/^https?:/]
          count: 0
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0
```

## Scoping a link rule to one section

```gherkin
Feature: A link rule judges only the links its selector reaches

  Rule: A link rule scoped to a section judges that section's links only; links elsewhere in the document are never selected and never reported

    @LNK-N8
    Scenario: An inline link outside the scoped section is not judged
      Given a document:
        """markdown
        # T

        ## Notes

        See [ext](https://e.com).

        ## Refs

        See [[features/x|X]].
        """
      And the rules:
        """yaml
        - select: SECTION("Refs") A
          wiki: true
        """
      When mdcheck checks the document
      Then the run passes with no findings
      And the exit code is 0

    @LNK-N9
    Scenario: An inline link inside the scoped section is judged
      Given a document:
        """markdown
        # T

        ## Notes

        See [[features/x|X]].

        ## Refs

        See [ext](https://e.com).
        """
      And the rules:
        """yaml
        - select: SECTION("Refs") A
          wiki: true
        """
      When mdcheck checks the document
      Then a finding is reported at line 9: "`[ext](https://e.com)` is not a wiki-link"
      And no other finding is reported
      And the exit code is 1
```
