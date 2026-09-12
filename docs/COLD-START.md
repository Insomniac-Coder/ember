# Cold start — read this first

State as of 2026-09-12. Read `docs/MIGRATION-0.9.5.md` next for the active
migration, then `docs/HANDOFF.md` §0 for the process and verified implementation
state. `docs/MIGRATION-0.8.3.md` remains historical context for the original
phase order.

| Ledger | Answers |
|---|---|
| `docs/DEFECTS.md` | every compiler defect, its status, **and how the fix was verified**. Its header carries the four-way sort below |
| `docs/DEVIATIONS.md` | where the compiler knowingly differs from the document, and why |
| `docs/spec-amendments.md` | every difference between the owner's file and the normative copy, each with a class |
| `docs/spec-errata.md` | defects in the *document*, and the reading taken |
| `docs/DECISIONS.md` | ADR-001..029 |
| `docs/OWNER-QUEUE.md` | **questions an agent may not answer.** ODR-001/002 and ODR-004..009 are closed; ODR-003 is deferred editorial; there is currently no open owner semantic decision |

---

## 1. State

**`main`, pushed, clean.** `https://github.com/Insomniac-Coder/ember.git`

    178 tests green      cargo test --workspace
    0 warnings           cargo build          <- keep it there
    6 gates green:
      python tools/hardening_check.py      no undeclared change to the specification
      python tools/rule_index.py           rules, codes, references, baselines
      python tools/spec_check.py           fenced `ember` blocks parse
      python tools/error_pages.py          every documented fix compiles
      python tools/check_branding.py       no hard-coded project names
      python tools/split_spec.py --check   docs/spec/ is the split of the source

 80 conformance rule directories, 229 cases. 58 defects recorded, **none open**.
 **4 open deviations** (D1–D4; D5 and D6 closed 2026-09-10).
**ERR-050 / ODR-009 is closed by the H10 owner rulings** on `Zeroable`,
`MaybeUninit`, and Arena bulk initialization. Their compiler/runtime/conformance
work remains open. ERR-047 and ERR-048 were closed
by the H7/H8 owner rulings; ERR-049 was closed by the H9
Arena-provenance ruling; earlier ERR-041 and ERR-043 were
decided on 2026-09-10 and ERR-042 was withdrawn as wrong. See `HANDOFF.md`.
Ratchets in
`tools/*_baseline.json` may shrink and never grow; `--allow-growth` needs a
reason in the commit message.

**The working process is `docs/HANDOFF.md` §0.0** — authority, versions, the
five-way sort, the four-document write path, probe-first, test discipline,
claim discipline, escalation, scope reporting, and the pre-commit checklist.
Read it before starting a task, not after.

**0.9.5 status:** the owner-supplied H2 and H3 files are preserved unchanged
under `docs/spec-source/as-received/`. The repaired and frozen
`Ember_v0.9.5_Hardened_4.md` remains the source-gap audit record. The owner then
supplied `[LT-8]`–`[LT-13]`, frozen as H5, and resolved the mutable-helper
boundary in H6. The owner then specified ordinary borrowed/default, `mut`, and
`owned` callable parameter modes and explicit `mut` callback modes in
H7. The owner then closed the helper-input and `Callable` bridge boundaries in
`Ember_v0.9.5_Hardened_8.md`: mutable helper inputs are `mut` reborrows, no
helper consumes a view, and callable modes remain compile-time canonical type
metadata through the existing abstraction. H9 then resolved the Arena-backed
return-provenance boundary with the narrow `@borrows(arena)` exception. H10
now resolves deterministic bulk initialization and the complete `Zeroable` /
`MaybeUninit` safety/API boundary. H10 is the new frozen development
target, but not yet the normative repository source. It retains the owner-
selected multi-region-view target and separate shared/all-mutable callback-
helper families; no mixed overloads are implied. No 0.9/0.9.5 implementation
or conformance is implied by the target's version label.

## 2. The rules. Read these before touching `docs/spec-source/`

Learned expensively. Two amendments were withdrawn for breaking them.

1. **The specification is the contract and is never edited to make the compiler
   agree with it.** Where the two disagree and the rule is sound, *the compiler
   moves*.
2. **Where the document contradicts itself, neither side moves** until the owner
   rules. Do not pick the reading that matches what is already built — that is
   how an implementation workaround becomes the language.
3. **`docs/spec-source/as-received/Ember_v0.8.3_spec.md` is never edited**, ever.
   md5 `2bdffee6510b8668cf828185266efedb`.
