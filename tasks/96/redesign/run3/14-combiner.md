# 14 combiner: report (run 3, phase 3)

Power: `PowerLineStatus` printed `Online` at the start, after the first build, and at the end.
No compiler change, no git command, no Ember program run; nothing written outside this report,
`RUN3/combined.md`, `RUN3/vocabulary-combined.md` and `target/redesign3/14-combiner/`.

## What I wrote

| file | what is in it |
|---|---|
| `RUN3/combined.md` | the one memory model. Sections 0-9 (mine): terms; the model in plain words as 17 rules (R1 .. R17), what still runs while the program runs against C (with every number's source), what stays refused; the ruling on every new word; the 16 clashes and how each was settled; every rewrite made to an area's text; the re-checker's mismatches (none); the two completed items; the agreed rows 3-9 and where each sits; the error codes; five open points for the owner. Sections 10-14: all 84 answers in the eight-part form, grouped by area, each area's terms and rules first, copied byte for byte from the area reports with every change marked **14:** |
| `RUN3/vocabulary-combined.md` | the one vocabulary: 06's table with 43 rows rewritten (each keeps 06's "today" cell and says **14:** and who decided), 8 rows added (`thread_shared`, `weak`, `set by`, `@cached`, `guarded by`, `Lock`, `Commands.create()`, `from static`), the final spelling of every placeholder, and the words left open |
| `target/redesign3/14-combiner/build.py`, `parts/` | builds `combined.md`: applies the 35 changes as exact replacements that must each match once, and checks that AN-01 .. AN-84 are all there in order with their eight parts, and that every agreed spelling of rows 3-9 appears. Last run: `edits applied: 35; answers: 84; agreed spellings found: 11; **14:** marks: 34; bytes: 466608` |
| `target/redesign3/14-combiner/vocab.py` | builds `vocabulary-combined.md` from `vocabulary.md`. Last run: `rows rewritten: 43; rows added: 8; bytes: 80827` |

## The decisions, in short

**New words** (the four the areas brought): all **accepted**. `Commands.create()` (A, a library method);
`static` in `from static` (B, a new position of two existing words); `guarded by <lock>` (E); `Lock` (E).
None rejected, so no answer had to be rewritten for a word. 07's five planned words were used as planned
and are confirmed; `RuntimeChecked` (with `ReadGuard`, `WriteGuard`) and `@thread_shared` are removed, as
areas C and E ruled.

**Clashes settled** (`combined.md` section 3 has the reasons):
1. `x = <part>`: a read-only view of the part (A's rule, extended to fields; row 8's plain `path` rule),
   not C's partial hand-over; numbers, `bool` and `weak` links are copied; a stored part is handed over.
2. Element names are positions (A); every other view is an address that ends at a reshape (C6).
3. Kept `ref mut` views may name one object (C's AN-52 (b)); B's refusal lifted; parameters keep the
   "different places" promise, widened to a `ref mut` and any other view argument.
4. One first-line order (parameters, `reads`, `changes`, `->`, `from`) and one `cast` rule; `get_mut[T]`
   rewritten the same way in B, C and D; B's missing `dyn` added.
5. `RuntimeChecked` removed; B's and E's mentions rewritten.
6. `@cached` across threads: E's once-flag replaces B's refusal.
7. A plain name never changes anything, a `shared` link included: C's AN-51 takes `ref mut ws`.
8. E2105 for every first line that does not match its body; E3023 for a change through a plain name.
9. No word at the call site for a `ref mut` argument (row 3's own example).
10. One rule for keeping an object alive under a view (C7 + D5), with an atomic count added for a view of
    a `thread_shared` object reached through `weak` (a deferred-drop region is per thread).
11. A map key from a `StrView`: no word.
12. A-3 governs `for` loops; D's `while` dispatch by position may change the list.
13. Inferred facts at a package's edge: the written word where one exists, the interface file otherwise.
14. `@thread_shared` removed. 15. `class` has one owner (C), as B and E assumed. 16. AN-76's `worker`
    gains `changes RESULTS` (its own answer's rule 2).

**Completed**: AN-16 (the owner's test applied to every name in every line A's messages give) and AN-84
(the programs E's new codes lead to, in NEW spelling, with the test applied).

**Re-checker mismatches**: none to correct (agent 13 found none in 245 files).

**Agreed rows 3-9**: unchanged; each is listed with its place in `combined.md` section 7, and `build.py`
checks its spellings.

**No "means exactly" translation changed**: every rewrite either changes a refusal's text only, or gives
NEW code whose meaning the area's existing translation already shows (listed in `combined.md` section 4).

## Open for the owner (`combined.md` section 9)

A call-site word for a changed argument; `Box[T, A]` with an allocator; `=` viewing a part but handing a
whole name over; kept `ref mut` views of one object; AN-60's `while` dispatch changing its list.
