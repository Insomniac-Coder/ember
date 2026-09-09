# Ember — defect ledger

Every defect found and what closed it. One row per defect, newest first.

**Why this exists.** Defects were recorded in prose, spread across
`docs/HANDOFF.md`'s per-block sections, where "found" and "fixed" read the
same. A reader could not tell which of them are closed. This file answers that
one question, and links to where the reasoning lives.

**How to use it.** Fixing a defect updates four documents, and the row records
which:

1. **this ledger** — the row, with how the fix was *verified*: a program that
   failed before and passes now, or a test that goes red when the fix is
   removed;
2. **`docs/HANDOFF.md`** — the reasoning, in the section for the block it
   belongs to;
3. **`docs/DECISIONS.md`** — an ADR, where the fix took a decision the
   specification does not force;
4. **the specification** — `docs/spec-source/ember-spec.md`, regenerated into
   `docs/spec/`, where the document's own wording admitted the wrong reading.
   That goes through `docs/spec-errata.md` and its procedure, never by hand.

The fourth is the one to be careful with: **an implementation gap is not a spec
defect.** Where the document was right and the compiler was wrong, say so in the
row and leave the specification alone — ERR-014, ERR-019 and ERR-022 record what
the opposite mistake costs.

Status is one of **fixed**, **open**, or **won't fix** with the reason.

---

## 2026-09-09 — the region work

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-001 | A borrow copied into another local was not tracked: `s = r` killed the loan with `r`, so writing the owner afterwards compiled | `[BRW-1]`, `[LT-5]` | **fixed** | `1204a5e`, ADR-010 |
| D-002 | A reborrow (`q = ref mut r`) did not keep the borrow it derived from alive | `[BRW-6]`, `[LT-5]` | **fixed** | `1204a5e` |
| D-003 | A call handing a reference back did not keep its argument borrowed | `[LT-1]` | **fixed** | `1204a5e` |
| D-004 | `[DIA-3]`'s "later used here" label was missing from every borrow diagnostic, and the help named the local the borrow *started* in | `[DIA-3]` | **fixed** | `1204a5e` |
| D-005 | `@borrows` was checked as a signature only: `@borrows(a)` on a function returning `b` compiled | `[LT-1a]` | **fixed** | `c1bd89c`, ADR-011 |
| D-006 | `E3060` exempted every parameter, so a borrow of a **by-value** parameter could be returned | `[LT-1]`, XVIII §4.7 step 6 | **fixed** | `c1bd89c`, ERR-024, ADR-011 |
| D-007 | `[TYP-14]`'s read-through worked only for a named local, so a call returning `ref i32` was not an operand of `+` (`E2020`) and reached `println` as a pointer | `[TYP-14]` | **fixed** | `c1bd89c`, ERR-023 |
| D-008 | `ember fmt` deleted a doc comment on a method | `[FMT-1]` | **fixed** | `28aa05d` |
| D-009 | `ember fmt` deleted a doc comment on an enum variant | `[FMT-1]` | **fixed** | `28aa05d` |
| D-010 | The AST printer showed a variant as its bare name, so `[FMT-1]`'s round-trip test could not see D-009 | `[FMT-1]`, `[TST-1]` | **fixed** | `28aa05d` |
| D-011 | `E3064` (two independent regions in one view struct) is registered and emitted by nothing | `[LT-2]` | **open — under semantic review** | I closed this on the reading that `[LT-2]`'s intersection is *taken*, so construction never fails and no v1 source can demand two regions; the owner reopened it (2026-09-09) because that is a determination, not an observation — the evidence does not yet say whether the compiler narrows something the rule means to reject, or the rule is stricter than it needs to be. Amendment A4, which had written my reading into `[LT-2]`, is **withdrawn**; the rule reads as the owner wrote it. What is established and not in doubt: the intersection is enforced through a call — `tests/conformance/LT-2/` shows `pick(p, q)` borrowing both even though it returns one. ADR-012 stands as a *decision to defer*, not as an answer |