4. **Every edit to `ember-spec.md` declares one of five classes**, and only four
   are permitted in a hardening:

       SEMANTICALLY NEUTRAL CLARIFICATION   permitted
       IMPLEMENTATION INVARIANT             permitted
       SOURCE RECOVERY                      permitted
       EDITORIAL REPAIR                     permitted
       OWNER-APPROVED SEMANTIC CHANGE       forces a language version bump

   The line: **the moment an edit answers *what Ember means* rather than *how to
   implement what Ember already means*, it stops being a hardening.**
5. **`tools/hardening_check.py` enforces declaration**, not correctness. It
   refused three of my own correct-but-undeclared edits. It would not have
   caught either withdrawn amendment — both were declared. It is a floor.

**Both withdrawals had the same shape and you will be tempted the same way:**
the analysis was right, and the answer went into the normative text instead of
the ledger. A4 wrote a determination about `E3064` into `[LT-2]`. A6 wrote an
unruled reading of `[FN-1]` into the rule, with "not yet ruled on by the owner"
sitting directly above it in the ledger — *the document carries the sentence, not
the caveat.*

### Which ledger a finding belongs in

| Kind | Where | Who moves |
|---|---|---|
| rule clear, compiler wrong | `DEFECTS.md` | the compiler |
| maybe right, never proved | a conformance case | nobody |
| two rules disagree, neither governs | `spec-errata.md` | **nobody, until the owner rules** |
| rule clear, compiler knowingly differs | `DEVIATIONS.md` | the compiler, later |

## 3. Versioning

The document is **v0.8.5_Hardened_1**. Two numbers move independently:

* **language version** — moves when the set of accepted programs changes, and
  **resets the hardening number to 1**. 0.8.4 exists for exactly one change: S1,
  the owner's resolution of ERR-044. **0.8.5 exists for three** — S2
  (`UnsafeCell` becomes a real primitive), S3 (`RefCell` is never `Copy`), S4
  (`[FN-1a]`) — all owner rulings of 2026-09-10, all additive.
* **hardening number** — moves when the document gains implementation detail and
  no rule changes meaning. **The number itself is the owner's call**: `8101389`
  records the last one that way, and `207c69f` is the shape to follow when the
  file has outrun its header and the decision has not been made.

`LANGUAGE_VERSIONS` in `compiler/ember_parser/src/lib.rs` accepts `"0.8.3"`,
`"0.8.4"` and `"0.8.5"`; each is additive, so nothing valid became invalid.
`docs/spec-source/Ember_v0.8.5_Hardened_1.md` is the current frozen snapshot —
the next revision diffs against **that**, not against as-received.
`Ember_v0.8.4_Hardened_1.md` and `Ember_v0.8.4_Hardened_2.md` are kept as prior
baselines. The working source and the current snapshot are **identical** right
now; where they ever differ, the working source governs for implementation and
`docs/HANDOFF.md` §0.17 is the authoritative statement of which artifact is
normative for what.

The current development target is `0.9.5_Hardened_10`, per the owner's
instruction that each issued hardening pass increments the hardening number.
H9 is the immediate predecessor and remains frozen. H5 recovered the missing
source; H6 records the mutable-helper family; H7 records callable parameter
modes; H8 records helper input modes and compile-time mode preservation through
`Callable`; H9 records Arena-backed return provenance; H10 records the complete
Arena initialization and `MaybeUninit` contract. The 0.9.5 multi-region-view
feature itself is the owner-selected language change. Any H10 correction must
be H11.

**Do not couple a tool to a version string.** `rule_index.py` decided which
change log was current by matching `"0.8.3"` and would have silently stopped
checking the current section the moment the version moved. It was one commit
from happening.

## 4. Where the work is: finish Phase 2

Exit criteria, from Part XXI: *all `[OWN-*]`, `[BRW-*]`, `[LT-*]`, `[DRP-*]`,
`[SPN-*]`, `[CELL-*]`, `[DIA-7..10]` tests; milestone M2; zero unclassified
borrow errors across the whole corpus.*

