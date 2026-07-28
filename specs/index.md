---
confirmed: No
---

# mdcheck — E2E use-case index

mdcheck applies a YAML rule file of CSS-like selectors and assertions to markdown documents and emits uniform findings. Every row in the tables below is one end-to-end test case: one Scenario or Scenario Outline in `e2e/`, addressed by the stable tag it carries. Ids without an `N` come from [scenarios.md](scenarios.md) and all 34 of them are covered here; ids of the form `<AREA>-N<k>` are cases derived from [specs.md](specs.md) rather than taken from scenarios.md. The Gherkin in `e2e/` is the normative form — this index only points at it.

## Areas

| area | scope | file | cases |
|---|---|---|---|
| Loading | Bringing a document into a checkable state; load-stage findings | [loading](e2e/loading.md) | 8 |
| Rules | Rule-file shape, unknown keys, selector and value-shape errors | [rules](e2e/rules.md) | 15 |
| Selectors | Scoping, combinators, attribute filters, call sugar, index | [selectors](e2e/selectors.md) | 25 |
| Frontmatter | `has:` and `not_has:` over the frontmatter key block | [frontmatter](e2e/frontmatter.md) | 10 |
| Sections | `ordered:`, optional members, `strict:`, title regexps | [sections](e2e/sections.md) | 14 |
| Lists and text | Regexp modes, prefix/suffix, length, enum, emptiness, cardinality | [lists-and-text](e2e/lists-and-text.md) | 19 |
| Tables | `columns:`, `rows:` bounds, per-cell assertions by column name | [tables](e2e/tables.md) | 11 |
| Links | `wiki: true`, `resolve:` against `--root`, forbidden link classes | [links](e2e/links.md) | 12 |
| Nesting | `each:` — an inner rule list run once per outer match | [nesting](e2e/nesting.md) | 12 |
| CLI | Paths, stdin, output formats, stream separation, exit codes | [cli](e2e/cli.md) | 20 |

## Use cases

