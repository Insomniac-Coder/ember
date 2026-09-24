# Owner decision queue

Questions an agent may not answer, in the format `SPEC-FEED-0.8.5-H1` §28 asks
for. **Nothing here is resolved silently.**

**This queue is not a second specification.** `docs/spec-source/ember-spec.md`
remains the sole normative language contract; an entry here describes a
*question*, and where it quotes the specification the specification governs.

Closed entries stay, with the authority that resolved them, because a closed
question is evidence about what kind of question this project generates — and
because a future agent who cannot find where a decision was made will reopen it.

*(Referred to as `OWNERQUEUE.md` in owner feedback; the file is
`docs/OWNER-QUEUE.md`.)*

---

## Queue summary

| ID | Status | Category | Priority | Semantic decision required |
|---|---|---|---|---|
| ODR-001 | **CLOSED** — retain the API exactly | Language / API | — | **No** — ruled 2026-09-10 |
| ODR-002 | **CLOSED** — `RIDX-1` landed | Tooling / document structure | — | **No** |
| ODR-003 | **DEFERRED EDITORIAL CLEANUP** — open for a future revision | Editorial / documentation | **P3** | **No** |
| ODR-004 | **CLOSED** — owner supplied `[LT-8]`–`[LT-13]` | Specification source recovery / library API | — | **No** |
| ODR-005 | **CLOSED** — explicit all-mutable `_mut` helpers | Language / standard-library API | — | **No** — ruled 2026-09-12 |
| ODR-006 | **CLOSED** — mutable helper inputs are `mut` reborrows | Language / callable API | — | **No** — ruled 2026-09-12 |
| ODR-007 | **CLOSED** — mode vector is compiler-known `Callable` metadata | Language / callable abstraction | — | **No** — ruled 2026-09-12 |
| ODR-008 | **CLOSED** — `Arena` is a narrow `@borrows` provenance source | Language / region provenance | — | **No** — ruled 2026-09-12 |
| ODR-009 | **CLOSED** — H10 defines Arena initialization completely | Language / unsafe initialization / API | — | **No** — ruled 2026-09-12 |
| ODR-010 | **CLOSED** — abort-only panic needs no observable Arena rollback | Language / Arena failure semantics | — | **No** — ruled 2026-09-13 |
| ODR-011 | **CLOSED** — complete fixed-capacity Arena collection contract | Language / standard-library API / regions | — | **No** — ruled by owner |
| ODR-012 | **CLOSED** — public hashing protocol and Map key boundary | Standard-library API / equality coherence | — | **No** — ruled 2026-09-13 |
| ODR-013 | **CLOSED** — static generic `H: Hasher` parameter | Language / callable-interface ABI | — | **No** — ruled 2026-09-13 |
| ODR-014 | **CLOSED** — exact Span iterator/chunk/raw-pointer contract | Standard-library API / views / unsafe boundary | — | **No** — ruled 2026-09-13 |
| ODR-015 | **CLOSED** — `@latebound` callable-type boundary | Language / lifetime callback API | — | **No** — ruled 2026-09-14 |
| ODR-016 | **CLOSED** — diagnostic identity for existing multi-region and callable-mode rejections | Diagnostics / conformance | — | **No** — incorporated in 0.9.7_Hardened_3 |
| ODR-017 | **CLOSED** — `Shared[T]` strong-owner and generalized `Weak[O]` surface | Standard-library API / ownership / borrowing | — | **No** — ruled 2026-09-20 |
| ODR-018 | **CLOSED** — explicit shared cycle-analysis root | CLI / static diagnostics / package resolution | — | **No** — ruled 2026-09-20 |
| ODR-019 | **CLOSED** — option 1; N1 cross-file ties use canonical identity | Diagnostics / name resolution | — | **No** — diagnostic/tooling hardening only |
| ODR-020 | **CLOSED** — resolved prefix, visible final-segment N1; qualified help, token-local edit | Diagnostics / name resolution | — | **No** — incorporated in 0.9.7_Hardened_4 |
| ODR-021 | **CLOSED** — float `//`/`%` are Python's (exact floor modulo, rounded once) | Language / floating point | — | Delegated for 0.9.9 — ruled 2026-09-23, 0.9.9_Hardened_3 |
| ODR-022 | **CLOSED** — `x = 0` is `int` at the declaration; later uses never change it | Language / type inference | — | Delegated for 0.9.9 — ruled 2026-09-23, 0.9.9_Hardened_3 |
| ODR-023 | **CLOSED** — a value function reaching its end is `E2182` | Diagnostics / functions | — | Delegated for 0.9.9 — ruled 2026-09-23, 0.9.9_Hardened_4 |
| ODR-024 | **CLOSED** — a borrowed or `mut` parameter that is not `Copy` is a source, passed by address | Language / regions / functions | — | Delegated for 0.9.9 — ruled 2026-09-24, 0.9.9_Hardened_5 |
| ODR-025 | **CLOSED** — `[ERR-4]`'s function-taking methods are eager, move the payload in, take `once fn`; a lambda infers `owned` | Standard library / closures | — | Delegated for 0.9.9 — ruled 2026-09-24, 0.9.9_Hardened_6 |
| ODR-026 | **CLOSED** — a type with its own `drop` is `Clone` only when it says so | Language / ownership | — | Delegated for 0.9.9 — ruled 2026-09-24, 0.9.9_Hardened_7 |
| ODR-027 | **CLOSED** — a range is a `Copy` value with public bounds; a `for` counts over a copy of them | Language / standard library | — | Delegated for 0.9.9 — ruled 2026-09-24, 0.9.9_Hardened_8 |
| ODR-028 | **CLOSED** — a method named like an inherited one replaces it: `E2111` without `override` over a virtual one, `E2110` over a non-virtual one; an `override` is virtual | Language / classes | — | Delegated for 0.9.9 — ruled 2026-09-24, 0.9.9_Hardened_9 |
| ODR-029 | **CLOSED** — `parse[T]()` is strict (Rust's grammar, no white space) and `ParseError` is `Empty`, `Invalid` or `Overflow` | Standard library / text | — | Delegated for 0.9.9 — ruled 2026-09-25, 0.9.9_Hardened_10 |
| ODR-030 | **CLOSED** — `extend` is a contextual keyword: a keyword only at the start of an item, before the type it extends | Language / lexical | — | Delegated for 0.9.9 — ruled 2026-09-25, 0.9.9_Hardened_11 |

**ODR-001 through ODR-003 were resolved by the owner on 2026-09-10.** ODR-002
is now fully closed because `RIDX-1` landed; ODR-003 remains deferred editorial
work and needs no semantic decision. ODR-004 was discovered during the 0.9.5
intake and closed when the owner supplied the missing definitions on 2026-09-12.
ODR-005 was then closed by the owner's explicit all-mutable helper ruling.

ODR-001, ODR-002, and ODR-004 through ODR-020 are closed; ODR-003 is deferred
editorial work, with no semantic impact. ODR-018 is
closed by the explicit shared cycle-analysis-root ruling; ODR-019 is closed
by the owner's option-1 N1 ranking ruling, incorporated into
0.9.8_Hardened_3. ODR-020's qualified N1 policy is incorporated in the new
0.9.7_Hardened_4 target, authored from immutable 0.9.7_Hardened_3; neither
predecessor nor the separate 0.9.8_Hardened_3 target was edited. H8 records the complete helper-mode and callable-abstraction
ruling; H9 records the Arena-backed return-provenance ruling; H10 records the
Arena allocation and initialization contract; 0.9.6_Hardened_1 records the
abort-only `[ARN-10]` clarification and simplicity consolidation; H2 records the
Arena-backed collection contract; and H3 records the public hashing contract.
H8/H9/H10/H1/H2/H3/H4 implementation remains incomplete as a whole, but its
Arena core, wrapper provenance, initialization, `[TST-23]`, generic-method,
Arena-collection, hashing, and UnsafeCell slices now have executable evidence.
H1's `@latebound` implementation and remaining conformance matrix are
implementation/conformance work, not owner questions. The current worktree
has verified the first `@latebound` slice, including nested scalar composition,
nested view-escape rejection, owned-capture publication rejection, and a
statically independent capture-free callback result through `Box[str]`;
sequential-invocation freshness, local callback-value escape, and all shared/
mutable helper arities are now directly covered; FFI and separate-compilation
evidence remain implementation work.
Other gaps are likewise implementation/conformance work.

Priorities: **P1** blocks a language or implementation decision · **P2** changes
no language semantics but affects conformance or tooling confidence · **P3**
editorial cleanup that can safely wait.

---

## ODR-018 — source/package selection for `ember explain --cycle` — **CLOSED**

    ID:        ODR-018
    Status:    CLOSED — owner-selected shared analysis-root resolution
    Category:  CLI / STATIC DIAGNOSTICS / PACKAGE RESOLUTION
    Priority:  P1
    Location:  Ember_v0.9.8_Hardened_2.md [CLI-17], [CLI-18], and [WK-9]

    Resolution: `ember explain --cycle <path> <Class[.field]>`; `<path>` uses
                the same package-directory or standalone-file resolution as
                `ember inspect --cycle <path>`.

    Blocks implementation:            NO — implementation is authorized
    Blocks conformance:               NO — canonical invocations are defined
    Blocks specification freeze:      NO — H2 records the tooling hardening
    Blocks normative specification adoption: implementation evidence still required
    Requires owner semantic decision: NO

**Owner ruling.** A valid path is exactly a package directory containing
`ember.toml`, resolved through its manifest and normal module/import closure,
or one standalone `.em` source file under the existing single-file rules. Both
cycle commands use this one resolver. They do not scan cwd, consult a stale or
unrelated build, infer a prior root, or select a package arbitrarily; an invalid
root fails before ownership analysis with no fallback. A module file is a
standalone root, never a third partial-package interpretation.

`Class`, `Class.field`, `module::Class`, and `module::Class.field` resolve only
inside that universe. Existing unknown-name/member diagnostics apply to missing
targets. An unqualified ambiguity fails and names its qualified candidates.
The ruling is preserved at
`docs/spec-source/as-received/ODR-018_Cycle_explanation_analysis_root.md` and
is recorded by ADR-038. It is CLI/tooling hardening only: no source-language,
ownership, lifetime, ABI, runtime, or graph-semantics decision changed.

---

## ODR-019 — N1 declaration proximity across source files — **CLOSED**

    ID:        ODR-019
    Status:    CLOSED — owner-selected option 1, canonical cross-file ordering
    Category:  DIAGNOSTICS / NAME RESOLUTION
    Priority:  —
    Location:  Ember_v0.9.8_Hardened_3.md §XX.6.2, N1 ranking algorithm

    Question:  How are N1 candidates ranked when declarations and the use
               span multiple source files?

    Blocks implementation:            NO — implementation is authorized
    Blocks conformance:                NO — deterministic ordering is specified
    Blocks specification freeze:       NO — incorporated into 0.9.8_Hardened_3
    Blocks normative specification adoption: implementation evidence still required
    Requires owner semantic decision:  NO — accepted/rejected programs do not change

**Existing wording and gap.** `[DIA-12]` requires N1 candidates to be ranked
first by Damerau–Levenshtein distance, then by declaration proximity. A span's
byte offset is file-relative, so the text did not define a meaningful
cross-file proximity comparison. This gap affects imported/aliased items,
inherited members, and prelude candidates when they participate in N1; it does
not affect source-language validity or the candidate set.

**Owner ruling — option 1 adopted.** Rank by spelling distance first. Only
when distances tie, rank same-file candidates before cross-file candidates.
For same-file candidates, retain the existing declaration-proximity calculation
unchanged and compare offsets only within the same source file. For cross-file
candidates, do not compute proximity; order by the canonical qualified
declaration identity using deterministic locale-independent lexicographic
ordering. Preserve N1's existing top-three limit and apply it after the full
ordering. Use visible spellings for display and only as a final fallback if
all specified keys tie.

Do not add a project-wide file/module distance model. Do not use source offsets
from unrelated files, import/alias locations, filesystem traversal, import
graph distance, hash iteration, file/source discovery order, operating-system
directory order, or build/cache order as ranking keys. Do not create separate
ranking mechanisms for imports, aliases, inherited members, or prelude
candidates. Candidate discovery and visibility remain unchanged; no diagnostic
code is added; no accepted program changes validity; no type, ownership,
lifetime, runtime, ABI, overload, import, or module semantics change.

**Implementation checklist.**

- [x] Keep the current candidate set, visibility, and discovery paths unchanged.
- [x] Preserve Damerau–Levenshtein as the primary key and the existing distance threshold.
- [x] Apply same-file-before-cross-file only after equal spelling distance.
- [x] Preserve same-file declaration-proximity behavior; never compare unrelated offsets.
- [x] Order cross-file ties by canonical qualified declaration identity, not alias/import position.
- [x] Apply the existing three-suggestion cap after complete ranking; add no diagnostic code.
- [x] Add ordered unit, conformance, and UI tests for local/cross-file ties, better cross-file spelling, import/alias, inherited/prelude candidates, top-three truncation, and reversed file/import discovery order.
- [x] Run the full workspace and repository CI-equivalent checks; all passed on 2026-09-23.

The owner-provided ruling is preserved byte-for-byte at
`docs/spec-source/as-received/ODR-019_N1_cross_file_suggestion_ranking.md`.
The supplied request named `_2`; the owner subsequently clarified that the
new artifact must be `_3` and that `_2` must not be edited. Accordingly, this
resolution is recorded in `Ember_v0.9.8_Hardened_3.md`, authored from immutable
immediate predecessor `_2`; `_2` remains untouched. The ODR changes only
diagnostic suggestion ordering and does not require a language-version bump.

---

## ODR-030 — how does `Array` have a method named `extend`? — **CLOSED**

    ID:        ODR-030
    Status:    CLOSED — ruled 2026-09-25 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_11
    Category:  LANGUAGE / LEXICAL
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_10.md Part II §4 (keyword table, `[LEX-15]`),
               `[STD-15]`, Appendix E

    Question:  Part II §4 reserves `extend` everywhere, but `[STD-15]` gives `Array`
               a method `extend(iterable)` and Appendix E maps Python's
               `xs.extend(ys)` to `xs.extend(ys)`. A reserved word cannot be a
               method name, so `xs.extend(ys)` does not parse; only
               `xs.r#extend(ys)` (`[LEX-14]`) would.

    Blocks implementation:            YES — `Array.extend` cannot be called
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**Options.**
- **(A) Rename the method.** Rejected: the owner's standing rule for a keyword
  that blocks a standard-library method name is to make the word contextual,
  never to rename the member; and Appendix E promises `xs.extend(ys)`.
- **(B) Write `xs.r#extend(ys)`.** Rejected under the same rule.
- **(C) Accept any keyword as a member name after `.`.** Rejected: it changes
  every keyword to settle one.
- **(D) Make `extend` contextual**, as ERR-017 did for `from` so that
  `interface From` can declare `from`.

**Ruling: (D).** `extend` leaves the keyword table (48 words) and joins
`[LEX-15]`'s contextual keywords: it is a keyword at the start of an item when
the type it extends follows, directly or after generic parameters
(`extend Point:`, `extend[T] Holder[T]:`), and an identifier everywhere else, so
a field, method or variable may be named `extend`.

**Implementation (2026-09-25).** The lexer no longer has `Kw::Extend`;
`"extend"` is in `CONTEXTUAL_KEYWORDS`. The parser's `at_extend_decl` decides:
`extend` followed by a name, or by a bracketed list whose closing `]` is
followed by a name, begins an `extend` block, so a script's `extend[0] = 1` and
`extend = 1` are still statements. Test: `LEX-15/accept_extend_is_contextual.em`.

---

## ODR-029 — what does `parse[T]()` accept, and what is a `ParseError`? — **CLOSED**

    ID:        ODR-029
    Status:    CLOSED — ruled 2026-09-25 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_10
    Category:  STANDARD LIBRARY / TEXT
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_9.md [TXT-10], §XV module table (`std.string`)

    Question:  `[TXT-10]` lists `parse[T]() -> Result[T, ParseError]` and the module
               table puts `ParseError` in `std.string`, but nothing says which `T`
               it reads, what text each accepts (white space? `+`? underscores?
               `inf`?), or what a `ParseError` is.

    Blocks implementation:            YES — `parse` cannot be written without it
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**Options.**
- **(A) Python's `int()`/`float()`:** white space around the number, `_` between
  digits. Rejected: `parse` is not a Python spelling (`int(s)` is, and the
  Python-habit table maps it to `s.parse[int]()`), and every example in the
  document trims before parsing (`value.trim().parse[int]()`), which only makes
  sense if `parse` does not.
- **(B) Strict, as Rust's `str::parse`:** the whole text, no white space.

**Ruling: (B).** `parse[T]()` reads an integer type, a float type, `bool` or
`char`, from the whole text: an integer is an optional sign (`-` only for a
signed type) and ASCII decimal digits, and must fit `T`; a float is an optional
sign and `inf`, `infinity` or `nan` in any case, or decimal digits with an
optional `.`, fraction and exponent, rounded to the nearest `T`; a `bool` is
`true` or `false`; a `char` is exactly one character. `ParseError` is a
unit-only enum in `std.string`, `Empty`, `Invalid` or `Overflow`, naming the
first problem from the left.

**Implementation (2026-09-25).** `std/src/string.em` declares `ParseError` and
is loaded with the prelude modules. The checker builds the `Result` from a
runtime status (`ember_parse_*_status`, strict validation) and reads the value
only when the status is 0 (`strtod`/`strtof` in the "C" locale for floats). Not
built: `i128`/`u128`. Tests: `TXT-10/accept_parse_is_strict.em`,
`TXT-10/reject_parse_needs_a_type_it_reads.em`.

---

## ODR-028 — what is a method that shares an inherited method's name? — **CLOSED**

    ID:        ODR-028
    Status:    CLOSED — ruled 2026-09-24 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_9
    Category:  LANGUAGE / CLASSES
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_8.md [CLS-4], §XVII.6 (the code registry)

    Question:  `[CLS-4]` says methods are non-virtual unless `virtual`, that
               "replacing one requires `override`", and that overriding a
               non-virtual method is `E2110`. It gives no code for a method that
               replaces an inherited virtual method without saying `override`,
               does not say whether a same-named method *without* `override`
               over a non-virtual method is that `E2110`, and does not say
               whether an `override` may itself be overridden further down.

    Blocks implementation:            YES — the check cannot be written without it
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**What the compiler did.** Only methods marked `override` were checked. A derived
`fn speak(self)` over an inherited `virtual fn speak` compiled and silently
hid it for calls through the derived type while the base's slot stayed in place,
and a grandchild's `override` of a parent's `override` was `E2110` (D-239).

**Options.**
- **(A) One code for every `[CLS-4]` violation (`E2110`).** Rejected: `E2110`
  says "override of a method that is not virtual", which is the wrong
  description of a missing `override`, and the fix differs (add a word, not
  change the base).
- **(B) A new code for the missing `override`,** `E2111`, as Kotlin and Swift
  treat it; `E2110` keeps its meaning and covers any method over a non-virtual
  one.

**Ruling: (B).** A method with a receiver named like an inherited one replaces
it: over a virtual method without `override` it is `E2111` (the fix-it adds
`override`), and over a non-virtual one it is `E2110`, `override` or not.
`init`, `drop` and associated functions are each class's own. An `override` is
itself virtual, so a further subclass may override it again.

**Implementation (2026-09-24).** `validate_class_methods` applies it to every
method in a class body; `extend` blocks keep their `override` check, with an
inherited `override` counting as virtual. Tests: `CLS-4/`.

---

## ODR-027 — what is a range value? — **CLOSED**

    ID:        ODR-027
    Status:    CLOSED — ruled 2026-09-24 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_8
    Category:  LANGUAGE / STANDARD LIBRARY
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_7.md [CTL-3], [STD-8], [STD-26], §V prelude table

    Question:  `[CTL-3]` names `Range`, `RangeInclusive` and `RangeFrom`, the prelude
               table adds `RangeTo`, `[STD-8]` makes `Range[T]` a `Contains` and
               `[STD-26]` gives a range a `len`. Nothing says what a range *value*
               is: whether its bounds can be read, whether it is `Copy`, and
               whether a `for` over it uses it up.

    Blocks implementation:            YES — `r = 0..3` cannot be built without it
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**Why it is visible.** `r = 0..4`, then `for i in r:` twice, then `s = r` and
`print(r.start)`. Under one reading every line compiles; under another the
second loop and the last line are use-after-move errors.

**Options.**
- **(A) Rust's model.** A range is its own iterator: not `Copy` (so advancing
  a copy is not a silent surprise), consumed by `for`, and `RangeInclusive`
  hides its bounds behind an "exhausted" flag. Rejected: `[CTL-3]` and
  `[CTL-3b]` already require a counted loop with no iterator object, so a
  `for` never advances the range; forbidding its reuse would be a move error
  with nothing moved.