M2 exists (`tests/milestones/`) and no unclassified log is produced, so what is
left is coverage:

    OWN   6/8    missing OWN-6 (mem.take/replace/swap/forget), OWN-8 (Clone,
                 @derive(Clone)) — both are unbuilt features, not missing tests
    BRW   7/9    missing BRW-8 (an ABI decision, "never observable" — assert on
                 emitted C), BRW-9
    LT    6/10   missing LT-1b (L3014, an opt-in lint with no opt-in mechanism),
                 LT-2a, LT-5, LT-7 (callback regions)
    DRP   4/6    missing DRP-4 (needs effects, Phase 4),
                 DRP-6 (Box/handle/Shared — Phase 3). DRP-5 has cases since D-030
                 was fixed (drop-body moves rejected)
    SPN   3/3    done
    OWN-5        both clauses now, after D-035 — see the note below
    CELL  11/13  Cell/RefCell cases exist for every currently buildable rule;
                 CELL-3 and CELL-8 (`!Sync`) wait for Send/Sync/threads. See §5
    DIA   0/5    needs tests/ui snapshots — see §6

**A directory named for a rule is not coverage of the rule.**
`tests/conformance/OWN-5/` existed and passed while the compiler ran no
destructor on an overwrite at all (D-035). Its case tested the sentence's
parenthetical and was written so the main clause could not fire. When a rule
states two things, count two cases.

### The method that found twelve defects — keep using it

**Probe every normative rule with a minimal executable program before changing
any implementation code.** Not "does the suite pass" — write the three-line
program the rule describes and check the number. Four of the twelve were
*silent*: a declared `drop` that never ran, temporaries that never dropped
(output byte-identical either way), an explicit `drop()` that double-freed, a
write through a shared `ref` caught only by clang's `const`.

Where behaviour cannot be observed from output, **assert on the emitted C**
(`#$ assert-c: contains("ember_vec_free")`).

## 5. Next task: implement H10's Arena initialization foundation

**`Cell[T]` and `RefCell[T]` are both built.** The remaining buildable
Cell/RefCell coverage closed on 2026-09-12: `[CELL-6a]` now runs both fallible
borrow paths in debug, release, and shipping; `[CELL-9]` pins the one-word
counter in generated C and the every-profile runtime check; `[CELL-10]` pins
ordered positive and forbidden negative suggestions. The class-only
`exclusivity = "unchecked"` interaction has no reachable trigger until classes
and package exclusivity exist; it is a dependency gap, not a current compiler
defect. `[CELL-3]`/`[CELL-8]` remain correctly blocked on `CELL-SYNC-1`.

**D-042 is fixed:** drop elaboration now carries recursive per-field move paths,
splits partial aggregate cleanup into live-field drops, uses per-path flags for
conditional moves, preserves disjoint siblings, and emits `E3042` for a whole-
value use after a partial move. Six adversarial `[EXP-6]` cases pin exact drop
counts, conditional and call-terminator flags, nesting, reinitialisation and the
diagnostic. This is the ownership foundation 0.9.5 `[LT-38]` needs; it does not
itself implement region vectors or any 0.9.5 view rule.

The same probe found and closed **D-043**: `owned` parameters were not in the
callee's destruction scope and leaked when not moved onward. `[OWN-2]` now has
a direct move-onward/not-moved parameter test. Neither fix changed the
specification or frozen H8 target.

The `[CELL-10]` probe found and closed **D-044**: default-mode value parameters
lost their shared-borrow provenance at type checking, so writes compiled and
changed only a private ABI copy. Parameter modes now remain explicit type-
checker state across assignment, `ref mut`, mutable-view, `mut self`, and
`mut`-argument paths. E3023/B4 leads with the structural `mut` repair and only
offers costed interior mutability where non-simultaneity is proven. The rule was
already clear; no specification or ADR changed.

**What follows in Gate B:**

1. ~~**D-038:** implicit `String`→`str` coercion.~~ **Fixed.** The coercion
   reuses `view_of`, region elision, MIR view verification, and the D-037
   backend path. Three mutation-tested `[SPN-1]` cases pin assignment, argument,
   NLL, mutation, and return-region behavior.
2. **`Arena` core (`[ARN-1]`–`[ARN-4]`, `[ARN-6]`, `[ARN-7]`, `[LT-4]`).**
   **Implemented.** Growing, fixed, and scoped compiler-known arenas have
   stable aligned bump allocation, reset/rewind, nested LIFO scopes, drop-free
   enforcement, `E3090`/`E3096`, and H9 wrapper provenance. Generic
   instantiations re-check `[ARN-3]`, so `alloc(Array[T]())` cannot hide behind
   an opaque type parameter.