| id | use-case | area | file |
|---|---|---|---|
| LOAD-N1 | Load a document with a closed frontmatter block and a body cleanly | Loading | e2e/loading.md |
| LOAD-N2 | Load a document with no frontmatter when no rule asks for one | Loading | e2e/loading.md |
| LOAD-1 | Report an unclosed frontmatter block as a load-stage finding | Loading | e2e/loading.md |
| LOAD-N3 | Report unterminated wiki-link markup as a load-stage finding | Loading | e2e/loading.md |
| LOAD-N4 | Run the declared rules after a load-stage finding and report both | Loading | e2e/loading.md |
| LOAD-N5 | Fail the run on a load-stage finding alone when every rule passes | Loading | e2e/loading.md |
| LOAD-N6 | Abort with a stderr diagnostic when the document path does not exist | Loading | e2e/loading.md |
| LOAD-N7 | Abort with a stderr diagnostic when the document cannot be read | Loading | e2e/loading.md |
| RULE-N1 | Accept a rule file with a `doc:` header and a flat `rules:` list | Rules | e2e/rules.md |
| RULE-N2 | Apply every rule in the list, not only the first | Rules | e2e/rules.md |
| CLI-4 | Reject an unknown assertion key, naming the key and the rule index | Rules | e2e/rules.md |
| RULE-N3 | Emit the rule-file error instead of the document's own findings | Rules | e2e/rules.md |
| RULE-N4 | Reject an unknown key inside a nested `each:` rule | Rules | e2e/rules.md |
| RULE-N5 | Reject an unparseable selector, naming it and its rule index | Rules | e2e/rules.md |
| RULE-N6 | Reject selectors left open at a combinator, filter or regexp | Rules | e2e/rules.md |
| RULE-N7 | Reject an element name outside the selector grammar | Rules | e2e/rules.md |
| RULE-N8 | Reject a numeric bound written as a word | Rules | e2e/rules.md |
| RULE-N9 | Reject assertion values whose shape the key does not take | Rules | e2e/rules.md |
| RULE-N10 | Reject a rule file that is not valid YAML | Rules | e2e/rules.md |
| RULE-N11 | Reject a rule file carrying no `rules:` list | Rules | e2e/rules.md |
| RULE-N12 | Reject a `rules:` key holding one mapping instead of a list | Rules | e2e/rules.md |
| RULE-N13 | Pass a rule that matches nothing and asserts no cardinality | Rules | e2e/rules.md |
| RULE-N14 | Report that same empty match when the rule demands a minimum | Rules | e2e/rules.md |
| SEL-1 | Scope a rule to one section so a neighbour's items never count | Selectors | e2e/selectors.md |
| SEL-N1 | Report a section not credited with its sibling section's items | Selectors | e2e/selectors.md |
| SEL-N2 | Span a parent section over its subsection's content | Selectors | e2e/selectors.md |
| SEL-N3 | Limit a subsection addressed on its own to its own body | Selectors | e2e/selectors.md |
| SEL-N4 | Stop the child combinator at the boundary of a subsection | Selectors | e2e/selectors.md |
| SEL-N5 | Reach nested list items by descendant but not by child | Selectors | e2e/selectors.md |
| SEL-N6 | Nest headers by level under their parent header | Selectors | e2e/selectors.md |
| SEL-N7 | Select only the list immediately after the named paragraph | Selectors | e2e/selectors.md |
| SEL-N8 | Report a list separated from its paragraph as not adjacent | Selectors | e2e/selectors.md |
| SEL-N9 | Select paragraphs by text equals, prefix, suffix, substring, regexp | Selectors | e2e/selectors.md |
| SEL-N10 | Select sections by title with every text operator | Selectors | e2e/selectors.md |
| SEL-N11 | Select the same elements through call sugar and attribute form | Selectors | e2e/selectors.md |
| SEL-2 | Address the first paragraph of a section with index 0 | Selectors | e2e/selectors.md |
| SEL-N12 | Report the first paragraph, not a later one, under index 0 | Selectors | e2e/selectors.md |
| SEL-N13 | Select the last paragraph in scope with index -1 | Selectors | e2e/selectors.md |
| SEL-N14 | Count an index from the start of its scope, not the document | Selectors | e2e/selectors.md |
| SEL-3 | Accept an index over an empty scope when the rule is optional | Selectors | e2e/selectors.md |
| SEL-N15 | Report an index over an empty scope when the rule is not optional | Selectors | e2e/selectors.md |
| SEL-N16 | Select only the fence whose language the filter names | Selectors | e2e/selectors.md |
| SEL-N17 | Select every fence with an unfiltered `CODE` selector | Selectors | e2e/selectors.md |
| SEL-N18 | Keep a fence of another language outside the selection | Selectors | e2e/selectors.md |
| SEL-N19 | Match `DOCUMENT` once whatever the document contains | Selectors | e2e/selectors.md |
| SEL-N20 | Span a `DOCUMENT` scope over every section | Selectors | e2e/selectors.md |
| SEL-N21 | Match `FRONTMATTER` once | Selectors | e2e/selectors.md |
| SEL-N22 | Keep frontmatter values out of the body element selection | Selectors | e2e/selectors.md |
| FM-1 | Pass a required key satisfying an exact, enum or regexp value spec | Frontmatter | e2e/frontmatter.md |
| FM-2, FM-5 | Report a key violating its value spec at that key's own line | Frontmatter | e2e/frontmatter.md |
| FM-N1 | Point the finding at the offending key, not the start of the block | Frontmatter | e2e/frontmatter.md |
| FM-N2 | Report a required key absent from a present frontmatter block | Frontmatter | e2e/frontmatter.md |
| FM-3 | Report a document with no frontmatter block at all | Frontmatter | e2e/frontmatter.md |
| FM-N3 | Report one missing-block finding however many keys were required | Frontmatter | e2e/frontmatter.md |
| FM-4 | Report a forbidden key at that key's own line | Frontmatter | e2e/frontmatter.md |
| FM-N4 | Pass a block carrying none of the forbidden keys | Frontmatter | e2e/frontmatter.md |
| FM-N5 | Ignore frontmatter keys the rule file never names | Frontmatter | e2e/frontmatter.md |
| FM-N6 | Report every frontmatter violation of a block in a single run | Frontmatter | e2e/frontmatter.md |
| SEC-1 | Pass required sections present in the declared order | Sections | e2e/sections.md |
| SEC-2 | Report two required sections swapped, at the later one's header | Sections | e2e/sections.md |
| SEC-3 | Report a required section the document never provides | Sections | e2e/sections.md |
| SEC-4 | Pass an optional member present in its declared position | Sections | e2e/sections.md |
| SEC-4 | Pass an optional member that is absent | Sections | e2e/sections.md |
| SEC-N1 | Report an optional member present but out of the declared order | Sections | e2e/sections.md |
| SEC-5 | Report a stray section under `strict:`, at its own header | Sections | e2e/sections.md |
| SEC-N2 | Tolerate an extra section when `strict:` is not declared | Sections | e2e/sections.md |
| SEC-6 | Pass a section required by a title regexp | Sections | e2e/sections.md |
| SEC-N3 | Report a title-regexp selector that matches no section | Sections | e2e/sections.md |
| SEC-N4 | Constrain only the level the skeleton selector picks | Sections | e2e/sections.md |
| SEC-N5 | Leave the level above unconstrained by a nested strict rule | Sections | e2e/sections.md |
| SEC-N6 | Report a nested member out of order at its own header | Sections | e2e/sections.md |
| SEC-N7 | Report missing, misplaced and stray members together in one run | Sections | e2e/sections.md |
| LST-1 | Report a list item whose whole text does not match the pattern | Lists and text | e2e/lists-and-text.md |
| TXT-N1 | Pass a value that merely contains the pattern under `require` | Lists and text | e2e/lists-and-text.md |
| TXT-N2 | Report under `match` the value that `require` accepted | Lists and text | e2e/lists-and-text.md |
| TXT-N3 | Report under `forbid` the value that `require` accepted | Lists and text | e2e/lists-and-text.md |
| TXT-N4 | Report a value the required pattern never appears in | Lists and text | e2e/lists-and-text.md |
| TXT-N5 | Report a paragraph missing the literal prefix, at its own line | Lists and text | e2e/lists-and-text.md |
| TXT-N6 | Report a header missing the literal suffix, at its own line | Lists and text | e2e/lists-and-text.md |
| LEN-1 | Report a header outside its `min_length` or `max_length` bound | Lists and text | e2e/lists-and-text.md |
| LEN-2 | Report each oversized paragraph separately in one run | Lists and text | e2e/lists-and-text.md |
| DOC-1 | Report a document shorter than `min_lines` at line 1 | Lists and text | e2e/lists-and-text.md |
| TXT-N7 | Pass a document with exactly the required number of lines | Lists and text | e2e/lists-and-text.md |
| LST-2 | Report fewer matches than the minimum at the scope line | Lists and text | e2e/lists-and-text.md |
| TXT-N8 | Report more matches than the maximum at the scope line | Lists and text | e2e/lists-and-text.md |
| TXT-N9 | Report an exact `count` missed from either direction | Lists and text | e2e/lists-and-text.md |
| TXT-N10 | Report a list item whose value is outside the allowed set | Lists and text | e2e/lists-and-text.md |
| TXT-N11 | Report an empty list item under `no_empty` | Lists and text | e2e/lists-and-text.md |
| SEL-3 | Pass a rule marked optional whose selector matches nothing | Lists and text | e2e/lists-and-text.md |
| TXT-N12 | Assert an optional rule against the elements it does match | Lists and text | e2e/lists-and-text.md |
| TXT-N13 | Report a non-optional rule whose selector matched no elements | Lists and text | e2e/lists-and-text.md |
| TBL-1 | Pass a table header carrying the declared columns in order | Tables | e2e/tables.md |
| TBL-2 | Report swapped columns as one order mismatch at the header line | Tables | e2e/tables.md |
| TBL-N1 | Report a declared column absent from the header as missing | Tables | e2e/tables.md |
| TBL-N2 | Report a header column the rule does not name | Tables | e2e/tables.md |
| TBL-5 | Report a body row count outside the `rows:` bounds | Tables | e2e/tables.md |
| TBL-N3 | Count body rows only, excluding header and delimiter lines | Tables | e2e/tables.md |
| TBL-N4 | Pass a column whose every cell satisfies its assertion | Tables | e2e/tables.md |
| TBL-3, TBL-4, TBL-6 | Report a cell breaking its column's assertion, at its own row | Tables | e2e/tables.md |
| TBL-6 | Report each offending cell of a column separately in one run | Tables | e2e/tables.md |
| TBL-N5 | Leave an identical value in an unnamed column alone | Tables | e2e/tables.md |
| TBL-N6 | Report nothing for a cell rule naming a column the table lacks | Tables | e2e/tables.md |
| LNK-1 | Report an inline link among wiki-links under `wiki: true` | Links | e2e/links.md |
| LNK-N1 | Pass a document whose links are all wiki-links | Links | e2e/links.md |
| LNK-N2 | Report each inline link separately, naming its own target | Links | e2e/links.md |
| LNK-2 | Report a wiki-link whose target file is absent under the root | Links | e2e/links.md |
| LNK-N3 | Pass a wiki-link whose target file exists under the root | Links | e2e/links.md |
| LNK-N4 | Resolve an aliased wiki-link on its path, not its display label | Links | e2e/links.md |
| LNK-N5 | Report each unresolvable wiki-link separately | Links | e2e/links.md |
| LNK-N6 | Reject `resolve: true` when no vault root was supplied | Links | e2e/links.md |
| LNK-3 | Report each external link forbidden by a target filter | Links | e2e/links.md |
| LNK-N7 | Pass a document holding no link of the forbidden class | Links | e2e/links.md |
| LNK-N8 | Leave a link outside the scoped section unjudged | Links | e2e/links.md |
| LNK-N9 | Report a link inside the scoped section | Links | e2e/links.md |
| SUB-1 | Report a story block with no acceptance list, sparing its sibling | Nesting | e2e/nesting.md |
| SUB-N1 | Pass a document where every block satisfies every inner rule | Nesting | e2e/nesting.md |
| SUB-N2 | Report two malformed blocks, each on its own, in one run | Nesting | e2e/nesting.md |
| SUB-N3 | Report only the malformed block between two well-formed ones | Nesting | e2e/nesting.md |
| SUB-N4 | Report two findings for a block that breaks two inner rules | Nesting | e2e/nesting.md |
| SUB-N5 | Report a block with no list although a sibling has several | Nesting | e2e/nesting.md |
| SUB-N6 | Count inner matches per block rather than pooled across blocks | Nesting | e2e/nesting.md |
| SUB-N7 | Anchor an inner rule broken by one list item at that item | Nesting | e2e/nesting.md |
| SUB-N8 | Excuse an optional inner rule per block, not per document | Nesting | e2e/nesting.md |
| SUB-N9 | Report a lead paragraph that exists but breaks its assertion | Nesting | e2e/nesting.md |
| SUB-N10 | Pass an `each:` rule whose outer selector matches no block | Nesting | e2e/nesting.md |
| SUB-N11 | Report the absent blocks when the outer rule also demands one | Nesting | e2e/nesting.md |
| CLI-N1 | Tag a finding with the path of the document it came from | CLI | e2e/cli.md |
| CLI-N2 | Accept a link-resolution root on a run that asserts no links | CLI | e2e/cli.md |
| CLI-1 | Report a piped document's violation against file `<stdin>` | CLI | e2e/cli.md |
| CLI-1 | Pass a piped document with no violation | CLI | e2e/cli.md |
| CLI-2 | Report two failing documents, each finding tagged with its file | CLI | e2e/cli.md |
| CLI-2 | Fail a run where one document is clean and one is not | CLI | e2e/cli.md |
| CLI-2 | Pass a run only when every named document is clean | CLI | e2e/cli.md |
| CLI-N3 | Report three independent failing rules over one document | CLI | e2e/cli.md |
| CLI-N4 | Report every offending match of one rule, not only the first | CLI | e2e/cli.md |
| CLI-N5 | Present each finding with its document, its line and its message | CLI | e2e/cli.md |
| CLI-3 | Emit a single json object with files, findings and summary | CLI | e2e/cli.md |
| CLI-3 | Emit that same json object with an empty findings list when clean | CLI | e2e/cli.md |
| CLI-3 | Summarise a two-document run as two checked and one failed | CLI | e2e/cli.md |
| CLI-N6 | Report the same findings in json as in the default output | CLI | e2e/cli.md |
| CLI-N7 | Keep everything but the findings off stdout | CLI | e2e/cli.md |
| CLI-N8 | Leave stdout empty when the run cannot start | CLI | e2e/cli.md |
| CLI-N9 | Treat an invocation naming no document as a usage error | CLI | e2e/cli.md |
| CLI-N10 | Report a named document that does not exist as a usage error | CLI | e2e/cli.md |
| CLI-1, CLI-2, CLI-3 | Map each outcome class to exit code 0, 1 or 2 | CLI | e2e/cli.md |
| CLI-N11 | Report an internal failure as exit 3, never as a finding | CLI | e2e/cli.md |