- **(B) Python's model.** A range is an immutable value: re-iterable, as
  Python's `range` is (`[PHIL-14]`), with readable bounds.

**Ruling: (B).** The four range types are prelude structs with public bounds
(`start` and `end`; a `RangeFrom` has only `start`, a `RangeTo` only `end`),
`Copy` when the bound type is. A `for` over a range value counts over a copy of
its bounds and leaves the range as it was. `a..` has no end: counting up to its
type's maximum is an overflow (`[TYP-8]`), as producing the next value would be.
`..=b` and `..` name no prelude type, so as values they are `E1010`. Printing a
range uses the implicit `Debug` of `[STR-5]` (`Range(start=0, end=4)`).

**Implementation (2026-09-24).** `std/src/core.em` declares the four structs
with `@derive(Copy)` and the prelude exports them. The checker builds `a..b`,
`a..=b`, `a..` and `..b` as values (an expected range type gives the bound's
type, as `f(0..3)` for `r: Range[u8]`), counts a `for` over `a..`, over a range
value, and over `range(n)`/`range(a, b)` values, and gives range values `x in r`
/ `r.contains(x)` (`[STD-8]`) and `len(r)` / `r.len()` (`[STD-26]`, a count too
large for an `int` panics). Not built: a stepped `range(a, b, step)` as a value
(it is a `for` head), `len` of an `i128`/`u128` range, and the `Iterator`
methods on a range variable, which come with `[STD-19]`'s adapters. Tests:
`CTL-3/` (three files), `STD-8/accept_a_range_value_contains_by_two_comparisons.em`,
`STD-26/accept_range_is_a_value.em`, `STD-26/reject_len_of_an_unbounded_or_float_range.em`,
`STD-26/run_fail_len_of_a_range_too_long_for_an_int.em`.

---

## ODR-026 — is a type with its own `drop` implicitly `Clone`? — **CLOSED**

    ID:        ODR-026
    Status:    CLOSED — ruled 2026-09-24 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_7
    Category:  LANGUAGE / OWNERSHIP
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_6.md [STR-5], [OWN-8], [DRP-1]

    Question:  `[STR-5]` makes a struct or enum implement `Clone` field-wise,
               automatically, when every field implements it. A type with its own
               `drop` releases something when it dies. Does the implicit rule
               cover it?

    Blocks implementation:            YES — implicit `Clone` cannot be built without it
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**Reproducer.** `struct Buffer: p: *mut u8` with `fn drop(mut self)` freeing
`p` (in an `unsafe` block, as `[UNS-*]` allows). Every field is `Copy`, so the
literal rule makes `Buffer` `Clone`; `b.clone()` in Safe code copies `p`, and
both values free it: a double free with no `unsafe` at the call.

**Options.**
- **(A) The literal rule: every type whose fields are cloneable.** Rejected:
  Safe code reaches a double free, against `[BRW-9]` and `[UNS-4]`.
- **(B) No implicit `Clone` for a type with its own `drop`.** A destructor is
  the mark of a type that manages something its fields do not describe; its
  author writes `clone` (or `@derive(Clone)`, which asserts the field-wise copy
  is right).

**Ruling: (B).** `[STR-5]`'s implicit `Clone` does not apply to a struct or
enum that declares `drop`; `@derive(Clone)` or a written `clone` still gives it
one. A missing `clone` on such a type says so (`E1010`, with the help to write
`fn clone(self) -> T`). Implicit `Eq` and `Debug` are unaffected: neither
duplicates what `drop` releases.

