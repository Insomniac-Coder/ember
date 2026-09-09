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
| `docs/DECISIONS.md` | ADR-001..019 |

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

56 conformance rule directories, 126 cases. 48 defects recorded, **1 open**
(D-030). 5 deviations. 2 errata awaiting the owner. Ratchets in
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
hardening diffs against **that**, not against as-received.

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
    DRP   3/6    missing DRP-4 (needs effects, Phase 4), DRP-5 (= D-030, open),
                 DRP-6 (Box/handle/Shared — Phase 3)
    SPN   3/3    done
    CELL  0/12   needs the implementation — see §5
    DIA   0/5    needs tests/ui snapshots — see §6

### The method that found twelve defects — keep using it

**Probe every normative rule with a minimal executable program before changing
any implementation code.** Not "does the suite pass" — write the three-line
program the rule describes and check the number. Four of the twelve were
*silent*: a declared `drop` that never ran, temporaries that never dropped
(output byte-identical either way), an explicit `drop()` that double-freed, a
write through a shared `ref` caught only by clang's `const`.

Where behaviour cannot be observed from output, **assert on the emitted C**
(`#$ assert-c: contains("ember_vec_free")`).

## 5. Next task: `Cell[T]` (block I)

ADR-019 decides the approach and ERR-043 says why. **Read both.** Short version:
`[CELL-9]` names `UnsafeCell` as the primitive and the document defines it
nowhere, so `Cell`/`RefCell`/`Arena` are **compiler-known**, as `Array`, `Span`,
`Option` and `Result` already are under Part XX.1.

I began this and set the scaffolding aside so a version cut could land
warning-free. **The design, which was working:**

* `Cell[T]` is a **transparent one-field struct**, built like `option_of` —
  `cell_of(inner)` interns a `StructDef` named `Cell_<stem>` with a single
  private field, and records `cells: HashMap<StructId, Ty>` on the `Checker` so
  method dispatch can tell a cell from an ordinary struct.
* That gives `[CELL-4]` for free — `Copy` when `T` is, move-only when it is not,
  `Drop` iff `T` is — because all three are read off the field. `[CELL-2]`'s "no
  overhead relative to a plain field" is why a struct is right rather than a new
  `TyKind` (`Span` needs 20 sites; this needs almost none).
* The field is private and **unreachable from source**. Every path to the value
  goes through a builtin, which is what makes `[CELL-2]`'s "never hands out a
  reference to its contents, so no aliasing rule can be violated" true by
  construction.

**What is left, and it is the actual point of the type:** `[CELL-1]`'s `set`
takes `self` — a *shared* borrow — and writes. Model the methods on
`synth_span_method`, with builtins `CellNew`/`CellGet`/`CellSet`/`CellReplace`/
`CellIntoInner`; the borrow checker sees a builtin call rather than a user write,
so it needs no exemption. `get` is `T: Copy` only.

**`[CELL-1]`'s invariant, which the owner called out twice:** `set` and `replace`
**MUST store the new value before dropping the old one.** A drop can re-enter the
same cell, and a drop-then-store leaves it observably uninitialised across that
window. Make this a conformance case, not a comment.

Then `RefCell` (a borrow counter, `[CELL-5..8]`, checked in **every** profile per
`[CELL-9]`) and `Arena` (`[ARN-*]`, a region proved statically). Amendment A13
records that the three share the implementation concern and **not** the concept.

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

* **ERR-041** — `[FN-1]` says a `mut` argument "MUST be a mutable place"; Part
  VII §7 passes `buf.as_mut_span()`, a call result. Holds **D5** open: the
  compiler passes a `MutSpan` by value and says so in `mut_param_ty`'s comment.
  Complying with the letter makes the document's own example uncompilable.
* **ERR-043** — `UnsafeCell` appears once in 5,526 lines and no rule defines it.
  Blocks a *third-party* package writing its own interior-mutability primitive;
  blocks nothing in `std` (ADR-019 takes the other route).
* **`[EFF-18]`** states the effect set without `Nondet`; §X.1, which defines the
  set, includes it. Two normative statements disagreeing.
* **ERR-042** — nine rule ids cited and defined nowhere, the whole `IDE-*` family
  among them. Part XX cites five IDE rules on one line and defines none.
* **D-030** — a `drop` body may move a field out of `mut self`, which `[DRP-5]`
  forbids. Filed rather than fixed: reaching it needs a `drop` that moves, and
  nothing in the corpus does.

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