## Open questions

| ref | question | assumed | where |
|---|---|---|---|
| LOAD-N3 | Which malformed link forms are load-stage findings, and worded how | An unterminated wiki-link, reported as "unterminated link syntax" | e2e/loading.md |
| LOAD-N6 | Exit code for an unreadable path, and whether one aborts a whole multi-document run | Exit 2, the usage-error code; the run aborts | e2e/loading.md |
| LOAD-N7 | Exit code when the path exists but cannot be read | Exit 2, as for a path that does not exist | e2e/loading.md |
| RULE-N4 | How a nested rule is identified — outer index only, or a path to the inner rule | The outer rule's index alone | e2e/rules.md |
| RULE-N7 | Whether an element name outside the grammar is rejected at load or matches nothing | Rejected as a rule-file error, exit 2 | e2e/rules.md |
| RULE-N9 | Whether every assertion key validates its value shape, or only numeric keys | Every key validates its value shape | e2e/rules.md |
| RULE-N11 | Whether a file with `doc:` and no `rules:` key is an error or a no-op | A rule-file error, exit 2 | e2e/rules.md |
| RULE-N12 | Whether a lone mapping under `rules:` is a one-rule shorthand | Rejected — `rules:` must be a list | e2e/rules.md |
| SEL-N15 | Message and anchor for an index over an empty scope without `optional:` | "selector matched no elements", at the scope's line | e2e/selectors.md |
| FM-2, FM-5 | Wording of the message for an exact-string value mismatch | "`status` is `rejected`, expected `approved`" | e2e/frontmatter.md |
| FM-N2 | Which line a required key that has no line of its own is reported at | Line 1, the block-level line | e2e/frontmatter.md |
| FM-N3 | Whether an absent block also emits a finding per required key | One block-level finding only | e2e/frontmatter.md |
| SEC-N3 | Anchor when a title-regexp selector matches no section | The selected scope's line — line 1 for a document-wide selector | e2e/sections.md |
| TXT-N13 | Whether a non-optional rule matching nothing is a finding or passes vacuously | An empty match set is itself a finding, at line 1 | e2e/lists-and-text.md |
| TBL-N2 | Whether an unnamed extra column is an error, and whether `strict:` governs it | `columns:` is exhaustive; the extra column is reported | e2e/tables.md |
| TBL-N6 | Whether `TD[col=…]` naming a column no table carries is an authoring mistake | Silence — it simply selects no cells | e2e/tables.md |
| LNK-N6 | What `resolve: true` does when `--root` is omitted | A usage error, exit 2 | e2e/links.md |
| SUB-1 | Whether `optional: true` excuses only an empty inner selector or the whole inner rule | Only SUB-1's stated finding is asserted; no claim on a second | e2e/nesting.md |
| SUB-N7 | Where an inner rule broken by a particular element is anchored | At the element's own line, as for a top-level rule | e2e/nesting.md |
| SUB-N9 | Whether a lead paragraph that exists but fails is a finding under `optional:` | It is a finding — `optional:` excuses only an empty selection | e2e/nesting.md |
| SUB-N10 | Whether an `each:` rule with zero outer matches needs `optional: true` | Vacuously satisfied | e2e/nesting.md |
| CLI-N9 | Whether an invocation naming no path is a usage error or falls back to stdin | A usage error, exit 2 | e2e/cli.md |
| CLI-N11 | Which condition triggers exit code 3 | None reachable from input; only the contract is fixed | e2e/cli.md |