**Implementation (2026-09-24).** Implicit `Clone` is built with this ruling:
every struct and enum without `@no_derive(Clone)`, a written `clone` or its own
`drop` gets a field-wise `clone` when all its fields are cloneable, generic
instances included (the derived body takes the declaration's span). An
implicit `clone` nothing calls is not emitted (`[COST-1]`:
`ember_mir::prune_unused_implicit`). Classes still need `@derive(Clone)`
(`[OWN-8]`). `@no_derive(Eq)` and `@no_derive(Debug)`
are `E0900` until implicit `Debug` exists and `Eq` can be opted out of. Tests:
`STR-5/accept_implicit_clone.em`, `STR-5/reject_no_derive_clone_opts_out.em`,
`STR-5/reject_implicit_clone_of_a_type_with_drop.em`, and `OWN-8`'s two reject
cases, retargeted at types with `drop`.

---

## ODR-025 — what do `[ERR-4]`'s methods that take a function accept? — **CLOSED**

    ID:        ODR-025
    Status:    CLOSED — ruled 2026-09-24 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_6
    Category:  STANDARD LIBRARY / CLOSURES
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_5.md [ERR-4], [CLO-7], [TYP-23], [CLO-3]

    Question:  `[ERR-4]` lists `map`, `map_err`, `and_then`, `or_else`,
               `unwrap_or_else`, `ok_or_else` and `filter` without signatures.
               Is the payload moved into the function or borrowed? Is the
               function a consumed parameter (`owned f: fn(…)`, whose call-site
               lambda captures by move, `[CLO-15]`), a `once fn`, or a plain
               `fn(…)` bound? And does a lambda written with no parameter mode
               take one from the expected callable type?

    Blocks implementation:            YES — the methods cannot be written without it
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**The two halves.** `[CLO-7]` names "`Option.map` returning a value computed
later" among APIs that store or send a callback, taking it `owned` or as
`once fn`, so a lambda at the call site captures by move (`[CLO-15]`). But
`[ERR-4]` lists `map` with `unwrap_or` and `expect`, which are eager, and no
Ember `Option.map` computes anything later. `[TYP-23]` says expected types flow
into lambdas; `[FN-6]` compares callable types by their parameter modes too;
`[FN-2]` makes an omitted mode borrowed. The corpus (`FN-6`, shape B15) rejects
`fn(x) => x` where `fn(mut i32) -> i32` is expected.

**Reproducer.** `name.map(fn(s) => s + suffix)`: under the consumed reading
`suffix` is moved into the closure and gone afterwards, for a call that
finishes before the next line. `opt.map(fn(s) => s)` needs `s` owned to hand
it on; with a borrowed payload it is `E3013`.

**Options.**
- **(A) Consumed parameter (`owned f: fn(owned T) -> U`), as `[CLO-7]` reads.**
  Rejected: an eager call gains nothing from moving captures, and loses the
  caller's variables.
- **(B) `f: once fn(owned T) -> U`.** The right shape: called at most once, so
  it accepts a closure that gives away a capture, and captures stay as the
  lambda infers them.
- **(C) `f: fn(owned T) -> U`, `[CLO-3]`'s bound.** Captures as inferred and
  zero cost; a closure that moves a capture out is refused (`E3030`).

**Ruling: (B) in the specification, with a borrowed payload for `filter`.**
- Each method consumes its receiver and calls its function at most once,
  before it returns. The payload is moved in (`owned T`, `owned E`), except
  `filter`'s, which it must give back: `filter(owned self, f: once fn(T) ->
  bool)`.
- `[CLO-7]`'s parenthesis loses "`Option.map` returning a value computed
  later": nothing in `[ERR-4]` stores its function.
- `[TYP-23]`: an omitted lambda parameter mode takes `owned` from the expected
  callable type, as the type does. `mut` is never inferred: a write to the
  caller's place is written where it happens, and an unwritten `mut` stays
  `E2228` (shape B15). Otherwise an omitted mode is borrowed (`[FN-2]`).
- `[ERR-4]` gains the signature table.

**Implementation (2026-09-24).** The methods are ordinary generic functions in
`std.core` (`option_map`, `result_map_err`, …), private to it; the type checker
routes `x.map(f)` to them, binding the receiver to a local no program can name,
so the callback is inferred, called and borrow-checked like any other
argument. `once fn` is not built yet (`[CLO-6]`'s parameter form), so the
helpers take `f: fn(…)`, option (C): a closure that moves a capture out is
`E3030` with a help naming the method. That difference is recorded in
`docs/DEVIATIONS.md`. Building the methods also needed three inference fixes,
each in the defect ledger: generic calls now read `Option[T]` and `Result[T, E]`
against their instances (D-229), a generic result nested in an enum no longer
blocks a lambda's body (`Option[U]`), and a ternary branch `None` or `[]` takes
the other branch's type (D-230).

Tests: `ERR-4/accept_methods_taking_a_function.em`,
`ERR-4/reject_map_consumes_its_receiver.em`,
`TYP-23/accept_owned_and_types_flow_into_a_lambda.em`, and `FN-6`'s B15 case,
unchanged.

---

## ODR-024 — may a returned view borrow a borrowed parameter that is not itself a view? — **CLOSED**

    ID:        ODR-024
    Status:    CLOSED — ruled 2026-09-24 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_5
    Category:  LANGUAGE / REGIONS / FUNCTIONS
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_4.md [LT-1], [LT-1a], [LT-1b], [LT-7], [LT-44],
               [FN-1], [FN-3], [FN-6], [BRW-8], [CORO-6], §VIII access table,
               §XVII.6 B7, §XVII.9 E2031

    Question:  `[LT-1]` rules 2 and 3 count only reference and view parameters as
               sources of a returned view. `[FN-1]`/`[FN-2]` make the default mode a
               borrow of the caller's value ("There is no by-copy mode"), and rule 1
               already makes any borrowed receiver a source. Can a borrowed `Array[T]`,
               `String` or `[T; N]`, or a struct or tuple holding one, be the source
               of a returned view?

    Blocks implementation:            YES — also decides how every borrowed parameter is passed
    Blocks conformance:                YES — decides SPN-1, B7, CELL-7, LT-1a and HEAP-5 corpus programs
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**The two halves.** `[LT-1]` rule 2 says: "if exactly one parameter is a reference or view, the result borrows from it". That reads as a test on the parameter's type. `[FN-1]` says: "The callee reads `a` … A `Copy` value no larger than two pointers is passed in registers; the meaning is the same". `[FN-2]` says: "There is no by-copy mode". Rule 1 says "a borrowed receiver (`self` or `mut self`)", with no restriction on its type. Milestone M2, which the spec required to "exist verbatim" from v0.6 through 0.9.8_Hardened_3, was `fn first(xs: Array[i32]) -> ref i32: return xs[0]  # ok: tied to xs`. The 0.9.9 audit closed F-160 as IMPL, which treats M2 as a valid program. So the default mode already stands for the caller's place, and the "view-typed parameters" wording of rule 2 (inherited from 0.6) was never reconciled with M2.

**Reproducer** (target/debug/ember.exe, 2026-09-24):

```ember
fn head(xs: Array[int]) -> Span[int]:
    return xs[..2]            # E3060 "`xs` is passed by value, so the copy's storage ends with the frame"

class Person:
    name: String
    fn name_ref(self) -> str:
        return self.name      # E3060 as well, although rule 1 makes `self` the source
```

The probes also exposed three live defects of the same by-copy convention:
- The spec's own IX.7 `Sprite`/`Scene` example prints `0` and `0`; it should print `2` and `100`.
- The `Scene` example with a four-element array and twenty `add` calls exits 0xC0000374 (STATUS_HEAP_CORRUPTION). The callee's `push` reallocates a buffer that the caller's header still points to.
- A `@derive(Copy)` struct holding a `Cell[int]` prints `0` after two `advance` calls. `[BRW-8]`'s "cannot be observed" is false for it.

A fourth defect goes the other way. The compiler counts every `mut` parameter as a source, so `fn next_token(mut pos: int, src: str) -> str` followed by `println(pos)` while the token lives is rejected with E3021. Under `[LT-1]` as written, that program is valid.

**Options.**
- **(A) Keep today's reading and improve the help.** Rejected. It contradicts rule 1, M2 and `[LT-44]`'s precedent, and it leaves the by-copy defects in place.
- **(B) Every borrowed parameter is a source.** Rejected. Ints and literal temporaries would join rule 3 (`take(xs, i + 1)` would fail at the caller), and every scalar would lose register passing.
- **(C) A parameter is a source only for views reached through its heap indirection.** Rejected. Whether a program is accepted would depend on layout (`Array[String]` yes, `[String; 4]` no, `ref p.name` no). It would be unsound under a future small-buffer optimisation. It is undefined for a generic `T`. And it does not fix the defects.
- **(D) A parameter is a source only as the receiver or when named in `@borrows`.** Sound, and no existing signature changes meaning. Rejected for four reasons:
  - it makes the most common Python shape need an annotation;
  - it contradicts M2;
  - it turns `@borrows` from a narrowing tool into a widening one, against `[LT-1a]`'s "needed only where rule 1 or 3 borrows more than the caller can afford";
  - it repeats the mistake F-070 undid for arenas.
- **(E) A borrowed or `mut` parameter whose type is not `Copy` is a source, and is passed by address.**

**Ruling: option E, refined.** A **source parameter** is either of these:
- a parameter whose type is a reference or view (as today);
- a borrowed or `mut` parameter whose type is not `Copy`, taking each type parameter to be `Copy`.

Rules 2 and 3 count source parameters. Rule 1 stays as written: a borrowed receiver of any type, including a `Copy` one, is the source. The spec's `Index.index(self, i) -> ref Output` therefore works for a `Copy` matrix over `[f64; 16]`. A source that is not a view is the caller's place, passed by address. The result may point into its own storage or into storage it owns, and the call's loan is on the argument, the same machinery `[SPN-1]` already uses.

Not sources:
- a borrowed or `mut` parameter of a `Copy` type. Such a value can be returned instead of viewed, and this keeps ints out of rule 3 and in registers;
- a plain `x: T`, and a callable parameter (`[CLO-3]`);
- an `owned` parameter that is not a view.

`@borrows` may name any source parameter, and also a `mut` parameter of a `Copy` type, which is already passed by address. Naming a borrowed `Copy` parameter, or an `owned` parameter that is not a view, is `E2031`.

`[BRW-8]` is restated around one idea: a borrowed parameter is passed by address. The only exceptions are `ref mut`/`MutSpan`, which are passed as themselves, and a `Copy` value that contains no `Cell` or `UnsafeCell` and is not the receiver of a view-returning method, which may be passed as a copy. So `[FN-1]`'s "the meaning is the same" becomes true. `restrict` is not emitted (`[LT-27]`).

The consequence is that `head(xs: Array[int])`, `name_of(s: String) -> str`, `name_of(p: Person) -> str` on a plain struct, `ref xs` and a `[String; 3]` element view all compile. The class getter compiles through rule 1, with its access carried to the caller by `[EXC-18]`. `fn first2(a: [int; 4]) -> Span[int]` stays `E3060`, with the help `a: Span[int]`.

The sources are fixed by the declared signature (types, modes, `@derive(Copy)`, `@borrows`) and never by the body. Callers, callable types (`[LT-7]`), `dyn` adapters and other packages therefore all agree. Hardened_5 carries the amended text. The four defects above are D-209 to D-212 in `docs/DEFECTS.md`, fixed by this change.

**Implementation (2026-09-24).** Built as ruled, with these differences:

- **Views pass as themselves.** A `@view` struct is passed as itself, not by
  address, so each field keeps its own region (`[LT-35]`). A view holds no
  `Cell`, so the copy cannot be observed. Hardened_5's `[BRW-8]` says so.
- **Class-handle receivers are left out.** A class handle is `Copy`, so it is
  not a source, and the receiver rule waits for `[EXC-18]`. The class getter
  above is still `E3060` (D-218, open).
- **`[TYP-5]` rule 7 was not built** (D-216). It is now, so the help "take `x`
  as a `ref` parameter" can add that callers keep writing the same call.
- **Diagnostics.** A view into an `owned` parameter is `E3060`, with the help
  "borrow `s` instead of taking it `owned`". A view into a parameter that is
  not a source is `E3062`. That help offers `@borrows` only when `@borrows` may
  name the parameter; otherwise it says to return the value. A view outliving a
  temporary argument is `E3060`, naming the temporary (D-214).
- **The source set** is computed once by the type checker from the declared
  signature (for an instantiation, the generic one). It is carried as
  `sources` on the MIR body, so every instantiation agrees (D-213). A method of
  a generic type is instantiated with its owner, so its non-receiver
  parameters are still read from the owner's instantiation.
- **A reference to a view parameter's own slot** (`return ref x`, `x: str`)
  used to escape (D-217, pre-existing). It is now `E3060`, as `[BRW-8]` says of
  a parameter passed as a copy.
- **The interface schema is version 8.** The compiler identity includes the
  executable's size and modification time, so no cache written under the old
  ABI is loaded (D-215).

Tests: `LT-1/` (thirteen cases), `LT-1a/accept_borrows_names_a_mut_copy_parameter.em`,
`LT-1a/reject_borrows_names_a_copy_struct_holding_a_cell.em`,
`LT-44/accept_one_arena_parameter_needs_no_borrows.em`,
`FN-1/accept_cell_and_refcell_through_borrowed_parameters.em`,
`BRW-8/accept_a_copy_struct_holding_a_cell_is_passed_by_address.em`, and the
`CELL-7`, `SPN-1`, `HEAP-5` and B7 cases this ruling changed.

---

## ODR-023 — which diagnostic reports a function that can reach its end without a value? — **CLOSED**

    ID:        ODR-023
    Status:    CLOSED — ruled 2026-09-23 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_4
    Category:  DIAGNOSTICS / FUNCTIONS
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_3.md [FN-10], §XVII.9 registry

    Question:  `[FN-10]` gives `void` and `Result[void, E]` functions an implicit
               value at the end of the body and says "No other return type has
               an implicit value". A body that can reach its end therefore has
               no meaning, but no rule names the diagnostic, and the registry
               has no code for it.

    Blocks implementation:            YES — the compiler must reject with some code
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**Reproducer.** `fn sign(x: int) -> int: if x > 0: return 1` — the compiler
accepted it and returned an uninitialised value (D-186).

**Options.** (1) Report it under an existing code, such as `E2020` (a type
mismatch between `void` and `int`). (2) A dedicated code.

**Ruling: option 2, `E2182`** — "a function that returns a value can reach the
end of its body", citing `[FN-10]`, shown at the function's name, with the help
to return on every path or end with `panic(…)`. It is not a type mismatch the
programmer wrote, and a dedicated code gets its own explanation and error page.
Hardened_4 adds the sentence to `[FN-10]` and the row to §XVII.9.

## ODR-022 — does a later use change the type of `x = 0`? — **CLOSED**

    ID:        ODR-022
    Status:    CLOSED — ruled 2026-09-23 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_3
    Category:  LANGUAGE / TYPE INFERENCE
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_2.md [LEX-16], [LEX-17], [TYP-23]

    Question:  An unannotated local initialised by an untyped literal
               (`total = 0`) — is its type fixed at the declaration (`int`), or
               left open and fixed by later uses (`return total` in an `i32`
               function)?

    Blocks implementation:            YES until ruled — every literal-initialised local
    Blocks conformance:                YES — decides which corpus programs are valid
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**The two halves.** `[LEX-16]`: an untyped integer literal "takes the type its
context expects …, and with no context it is `int` (`i64`)". `[TYP-23]`: "A
local declared by `x = e` takes `e`'s type; a type left open by `e` (`[]`,
`Map()`, `None`) is fixed by later uses in the same function … Untyped literals
are resolved last." Read together, `x = 0` either has no context (so `int`), or
has an untyped type left open (so later uses decide). Both readings are
plausible; the listed examples of "left open" do not include a literal, and
"resolved last" does not say what a literal's context is.

**Reproducer.**

```ember
fn sum_to(n: i32) -> i32:
    total = 0
    i = 1
    while i <= n:
        total += i
        i += 1
    return total
```

Option 1 accepts only if later uses decide; option 2 rejects `i <= n` and
`return total` with `E2020`.

