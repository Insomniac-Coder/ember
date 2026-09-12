# Ember — defect ledger

## Which ledger a finding belongs in

Every finding sorts into one of four, and the sort decides who moves. Getting
this wrong in either direction is how an implementation workaround quietly
becomes the language's semantics.

| Kind | Evidence | Where it goes | Who moves |
|---|---|---|---|
| **Implementation defect** | the rule is clear, the compiler violates it | `DEFECTS.md` (here) | the compiler |
| **Missing coverage** | the compiler may be right and nobody had proved it | a conformance case; no ledger entry unless it fails | nobody |
| **Normative contradiction** | two rules require different things, neither marked as governing | `spec-errata.md` | **neither, until the owner rules** |
| **Deliberate divergence** | the rule is clear, the compiler knowingly differs, with a reason | `DEVIATIONS.md` | the compiler, later |

The third is the one that needs discipline, because a compiler cannot be called
wrong against a document that says both things — so the temptation is to pick
the reading that matches what is already built and move on. ERR-044 is the
worked example: `[TYP-15]`'s enumeration forbids a static-region `str` in a
class field, `[LT-3]` permits it in as many words, and `[TYP-15]`'s own
principle sides with `[LT-3]`. The compiler follows the enumeration and stays
there, unchanged, until the owner decides.


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

## 2026-09-12 — `[CELL-10]` / borrowed-parameter write closure

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-044 | **Writes through default-mode value parameters were accepted and silently lost.** `[FN-1]` makes an omitted parameter mode a shared borrow, but the type checker retained only the parameter's ABI-local type. A small value therefore looked like an ordinary writable local: `fn increment(counter: Counter): counter.value = counter.value + 1` compiled, mutated only the callee's private by-value copy, and returned with the caller unchanged. The same hole admitted `ref mut`, mutable-view creation, `mut self`, and forwarding to a `mut` parameter | `[FN-1]`, `[BRW-1]`, `[CELL-10]`, `[DIA-7]` shape B4 | **fixed** | type checking now retains default-mode parameter provenance independently of ABI representation and rejects every write-capable access rooted there as `E3023`. The diagnostic's first help is the structural single-owner/`mut` repair; `Cell`/`RefCell` and class are later, costed alternatives only at sites proven not simultaneous. Call boundaries conservatively omit `RefCell` until the whole call can establish that condition. **Verified:** the minimal assignment case failed the conformance suite before the fix; disabling the new provenance check made all five E3023 probes under `tests/conformance/CELL-10/` compile with exit 0. Ordered and forbidden-help directives were each mutation-tested red, and `docs/errors/E3023.md` contains an executable failing/fixed pair. The specification was already explicit; neither it nor an ADR changed |

## 2026-09-12 — D-042 move-path closure

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-043 | **An `owned` parameter was never destroyed by its callee.** `[FN-1]` transfers ownership at the call, but MIR lowering registered only body locals for scope-end destruction; owned parameters were absent from `Builder.owned`. A callee that accepted `owned resource: Resource` and did not move it onward returned without running `Resource.drop`, leaking everything the argument owned. The per-field call-argument probe for D-042 exposed this as a missing destructor line, independently of D-042's source-field flag | `[FN-1]`, `[OWN-2]`, `[OWN-3]` | **fixed** | owned, droppable parameters are registered as the function's outer ownership scope; explicit returns already discharge that scope and fallthrough now does so after body locals. Move-path flags start live for arguments and call-terminator moves clear them. **Verified:** `tests/conformance/OWN-2/accept_owned_parameters_drop_in_the_callee.em` covers both the move-onward and not-moved paths; `tests/conformance/EXP-6/accept_partial_move_into_a_call_clears_the_field_flag.em` first failed with the transferred value never dropped by the callee (`10, 2, 2, 1` instead of `10, 1, 2, 2, 1`). The specification was already explicit; it was not changed |

