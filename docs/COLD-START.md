# Cold start — read this first

State as of 2026-09-09, everything pushed. Then `docs/HANDOFF.md` for the phase
plan (its own "Start here" is marked superseded — ignore it) and
`docs/MIGRATION-0.8.3.md` for the analysis that set the order.

| Ledger | Answers |
|---|---|
| `docs/DEFECTS.md` | every compiler defect, its status, **and how the fix was verified**. Its header carries the four-way sort below |
| `docs/DEVIATIONS.md` | where the compiler knowingly differs from the document, and why |
| `docs/spec-amendments.md` | every difference between the owner's file and the normative copy, each with a class |
| `docs/spec-errata.md` | defects in the *document*, and the reading taken |
| `docs/DECISIONS.md` | ADR-001..020 |

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

61 conformance rule directories, 154 cases. 55 defects recorded, **2 open**
(D-038, D-041). 5 deviations. 2 errata awaiting the owner. See `HANDOFF.md`.
Ratchets in
`tools/*_baseline.json` may shrink and never grow; `--allow-growth` needs a
reason in the commit message.

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

The document is **v0.8.4_Hardened_1**. Two numbers move independently:

* **language version** — moves when the set of accepted programs changes, and
  **resets the hardening number to 1**. 0.8.4 exists for exactly one change: S1,
  the owner's resolution of ERR-044.
* **hardening number** — moves when the document gains implementation detail and
  no rule changes meaning.

`LANGUAGE_VERSIONS` in `compiler/ember_parser/src/lib.rs` accepts `"0.8.3"` and
`"0.8.4"`; 0.8.4 is additive, so no 0.8.3 program became invalid.
`docs/spec-source/Ember_v0.8.4_Hardened_1.md` is the frozen snapshot — the next
hardening diffs against **that**, not against as-received. The working source
`docs/spec-source/ember-spec.md` currently runs one declared editorial repair
(E5, `[EFF-18]` gains `Nondet`) ahead of the frozen snapshot; for
implementation the working source governs — see `docs/HANDOFF.md` §0.17, which
is the authoritative statement of which artifact is normative for what.

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
    LT    5/10   missing LT-1b (L3014, an opt-in lint with no opt-in mechanism),
                 LT-2a, LT-4 (Arena), LT-5, LT-7 (callback regions)
    DRP   4/6    missing DRP-4 (needs effects, Phase 4),
                 DRP-6 (Box/handle/Shared — Phase 3). DRP-5 has cases since D-030
                 was fixed (drop-body moves rejected)
    SPN   3/3    done
    OWN-5        both clauses now, after D-035 — see the note below
    CELL  4/12   Cell is built (CELL-1, 2, 4, 11). RefCell is CELL-5..8,
                 CELL-9, CELL-10, CELL-6a; Arena is ARN-*. See §5
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

## 5. Next task: `RefCell[T]`, then `Arena` (block I, the rest)

**`Cell[T]` is done, 2026-09-09.** `Cell()`, `set`, `replace`, `into_inner`,
`get` (`T: Copy`) and `update`'s `T: Copy` arm; `[CELL-1]`, `[CELL-2]`,
`[CELL-4]` and `[CELL-11]` have conformance cases. `take` and `[CELL-3]`'s
`!Sync` are filed as `CELL-DEF-1` and `CELL-SYNC-1` in `docs/BACKLOG.md` —
`Default`, `Send`, `Sync` and threads do not exist yet, and neither rule was
softened to fit. The reasoning is in `docs/HANDOFF.md`'s **Block I** section;
read it before starting `RefCell`, because three of its findings apply directly.

ADR-019 still governs: `[CELL-9]` names `UnsafeCell` as the primitive and the
document defines it nowhere (ERR-043), so all three of these are
**compiler-known**, as `Array`, `Span`, `Option` and `Result` are under
Part XX.1. Amendment A13 records that the three share the implementation
concern and **not** the concept — do not build `RefCell` by generalising
`Cell`.

**What carries over from `Cell`:**

* The **transparent-struct shape**. `cell_of` interns a `StructDef` per `T` and
  records it in `cells` on the `Checker`; `RefCell` wants the same, with a
  second field for `[CELL-5]`'s one-word borrow counter. `[CELL-4]`'s `Copy`
  and `Drop` questions answered themselves off the field for `Cell`, and the
  same machinery will answer them for `RefCell` — but whether `RefCell[T]` may
  be `Copy` is an **open question to escalate, not a settled rule**: copying
  the counter would fork the borrow state (sound inference), and Part IX
  states `[CELL-4]` for `Cell` only. See `HANDOFF.md` §0.14.
* **Privacy is the mechanism**, not an unspellable name. `declaring_module:
  usize::MAX` plus a private field refuses read, write and `ref` everywhere with
  `E1020`. A `$`-prefixed name breaks `[CG-C-1]` — the backend writes field
  names into the C verbatim and `$` in an identifier is a compiler extension,
  which only `-pedantic` reports.
* **Order rules need `assert-c-order`**, added to the harness for this.
  `assert-c` matches one line and cannot express "a before b"; a drop that
  re-enters the value being replaced is unobservable in output, and no safe
  program can build the back-pointer that would make it observable. Anchor the
  needles on text that survives MIR renumbering, and remember that a bare
  function name matches its own prototype at the top of the file.

**What `RefCell` adds, and it is the hard half:** `[CELL-5]`'s `borrow` and
`borrow_mut` hand out `Ref[T]`/`RefMut[T]`, which `[CELL-7]` makes **view
types** whose region borrows the cell and whose `drop` releases the borrow
state. So unlike `Cell`, something *does* escape, `[TYP-15]` applies to it, and
the region work in `regions.rs` is load-bearing. `[CELL-9]` puts the check in
**every** profile and `[CELL-6a]` forbids any profile making `try_borrow`
infallible. `[CELL-7]`'s `L3011` (`RefCell` guard held across a call) fires per
the rule — registered, emitted by nothing, and part of this task's work, not
blocked and not opt-in (`LNT-CFG-1` is about `[LT-1b]`'s `L3014`).

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

* **D5** — `[FN-1]` says a `mut` argument "MUST be a mutable place"; Part
  VII §7 passes `buf.as_mut_span()`, a call result. **ERR-041 is decided**
  (Part VII §7's example governs; ADR-017 records the reading). What holds
  open is **D5**: the compiler passes a `MutSpan` by value and says so in
  `mut_param_ty`'s comment, unratified and awaiting an owner decision.
  Complying with the letter makes the document's own example uncompilable.
* **ERR-043** — `UnsafeCell` appears once in 5,526 lines and no rule defines it.
  Blocks a *third-party* package writing its own interior-mutability primitive;
  blocks nothing in `std` (ADR-019 takes the other route).
* **ERR-042** — nine rule ids cited and defined nowhere, the whole `IDE-*` family
  among them. Part XX cites five IDE rules on one line and defines none.

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