## 2026-09-09 — v0.8.3, the standard library and range types

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-014 | A `##` doc comment above an `import` was `E0100 expected a declaration`. `[LEX-11]` says such a comment "documents nothing and is **discarded in silence**: a comment never affects compilation, and that includes producing a warning" — and an import is not a declaration | `[LEX-11]` | **fixed** | the parser skips a doc-comment run followed by an import; two parser tests and `tests/conformance/LEX-11/` |
| D-015 | Interfaces were collected per module, immediately before that module's `implements`, so a module checked before the one declaring an interface could not implement it. `[MOD-4]` allows import cycles inside a package, so **no** load order would have worked | `[IFC-1]`, `[MOD-4]` | **fixed** | interface collection is whole-program, like the name pass above it; `tests/conformance/TYP-17/` |
| D-016 | A generic bound, an `implements` clause and a supertrait each recorded the name **as written**, so an imported `Ord` did not match an implementation of the same interface | `[TYP-17]`, `[IFC-3]` | **fixed** | all three record the resolved name |
| D-017 | `Self` did not resolve in any signature: `interface Clone: fn clone(self) -> Self` was "this type is not supported yet". Part IV §8 declares nine interfaces over `Self`, so `std.core` could not be written at all | Part IV §8, `[TYP-22]` | **fixed** | `Self` is the concrete type in a body and a parameter in an interface, substituted at each use |
| D-025 | `RangeError` was created on **first use**, so it did not exist while signatures were being collected and `[RNG-3]`'s own worked example — `fn from_slider(x: f32) -> Result[Roughness, RangeError]` — did not compile. Found by `tools/error_pages.py` refusing a page whose "fix" did not compile | `[RNG-3]` | **fixed** | registered as a **prelude type** before any module is walked. Not a `std.core` declaration: `checked` is a language-defined construction under `[RNG-10]`, not a library function, so its error type cannot wait on an import; `tests/conformance/RNG-3/accept_range_error_is_nameable.em` is the spec's own signature, compiled and run |
| D-026 | A write through a **shared** `ref` passed every check in the compiler and was caught only by the C backend's `const` (`error: read-only variable is not assignable`). ADR-010 makes a reference local non-re-seatable, so `r = 99` and `r = ref y` both write *through* `r` — sound for `ref mut`, and exactly the write `[BRW-1]` forbids for a shared borrow. Found while auditing `[LT-2]`/`E3064` on the owner's list | `[BRW-1]`, `[TYP-14]`, `[CG-C-1]` | **fixed** | rejected in typeck, where the type alone answers it and no flow analysis is needed; `tests/conformance/BRW-1/` has the reject and the `ref mut` accept beside it. Aliasing-XOR-mutability was being upheld by the backend rather than the language, which is luck, not a rule |
| D-024 | A call argument was parsed as a full expression, so a lambda's `:` body inside brackets consumed an indented block. `[LEX-6a]` makes it "a single `small_stmt`, terminated by the enclosing closing bracket or by a `,`", and indentation is not significant inside brackets, so there was nothing for a second statement to belong to | `[LEX-6a]`, `[GRM-17]` | **fixed** | arguments parse with block lambdas disabled, and `E0106` — registered and emitted by nobody until now — reports a body that is not one statement, with `[GRM-17]`'s mandated help |
| D-022 | `[SPN-1]`'s coercion built a view without borrowing its container, so `v: Span[i32] = a` followed by `a.push(…)` compiled — the push reallocates and the view dangles. Found by asking the question rather than by a test failing | `[BRW-1]`, `[SPN-1]`, `[UNS-4]`, `[PHIL-10]` | **fixed** | the coercion takes an explicit borrow, which is what `collect_loans` looks for, and the call's elision carries the loan's region to the view; `tests/conformance/BRW-1/` |
| D-023 | `lower_span_get` assumed `Some` was `Option`'s variant 0. `option_of` builds `None` first | `[SPN-2]` | **fixed** | the variant order is read from the type table: it is a choice inside one function, not a language rule |
| D-019 | Field visibility was not enforced at all: `[MOD-2]` makes a field private unless `pub`, and every field of every struct could be read from any module | `[MOD-2]` | **fixed** | `FieldDef` carries `FieldVis`; a private field read from another module is `E1020`. It rejected `tests/run-pass/modules_across_files.em`, which had been passing on the strength of the gap |
| D-020 | `[MOD-7]`'s `pub(read)` was parsed and then dropped: `E1050` was registered and emitted by nobody, so a read-only field was writable from anywhere | `[MOD-7]` | **fixed** | the flag reaches `FieldDef`; assignment, augmented assignment, `ref mut` and a `mut` argument each check the whole projection chain |
| D-021 | `[STR-1]`'s memberwise constructor was `pub` regardless of its fields, so a struct with private or `pub(read)` fields could be constructed from any module — which writes them | `[STR-1]`, `[MOD-7]` | **fixed** | the constructor is checked against every field's visibility at the call |
| D-018 | `[RNG-4]`'s range tracking derives a range for constants only. `[RNG-10]`(d) — "a value whose `[RNG-4]` range is contained in the target's" — therefore admits a constant and nothing else, and `[RNG-5a]`'s range-preserving clamp family has no `min`/`max`/`clamp` to preserve through | `[RNG-4]`, `[RNG-4a]`, `[RNG-5a]` | **open — precision, not soundness** | the emitted checks are correct and complete; what is missing is the analysis that removes some of them, and half of it waits for `std.math` |