## 2026-09-09 — task 2, the D-035 sweep

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-036 | **Pushing a moved value destroyed it twice.** `Array.push` spilled its value argument through a temporary and erased the move to a copy, so the temporary kept its statement-end drop while the buffer took ownership and dropped the element at scope end. `a.push(R(1))` printed 1, 2, 1, 2 — for a value owning a buffer that is a double free. An `owned` parameter of a user function never had this shape (`take(R(a))` destroys once); only the builtin push path erased the move | `[OWN-3]`, `[OWN-2]` | **fixed** | the push argument keeps the shape `lower_operand` gives it; only constants (no address: `&10` is not C) still spill. **Verified:** `tests/conformance/DRP-2/accept_array_elements_drop_in_index_order.em` prints 1, 2 and goes "1 2 1 2" red when the fix is reverted — checked by doing exactly that |
| D-037 | **`as_str()` built a view with no borrow behind it.** It lowered straight to the builtin, so `collect_loans` saw no `Rvalue::Ref`, the elision table tied the result to nothing, and `s.push_str("!")` while the `str` was live compiled — the view dangled across the reallocation. The twin of D-022, which fixed the same hole for spans; strings were missed | `[SPN-1]`, `[BRW-1]`, `[UNS-4]`, `[PHIL-10]` | **fixed** | `as_str()` goes through the one producer (`view_of`, generalised to take the builtin tag), the elision table ties `StringAsStr` results to the receiver, the backend reads the borrowed pointer, and `verify_views` checks the new path too. **Verified:** `tests/conformance/SPN-1/reject_mutating_around_a_live_str_view.em` fails to compile as required and compiles when the fix is reverted — checked by doing exactly that |
| D-038 | **Implicit `String`→`str` coercion was rejected.** `[SPN-1]` lists "String to `str`" among the coercions and Part XIX §5 records it as one the checker applies, but `v: str = s` was `E2020`. The explicit `as_str()` spelling already worked soundly after D-037, so this failed closed | `[SPN-1]` | **fixed** | `coerce` recognizes the compiler-known `String` representation when `str` is expected and routes it through the existing `view_of(..., StringAsStr)` producer. The implicit and explicit spellings therefore share the same explicit IR borrow, elision, verifier, and backend path. **Verified:** three `[SPN-1]` cases cover assignment and argument coercions, NLL ending a call-local borrow, E3021 on mutation across a live implicit view, and E3060 when returning a view of non-view parameter storage. Removing only the coercion arm restores E2020 in all three probes. The specification was already explicit and was not changed |
| D-039 | **Reservation windows opened for user-local borrows, misclassifying conflicts.** Any borrow whose borrower was later used as a call argument was treated as two-phase-reserved — including plain bindings — so two `ref mut` borrows of one field reported B3/`E3021` instead of B1/`E3022`. A reservation is taken *for* a call; its borrower is always a lowering temporary | `[BRW-3]`, `[BRW-4]`, `[DIA-7a]` | **fixed** | windows open only for temporaries. **Verified:** `tests/conformance/BRW-4/reject_two_mutable_borrows_of_one_field.em` reports `E3022` and reports `E3021` when the fix is reverted — checked by doing exactly that; the full suite stays green, including the two-phase accept cases |
| D-040 | **Method-call conflicts have no code: `E3025` is registered, shape-mapped, and emitted by nothing.** A method call defeating disjoint-field access (`[BRW-4]`'s parenthetical) reports `E3022`, where `[DIA-7a]` keys shape B8 to `E3025`. Keying it needs call provenance at the report point — the conflicting access is recorded as a plain borrow — which is design work, not a lookup | `[BRW-4]`, `[DIA-7a]` | **fixed** | the reporter walks the autoref temporary forward to its consuming call and reports `E3025`/B8 when the callee is a method body; free-function `mut` arguments keep `E3022`. **Verified:** `tests/conformance/BRW-4/reject_method_defeats_disjoint_fields.em` reports `E3025` and reports `E3022` when the fix is reverted — checked by doing exactly that; `docs/errors/E3025.md` holds the page |
| D-041 | **Moves out of borrowed places are unchecked.** `x = r.inner` through a shared `ref` and a field move out of a borrowed call parameter both compile; each value then drops twice — once with its new owner, once with its original place. `[EXP-6]` names `E3013` ("cannot move out of a reference") for exactly this, and it has no emitter outside drop bodies. A borrowed parameter arrives as a bitwise copy whose borrowed-ness reaches no analysis (no loan, no marker), so the callee cannot tell it apart from owned data | `[EXP-6]`, `[BRW-1]`, `[OWN-3]`, `[FN-1]` | **fixed** | borrowed-ness threaded HIR→MIR (`borrowed_params` on `Body`, from the parameter modes); `check_borrowed_moves` in `compiler/ember_analysis/src/drops.rs` reports every owning `Move` through a safe-reference `Deref` and every owning `Move` out of a borrowed value parameter (whole or field, assignment and call-argument/return paths) as `E3013`/O2 — raw-pointer derefs exempt (`[UNS-*]` territory), `drop`'s `self` keeps D-030's messages (skipped, never double-reported), moves of values owning nothing stay legal. **Verified:** `tests/conformance/EXP-6/` holds five rejects that each compile with the check reverted (exit 0; the suite fails "expected compilation to fail, but it succeeded") plus a run-pass accept whose stdout mutation fails the suite — every new case broken red once, checked by doing exactly that (the return-path case additionally fails on a deliberately wrong code, pinning `E3013` specifically). `docs/errors/E3013.md` holds the page (baseline shrinks by one). The specification is untouched: `[EXP-6]`/`[FN-1]` already forbid both shapes (cf. D-035). Adjacent symptom, lost-write only: writes through borrowed places go nowhere (same by-copy ABI), e.g. through a borrowed `o` — needs a spec reading on the intended code before filing separately (not ERR-041) |
| D-042 | **Partial moves out of owned places double-destroyed at scope end.** `[EXP-6]` allows moving a field out of a plain struct ("leaves the struct partially moved"), but drop elaboration tracked whole-local movedness only — `moved_by_operand` discarded field projections — so the scope-end drop destroyed the moved-from field again. `x = o.inner` emitted both the new owner's drop and `o.inner`'s old drop. `E3042` was registered and shape-mapped (O4) but unreachable | `[EXP-6]`, `[OWN-2]`, `[OWN-3]`, `[DRP-2]` | **fixed** | `drops.rs` now builds recursive move paths for plain structs/tuples, carries `Live`/`Moved`/`Partial` possibilities through control flow, preserves disjoint siblings, restores an aggregate after field reinitialisation, expands a partial aggregate's cleanup into reverse-order live-field drops, and maintains per-path flags for conditional moves including call terminators. Whole-value use after a partial move emits `E3042`; `docs/errors/E3042.md` is the page. **Verified:** six adversarial cases in `tests/conformance/EXP-6/` cover exact destructor count, conditional flags, call-argument transfer, nested paths, reinitialisation, sibling use and `E3042`; the old implementation failed the first probe with `1, 2, 1` and failed the conditional probe with `10, 1, 2, 1`. Removing projected-move tracking makes the new suite red. The normative specification and frozen H8 target were unchanged: the compiler was wrong |

## 2026-09-09 — block I, `Cell[T]`

| # | Defect | Rule | Status | Fixed in |
|---|---|---|---|---|
| D-035 | **Overwriting a place ran no destructor at all.** `[OWN-5]`: "Overwriting a place that holds a live value drops the old value first (after evaluating the new value)." `[OWN-2]` names three occasions to drop — scope end, overwrite, temporary at statement end. D-027 built the first and D-031 the third; **the second was never built.** `r = R(1)` then `r = R(2)` ran `R(1)`'s `drop` zero times, and `a = Array[i32]()` over a live array emitted one `ember_vec_free` for two allocated buffers, leaking the first. Silent in both directions: no diagnostic, and for the container the program's output is identical either way | `[OWN-5]`, `[OWN-2]` | **fixed** | ADR-020. `lower_assign` in `ember_mir/src/lower.rs`: the new value goes to a temporary, then `Drop`, then the store — the order the rule gives. Which of those drops survives is left to `[OWN-3]`'s existing elaboration, so a first initialisation (local `Moved` after `StorageLive`) deletes it and a conditionally-moved local gets a flag. **Verified:** two cases in `tests/conformance/OWN-5/`, both of which lose their first line of output when `lower_assign` is reverted to `lower_into` — checked by doing exactly that. The buffer case is written observably (a `Bag` whose `drop` prints its length) rather than as `assert-c: contains("ember_vec_free")`, which would have passed on the broken compiler: it emitted one free and `contains` cannot count |
| — | **Why the specification does not move.** `[OWN-5]` states the requirement and the order in one sentence and admits no other reading; the compiler simply did not implement it. An implementation gap is not a spec defect, so there is no errata entry and `ember-spec.md` is untouched | `[OWN-5]` | n/a | — |
| — | **Why `tests/conformance/OWN-5/` did not catch it.** The directory's one case, `accept_the_new_value_is_evaluated_first.em`, tests the parenthetical only, and it is built so the main clause cannot fire: `x = grow(x)` **moves** `x` into the call, so nothing is live at the store and the missing drop is unobservable. A rule stated in two clauses needs a case per clause | `[OWN-5]`, `[TST-4a]` | n/a | — |

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
| D-011 | `E3064` (two independent regions in one view struct) is registered and emitted by nothing | `[LT-2]`, `[DIA-7a]` | **fixed** | **`E3064` was reachable all along and these programs were being reported as B3.** A `@view struct` bundling two views holds *both* loans for as long as any part of it is live, so touching it keeps the shorter one alive even where only the longer-lived field is read — which is exactly `[DIA-7a]`'s shape B13. The rejection was already right; the *classification* was wrong, and so was the advice: B3 says shorten the borrow's last use, and the use keeping the loan alive is of the other field. The conflict now reads the type of the local holding the loan and reports `E3064` with B13's help when it bundles two views. `tests/conformance/LT-2/` holds the reject and, beside it, the program B13's help prescribes — compiled and run, because a help that does not work is worse than none |

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
| D-018 | `[RNG-4]`'s range tracking derived a range for constants only, so `[RNG-10]`(d) — "a value whose `[RNG-4]` range is contained in the target's" — admitted a constant and nothing else | `[RNG-4]`, `[RNG-4a]`, `[RNG-10]` | **fixed** | `range_of` returns an **interval**, not a point. The fact carrying most of the weight is not arithmetic: a value at a range type is in its declared range, which `[RNG-9]` makes an invariant and `[RNG-4]` may assume. Intervals propagate through `+`, `-`, `*` and survive a binding (a fact that dies at the `=` is worth nothing), and a later write to the local drops it. `[RNG-4a]` is honoured: a derived fact is kept only where the operation provably cannot overflow its representation, in any profile. Four conformance cases in `tests/conformance/RNG-4/`. **Still open as later work**: branch refinement (the arms of an `if` that compared the value) needs a flow-sensitive pass, and `min`/`max`/`clamp` need `std.math` |

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
| D-027 | **A declared `drop` method never ran.** `drop_lines` dropped a struct's *fields* and never called the type's own destructor, so a struct whose only claim on `needs_drop` was its `drop` method produced no lines at all and the `Drop` statement lowered to nothing. Silent: no diagnostic, and output that looks right because the destructor's effects are simply absent | `[OWN-2]`, `[DRP-1]` | **fixed** | the call is emitted **before** the fields, as the rule orders it, so the destructor still sees a whole value — which is the only reason it can read its own fields. Enums too, unit-only ones included. `tests/conformance/OWN-2/` covers the call and reverse declaration order. `drop_symbol` in the backend must agree with `method_symbol` in typeck and nothing checks that it does; those tests are what would notice |
| D-028 | A move inside a loop was reported as `E3040`, shape O1 ("use after move"), where `[OWN-4]` specifies `E3041` and `[DIA-7a]` keys it to shape O3 ("move in a loop"). The rejection was right and the **help was wrong** — O1 says clone it or borrow it, when the problem is that the next iteration finds nothing there | `[OWN-4]`, `[DIA-7a]` | **fixed** | the reporter learns whether its block can reach itself, and a *maybe*-moved local inside a cycle is O3. The states discriminate cleanly: a move and a use in one iteration leaves it definitely moved (O1); a move reaching its own use round a back edge leaves it maybe-moved, because the loop head joins "not yet moved" with "moved last time". `tests/conformance/OWN-4/` has the reject and the escape hatch the rule names |
| D-029 | **Calling `drop` explicitly was accepted, and ran the destructor twice.** `[DRP-1]` says `drop` "is invoked exactly once per value at the end of its life. It may not be called explicitly (`E3070`)" — and an explicit call does not replace the scope-end one, it adds to it. For a struct owning an `Array[T]` that is a **double free**, whose second run also reads the buffer after freeing it, with no `unsafe` in the program | `[DRP-1]`, `[OWN-2]` | **fixed** | `E3070` at the call, with `mem.drop(owned x)` named as the way to end a life early. `tests/conformance/DRP-1/` has the reject and the accept that must keep working |
| D-030 | `[DRP-5]` — a `drop` body moving a field out of `mut self` was accepted, where the rule says it may not (`[EXP-6]`). The field was then dropped again after `drop` returns | `[DRP-5]`, `[EXP-6]` | **fixed** | `check_drop_moves` in `ember_analysis/src/drops.rs` reports every `Move` out of `drop`'s `self` (field moves as `E3010`, whole-through-borrow moves as `E3013`); only a receiver named `self` counts, so a free function named `drop` is untouched. **Verified:** `tests/conformance/DRP-5/` holds two rejects that compile with the check reverted, plus an accept proving reads still work — checked by doing exactly that |
| D-031 | **Temporaries were never dropped.** `[DRP-3]`: "Temporaries drop at the end of the enclosing statement." A bare `R(1)` ran no destructor at all, and a temporary owning an `Array[T]` leaked its buffer. Silent — the program's output is identical either way, which is why it survived block A's leak work | `[DRP-3]`, `[EXP-4]` | **fixed** | every temporary comes through one function, so it registers there rather than at each construction site; a site that forgot would be invisible. Dropped at statement end, last first, kept separate from `[OWN-2]`'s scope-end list because the two differ in *when* and running them through one list gives a temporary the wrong lifetime in either direction. `tests/conformance/DRP-3/` asserts on the emitted C, since the output does not change |
| D-032 | **`[BRW-5]`'s constant-index exemption was unreachable.** The rule says `ref mut a[i]` and `ref mut a[j]` conflict "unless both indices are constants and different", and `overlaps` is written to tell two `ConstIndex` projections apart — but `lower_index` put *every* index into a runtime slot, so no `ConstIndex` was ever produced by an index expression and `v[0]`/`v[1]` were rejected as though the indices might be equal | `[BRW-5]` | **fixed** | a constant index lowers to `ConstIndex`. The bounds check is still emitted: for an `Array[T]` the *length* is dynamic even when the index is a literal, so what the constant buys is disjointness, not the elision of a check. A side benefit — the diagnostic now names `v[0]` instead of `v[…]` (`[DIA-2]`) |
| D-033 | For an `Array[T]`, a second thing blocked the same exemption: `lower_index` reads the length through the container's internal `Field(1)` to bounds-check, and `overlaps` treated that read as overlapping an element borrow | `[BRW-5]`, `[BRW-4]` | **fixed** | a `Field` beside an `Index` is disjoint. Ember has no `v.len` *field* — only a `len()` method — so that pair is only ever the compiler's own header access, never something a program can write. Nothing user-visible is weakened: `push` takes `mut v` with an empty projection, which overlaps every place under `v`, so the reallocation hazard is still caught, and `v.len()` while an element is borrowed is still rejected because `[BRW-4]` says a method takes all of `self`. Both are conformance cases |
| D-034 | Two mutable index borrows of an `Array[T]` reported `E3021` ("cannot be read while mutably borrowed") where `[DIA-7a]` keys shape B1, "two mutable indices", to `E3022`. The fixed-array path already used `E3022` | `[BRW-5]`, `[DIA-7a]` | **fixed** | a consequence of D-033: with the header read no longer overlapping, the conflict reported is the second *borrow* rather than the bounds check's read, which is both the right code and the right description |