3. **Remaining Arena surface.** `alloc_array` needs `Default`/`Zeroable`;
   `alloc_uninit` needs `MaybeUninit`; `ArenaArray`/`ArenaMap` need the
   corresponding collection machinery; allocation effects and `ThreadArena`
   wait for the effects/concurrency phases. **ODR-009 is closed:** H10 defines
   the zero-bit validity set, `MaybeUninit` state transitions, canonical APIs,
   and exact bulk-allocation fallback/drop/rollback behavior. Explicit type
   arguments on methods remain the separate compiler gap `GEN-METHOD-1`.
   Implement the H10 contract exactly; do not substitute ad-hoc initialized
   bytes or implementation-defined conversion APIs.

Arena is **not a third interior-mutability primitive**. It is a region
allocator, and amendment A13 records that it shares implementation machinery
with `Cell`/`RefCell`, not their semantic concept.

**Blocked, correctly:** `[CELL-3]`/`[CELL-8]` (`!Sync`) on `CELL-SYNC-1`;
`Cell.take` on `CELL-DEF-1`. Neither rule was softened to fit.

**Open compiler defects:** none.

**`UnsafeCell` is specified and unbuilt** — `[UNS-10]`/`[UNS-10a]`/`[UNS-10b]`,
0.8.5. **Do not build `RefCell` on it**; ADR-019's compiler-known route stands.

## 6. After that: `[DIA-7..10]` and `tests/ui/`

A Phase 2 *exit* criterion and currently zero. `compiler/ember_diag/src/shapes.rs`
holds every shape; `[DIA-13]` wants a rendered snapshot per shape plus
`[PHIL-8a]`'s `.fixed.em` companion, and `tests/ui/` is empty.

Note the pattern three defects took: **the compiler rejects the right program
under the wrong shape**, so the *help* is wrong. D-011 (`E3064` reported as B3),
D-028 (a loop move as O1 not O3), D-034 (two mutable indices as `E3021` not
`E3022`). `[DIA-7a]` makes the shape part of the conformance contract, so a
snapshot per shape is what stops the next one.

## 7. Open, and the owner's to answer

**Everything that stood here on 2026-09-09 has been ruled on or closed.**
ERR-041 and ERR-043 were decided by the owner on 2026-09-10; ERR-042 was
withdrawn as wrong; D5 closed with the compiler right; D-030 was fixed. What
remains:

**The queue lives in `docs/OWNER-QUEUE.md`.** ODR-001, ODR-002, and ODR-004
through ODR-009 are closed. ODR-003 is deferred editorial work. **There is no
current open owner semantic question.**

* **ODR-001 — CLOSED.** `[UNS-10]`'s `UnsafeCell` API stays exactly as written.
* **ODR-002 — CLOSED as tooling work, not spec work.** The six rules stayed
  untouched. `RIDX-1` landed in `6c77723`; the extractor now recognises the
  legitimate structural forms and its tests reject reference-shaped false
  definitions.
* **ODR-003 — deferred editorial cleanup.** No semantic change for 0.8.5; the
  next suitable revision classifies each `[FFI-17]` item A/B/C/D against its
  authoritative rule.
* **ODR-004 — CLOSED.** The owner supplied `[LT-8]`–`[LT-13]` on 2026-09-12;
  H5 reproduces them with transport-only Markdown normalization and preserves
  H4 unchanged.
* **ODR-005 — CLOSED.** H6 adds distinct all-mutable
  `with_views2_mut/3_mut/4_mut` helpers using canonical `MutSpan[T]`. Shared
  helpers remain shared; no mixed `Span`/`MutSpan` overloads are implied.
* **ODR-006 — CLOSED.** H8 makes mutable helper inputs `mut` reborrows; shared
  inputs stay borrowed and neither family consumes a view.
* **ODR-007 — CLOSED.** H8 retains `Callable[Args, R]` and preserves the full
  callable mode vector as compiler-known, compile-time-only canonical type
  metadata. No new public generic or runtime mode mechanism is implied.
* **ODR-008 — CLOSED.** H9 permits `@borrows(arena)` only when a returned view
  is proven to use storage owned by that growing Arena parameter. Arena remains
  non-view; arbitrary non-view parameters remain E2031.
* **ODR-009 — CLOSED.** H10 defines `Zeroable`, `MaybeUninit`, `alloc_array`,
  and `alloc_uninit`, including bit validity, canonical value/span transitions,
  `!needs_drop`, deterministic fallback, E2040, rollback, and phase ordering.
  The remaining Arena bulk surface is now ordinary implementation work.