## 2026-09-09 — the v0.6 draft

Defects in a **proposed** document, not in the compiler. Re-check both against the
expanded v0.6 specification when it arrives: they were introduced by fixes, and the
same fixes are likely to have been carried forward.

| # | Defect | Rule | Status | Where |
|---|---|---|---|---|
| D-012 | `E1020` is used for "you referred to a type from a library layer this build omitted", and `[GRM-4]` already uses `E1020` for redeclaring a name in the same block | `[BLD-11]`, `[GRM-4]` | **fixed by v0.8.3** | `[BLD-11]` now reports `E1021` and says in terms that `E1020` "remains `[GRM-4]`'s and MUST NOT be reused" |
| D-013 | `[STD-7a]` says const generics are governed by `[CG-1]`..`[CG-4]`; none of those four rules is defined anywhere in the document | `[STD-7a]` | **fixed by v0.8.3** | it now cites `[GRM-8]`, `[CT-1]`, `[MONO-1]` and `[TYP-19]`, all of which exist |

**Two claims of mine that were wrong**, recorded so they are not re-raised as
defects. I reported that `roughness + metallic` compiles — unprovable, because the
conversion rule never says *where* an implicit conversion applies and an operand has
no expected type to convert against, so the real gap is the missing "where". And I
reported that fixed-capacity containers need a language feature Ember lacks — the
grammar has const-generic arguments and fixed arrays already use them, so the gap is
in the compiler, not the design. A claimed contradiction that turns out not to exist
has cost this project three errata already; check before recording one.

### How each was verified

**D-001, D-002, D-003.** Three programs that write through a dangling
reference. Each compiled in silence before and is `E3021` now:

```ember
r: ref mut i32 = ref mut n
s: ref mut i32 = r      ## the loan is held by `s` now
n = 5                   ## …and this was accepted
s = 7
```

`tests/compile-fail/borrows_travel_with_the_reference.em` holds all three, and
`tests/run-pass/references_that_travel.em` holds the same shapes written
correctly, so the fix is not over-rejection.

**D-004.** Visible in the rendered diagnostic: the conflict now carries a
`borrow later used here` label at the next read, and the help names the local
that actually keeps the loan alive — `s` in the program above, where it used to
say `r`.