**Options.** (1) Later uses decide, with `int` only when nothing constrains the
local (Rust's rule). (2) The declaration decides: a literal's context is the
expected type at the literal itself, an unannotated declaration supplies none,
so `total` is `int`.

**Ruling: option 2.** Option 1 is action at a distance: a use far down a
function would change an earlier variable's width, and with it where its
arithmetic overflows (`[TYP-8]`) and whether a large literal at the declaration
fits at all. A Python programmer reads `total = 0` as an `int`; option 2 makes
that true everywhere and makes a declaration say its type without reading the
rest of the function. `[]`, `Map()` and `None` stay open because they have no
default. The cost is an annotation (`total: i32 = 0`) in code that works in a
narrower type, and the `E2020` at the later use carries a help naming the
declaration and that annotation. Hardened_3 states the rule in `[TYP-23]`.

## ODR-021 — float `//` and `%`: the formula in IEEE arithmetic, or Python's result? — **CLOSED**

    ID:        ODR-021
    Status:    CLOSED — ruled 2026-09-23 under the owner's delegation for 0.9.9;
               incorporated in 0.9.9_Hardened_3
    Category:  LANGUAGE / FLOATING-POINT SEMANTICS
    Priority:  —
    Location:  Ember_v0.9.9_Hardened_2.md [TYP-29], [PHIL-14], [TYP-9]

    Question:  `[TYP-29]` defines float `a // b` as `floor(a / b)` and `a % b` as
               `a - b * floor(a / b)` "(the sign of `b`, Python)". Evaluated in
               IEEE arithmetic, the formula and Python disagree. Which governs?

    Blocks implementation:            YES until ruled — the runtime needs one algorithm
    Blocks conformance:                YES — the printed results differ
    Requires owner semantic decision:  delegated to the agent for 0.9.9 (owner, 2026-09-23)

**The two halves.** The formula, evaluated with IEEE rounding, gives
`1.0 // 0.1 == 10.0` and `1.0 % 0.1 == 1.0 - 0.1 * 10.0`, about `-5.55e-17`
(the sign of neither operand). Python gives `9.0` and `0.09999999999999995`,
because `0.1` is slightly more than a tenth, so ten of them exceed `1.0`.
`[PHIL-14]` (a Python spelling has Python's meaning) and the parenthesis name
Python; the formula names a computation.

**Options.** (1) The formula as written, in IEEE arithmetic. (2) The exact
floor modulo rounded once — computed exactly with `fmod`, then moved into the
divisor's sign — and the floor quotient consistent with it, which is what
Python computes.

**Ruling: option 2.** Option 1 breaks the sign promise the same sentence makes
and loses `a == (a // b) * b + a % b` far beyond rounding. Option 2 is
Python's, keeps the identity to within one rounding, and costs a few operations
more than a bare `floor`. Zero divisors, infinities and NaN follow IEEE through
`fmod` and the division, as `[TYP-29]` already says. Hardened_3 rewrites
`[TYP-29]` to state the exact definition. Implemented in the runtime as
`ember_floordiv_f32/f64` and `ember_floorrem_f32/f64`.

## ODR-020 — N1 suggestions for namespace-qualified paths — **CLOSED**

    ID:        ODR-020
    Status:    CLOSED — owner ruling incorporated in 0.9.7_Hardened_4
    Category:  DIAGNOSTICS / NAME RESOLUTION
    Priority:  —
    Location:  Ember_v0.9.8_Hardened_3.md [MOD-2], [MOD-3], §III path grammar,
               and §XX.6.2 [DIA-12] N1; resolved in Ember_v0.9.7_Hardened_4

    Question:  For a qualified path whose namespace resolves but whose final
               item does not, what is the N1 candidate set, visible spelling,
               and edit span?

    Blocks implementation:            NO — owner ruling is explicit
    Blocks conformance:                NO — qualified N1 output is defined
    Blocks specification freeze:       NO — incorporated in new H4; H3 stays immutable
    Requires owner semantic decision:  NO — accepted-program validity is unchanged

**Existing wording and gap.** The grammar permits a qualified path of the form
`identifier "::" identifier {"::" identifier}`. `[MOD-3]` says importing a
module binds a namespace, while `[MOD-2]` defines which of its items are
visible. `[DIA-12]` requires N1 to suggest an in-scope item within its distance
threshold and applies the ranking algorithm to ordinary imports and aliases.
It does not say whether `N1` compares only the unresolved terminal component
or the whole path, which namespace items form the candidate set (including
visibility/re-exports), whether a suggestion is displayed qualified or
unqualified, or whether the fix replaces the terminal identifier or the full
path. A misspelled namespace prefix and its relationship to N2 are also not
covered by this entry and must not be silently folded into this policy.

**Minimal reproducer and current behavior.** With `support.io` exporting
`pub fn print(value: i32) -> i32`, compile:

```ember
import support.io as io

fn main():
    io::pritn(1)
```

The current compiler reports:

```text
error[E1010]: cannot find `support.io.pritn` in this scope
```

It underlines `io` and provides no N1 help. `resolve_qualified` has
already resolved the namespace; the direct-call unknown-name path only adds
suggestions for one-segment names, uses the first segment's span, and displays
the canonical internal name. This was reproduced against the working tree on
2026-09-23. It is a diagnostics implementation gap, not a language-semantic
decision.

**Options considered before the ruling.**

1. Compare the unresolved terminal identifier against only items accessible
   through the resolved namespace under `[MOD-2]`. Display the item spelling
   (`print`) and replace only the final identifier; the already-written
   namespace alias supplies context. Apply the existing N1 ordering to the
   candidates using canonical declaration identity for cross-file ties.
2. Use the same terminal-segment candidates, but display the visible qualified
   path (for example, `io::print`) while still replacing only the terminal
   identifier. This makes the namespace context explicit without rewriting an
   alias.
3. Rank and display whole qualified paths, replacing the entire path. This
   would need a further rule for aliases versus canonical module paths and for
   the edit range when only one segment is wrong.

**Owner ruling — adopted.** The owner selected final-segment comparison,
resolver-visible candidates constrained by `[MOD-2]`, full source-qualified
help spelling, and a machine-applicable edit limited to the unresolved final
identifier. Namespace prefixes and aliases stay exactly as written. A failed
prefix is diagnosed through ordinary path resolution and does not trigger
final-member search. Qualified N1 reuses the existing threshold, ranking and
top-three limit; it adds no diagnostic code or independent ranking system.
ODR-019's cross-file tie policy is retained in source revisions that already
include it; this H4 is based on 0.9.7_Hardened_3 and does not redefine that
separate policy. No accepted/rejected program set, module/import semantics,
visibility semantics, overload resolution, type checking, ownership/lifetime,
runtime or ABI behavior changes.

The ruling is archived at
`docs/spec-source/as-received/ODR-020_qualified_path_N1.md`. The normative rule
is in `docs/spec-source/Ember_v0.9.7_Hardened_4.md` XX.6.2; immutable
0.9.7_Hardened_3 and 0.9.8_Hardened_3 remain unchanged.

**Conformance and implementation mapping.**

- Qualified alias, public candidate, alias preservation and corrected source:
  `DIA-12/reject_qualified_alias_typo.em` and
  `DIA-12/accept_qualified_alias_correction.em`.
- Nested path spelling: `DIA-12/reject_qualified_nested_typo.em`.
- Private perfect-match exclusion: `DIA-12/reject_qualified_private_name_is_not_suggested.em`.
- `pub(package)` in the same package:
  `DIA-12/reject_qualified_package_item_is_visible_inside_package.em`; an
  isolated `std` package UI case verifies it is not suggested cross-package.
- Existing ranking and top-three cap:
  `DIA-12/reject_qualified_top_three_visible_candidates.em`.
- Prefix failure and no final search:
  `DIA-12/reject_qualified_prefix_typo_is_not_searched.em`.
- No nearby member candidate:
  `DIA-12/reject_qualified_name_without_candidate.em`.
- Direct `from` import continues through unqualified N1:
  `DIA-12/reject_n1_alias_uses_canonical_cross_file_order.em`.
- `compiler/ember_driver/tests/ui.rs` verifies final-token caret/edit ranges,
  fixed alias source, stable repeated ordering, and cross-package `pub(package)`.

The machine-readable diagnostic continues to use this compiler revision's
existing `E1010` unknown-name code. No `E1060` code is created; the code in the
owner ruling's illustrative example is not a request to add a diagnostic
identity.

---

## ODR-017 — `Shared[T]` strong-owner and generalized weak-owner surface — **CLOSED**

    ID:        ODR-017
    Status:    CLOSED — owner-approved `Shared[T]` / `Weak[O]` API
    Category:  STANDARD-LIBRARY API / OWNERSHIP / BORROWING
    Priority:  —
    Location:  Ember_v0.9.8_Hardened_1.md IX.1, [GRM-3], [TYP-15], [DRP-6],
               [RC-5], [RC-2e], and Part VIII [OBJ-3], [WK-1]–[WK-3],
               [WK-11]–[WK-14]

    Question:  What is the complete public construction, borrowing, mutation,
               and weak-reference API for `Shared[T]`?

    Blocks implementation:            NO — implementation is authorized
    Blocks conformance:               NO — `[TST-26]` evidence is authorized
    Blocks specification freeze:      NO — 0.9.8_Hardened_1 is frozen
    Blocks normative specification adoption: implementation evidence still required
    Requires owner semantic decision: NO

**Existing wording.** IX.1 calls `Shared[T]` a "reference-counted heap `T`"
with the same header as classes, says it is Copy by retain, names `s.get()`,
and says mutation follows Part VIII §3. It also says it has `Weak[T]`.
`[GRM-3]` makes both names ordinary library types, `[DRP-6]` requires a
`Shared[T]` drop to release, and `[TYP-15]` constrains what may be stored in its
contents. Part VIII completely defines a different, class-only `Weak[C]`
surface: `Weak(h)`, `Weak[C].empty()`, and `upgrade() -> Option[C]`.

**Gap.** The target never supplies a construction spelling for `Shared[T]`, the
return type and receiver mode of `get()`, or any operation, construction, empty
value, upgrade result, or relationship to `Weak[C]` for `Weak[T]`. Therefore an
implementation would have to choose whether construction is `Shared(value)` or
an associated function; whether `get()` returns a shared or mutable borrow and
how it enters dynamic exclusivity; and whether `Weak[T]` is a companion for
`Shared[T]`, an alias for a `Weak[Shared[T]]` form, or something else. Each
choice changes accepted programs and memory/borrowing behavior.

**Why this cannot be resolved safely by an agent.** Reusing the class-only
`Weak[C]` behavior would reject or reinterpret the explicitly named `Weak[T]`
surface. Inventing constructors, borrow modes, or a wrapper representation
would create public language-library API and decide the exclusivity boundary,
not merely repair a compiler defect. No existing rule selects among those
choices.

**Requested owner resolution.** Supply the exact public declarations and
semantics for `Shared[T]` construction, `get()` receiver/result modes and
mutation access, copying and destruction, the `Weak[T]` constructor/empty/
upgrade/drop behavior, payload eligibility and drop ordering, and the explicit
non-interoperation rule with class `Weak[C]` and C++ bridge types. The ruling
should identify the corresponding conformance cases and use the existing object
header, ARC, and dynamic-exclusivity mechanisms where appropriate.

**Owner resolution — 2026-09-20.** `Shared(value: T) -> Shared[T]` constructs
one strong owner. `get()` returns `ref T`; `get_mut(mut self)` returns `ref mut
T`, using ordinary static borrowing and the existing dynamic-exclusivity
mechanism rather than unique ownership or a `Shared`-specific aliasing model.
`Shared[T]` is Copy by strong retain and releases its payload at final strong
release.

`Weak[O]` means the weak form of the counted owner `O`. The v1 owner forms are
a class handle `C` and `Shared[T]`; therefore class weak handles remain
`Weak[C]` and the companion of `Shared[T]` is `Weak[Shared[T]]`. `Weak(owner)`,
`Weak[O].empty()`, Copy-by-weak-retain, weak drop, and
`upgrade() -> Option[O]` reuse the existing weak-count/deinitialization model.
The two Ember weak-owner forms do not interconvert with each other or with C++
bridge owners. The ruling is preserved verbatim at
`docs/spec-source/as-received/ODR-017_Shared_Weak_API_completion.md`; ADR-037
and `0.9.8_Hardened_1` are the authority chain. This is an owner-approved
semantic change, so it begins language version 0.9.8 rather than a 0.9.7
hardening.

---

## ODR-016 — diagnostic identity for existing rejections — **CLOSED**

`0.9.7_Hardened_3` H3.3 assigns existing multi-region provenance rejections to
`E3065`/B14 and callable parameter-mode mismatches to `E2228`/B15. This changes
diagnostic and conformance identity only: the underlying acceptance rules were
already defined. The target records ODR-016 as closed; it does not define any
`Shared[T]` or `Weak[T]` semantics.

---

## ODR-015 — `[LT-7]` late-bound callback-region declaration mechanism — **CLOSED**

    ID:        ODR-015
    Status:    CLOSED — owner-approved `@latebound` callable-type boundary
    Category:  LANGUAGE / LIFETIME CALLBACK API
    Priority:  —
    Location:  Ember_v0.9.7_Hardened_1.md [FN-6b], [LT-7], [LT-8]–[LT-10], [TST-16], [TST-21]

    Existing wording: "A callback-taking API MAY expose a callback boundary whose
                      borrow region is chosen by the callee for each invocation."
                      `with_views` is required to use that boundary and reject
                      any callback-local view that escapes.

    Historical conflict: H6 defined the required lifetime behavior but did
              not define a source declaration, signature annotation, interface
              artifact field, or other general marker by which a library API
              says that one of its callback parameters is late-bound. Plain
              `fn(...)` means an ordinary callable boundary today. Treating all
              callbacks as late-bound would change unrelated APIs; recognizing
              only `std.borrow.with_views*` by compiler name would leave [LT-7]
              non-general and conflicts with the helpers being ordinary library
              APIs rather than compiler-special ownership types.

    Resolution:                       `@latebound fn(...) -> R` is the one general callable-type
                                      modifier. It binds fresh invocation-local regions for each
                                      borrowed/view parameter; results and publication paths must
                                      be free of those regions.
    Authority:                        Owner ruling, 2026-09-14; 0.9.7_Hardened_1; ADR-036.
    Semantic impact:                  owner-approved language revision
    Blocks implementation:            NO — implementation is now authorized
    Blocks conformance:               NO — conformance work is now authorized
    Blocks specification freeze:      NO — H1 is frozen
    Blocks normative specification adoption: implementation evidence still required
    Requires owner semantic decision: NO

**Historical minimal reproduction.** The ordinary generic
`std.borrow.with_views2` wrapper can forward views and preserve callable modes.
Before the current implementation it accepted the following program even
though `[LT-10]` requires rejection:

```ember
from std.borrow import with_views2

fn first(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a

fn main():
    a: Array[i32] = Array[i32]()
    b: Array[i32] = Array[i32]()
    a.push(1)
    b.push(2)
    escaped: Span[i32] = with_views2(a.as_span(), b.as_span(), first)
```

The current compiler rejects this case with `E3062`, and the executable
conformance case remains as regression evidence. The earlier acceptance was an
implementation/adoption gap, not evidence that `[LT-10]` should be weakened.

**Historical alternatives rejected by the owner.**

1. The selected approach is the deliberately small general modifier
   `@latebound` on a callable type. It is source-visible only as the boundary
   marker; region identities remain compiler-internal, and the fact travels in
   compile-time type/EMIF identity rather than runtime or ABI data.
2. Give only the canonical `std.borrow.with_views*` names this meaning in the
   compiler. This is narrow but risks making an ordinary library API
   compiler-special and leaves general `[LT-7]` APIs unexpressible.
3. Infer late-bound regions for every callback-taking API. This is not
   recommended: it silently changes unrelated callback APIs and accepted
   programs.
4. Supply another explicit owner-selected mechanism consistent with `[LT-7]`.

**Resolution evidence.** The owner explicitly rejected both name-specific
compiler behavior and global inference. The decision adds one reusable
callable-boundary modifier rather than named lifetime syntax. It is a language
revision because it changes callback-return and escape acceptance; H6 remains
immutable and H1 is the correct successor. The minimal reproducer below remains
the first required negative conformance case, now against the H1 contract.

---

## ODR-014 — remaining `Span` / `MutSpan` method surface — **CLOSED**

    ID:        ODR-014
    Status:    CLOSED — resolved by the owner and incorporated in 0.9.6_Hardened_6
    Category:  STANDARD-LIBRARY API / VIEWS / UNSAFE BOUNDARY
    Priority:  —
    Location:  Ember_v0.9.6_Hardened_5.md IV.4, IV.8, VI.4 [CTL-1]/[CTL-3b],
               VII.7 [SPN-1]–[SPN-3], IX.2, and [UNS-1]

    Existing wording: "Span[T] ... Bounds-checked indexing; `.len()`, `.iter()`,
                      `.iter_mut()`, `.split_at(i)`, `.chunks(n)`, `.as_ptr()`
                      (unsafe result)."

    Conflict: The document names operations but does not define enough of the
              public contract to implement them without selecting semantics.
              `[CTL-3b]` also names `chunks_mut`, although IV.4 lists only
              `chunks`. The existing Iterator interface requires a concrete
              associated iterator type, but no ordinary Span iterator/chunk
              types or exact `Item` relationships are named. `as_ptr()` does
              not say whether calling it is unsafe, whether only dereference is
              unsafe, or which shared/mutable pointer forms a MutSpan exposes.

    Semantic impact:                  RESOLVED by owner — API identity, borrowing, failure,
                                      pointer authority, and public module boundary are fixed
    Blocks implementation:            NO
    Blocks conformance:               NO
    Blocks specification freeze:      NO
    Blocks normative specification adoption: NO — implementation/evidence gates still apply
    Requires owner semantic decision: NO

**Questions that required the ruling.**

1. What named concrete iterator types and signatures implement `iter` and
   `iter_mut`, and which receivers are shared, mutable-reborrowed, or consumed?
2. Does the canonical chunk surface include both `chunks` and `chunks_mut`?
   What are their named result/iterator types and `Iterator.Item` types?
3. What happens when a chunk size is zero: a required panic in every profile,
   a fallible result, an empty iterator, or another already-defined behavior?
4. Is `as_ptr()` itself safe and merely returns a raw pointer whose use is
   unsafe under `[UNS-1]`, or must the call occur in `unsafe`? For `MutSpan`,
   does the public surface include shared `as_ptr() -> *T`, mutable
   `as_mut_ptr(...) -> *mut T`, or a different exact pair?
5. Which module publicly owns the concrete iterator/chunk types, and are any of
   them prelude names? No new opaque-return syntax currently exists.

**Resolution.** The owner selected named public, non-prelude `@view` types
`SpanIter`, `MutSpanIter`, `SpanChunks`, and `MutSpanChunks` under
`std.collections`, using the existing associated-type `Iterator` interface.
Shared forms yield `ref T`/`Span[T]`; mutable forms reborrow rather than consume
and yield `ref mut T`/disjoint `MutSpan[T]`. Zero-sized chunks panic through the
existing failure path in every profile. `as_ptr`/`as_mut_ptr` extraction is
safe, source lifetimes are not extended, and all pointer use remains subject to
the existing unsafe contract. The supplied `*const T` spelling is normalized
to Ember's already-canonical shared raw pointer `*T`, without changing the
selected authority. No opaque return or new ownership/lifetime mechanism was
introduced.

    Resolution: Owner-approved Span/MutSpan API completion
    Authority:  Owner ruling preserved as ODR-014_Span_MutSpan_API_completion.md
    Revision:   Ember 0.9.6_Hardened_6
    ADR:        ADR-035
    Result:     Closed; [SPN-4]–[SPN-10] and [TST-25] are authoritative for H6;
                implementation and mutation-sensitive evidence landed in d077563

**Historical alternatives considered before the ruling.**

1. **Recommended conventional paired API:** use named compiler/library `@view`
   iterators implementing the existing associated-type `Iterator`; shared
   iteration/chunking yields `ref T`/`Span[T]`, mutable reborrowed forms yield
   `ref mut T`/`MutSpan[T]`; provide `chunks_mut` because `[CTL-3b]` already
   names it; define an explicit all-profile zero-size failure; and make pointer
   extraction safe while raw-pointer operations remain unsafe. The owner must
   still approve the exact names, modes, signatures, failure identity, module,
   and prelude status.
2. Restrict v1 to a smaller surface and amend every normative example and
   `[CTL-3b]` reference consistently. This changes the advertised API and may
   invalidate existing accepted source expectations, so it cannot be selected
   as editorial cleanup.
3. Introduce opaque iterator return types. Ember currently has no such public
   type mechanism; this would be a larger language feature and is not
   recommended merely to finish Span.

**Why this could not be resolved safely by an agent.** The alternatives determine
which programs type-check, when a mutable view is reborrowed or consumed,
whether zero is a recoverable/error/abort boundary, what pointer authority safe
code can obtain, and whether a new public type or return abstraction exists.
Those are language/library safety and accepted-program decisions. Following the
project's hardening rule, implementation stopped until the owner resolution was
incorporated in `0.9.6_Hardened_6`.

---

## ODR-013 — `Hasher` interface parameter representation — **CLOSED**

    ID:        ODR-013
    Status:    CLOSED — resolved by the owner and incorporated in 0.9.6_Hardened_4
    Category:  LANGUAGE / CALLABLE-INTERFACE ABI
    Priority:  —
    Location:  Ember_v0.9.6_Hardened_3.md [HASH-1] with [TYP-22], [FN-1], [TYP-17]

    Existing wording: "fn hash(self, mut h: Hasher)"

    Conflict: A concrete DefaultHasher must be passed to Hash.hash, but [TYP-22]
              makes `dyn I` unsized behind explicit indirection and permits bare
              interface `I` only in class-handle position. DefaultHasher is not
              specified as a class handle. The signature therefore does not
              identify a legal concrete-to-interface representation.

    Possible interpretations:
      A. Static protocol: `fn hash[H: Hasher](self, mut h: H)`.
      B. Dynamic protocol: explicitly pass a mutable `dyn Hasher` view using
         the exact indirection/mode spelling selected by the owner.
      C. Another owner-specified representation that fits existing interface
         and receiver-mode rules.

    Semantic impact:                  YES — accepted implementations, dispatch, ABI, monomorphisation
    Blocks implementation:            NO — the generic contract is executable
    Blocks conformance:               NO — the required static-dispatch evidence is defined
    Blocks specification freeze:      NO — H3 remains an immutable finding record
    Blocks normative specification adoption: NO — implementation/adoption gates remain
    Requires owner semantic decision: NO

    Resolution: Static generic `fn hash[H: Hasher](self, mut h: H)`; concrete
                hasher inference and ordinary monomorphization; DefaultHasher
                implements Hasher; no mandatory dynamic dispatch
    Authority:  Owner ODR-013 ruling supplied 2026-09-13; ADR-033; HC-096-04
    Revision:   Ember 0.9.6_Hardened_4
    Result:     Closed; H4 [HASH-1] is authoritative within the development target

    Recommended option: A — generic `H: Hasher`. It permits a concrete
                        move-only struct, uses existing generic/interface
                        machinery, keeps dispatch static, and introduces no
                        new unsized mutable-interface parameter ABI.

**Why this is not an implementation detail.** Choosing static generic dispatch
changes the public method signature and monomorphization obligations. Choosing
dynamic dispatch changes the source spelling, object representation, call ABI,
and possibly which implementations are dyn-compatible. A compiler-only
coercion from a struct to bare `Hasher` would create a third interface model
contrary to `[TYP-22]`.

**Owner answer.** The owner selected option A exactly: the hasher parameter is
statically generic, `H` is inferred from the concrete argument and normally
monomorphized, and `DefaultHasher implements Hasher`. `Hash.hash` itself is not
a dynamic API; a future explicit `dyn Hasher` API remains a separate decision.

**Historical stop.** The agent stopped before baking an unapproved dispatch/ABI
choice into HIR, MIR, code generation, and conformance. The owner ruling now
closes that stop. H4 normalizes the supplied diff's angle brackets around the
ordinary parameter list to Ember's existing parenthesized grammar; this does
not alter the selected generic semantics. ERR-054 records the same chain.

---

## ODR-012 — `Hasher` public API and Map key invariant — **CLOSED**

    ID:        ODR-012
    Status:    CLOSED — resolved by the owner and incorporated in 0.9.6_Hardened_3
    Category:  STANDARD-LIBRARY API / EQUALITY COHERENCE
    Priority:  —
    Location:  Ember_v0.9.6_Hardened_2.md interface Hash, [ARN-5a], [ARN-5d], [TST-24]

    Existing wording: "interface Hash: fn hash(self, mut h: Hasher)"

    Semantic impact:                  YES — public protocol, custom-key acceptance, Map invariant
    Blocks implementation:            NO — the complete contract is now executable
    Blocks conformance:               NO — H3 extends [TST-24]'s required matrix
    Blocks specification freeze:      NO — H2 remains immutable
    Blocks normative specification adoption: NO — implementation/adoption gates remain
    Requires owner semantic decision: NO

    Resolution: Public Hash/Hasher protocol, concrete DefaultHasher, Eq/hash coherence,
                implementation-defined mixing, and read-only resident keys
    Authority:  Owner ODR-012 ruling supplied 2026-09-13; ADR-032; HC-096-03
    Revision:   Ember 0.9.6_Hardened_3
    Result:     Closed; [HASH-1]–[HASH-4] are authoritative within H3

**Conflict discovered during implementation.** H2 required `K: Eq + Hash`
for both ordinary and Arena-backed Maps, and inherited a `Hash` interface whose
method accepted a `Hasher`. The complete specification lineage defined no
`Hasher` type, methods, construction/finalization behavior, standard-module
owner, default implementation, or Eq/hash coherence requirement. Implementing
custom keys would therefore have required an invented ABI and behavioral
contract.

**Owner resolution.** `std.collections` publicly exports `Hash`, `Hasher`, and
`DefaultHasher`. Existing `[MOD-5]` keeps `Hash` in the prelude; the latter two
names are not added. `Hasher` is stateful and move-only, supports the canonical
byte/integer write surface, and is consumed by `finish`. Equal values must hash
equally; supplied spans cannot be retained. Map/Set use `DefaultHasher`, but its
mixing algorithm is not frozen. Safe Map lookup and iteration expose keys only
read-only, never as `ref mut K`.

The supplied sketch's `finish(self)` spelling conflicted with its explicit
statement that finalization consumes the move-only context. Existing receiver
modes settle the spelling without a new semantic choice: H3 writes
`finish(owned self)`. The owner authority is preserved in
`docs/spec-source/as-received/ODR-012_Hasher_public_API_and_hashing_contract.md`.

No further owner decision is required. Arbitrary custom-key compiler support
and conformance are implementation work against H3, not permission to freeze a
particular hashing algorithm.

---

## ODR-011 — `ArenaArray` / `ArenaMap` public contract — **CLOSED**

    ID:        ODR-011
    Status:    CLOSED — resolved by the owner and incorporated in 0.9.6_Hardened_2
    Category:  LANGUAGE / STANDARD-LIBRARY API / REGIONS
    Priority:  —
    Location:  Ember_v0.9.6_Hardened_1.md [ARN-5], [TYP-15], [ARN-1]–[ARN-4];
               Part XII §1 standard-library table

    Existing wording: "ArenaArray[T], ArenaMap[K,V] are container variants
                       whose backing storage is an arena view; they are view
                       types (@view) and follow [TYP-15]."

    Semantic impact:                  YES — public API, accepted programs, mutation, regions, failure
    Blocks implementation:            NO — the complete contract is executable
    Blocks conformance:               NO — [TST-24] defines the required matrix
    Blocks specification freeze:      NO — H1 remains an immutable target
    Blocks normative specification adoption: NO — implementation/adoption gates remain
    Requires owner semantic decision: NO

    Resolution: Fixed-capacity, single-allocation Arena-backed view collections
    Authority:  Owner ODR-011 ruling and explicit completion approval; ADR-031; HC-096-02
    Revision:   Ember 0.9.6_Hardened_2
    Result:     Closed; [ARN-5]–[ARN-5g] and [TST-24] are authoritative for H2

**Conflict.** `[ARN-5]` establishes the representation/lifetime category but
does not define how either container is created or used. The standard-library
table mentions `ArenaArray` but omits `ArenaMap`. No normative text fixes:

1. constructor signatures and how the supplying Arena's region reaches the
   result;
2. whether growth is supported, fixed-capacity, or fallible, and what happens
   when reserved arena capacity is exhausted;
3. the minimum read/write/iteration/removal surface and its parameter modes;
4. whether growth invalidates element views and how that interacts with
   outstanding borrows;
5. `ArenaMap`'s key equality/hash capabilities, duplicate-key behavior, and
   iteration guarantees; or
6. whether either container may hold a `needs_drop` element and, if so, who
   owns and runs destruction.

**Owner direction received.** The owner selected fixed-capacity,
single-allocation `ArenaArray`/`ArenaMap` views; shared-Arena
`with_capacity` construction with `@borrows(arena)`; no growth, reallocation,
or hidden cursor mutation; recoverable capacity exhaustion; `!needs_drop`
keys/elements/values; ordinary borrowing; index-order Array iteration; and
unspecified-but-stable Map iteration for unchanged state/configuration. The
minimum Array and Map operations and duplicate-key replacement behavior were
also supplied. These decisions are accepted and must not be reopened.

**Residual owner details that were subsequently resolved.**

1. `CapacityError` is the public unit-only `std.collections` enum with sole
   variant `Full`; capacity failure returns `Err(CapacityError.Full)`. It is
   not a prelude name.
2. The public named `@view` types `ArenaArrayIter`,
   `ArenaArrayIterMut`, and `ArenaMapIter` implement the existing
   associated-type `Iterator[Item = ...]` interface. No opaque/dynamic return
   or second iterator model is introduced.
3. Both `with_capacity` constructors return an empty container with
   `len() == 0`.

Existing `mut self` semantics conservatively make every mutating collection
operation conflict with a live element borrow. That is derived from ordinary
`[BRW-*]` rules and the supplied ban on container-specific invalidation, not
a fourth owner question.

**Superseded initial interpretations.**

1. Arena-backed counterparts of `Array`/`Map` with explicit Arena-borrowing
   constructors and growth that allocates replacement storage from the same
   arena. This is familiar, but needs exact invalidation, failure, and drop
   rules.
2. Fixed-capacity views created from arena-allocated storage. This gives a
   simpler lifetime/failure model but a narrower accepted-program/API surface.
3. Builder-only construction followed by an immutable arena-backed view. This
   minimizes mutation but is materially different from a container variant.

**Recommendation for the residual.** Put a concrete unit-like
`CapacityError` in the same public standard-library module as both
collections, keep it out of the prelude unless explicitly desired, and define
named arena-backed iterator view types implementing the existing
`Iterator[Item = ...]` contract. Confirm empty construction. Preserve
`[ARN-5]`'s non-owning `@view` identity and do not implement an owning
`Array`/`Map` wrapper with a hidden Arena pointer.

**Why this could not be resolved safely by the agent.** The remaining choices
change names and values programs can construct/import and the public types
returned by iteration. Those are observable language/library semantics, not a
compiler mechanism choice.

**Final resolution.** The owner approved the recommendation exactly:
`std.collections` owns all six public names, none enters the prelude,
`CapacityError.Full` is canonical, construction is empty, and iteration uses
named Arena-backed view types through the existing associated-type interface.
ADR-031 and HC-096-02 carry this into frozen H2. No further owner decision is
required.

---

## ODR-010 — Arena rollback under abort-only panic — **CLOSED**

    ID:        ODR-010
    Status:    CLOSED — resolved by the owner and incorporated in 0.9.6_Hardened_1
    Category:  LANGUAGE / ARENA FAILURE SEMANTICS
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_10.md [ARN-10], [PAN-1], Default.default();
               Ember_v0.9.6_Hardened_1.md [ARN-10]

    Semantic impact:                  YES — clarifies the observable failure/rollback contract
    Blocks implementation:            NO — the ambiguity is resolved
    Blocks conformance:               NO — the required evidence is stated
    Blocks H10 identity freeze:        NO — H10 remains immutable
    Blocks normative specification adoption: NO — other adoption gates remain
    Requires owner semantic decision: NO

    Resolution: Abort-only v1 panic does not require observable Arena rollback or unwinding
    Authority:  Owner ruling supplied 2026-09-13; ADR-030; HC-096-01
    Revision:   Ember 0.9.6_Hardened_1
    Result:     Closed. The amended [ARN-10] is authoritative within the
                frozen development target; no [ARN-10a] was introduced.

**Owner resolution.** `Default.default() -> Self` has no recoverable failure
channel in v1. If it panics, `[PAN-1]` terminates the process through `abort()`;
there is no continuation from which Arena state can be observed, and the
compiler must not invent unwinding merely to restore the cursor. Rollback is
required only when a separately specified API defines both a recoverable
construction-failure path and transactional rollback. Such a path cannot make
a partially initialized result reachable. Ordinary `alloc_array[T]` retains
its `!needs_drop(T)` requirement, so rollback does not run element destructors.

**Conformance boundary.** Evidence must cover successful default construction,
termination through the v1 abort path, no reached continuation, no invented
unwinding, and no observation of post-panic Arena state. A future recoverable
construction API is governed and tested by its own explicit rollback contract.

**Historical conflict.** H10 used “fails or panics” while its `Default` and
panic contracts supplied no recoverable failure or unwind path. ERR-051
preserves that ambiguity and the owner resolution without mutating H10.

---

## ODR-009 — Arena bulk-initialization safety contract — **CLOSED**

    ID:        ODR-009
    Status:    CLOSED — resolved by the owner and incorporated in H10
    Category:  LANGUAGE / UNSAFE INITIALIZATION / STANDARD-LIBRARY API
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_10.md [ARN-2], [ARN-3], [ARN-8]–[ARN-13],
               [UNS-1], [UNS-5], [TST-23], Phase 2 and Phase 4

    Semantic impact:                  YES — validity, initialization, drop, API, accepted programs
    Blocks implementation:            NO — the semantic/API blocker is resolved
    Blocks conformance:               NO — implementation and evidence remain ordinary work
    Blocks H9/H10 identity freeze:     NO — H9 preserves the gap; H10 preserves the ruling
    Blocks normative specification adoption: NO — other adoption gates remain
    Requires owner semantic decision: NO

    Resolution: Owner-approved Arena allocation / initialization and MaybeUninit API contract
    Authority:  Owner rulings supplied 2026-09-12; ADR-029; HC-095-09
    Revision:   Ember 0.9.5_Hardened_10
    Result:     Closed. H10 [ARN-2], [ARN-3], and [ARN-8]–[ARN-13] are
                authoritative within the frozen development target.

**Owner resolution.** `alloc_array` deterministically prefers `Zeroable`, then
`Default`, and reports existing E2040 when neither is available; ordinary bulk
allocation always requires `!needs_drop(T)`. H10 originally described
`Default` construction as transactional; ODR-010 and H1 clarify that the v1
abort path creates no observable rollback or unwinding obligation. `Zeroable` means exactly that the
all-zero object representation is a valid initialized `T`; automatic proof is
recursive and manual implementation, if exposed, is unsafe and audited.

`MaybeUninit[T]` has `T`'s size/alignment, never drops `T`, is `Copy` iff `T`
is, and has the canonical `uninit`, `write`, and unsafe consuming
`assume_init` operations. `MutSpan[MaybeUninit[T]]` has canonical `write_at`
and unsafe consuming `assume_init`; the latter asserts full initialization.
The core predicate/built-ins are Phase 2, while general derive generation may
remain Phase 4.

**Signature normalization.** The ruling's prose says that `write` moves its
value and that `assume_init` consumes its source. Because an omitted Ember mode
means borrowed, H10 writes the corresponding `owned value`, `mut self`, and
`owned self` modes explicitly. This makes the supplied semantics executable;
it does not add a second API. The ruling's `[ARN-4]`–`[ARN-7]` headings collided
with frozen H9 rules, so H10 preserves the earlier meanings and assigns the new
clauses `[ARN-8]`–`[ARN-13]`. `[TST-ARN-MU]` is normalized to `[TST-23]`.

**Historical question — existing H9 wording.** `[ARN-3]` demonstrated
`alloc_array[T](count) -> MutSpan[T]`, “zero-initialised if `T: Zeroable` else
`Default`”, and `alloc_uninit[u8](bytes) -> MutSpan[MaybeUninit[u8]]`.
`[UNS-5]` says `MaybeUninit[T]` and `mem.zeroed[T]()` exist and describes
`Zeroable` as an unsafe marker interface auto-derived for “all-scalar/POD
structs”. `[UNS-1]` makes `assume_init` and implementing an unsafe interface
unsafe. The prelude lists `Default`, whose only stated signature is
`fn default() -> Self`.

**Historical conflict / missing contract.** Those sentences named the concepts but did not
define the safety boundary needed to emit code:

1. `MaybeUninit[T]` has no normative layout, `Copy`/`Drop` behavior, constructor,
   write API, initialization-state rule, or operation that turns a fully
   initialized value/span back into `T`/`MutSpan[T]`.
2. `Zeroable` has no canonical interface declaration or exact eligibility
   rule. “All-scalar/POD” is unsafe as an implementation test: references are
   scalar-shaped but zero is invalid, and range/enum validity can also exclude
   the all-zero representation.
3. `alloc_array` has no exact callable/bound contract for `Zeroable` versus
   `Default`, no specified behavior when neither is implemented, and no rule
   for partial initialization or failure while invoking `Default.default()`.
4. `[ARN-3]` is Phase 2 work, while the document schedules `Zeroable` derives
   in Phase 4. It does not say what Phase 2 implementation is expected to use.

The current compiler also rejects the minimal surface probe
`arena.alloc_array[Pixel](2)` with E1010, “only direct calls are supported in
this phase”, because explicit type arguments on methods are not implemented.
That is a separate compiler-side dependency (`GEN-METHOD-1`), not an answer to
the initialization semantics above.

**Historical possible interpretations.**

1. Define a minimal complete unsafe-initialization contract now: exact
   `Zeroable` validity/derivation rules, exact `MaybeUninit` representation and
   state-transition API, and exact `alloc_array` selection/failure/drop rules.
   **Recommended direction**, because it preserves the advertised Phase 2
   Arena surface and makes `[PHIL-10]` mechanically enforceable.
2. Restrict Phase 2 to `Default`-initialized arrays and defer zeroed and
   uninitialized storage. This is simpler but changes the current advertised
   API and phase contract.
3. Defer all Arena bulk allocation to the Phase 4 unsafe/derive work. This
   preserves implementation safety but changes the Phase 2 exit surface and
   leaves `[TST-22]`'s mutable-span clause unavailable until then.

**Answer supplied.** The owner selected and completed option 1, including the
canonical public API in a second follow-up ruling. The requested contract now
appears in H10.

The answer supplied the exact `Zeroable` eligibility claim, manual-implementation
boundary, `MaybeUninit` layout/drop/copy and transition APIs, `alloc_array`
selection, E2040 fallback, `needs_drop` rule, failure semantics, and H10
classification that the original question required.

**Why this could not be resolved safely by the agent.** Each choice changed which
bit patterns may become a typed safe value, when destructors are owed, which
programs type-check, and what API safe code can call. Guessing would silently
change Ember's accepted-program and memory-safety contract.

---

## ODR-001 — `UnsafeCell`'s API surface — **CLOSED**

    ID:        ODR-001
    Status:    CLOSED — superseded by owner ruling S2 / ADR-022
    Category:  Language / API
    Location:  [UNS-10], Part IX §4; docs/spec-amendments.md S2; ADR-022

    Resolution: Owner-approved UnsafeCell API surface
    Authority:  Owner ruling 2026-09-10; amendment S2; ADR-022
    Revision:   Ember 0.8.5_Hardened_1
    ADR:        ADR-022
    Result:     Closed. `[UNS-10]` is authoritative and needs no further
                owner decision.

    Blocks implementation:            NO
    Blocks conformance:               NO
    Blocks specification freeze:      NO
    Requires owner semantic decision: NO

**What the specification says, and it is settled.** `[UNS-10]` defines the
surface: `UnsafeCell(owned v: T)`, `get(self) -> *mut T` — which needs an
`unsafe` context, being a raw pointer under `[UNS-1]` — and
`into_inner(owned self) -> T`, which is safe because the cell is consumed and
nothing is shared. It lives in `std.mem`, and it is the language's lowest-level
interior-mutability primitive with the safety boundary `[UNS-10a]` and
`[UNS-10b]` state.

**None of the following is an open question, and this entry must not be read as
though any of them were:** whether `UnsafeCell` exists; whether it lives in
`std.mem`; whether it has a raw-pointer accessor; whether that accessor requires
`unsafe`; whether `into_inner` exists; whether it is the lowest-level primitive.
All six are decided and normative in 0.8.5.

### Historical context — kept because the conflict is instructive

This entry was opened because `SPEC-FEED-0.8.5-H1` §8 stated that *"the current
specification deliberately leaves exact raw-pointer API naming for a later
specification revision"* and instructed *"Do NOT invent API names during this
feed."* The document did name them, which looked like a contradiction.

It was not one, and the sequence is the point:

1. **The feed's wording was stale.** It described the specification as it stood
   before the API surface was settled.
2. **The owner ruled.** The surface and the module were put to the owner as an
   explicit question on 2026-09-10 — asked precisely *because* both change the
   accepted program set — and the owner chose `std.mem` with a raw-pointer
   accessor. The names were never invented.
3. **The current specification incorporates that ruling** as `[UNS-10]`,
   declared as amendment S2, class `OWNER-APPROVED SEMANTIC CHANGE`, which is
   what made the language version 0.8.5.
4. **No further owner decision is required.**

The entry was raised rather than resolved because settling it either way would
have meant overriding one owner statement with another — the feed or the ruling.
That was the correct call at the time.

### The owner's rationale, 2026-09-10 — why the API stays exactly as it is

> *"There is no benefit in reopening this."*

The design direction Ember has accepted is **safe by default, but capable of
expressing low-level systems mechanisms when the programmer explicitly crosses a
well-defined unsafe boundary.** `UnsafeCell` fits that precisely. It is not an
abstraction for ordinary Ember code; it is the floor underneath the higher-level
interior-mutability mechanisms:

    Safe code
       │
       ├── Cell
       ├── RefCell
       ├── Mutex
       └── RwLock
              │
              ▼
          UnsafeCell
              │
              ▼
         unsafe / raw pointer

**`UnsafeCell` does not weaken Ember's global safety model.** It is a narrowly
scoped escape hatch whose obligations stay explicit — which is what `[UNS-10a]`
and `[UNS-10b]` exist to state. Removing or demoting the API would make the
language *less* implementation-ready with no corresponding design benefit.

**No further action required.**

---

## ODR-002 — six rule definitions the extraction tool could not see — **CLOSED**

    ID:        ODR-002
    Status:    CLOSED — RIDX-1 landed in 6c77723
               No owner semantic decision required; no language change made.
    Category:  TOOLING / DOCUMENT STRUCTURE
    Priority:  P2
    Tracking:  RIDX-1 in docs/BACKLOG.md; design in docs/RFC-rule-extraction.md
    Location:  [TYP-26], [IFC-2], [HND-2], [GPU-7] — each the second rule on a
               line shared with its predecessor
               [VER-7]  — stated inline in a paragraph
               [CTL-3a] — stated inline in a parenthetical

    Semantic impact:                  NONE
    Blocks implementation:            NO
    Blocks conformance:               NO
    Blocks specification freeze:      NO
    Requires owner semantic decision: NO

> **These are not believed to be undefined language rules. They are rules whose
> current document structure is not recognised as a definition by the extraction
> tool.**

That distinction is the whole entry. **ERR-042 investigated this as an
undefined-rule problem and was withdrawn as wrong**; the inventory it produced
is what established that all six are fully stated. Do not resurrect it. Reading
this entry as "six rules are missing" would repeat the exact error that entry
was withdrawn for.

**What is actually true.** Each of the six states its rule in full. `[TYP-26]`,
for example: *"two functions with the same name in one scope is `E1030` — except
operator interface impls and `extend` blocks for distinct types."* That is a
complete rule. `rule_index.py` does not see it because it recognises a
definition structurally — the id must open a bullet, and **only the first id on
a bullet counts** — and its own comment says why: *"a later one is cited by it,
even when the citation reads like a rule. This is `[XXII.4]`'s 'a reference is
not a definition' made mechanical."* All six sit in `rule_index_baseline.json`.

### Neither of these is permitted without a separate tooling review

1. **Changing the six rules' wording or layout merely to satisfy the
   extractor.** The specification is authoritative; a tool that cannot read it
   is the thing that is wrong. Relaying out owner prose for a tool's benefit is
   the wrong direction by this project's cardinal rule.
2. **Weakening the extractor so that it accepts ambiguous rule definitions.**
   The strictness is deliberate. Loosening it admits false negatives, and a
   genuinely undefined rule slipping through is worse than a known-good one
   sitting in a baseline.

### The owner's resolution, 2026-09-10 — the tool is the defective component

> *"If a tool cannot correctly recognize a valid normative rule, the tool is the
> defective component, assuming the specification's structure is itself valid
> and intentional."*

**Keep the six rules exactly as they are. Do not restructure their normative
wording. Do not weaken the extractor. Improve the tool instead**, as separate
work: `RIDX-1`, designed in `docs/RFC-rule-extraction.md`.

**Resolution evidence.** `tools/rule_index.py` now recognizes the four
documented definition forms, and `tools/test_rule_index.py` includes positive
cases plus a `FalseDefinitionsRefused` suite. The dangling-reference baseline
fell from 12 to 4; the remaining four are genuine references/reservations, not
definitions. The specification did not move. Commit `6c77723` landed the
completed implementation and its tests.

The dependency that must not exist, and the architecture that must:

    Specification              ✗                    Specification ───┬── compiler
          ↓  must conform to                                         ├── conformance suite
    current tooling                                                  ├── rule extractor
                                                                     ├── diagnostics tooling
                                                                     └── documentation tooling

The RFC records the four legitimate forms the extractor should learn — canonical
bullet, a rule sharing a structural context, inline in a paragraph, and
parenthetical or granted by name — under one constraint that is the whole
difficulty: **recognising additional syntactic forms must not weaken the
definition/reference distinction.** The tool must judge whether the surrounding
text *defines* the rule, not whether the id sits in a newly permitted position.

Longer term, an explicit machine-readable annotation (`<!-- ember-rule: TYP-26 -->`
or equivalent) would carry rule ownership without relying on Markdown structure
at all. **That is a tooling and documentation evolution and must not be forced
into 0.8.5 to close a queue item.**

---

## ODR-003 — the `[FFI-17]` numbered list: keep in step, or delete the duplicates

    ID:        ODR-003
    Status:    DEFERRED EDITORIAL CLEANUP (owner, 2026-09-10).
               No semantic change required for 0.8.5.
               Technically OPEN for a future revision.
    Category:  EDITORIAL / DOCUMENTATION
    Priority:  P3
    Location:  Part XVI §7a, the numbered list under [FFI-17]

    Semantic impact:                  NONE
    Blocks implementation:            NO
    Blocks conformance:               NO
    Blocks specification freeze:      NO
    Requires owner semantic decision: NO

**There is no current semantic contradiction.** The list is already marked
*"`NON-NORMATIVE` under `[CAT-1]`: where it and a rule disagree, the rule
governs, and the rule is named in each item"*. So the normative FFI tables and
rules are authoritative today, and `SPEC-FEED-0.8.5-H1` §14's requirement is met
as the document stands.

**What remains is the document's own recommendation.** It records that the list
*"has been the site of four contradictions with the rules beside it
(`std::function`, `std::string_view`, C++ inheritance, and the CRT device
attributed to `[FFI-30]`), because a prose restatement of a rule drifts from it
and nothing detects that"*, and concludes: *"A future revision should delete from
the list every claim a rule already makes rather than keep two copies in step."*

**Explicitly not blocking:** compiler implementation, FFI conformance, ABI
implementation, RageV migration, or a language-version freeze. It is a future
cleanup opportunity and nothing more.

**Why it was not done in the 0.8.5 pass.** It is roughly thirty judgement calls
about which prose duplicates a rule and which carries explanation found nowhere
else. Deleting normative-adjacent prose in a pass whose first constraint is *do
not silently redesign* trades a known-safe state for an information-loss risk.
It wants a revision that can work item by item with the tables open beside it.

### The owner's resolution, 2026-09-10 — and the method for the next revision

**Do not simply delete the list.** Classify every item against its authoritative
rule or table, and act per category:

| | If the item | Then |
|---|---|---|
| **A** | merely restates a normative rule | **delete it** — the rule already exists |
| **B** | gives rationale, examples, migration advice or context absent from the rule | **keep**, clearly labelled explanatory / non-normative |
| **C** | is a useful quick reference | keep **only** if it can be mechanically generated from the authoritative rules, or mark it plainly as a non-authoritative summary |
| **D** | conflicts with the normative rule | **delete or rewrite immediately** — the rule wins |

The target shape, and why two representations is bad architecture even when the
second is marked non-normative:

    Normative FFI rules          rather than       Normative rules
            │ authoritative                              ↕
            ▼                                     duplicated prose
      Single source of truth                            ↕
            │                                         tables
            ├── generated summary
            └── explanatory prose (non-normative)       ⇒ drift is inevitable

That is exactly the drift the four recorded contradictions came from. A hardened
specification should eliminate the second source of truth, not keep two copies
in step — but the elimination is item-by-item work for a suitable revision, not
a consistency pass.

---

## ODR-004 — `[LT-8]`–`[LT-13]` source recovery — **CLOSED**

    ID:        ODR-004
    Status:    CLOSED — owner supplied the missing definitions; incorporated
               in H5
    Category:  SPECIFICATION SOURCE RECOVERY / LIBRARY API
    Priority:  —
    Location:  supplied Ember_v0.9.5_Hardened_3_Implementation_Ready_Spec.md,
               lines 105, 304–323, 1,683 and 4,360; frozen H4 target; H5 §H14.3

    Resolution: Owner-supplied Hardened 14 definitions of LT-8 through LT-13
    Authority:  Owner message, 2026-09-12
    Revision:   Ember 0.9.5_Hardened_5
    ADR:        ADR-024
    Result:     Closed. H5 §H14.3 is the recovered definition site.

    Semantic impact:                  NONE — source recovery
    Blocks implementation:            NO
    Blocks conformance:               NO — the source-recovery issue is closed
    Blocks H4 identity freeze:         NO — H4 remains the frozen predecessor
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Existing wording.** The supplied 0.9.5 document says the
`std.borrow.with_views2/3/4` helpers were introduced by 0.9_Hardened_14, remain
normative compatibility APIs, and are governed by `[LT-8]`–`[LT-13]`.
`[TST-16]` requires conformance tests for all six. The standard-library table
also cites `[LT-8]` through `[LT-12]`.

**Historical conflict.** No definition of any of the six rules existed in either supplied
0.9.5 file. H3 added a manifest row saying they were “recovered from Hardened
14”, but the definitions themselves are still absent. The available
0.9_Hardened_12 and 0.9_Hardened_13 files also contain none of them, and no
0.9_Hardened_14 source is available in the repository or supplied downloads.
The frozen H4 development target corrected the unsupported recovery claim.
The owner then supplied the six definitions; they were normalized only for
transport-corrupted Markdown and incorporated in H5 rather than editing H4.
This is not the former ERR-042 tooling problem: there is no hidden inline,
heading, or shared-line definition for the extractor to discover.

**Possible interpretations.**

1. The six rule definitions exist in an omitted 0.9_Hardened_14 source and
   should be recovered verbatim. **Recommended.** This is source recovery and
   creates no new semantics.
2. The references are stale and the helper surface should be removed or
   deferred. That changes the claimed standard-library contract and needs the
   owner.
3. The helpers are intended, but the six rules were never written. Defining
   their exact signatures, mutability, escape, allocation, and region behavior
   is new normative work and needs the owner.

**Semantic impact.** The difference is observable: options 2 and 3 decide
whether programs naming these helpers are accepted and what borrow/escape
behavior they have. An implementation agent cannot reconstruct six APIs from
the phrases “allocation-free late-bound callback composition” and
“two/three/four-view callbacks” without inventing semantics.

**Resolution.** The recommended source-recovery path occurred. H5 preserves the
shared-`Span` signatures, late-bound independent regions, no-escape rule,
ordinary borrowing, `@noalloc` requirement, and persistent-aggregate boundary.
It also reconciles the recovered pre-0.9.5 `[LT-13]` wording with the later,
owner-approved multi-region `[LT-2]` revision without weakening either contract.

**Why an agent could not close the original gap.** Before the owner supplied
the source, function signatures, callback-result restrictions, escape behavior,
and allocation guarantees were absent. Each affected accepted programs and
borrow behavior. H5 now carries those answers. The exact mutable overload
surface remains separately isolated as ODR-005 rather than being guessed.

See `docs/MIGRATION-0.9.5.md` for the complete intake, H4 custody record, and H5
source-recovery classification.

---

## ODR-005 — exact mutable `with_views` overload surface — **CLOSED**

    ID:        ODR-005
    Status:    CLOSED — explicit all-mutable `_mut` family selected
    Category:  LANGUAGE / STANDARD-LIBRARY API
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_6.md §H14.3, [LT-8] and [LT-11]; [TST-16]

    Resolution: Explicit all-mutable with_views2_mut/3_mut/4_mut helpers
    Authority:  Owner ruling, 2026-09-12
    Revision:   Ember 0.9.5_Hardened_6
    ADR:        ADR-025
    Result:     Closed. H6 [LT-8]/[LT-11] are authoritative within the
                development target.

    Semantic impact:                  Owner-approved API clarification
    Blocks implementation:            NO
    Blocks conformance:               NO — the decision is complete; evidence remains
    Blocks H5 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Existing wording.** `[LT-8]` gives structural signatures only for shared
`Span[A]` inputs and callbacks. `[LT-11]` nevertheless requires two mutable
inputs whose sources may alias to be rejected, and `[TST-16]` requires mutable
alias-rejection coverage.

**Historical conflict.** The safety rule was clear, but the callable API that exposes mutable
inputs was not. The recovered source did not state whether mutable helpers exist,
which arguments may be `MutSpan`, whether shared/mutable combinations are one
overload family, or what exact callback types result. Inventing those signatures
would change the accepted standard-library API.

**Possible interpretations.**

1. `with_views2/3/4` have an overload family covering every `Span`/`MutSpan`
   combination, with callback mutability matching each input.
2. Only particular mutable combinations are provided; the owner supplies the
   exact matrix.
3. The public helpers are shared-`Span` only; `[LT-11]` is a general safety
   boundary for any later or implementation-private mutable form, and TST-16's
   mutable-helper requirement should be corrected.

**Resolution.** The owner selected distinct `with_views2_mut`,
`with_views3_mut`, and `with_views4_mut` helpers whose inputs and callback
parameters are all `MutSpan`. H6 uses Ember's canonical spelling `MutSpan[T]`;
the ruling's `SpanMut[T]` spelling was a terminology mismatch and did not create
a second type. The original shared helpers stay shared. No mixed `Span`/
`MutSpan` overload family is specified. `[LT-11]` binds the mutable family to
ordinary exclusive borrowing and alias rejection.

**Why owner resolution was required.** The alternatives accepted different
programs and exposed different mutation capabilities. The existing borrow rules
could determine whether a selected API was safe, but could not choose which API
Ember promised.

---

## ODR-006 — mutable helper and callback parameter modes — **CLOSED**

    ID:        ODR-006
    Status:    CLOSED — mutable helper inputs are mut reborrows
    Category:  LANGUAGE / CALLABLE API
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_8.md §H14.3 [LT-8]/[LT-8a]/[LT-11];
               Part III fn_type; [FN-1]/[FN-2a]; [FN-6a]

    Resolution: Mutable helper inputs use mut; no helper input is owned
    Authority:  Owner rulings, 2026-09-12; ADR-026 / ADR-027
    Revision:   Ember 0.9.5_Hardened_8
    Result:     Closed. H8 [LT-8] is authoritative within the development
                target.

    Semantic impact:                  YES — callability and mutation authority
    Blocks implementation:            NO — implementation remains to be built
    Blocks conformance:               NO — evidence remains to be added
    Blocks H8 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Resolved portion.** H7 extends `fn_type` with borrowed/default, `mut`, and
`owned` modes under ordinary `[FN-1]`/`[FN-2]` semantics. The `_mut` callbacks
now explicitly use `fn(mut MutSpan[A], ...)`; `[LT-11a]` and `[TST-20]`
make the mode mismatch testable. This closes the callback side of the question.

**Historical remaining conflict.** The H7 helper functions' own inputs were written
`a: MutSpan[A]`, not `mut a: MutSpan[A]` or `owned a: MutSpan[A]`. `[FN-1]`
makes that an ordinary shared borrow which cannot itself be mutably borrowed or
moved when the helper invokes a callback requiring `mut MutSpan[A]`. The owner
ruling changed callback modes but did not state the helper input modes.

**Possible interpretations.**

1. Spell every helper input `mut a: MutSpan[A]`. This is the direct application
   of `[FN-1]`/`[FN-1a]` and is recommended if the helper reborrows the caller's
   mutable view for the callback.
2. Spell every helper input `owned a: MutSpan[A]`, consuming the view capability
   into the helper before it passes/reborrows it. This changes caller usability.
3. Define a narrow rule that an unmarked `MutSpan` parameter may mutate its
   referent. This conflicts with the general shared-parameter rule and risks an
   aliasing exception; it is not recommended.

**Former recommended owner action.** Select `mut` or `owned` for the helper's own
`MutSpan` inputs. Prefer `mut` if the intent is invocation-scoped reborrowing
that leaves the caller's view usable afterward. Do not choose option 3 without
explicitly amending `[FN-1]` and the safety argument.

**Why this cannot be resolved safely by an agent.** The alternatives change
ownership, post-call usability, and which arguments can be passed. The H7
callback-mode ruling does not select the helper's own mode.

**Owner resolution.** Every mutable helper input is explicitly `mut`; shared
helper inputs retain the default borrowed mode; neither family consumes a view.
This is invocation-scoped reborrowing, so caller usability resumes when the
helper returns. H8 applies the ruling and `[TST-21]` makes it testable. The
supplied `SpanMut[T]` and `[TST-LT-MODE]` spellings are normalized to canonical
`MutSpan[T]` and the next unused numeric rule ID, `[TST-21]`.

---

## ODR-007 — mode-bearing `fn` types and `Callable[Args, R]` — **CLOSED**

    ID:        ODR-007
    Status:    CLOSED — mode vector is compiler-known canonical metadata
    Category:  LANGUAGE / CALLABLE ABSTRACTION
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_8.md [FN-6a], [CLO-3], [CLO-6];
               std.core Callable/CallableOnce declarations

    Resolution: Preserve modes internally through existing Callable[Args, R]
    Authority:  Owner ruling, 2026-09-12; ADR-027
    Revision:   Ember 0.9.5_Hardened_8
    Result:     Closed. H8 [FN-6a] is authoritative within the development
                target.

    Semantic impact:                  YES — callable compatibility and dispatch
    Blocks implementation:            NO — implementation remains to be built
    Blocks conformance:               NO — TST-20/TST-21 evidence remains
    Blocks H8 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Existing wording.** H7 makes parameter modes part of callable types and says
they have ordinary ownership semantics. `[CLO-3]` still says `fn(A) -> R`
denotes an implicit generic bound `Callable[(A), R]`, while `std.core` declares
`Callable[Args, R]` and `CallableOnce[Args, R]`. `Args` is an ordinary tuple of
types and carries no mode vector.

**Conflict.** `fn(A)`, `fn(mut A)`, and `fn(owned A)` must be distinguishable
for compatibility and invocation, but the documented `Callable[Args, R]` bridge
maps all three to the same `Args = (A)` surface. The owner explicitly rejected
a separate callable ownership model, so an agent cannot add an unrelated
parallel interface ad hoc.

**Possible interpretations.**

1. Make the mode vector compiler-known metadata on the implicit callable bound;
   `Callable[Args, R]` remains the source spelling but `[CLO-3]` defines its
   mode-bearing use precisely.
2. Change the public interface shape so the complete `fn(...) -> R` signature,
   including modes, is the generic argument to `Callable`/`CallableOnce`.
3. Add a separate mode-vector generic parameter or family of callable
   interfaces. This is explicit but expands the public standard-library model.

**Former recommended owner action.** Prefer option 1 if preserving the current public
`Callable[Args, R]` spelling is important; specify how conformance and method
matching observe the compiler-known mode vector. Otherwise choose an exact
public interface signature. In every case, mode mismatches must remain type
errors and no runtime mode dispatch should be introduced.

**Why this cannot be resolved safely by an agent.** The options alter interface
identity, generic bounds, coherence, closure matching, dynamic callable
compatibility, and possibly ABI/interface hashes. `[FN-6]` establishes required
behavior but does not choose this bridge representation.

**Owner resolution.** Option 1 governs. The existing public
`Callable[Args, R]` abstraction remains, while the compiler's canonical
callable type carries the complete borrowed/`mut`/`owned` mode vector through
generic bounds, type and borrow checking, overload resolution, and
monomorphisation. The metadata is compile-time-only and introduces neither
runtime bookkeeping nor a second ownership system. `[FN-6a]`, `[LT-8a]`, and
`[TST-21]` record the contract in H8.

---

## ODR-008 — Arena-backed return provenance — **CLOSED**

    ID:        ODR-008
    Status:    CLOSED — `Arena` is a narrow `@borrows` provenance source
    Category:  LANGUAGE / REGION PROVENANCE
    Priority:  —
    Location:  Ember_v0.9.5_Hardened_9.md [LT-1a], [LT-4], [LT-4a], [LT-4b]

    Resolution: Permit @borrows(arena) only for Arena-backed returned views
    Authority:  Owner ruling, 2026-09-12; ADR-028
    Revision:   Ember 0.9.5_Hardened_9
    Result:     Closed. H9 [LT-4a]/[LT-4b] are authoritative within the
                development target.

    Semantic impact:                  YES — wrapper signatures and accepted programs
    Blocks implementation:            NO — ruling received and implementation started
    Blocks conformance:               NO — TST-22 evidence can now be built
    Blocks H9 identity freeze:         NO
    Blocks normative specification adoption: NO
    Requires owner semantic decision: NO

**Historical conflict.** `[LT-4]` tied every arena allocation to the Arena
borrow, while `[LT-1a]` rejected an Arena parameter in the only public
return-provenance annotation because Arena is not a view type. A safe wrapper
could therefore neither express the relationship nor omit it soundly.

**Owner resolution.** A function whose returned view is proven to derive from
storage owned by a growing `Arena` parameter writes `@borrows(arena)`. This is
a provenance-only exception. Arena stays non-view; arbitrary non-view
parameters, unrelated views, owned results, lifetime extension, ownership
transfer, and borrow-checker bypass remain forbidden. Nested wrappers repeat
the annotation. Omitted provenance is E3061; false provenance remains E3062;
an unrelated non-view parameter remains E2031.

**Normalization.** H9 uses the existing `alloc`/`alloc_array` API rather than
the ruling's otherwise-undefined illustrative `alloc_span` names, and records
the mnemonic conformance request as numeric `[TST-22]`.

---

## Closed — the audit trail

Kept so that a future agent can find where each decision was made rather than
reopening it.

| Question | Resolution | Authority | Revision | Result |
|---|---|---|---|---|
| **ODR-001** — `UnsafeCell`'s API surface | Owner-approved: `std.mem`, raw-pointer accessor | Owner ruling 2026-09-10; **S2 / ADR-022** | 0.8.5_Hardened_1 | Closed; `[UNS-10]` is authoritative |
| `RefCell[T]` and `Copy` | Not `Copy`; move-only whatever `T` is | Owner ruling 2026-09-10; **S3 / ADR-021** | 0.8.5_Hardened_1 | `[CELL-12]` normative; `[CELL-4]` explicitly does not reach `RefCell` |
| Cut `Hardened_2`, or defer E5 | Cut it — E5 is editorial repair, not a revision | Owner ruling 2026-09-10 | 0.8.4_Hardened_2 | Cut, then superseded by 0.8.5 |
| ERR-043 — `UnsafeCell` undefined | Retained as the lowest-level interior-mutability primitive | Owner ruling 2026-09-10; **S2 / ADR-022** | 0.8.5_Hardened_1 | `[UNS-10]`/`[UNS-10a]`/`[UNS-10b]`; implemented at `a02c0a5` |
| ERR-041 / deviation D5 — `[FN-1]` | Part VII §7's worked example governs | Owner ruling 2026-09-10; **S4** | 0.8.5_Hardened_1 | `[FN-1a]`; D5 closed, **no code moved** |
| ERR-042 — nine "undefined" rule ids | Inventory ordered; it found **zero** gaps | Owner ruling 2026-09-10 | — | Entry **withdrawn as wrong**; see ODR-002 for what is actually true |
| Scope changes by an implementation agent | Permitted when justified; **must be reported** | Owner ruling 2026-09-10 | — | `docs/HANDOFF.md` §0.0 I |
| **ODR-004** — missing `[LT-8]`–`[LT-13]` source | Owner supplied the Hardened 14 definitions | Owner message 2026-09-12; **ADR-024** | 0.9.5_Hardened_5 | Closed; H5 §H14.3 is authoritative within the development target |
| **ODR-005** — mutable `with_views` API | Explicit all-mutable `_mut` helper family using `MutSpan[T]` | Owner ruling 2026-09-12; **ADR-025** | 0.9.5_Hardened_6 | Closed; no mixed-mutability overloads are implied |
| **ODR-006** — helper/callback modes | Mutable helper inputs are `mut` reborrows; callback modes remain explicit | Owner ruling 2026-09-12; **ADR-026 / ADR-027** | 0.9.5_Hardened_8 | Closed; no helper consumes an input view |
| **ODR-007** — `fn`/`Callable` mode bridge | Preserve the complete mode vector as compiler-known canonical type metadata | Owner ruling 2026-09-12; **ADR-027** | 0.9.5_Hardened_8 | Closed; no runtime mode bookkeeping or second ownership system |
| **ODR-008** — Arena-backed return provenance | Permit narrow `@borrows(arena)` only for a view backed by that Arena | Owner ruling 2026-09-12; **ADR-028** | 0.9.5_Hardened_9 | Closed; Arena remains non-view and arbitrary non-view parameters remain forbidden |
| **ODR-009** — Arena bulk-initialization contract | Define deterministic initialization, `Zeroable` validity, canonical `MaybeUninit` transitions, rollback, and phase ordering | Owner rulings 2026-09-12; **ADR-029 / HC-095-09** | 0.9.5_Hardened_10 | Closed; implementation and conformance remain ordinary tracked work |
| **ODR-010** — Arena rollback under abort-only panic | Clarify that v1 abort has no observable rollback/unwind obligation; transactional rollback requires an explicitly recoverable API contract | Owner ruling 2026-09-13; **ADR-030 / HC-096-01** | 0.9.6_Hardened_1 | Closed; no `[ARN-10a]`; implementation and conformance remain ordinary tracked work |