* **Historical tooling lesson from ODR-002.** Six valid rules (`[TYP-26]`,
  `[IFC-2]`, `[HND-2]`, `[GPU-7]`, `[VER-7]`, `[CTL-3a]`) used structural forms
  the old extractor could not read. The specification was not rearranged for
  the tool. `RIDX-1` taught the extractor those forms and pinned the
  definition/reference boundary with tests. Withdrawn ERR-042 retains the
  original inventory.
* **No open compiler defect.** **D-038** now routes implicit `String`→`str`
  through the same explicit-borrow producer as `as_str()`. **D-042** (partial
  moves), **D-043** (owned parameters leaked), and **D-044**
  (writes through borrowed value parameters) are fixed with adversarial
  conformance evidence. **D-041** (moves out of
  borrowed places unchecked — `x = r.inner` compiled and the value dropped
  twice) was the serious one and is **fixed** this turn: borrowed-ness is
  threaded HIR→MIR and owning moves out of borrows are `E3013`.
* **Four open deviations**: D1 (`[RNG-5a1]`'s generated operator impls), D2
  (`[CLO-6]`'s `owned f`, the live residual of the closure work), D3
  (`extern class` parses and is refused), D4 (`E9012` registered and never
  emitted). D5 and D6 are closed.

## 8. Traps paid for

* **A stale binary makes a gate lie.** `error_pages.py` preferred
  `target/release/` over `debug` and validated pages against a compiler from an
  earlier session. Newest wins now, and it says which it used.
* **An assertion can satisfy itself.** A `compile-fail` case writes its expected
  code on the line that provokes it, and the renderer echoes that line — so
  `stderr.contains("E1050")` matched the test's own annotation. Every trailing
  expectation asserted nothing, twice over, because the harness did not parse
  them either. Found by writing a case with a deliberately wrong code and
  watching it pass. **After adding a test, break it once and watch it go red.**
* **A Python line-continuation eats a Rust `\`.** Two help strings shipped with
  eighteen spaces mid-sentence. Write Rust through the Write tool, never a shell
  heredoc, and check any `\` that survives a Python string.
* **A checker that knows one shape of a rule reports thirty false positives.**
  The document states a rule in at least four shapes. `hardening_check.py`'s
  attribution was wrong four ways; each time it named a *real* rule and the
  wrong one.
* **A new value-producing form must be walked through every analysis by hand.**
  D-022 was found by asking whether the borrow checker saw the view, not by a
  test failing.
* **CRLF.** The Edit tool writes CRLF on this host while `.gitattributes` pins
  LF. Normalise every changed file before every commit.
* **A name the compiler invents ends up in the C verbatim.** `Cell`'s payload
  field was called `$value` so that no Ember program could spell it. `$` in a C
  identifier is a **compiler extension**, which `[CG-C-1]` forbids relying on —
  and it compiles clean under `-Wall -Wextra`, so nothing said so until
  `-pedantic` was tried by hand. Privacy, not spelling, is what makes a field
  unreachable. **Read the emitted C after anything that puts a new identifier
  into it.**
* **A conformance directory named for a rule can test almost none of it.**
  `tests/conformance/OWN-5/` passed for as long as the rule went unimplemented
  (D-035): its one case covered the sentence's parenthetical, and was written so
  the main clause could not fire. Before trusting a green directory, read the
  rule and count its clauses.
* **`grep -c` returning zero exits 1**, which reads as a failed command in a
  chained shell line and can be mistaken for a failing test run. Twice.
* **A gate needs its real arguments.** `python tools/split_spec.py --check`
  alone throws `IndexError`; CI passes it the source and the output directory.
  `.github/workflows/ci.yml` is the authority on how each gate is invoked.

## 9. Where the code is

    compiler/ember_typeck/src/lib.rs        largest file: ranges, spans, closures,
                                            visibility, callable generics, cells
    compiler/ember_analysis/src/borrows.rs  NLL, loans, elision, the [DIA-7] shapes
    compiler/ember_analysis/src/drops.rs    moves, drop flags, [OWN-4]'s loop shape
    compiler/ember_analysis/src/regions.rs  region variables and the constraint graph
    compiler/ember_mir/src/lower.rs         MIR lowering; statement_temps is [DRP-3]
    compiler/ember_mir/src/verify.rs        structural verifier and verify_views
    compiler/ember_diag/src/codes.rs        the code registry, one entry per code
    compiler/ember_diag/src/shapes.rs       [DIA-7a]'s shape catalogue
    tools/hardening_check.py                the gate on the specification itself