**D-005.** `@borrows(a) fn pick(a: ref i32, b: ref i32) -> ref i32: return b`
is `E3062`. `[LT-1]` rule 1's case — a method whose result points into an
argument rather than into its receiver — is rejected with the `@borrows` that
would fix it in the help line.
`tests/compile-fail/a_returned_view_elision_cannot_tie.em`.

**D-006.** `fn peek(self) -> ref i32: return ref self.n` is `E3060`. The
exemption is now for view-typed parameters only: what the caller owns outlives
the call, a copy in this frame does not.

**D-007.** `println(give(ref n))` emitted C passing a `const int32_t *` where
`ember_str` was expected — clang caught it, Ember did not — and
`give(ref n) + 1` was `E2020`. Both work now;
`tests/run-pass/returned_views.em` runs them.

**D-008, D-009, D-010.** `ember fmt` on a file with a documented method and a
documented variant keeps both. Verified the way the handoff asks: the fix was
removed and `formatting_is_idempotent_and_preserves_the_tree` went red, then
restored. For D-009 that test went red **only after D-010 was fixed** — the
instrument was blind to the defect it exists to catch.

**D-008 and D-009 needed no specification change, and that is deliberate.**
`[FMT-1]` says "the formatter preserves comments" and `parse(fmt(x)) ≡
parse(x)`; a doc comment is a comment and the rule is unambiguous. Likewise
`[DIA-3]` for D-004: the label is a MUST and the compiler emitted none. Both are
recorded in the errata's closing section so the question is not re-asked.

**D-011 is open and deliberately unwritten.** `[LT-2]` gives a struct built
from several references the *intersection* of their regions, so the compiler
narrows rather than refusing, and no program has been found that reaches the
error. Writing the diagnostic before there is a program that needs it would
enforce the rule in a shape nobody asked for.

---

## Before 2026-09-09

Reconstructed from `docs/HANDOFF.md`, which is where the reasoning for each of
these lives. All are fixed; the ledger did not exist when they were.

| # | Defect | Rule | Status |
|---|---|---|---|
| D-101 | A `##` comment inside a function body deleted the statement under it — the parser returned "no statement" and the caller treated that as a parse failure | `ERR-007` | **fixed** |
| D-102 | `let` was dropped by the formatter, turning an immutable field into a mutable one | `[CLS-9]`, `[FMT-1]` | **fixed** |
| D-103 | Doc comments were dropped by the formatter at the item level | `[FMT-1]` | **fixed** |
| D-104 | The formatter dropped every type parameter list: `struct Pair[A, B]` printed as `struct Pair` | `[FMT-1]` | **fixed** |
| D-105 | The formatter flattened `unsafe:` blocks — the catch-all preserved text and destroyed structure | `[FMT-1]`, `[UNS-1]` | **fixed** |
| D-106 | Methods on a generic struct were silently dropped, so an instantiation had no methods | `[TYP-16]` | **fixed** |
| D-107 | `alloc`'s `arg_ty` was taken from its first argument, a count, so every allocation was `usize`-sized | `[UNS-4]` | **fixed** |
| D-108 | `StmtKind::CheckedBinaryOp` was invisible to the move analysis, so `bigger = cap * 2` read as a use after move | XVIII §4.6 | **fixed** |
| D-109 | A panicking program reported exit code 0: `abort()` arrives as `0xC0000409` and `clamp(0, 255)` made it a clean exit | `[TST-1]` | **fixed** |
| D-110 | `println(10.0)` printed `1e+01` — the shortest *precision* that round-trips, rather than the shortest text | `[FMT-2]` | **fixed** |
| D-111 | `n = match d:` with indented arms did not parse | `[GRM-11]` | **fixed** |
| D-112 | `t.0.1` did not parse: the lexer read `0.1` as one float literal | `[GRM-8]` | **fixed** |
| D-113 | An unused pattern binding tripped `-Wunused-but-set-variable`, breaking the warning-free C requirement | `[CG-C-1]` | **fixed** |
| D-114 | `interface From[T]` could not be declared because `from` was reserved | `ERR-017` | **fixed** |
