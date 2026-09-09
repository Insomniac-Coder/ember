# Ember — handoff

## 0. Current state — the authoritative snapshot (2026-09-09)

**Written as a migration hand-off. Everything in this section was verified
against the repository at the time of writing, not copied from a previous
note.** Sections 1 onward are the phase plan and are partly historical; where
they disagree with this section or with `docs/COLD-START.md`, this section and
COLD-START govern.

### 0.1 Git and build state — verified

| | |
|---|---|
| Remote | `https://github.com/Insomniac-Coder/ember.git` |
| Branch | `main` |
| HEAD | `5628a73` — task 1 (file `L3011` as D6). Below it: `9468602` (COLD-START twins), `a38aa59` (§0.18 + task list), `c4fa15c` (§0.16), `a2c0032` (snapshot), `365122d` (`Cell[T]`), `8459a1f` (D-035) |
| Last commit touching compiler sources | `365122d` — until this task lands (fixes in `ember_mir`, `ember_typeck`, `ember_codegen_c`, `ember_analysis` committed herein) |
| Working tree | task 2 (sweep: 3 compiler fixes, 13 cases, 5 ledger rows); committed herein, clean after push |
| Against `origin/main` | 0 ahead, 0 behind after push — everything committed is pushed |
| `cargo build` | **0 warnings** (no compiler sources changed since the verified state) |
| `cargo test --workspace` | **178 tests, all passing**, 0 failures |
| Gates | **all green** (six, run individually below) |

**The two commits this hand-off is about:**

| Commit | Date | What |
|---|---|---|
| `8459a1f` | 2026-09-09 | **D-035** — overwriting a place ran no destructor at all. 6 files: `ember_mir/src/lower.rs`, `DECISIONS.md`, `DEFECTS.md`, `HANDOFF.md`, two `tests/conformance/OWN-5/` cases |
| `365122d` | 2026-09-09 | **`Cell[T]`** — 18 files, 894 insertions: `ember_typeck`, `ember_mir`, `ember_hir`, `ember_codegen_c`, the test harness, 9 new conformance cases, `BACKLOG.md`, `COLD-START.md`, `HANDOFF.md` |

Preceding commits, for orientation: `aa2e1d4` (the cold-start file), `6a33bb0`
and `8101389` (cutting 0.8.4_Hardened_1), `207c69f`, `25335ec` (S1).

**The gates, with the invocation each needs.** `.github/workflows/ci.yml` is the
authority; six of the six are wired into CI.

    python tools/hardening_check.py                                          OK   (CI)
    python tools/split_spec.py --check docs/spec-source/ember-spec.md docs/spec   OK   (CI)
    python tools/rule_index.py                                               OK   (CI)
    python tools/check_branding.py                                           OK   (CI)
    python tools/spec_check.py                                               OK   (CI)
    python tools/error_pages.py                                              OK   (CI)

Two things about that list are worth carrying: `split_spec.py --check` throws
`IndexError` if you call it without both path arguments — it is not failing, you
called it wrong — and **`error_pages.py` was green but not wired into CI until
the consistency pass (§0.16) added it**, so it previously ran only when somebody
ran it. COLD-START says "6 gates"; six of them are now enforced.

### 0.2 What `Cell[T]` is, and exactly what is built

`Cell[T]` is a **transparent one-field compiler-known type** — a `StructDef`
interned once per `T` by `cell_of` in `compiler/ember_typeck/src/lib.rs`, with
`cells: HashMap<StructId, Ty>` on the `Checker` telling method dispatch that
this struct's `set` is a builtin rather than a missing method. **ADR-019** is
why it is compiler-known and not written in Ember on `UnsafeCell`.

**Implemented:**

| Member / rule | Notes |
|---|---|
| `Cell(owned v)` | a plain struct literal, not a builtin |
| `set(self, owned v: T)` | `Builtin::CellSet`, lowered by `lower_cell_store` |
| `replace(self, owned v: T) -> T` | same lowering, old value handed back instead of dropped |
| `into_inner(owned self) -> T` | `lower_cell_into_inner` |
| `get(self) -> T` **where `T: Copy`** | an `ExprKind::Field` read — no builtin |
| `update(self, f)` — **the `T: Copy` arm** | `Builtin::CellUpdate`, `set(f(get()))` |
| **`[CELL-1]` store-before-drop** | see §0.4; this is the load-bearing one |
| `[CELL-2]` | no reference escapes; no runtime check; no overhead over a plain field |
| `[CELL-4]` | `Copy` iff `T: Copy`, move-only otherwise, `Drop` iff `T` is |
| `[CELL-11]` | in the prelude — the name resolves with no import |

**9** conformance cases across `tests/conformance/CELL-1` (3), `CELL-2` (2),
`CELL-4` (3) and `CELL-11` (1). (An earlier revision said 11: it counted the two
`OWN-5` cases from D-035 into the `Cell` total.)

**`[CELL-4]` needed no code at all**, and this is worth reusing rather than
rediscovering: `is_copy` on a struct is already `derives_copy && !has_drop &&
every field Copy`, and `needs_drop` is already `has_drop || any field needs it`.
Set `derives_copy` and leave `has_drop` clear and both questions reduce to the
same question about `T`. That is also why a struct was chosen over a new
`TyKind` — `Span` needed twenty sites; this needed almost none — and it is what
makes `[CELL-2]`'s "no overhead relative to a plain field" true by construction
rather than by promise.

**Not implemented — these are dependency gaps, not defects.** Nothing here is a
compiler bug and nothing here weakened a rule:

| Item | Why it is not built | Tracking id |
|---|---|---|
| `take(self) -> T where T: Default` | **there is no `Default` interface anywhere in the compiler.** The member is refused by name with a diagnostic that says so, rather than reading as a typo | **`CELL-DEF-1`** (`docs/BACKLOG.md`) |
| `update`'s `T: Default` arm | same missing interface; the `T: Copy` arm is built | **`CELL-DEF-1`** |
| `[CELL-3]` `Cell[T]` is `!Sync` | **no `Send`, no `Sync`, no threads exist.** There is nothing for the marker to mean and nothing that could violate it — unenforceable, not unenforced | **`CELL-SYNC-1`** (`docs/BACKLOG.md`) |

`tests/conformance/CELL-1/reject_take_needs_default.em` pins the `take` message,
so the gap cannot silently become "no such method". **The rules were not
softened to fit the compiler** — what is missing is the interface, not the
requirement.

**`into_inner` needed one trick worth remembering.** It takes `owned self`, so
the payload leaves and the cell must not be dropped behind it — the payload was
the only thing it owned. A move out of a *field* is a partial move and
`[OWN-3]`'s analysis does not track one, so the receiver would still have looked
live and been dropped at scope end, freeing the value the caller now holds.
Moving the **whole cell** into a slot first is what marks the receiver moved,
and the slot is `temp_unowned` so nothing drops it either. Verified: exactly one
`ember_vec_free` in the emitted program.

**`[CELL-2]` rests on ordinary `[MOD-2]` privacy**, not on a clever name. The
payload field is private and its `declaring_module` is `usize::MAX`, which no
real module can be, so `check_field_visible` refuses it everywhere — read,
write and `ref` all `E1020`.

### 0.3 D-035 — a real compiler defect, found and fixed

**`[OWN-2]` requires a value to be destroyed on three occasions:**

1. **scope end** — built long ago (`emit_drops_from`)
2. **overwrite** — **was never built**
3. **temporary destruction at statement end** — built by D-031
   (`emit_statement_temps`)

The compiler handled the first and the third and silently failed the second.

**Minimal reproducer:**

    r = R(1)
    r = R(2)

`R(1)`'s destructor **did not run at all**. Where `R` declares `fn drop`, the
effect was simply a missing line of output; the same hole in the container path
was a leak — overwriting a live `Array[i32]` emitted **one** `ember_vec_free`
for **two** allocated buffers, and the program's output was identical either
way, so nothing anywhere reported it.

**D-035 was a compiler defect. The specification was already correct.**
`[OWN-5]` states the requirement and its ordering in one sentence and admits no
other reading. The compiler was fixed; **the specification was not weakened, and
there is no errata entry**, because an implementation gap is not a specification
defect.

**The fix** is `lower_assign` in `compiler/ember_mir/src/lower.rs`: the new value
is evaluated into a temporary, the old value is dropped, then the store happens
— `[OWN-5]`'s order exactly. Which of those inserted drops actually survives is
deliberately **not** decided there: `[OWN-3]`'s existing elaboration deletes the
drop where the local is `Moved` (which is what makes a first initialisation
free, since a local is `Moved` immediately after `StorageLive`) and attaches a
drop flag where it is `Maybe`. **ADR-020** records why the temporary is
unavoidable — a value arriving from a call is written by a terminator, so there
is no point between "evaluated" and "stored" to push a drop into — and why
lowering must not try to decide liveness for itself.

#### The testing lesson, which matters more than the fix

**`tests/conformance/OWN-5/` existed and passed for as long as the rule went
unimplemented.** Its single case tested only the sentence's *parenthetical* —
that the new value is evaluated before the store — and it was written as:

    x = grow(x)

which **moves `x` into the call**. Nothing was live at the store, so the main
clause could not fire and the missing destruction was unobservable.

> **A conformance directory named after a rule does not prove that the rule is
> actually covered.** Read the rule, count its clauses, and check that each one
> has a case that can fail. A green directory is evidence about the cases in it,
> not about the rule on the label.

Two cases were added and **both were verified to go red** by reverting
`lower_assign` to `lower_into` and watching them fail.

### 0.4 The ordering distinction — do not generalise this

**`[OWN-5]` and `[CELL-1]` require opposite orders, on purpose.**

| Operation | Order | Rule |
|---|---|---|
| ordinary assignment | evaluate new → **drop old** → **store new** | `[OWN-5]` |
| `Cell.set` / `Cell.replace` | evaluate new → **store new** → **drop old** | `[CELL-1]` |

`[CELL-1]` gives its own reason: *"A drop can run arbitrary user code that
re-enters the same `Cell` … a drop-then-store implementation would leave the
`Cell` observably uninitialised across that window, which is a read of
uninitialised memory."* Dropping the old value can re-enter the cell, so the
cell must already hold the new value when that happens.

**This is why `Cell.set` is specially lowered** — `lower_cell_store`, not an
assignment. If `set` were lowered as an assignment it would inherit `[OWN-5]`'s
order and violate `[CELL-1]`.

> **ADR-020's closing paragraph says explicitly that these two rules must not be
> generalised to each other.** Do not "simplify" them into one universal
> assignment rule, and do not read `Cell`'s exception as licence to vary
> `[OWN-5]`'s order anywhere else. They are two rules about two operations, and
> each states its own reason.

`replace` is the easy case of the same shape: the old value is handed back
rather than dropped, so the cell is never torn at all.

### 0.5 The C code-generation lesson — `$value`

`Cell`'s private payload field was first named **`$value`**, chosen because the
Ember lexer cannot produce `$` in an identifier, so no program could spell it.

**That was wrong.** The backend writes field names into the emitted C
**verbatim**, and `$` in a C identifier is a **compiler extension**, which
`[CG-C-1]` forbids depending on: *"…and not depend on compiler extensions except
through `ember_rt.h` macros that have portable fallbacks."*

It compiled **clean under `-Wall -Wextra`**. Only `-pedantic`, run by hand,
reported it (`-Wdollar-in-identifier-extension`). The field was renamed to an
ordinary identifier and privacy carries the whole burden.

> **A compiler-private name must still obey the target language's portability
> rules.** Unreachability comes from compiler-generated structure and the
> language's own privacy mechanism — not from spelling the name in syntax the
> target rejects. **Read the emitted C after anything that puts a new identifier
> into it**, and remember the project's warning contract (`-Wall -Wextra`, MSVC
> `/W3`) is a floor that will not catch this class.

### 0.6 `assert-c-order` in the conformance harness

`compiler/ember_driver/tests/milestones.rs` now understands:

    #$ assert-c-order: "a" then "b"

Both needles must appear in the emitted C, and the first before the second.

**Why it was needed.** `assert-c`'s needle is one line of literal text, so it can
say *what* the backend emitted but not *in what order* — and `[OWN-5]` and
`[CELL-1]` are **entirely about order**. Neither ordering is observable from a
program's output: seeing the difference requires a destructor that re-enters the
value being replaced, and no safe Ember program can build that back-pointer
today: `[STA-1]` requires a `static` initialiser to be **comptime-evaluable**,
and the compiler currently narrows that to a literal, so a `static` holding a
cell is refused with `E2130` — the one route by which a destructor could reach
the cell it is being replaced in. Without this, both rules could only ever be checked by reading the C
by hand, once, and would silently stop holding the next time either lowering
path was touched.

**This was deliberately built in the harness rather than contriving an unsafe
recursive scenario just to observe destructor ordering.** Both shapes were
priced: the unsafe route needs recursive types through a raw pointer or the
address of a private field, is brittle, and buys one test; the harness change is
about twenty lines, changes no existing behaviour, and makes a whole class of
ordering invariants testable.

**Verified the only way that counts:** the store and the drop in
`lower_cell_store` were swapped on purpose and the case went red with the right
message. Then restored.

**Two traps when writing one.** Anchor needles on text that survives MIR local
renumbering — `.value = ` and `em_Bag_drop(&`, not `_1.value = _4` and
`em_Bag_drop(&_5)`. And the `&` is load-bearing: a bare function name matches
its own **prototype** at the top of the emitted file, which precedes everything
and would make the assertion vacuously true.

### 0.7 Specification authority — the rules that govern everything here

**The current authoritative specification is `Ember_v0.8.4_Hardened_1`.**

    docs/spec-source/as-received/Ember_v0.8.3_spec.md   the owner's file, NEVER edited
    docs/spec-source/ember-spec.md                      the normative copy, carries declared amendments
    docs/spec-source/Ember_v0.8.4_Hardened_1.md         the frozen snapshot; the next hardening diffs against THIS
    docs/spec/                                          generated; split_spec.py --check fails CI on a hand edit

* **0.8.3 remains an accepted language version** — a file declaring
  `#! language "0.8.3"` compiles unchanged.
* **0.8.4 is the current language revision.**
* **0.8.4 is additive over 0.8.3** — no program valid under 0.8.3 became
  invalid. `LANGUAGE_VERSIONS` in `compiler/ember_parser/src/lib.rs` accepts
  both.
* **The current hardened specification is the normative source.**
* **Implementation difficulty does not justify changing language semantics.**
* **Compiler behaviour must conform to the specification, never the reverse.**

**Two numbers version the document and they move independently.** The *language
version* moves when the set of accepted programs changes, and resets the
hardening number to 1. The *hardening number* moves when the document gains
implementation detail and **no rule changes meaning**. Every edit to
`ember-spec.md` declares one of five classes, and only four are permitted in a
hardening:

    SEMANTICALLY NEUTRAL CLARIFICATION   permitted
    IMPLEMENTATION INVARIANT             permitted
    SOURCE RECOVERY                      permitted
    EDITORIAL REPAIR                     permitted
    OWNER-APPROVED SEMANTIC CHANGE       forces a language version bump

The line: **the moment an edit answers *what Ember means* rather than *how to
implement what Ember already means*, it stops being a hardening.**
`tools/hardening_check.py` enforces *declaration*, not correctness — it is a
floor, and it would not have caught either of the two amendments that were
withdrawn.

#### The five kinds of finding, and who moves for each

Keeping these apart is the single most important discipline in this project.
Getting it wrong in either direction is how an implementation workaround
quietly becomes the language.

| Kind | Evidence | Where it goes | Who moves |
|---|---|---|---|
| **Language-design decision** | the owner decided what Ember means | `DECISIONS.md` (ADR) + `spec-amendments.md`; forces a language version bump | the document, by the owner |
| **Specification ambiguity / contradiction** | two normative statements require different things, neither marked as governing | `spec-errata.md` | **nobody, until the owner rules** |
| **Compiler defect** | the rule is clear, the compiler violates it | `DEFECTS.md` | **the compiler** |
| **Test defect** | the rule is right, the compiler may be right, the case does not exercise the rule | a new conformance case | nobody — but see D-035 |
| **Implementation limitation** | the rule is clear, the compiler knowingly cannot meet it yet | `DEVIATIONS.md` or `BACKLOG.md`'s compiler-debt table | the compiler, later |

D-035 was the third kind and revealed the fourth. The `Cell` gaps
(`CELL-DEF-1`, `CELL-SYNC-1`) are the fifth. **None of them is the second, and
none was treated as one.**

### 0.8 Specification-change philosophy — a standing project rule

> **Do not modify the specification merely because the compiler cannot currently
> implement a rule.**

When the compiler and the document disagree, in this order:

1. **Reproduce it with a minimal executable program.** Not "does the suite
   pass" — write the three-line program the rule describes and check the number.
2. **Identify the exact normative rule**, and quote it.
3. **Determine whether the compiler is wrong.** If the rule is clear, it is.
4. **Check whether the test genuinely exercises the rule.** A passing directory
   named for the rule is not evidence — D-035 is the worked example.
5. **Only treat it as a specification problem if there is an actual semantic
   ambiguity or contradiction** — and before concluding that, check for a
   *sequencing* reading (two stages of one process), a *scoping* reading (a rule
   under a heading that narrows it), and a *generation* reading (an output may
   contain what no source may contain). Three claimed contradictions were
   dissolved by exactly these three readings.
6. **Obtain an explicit owner decision before changing language semantics.**

**Do not silently weaken safety, ownership, lifetime, region, interop, reload or
effect-system guarantees.** A requirement the compiler cannot meet yet goes in
`BACKLOG.md`'s compiler-debt table with an id. **Never soften the rule.**

**The failure mode this exists to prevent**, learned expensively: both withdrawn
amendments had one shape — the analysis was right, and **the answer went into
the normative text instead of the ledger**. One of them had "not yet ruled on by
the owner" written directly above it in the ledger. *The document carries the
sentence, not the caveat.*

### 0.9 Why 0.8.4 exists — ERR-044 and amendment S1. Do not regress this.

**The owner's semantic decision, taken 2026-09-09:** `[TYP-15]` is
**region-based**.

`[TYP-15]` stated a principle — a view may not be stored "in a place whose
region is not outlived by the view's region" — and then an enumeration saying
class fields, `static`s, `Box`/`Shared` contents, container elements and `owned
fn` captures are "**always** forbidden". `[LT-3]` said in as many words that a
`str` literal **may** be stored in a class field, because its region is
`static`. `[TYP-15]`'s own principle sides with `[LT-3]`.

**Decision: `[LT-3]`'s semantics govern.** A view may be stored in long-lived or
unbounded storage **when its region outlives the destination**. The enumeration
was what overreached, by assuming every listed place has no *possible*
sufficient region.

Therefore:

* **static-region views may be stored in long-lived storage** —
  `class Foo: greeting: str = "hello"` is admitted;
* **non-static views may not** — `foo.greeting = s` inside
  `fn set(mut foo: Foo, s: str)` stays refused, because `s` may be a caller's
  region;
* **`[TYP-15a]` still restricts arbitrary owning containers of views** —
  `Array[str]`, `Map[str, V]`, `Array[MutSpan[T]]` remain rejected whatever the
  region, because that rejection is at the *type* and not at the region.
  `BorrowList[T]` / `ViewList[T]` remain the specialised model.

**Do not regress the implementation to the old blanket prohibition.**
`tests/conformance/TYP-15/` holds `accept_a_static_region_view_in_a_static.em`
and `reject_a_non_static_view_in_a_static.em`, which are exactly this pair.
(`tests/conformance/LT-3/` is a different directory with one case,
`accept_a_literal_in_a_view_struct.em` — an earlier revision of this section
named it here and was wrong.)

The owner asked for this to be recorded as a semantic decision rather than a
hardening — *"Please resolve ERR-044 as an owner semantic decision, not as a
hardening-only change"* — and that is what made the language version 0.8.4.
Amendment **S1**, class `OWNER-APPROVED SEMANTIC CHANGE`, is the only semantic
change in the file; everything else is Hardened_1 on top of it. The reason to
take that route was never the size of the change but the boundary it protects:
**hardening must never quietly become language evolution.**

**Keep these two codes distinct** — they are different questions and the
diagnostics say so:

| Code | Rule | Means |
|---|---|---|
| **`E3060`** | `[LT-3]` | *borrowed value does not live long enough* — an ordinary borrow outliving its source |
| **`E3063`** | `[TYP-15]` | *stored view may not outlive its source* — a view stored in a place that outlives it |

(Nearby, so as not to confuse them: `E3062` is a returned reference not deriving
from a parameter, `E3064` is two independent regions in one view struct.)

### 0.10 Compiler-development philosophy

**The compiler implements the language's semantic model. It does not merely
satisfy the current tests.**

Prefer, in every case:

* proper compiler mechanisms over test-specific hacks;
* explicit semantic phases;
* strong internal representations;
* explicit ownership / lifetime / region information carried in the IR rather
  than recomputed by guess;
* clear verifier and checker boundaries;
* deterministic lowering;
* proper diagnostics — the right code, the right shape, and a `help` that
  names a concrete API;
* conformance tests tied to normative rule ids;
* adversarial tests — the program that should fail, not only the one that
  should pass;
* generated-C inspection where the behaviour is not observable from output.

> **A passing test suite is necessary but not sufficient evidence that a
> semantic rule is correctly implemented.** D-035 is the proof: 178 tests green,
> a conformance directory named for the rule, and the rule entirely
> unimplemented.

**When implementation exposes a genuine language-design question, stop and
escalate rather than guessing.** Guessing is how an implementation workaround
becomes the language, and this project has already had to unwind that once.

Three concrete habits that keep paying:

* **Probe every normative rule with a minimal executable program before
  changing any implementation code.** D-035 was found this way in the first ten
  minutes of a task about something else.
* **After adding a test, break it once on purpose and watch it go red.** A
  `compile-fail` case can satisfy itself; a directory of `run-pass` cases passes
  on an empty directory. Both have happened here.
* **Any new value-producing or place-writing form must be walked through every
  analysis by hand** — `drops.rs`, `borrows.rs`, `regions.rs`, `verify.rs`. It
  fails closed, and failing closed means rejecting a correct program. Several
  defects were found by asking, not by a test failing.

**Stop and escalate** on a genuine specification contradiction, two plausible
semantic interpretations, an owner-level tradeoff, a missing language rule, a
proposed safety relaxation, or any change to ownership / lifetime / region /
interop / reload / effect-system semantics. Bring: (1) the conflicting rules
quoted, (2) a minimal reproducer, (3) current compiler behaviour, (4) the
possible interpretations, (5) the consequences of each, (6) a recommended
resolution. Do not silently choose.

### 0.11 Using agents and agentic workflows — approved

**The owner has explicitly approved the use of subagents *and* multi-agent
workflows during development on this project** (2026-09-09). Both are available
to the next agent: individual subagents for parallel exploration, and full
orchestrated workflows (fan-out, pipeline, judge panels, adversarial verify)
where the task is large enough to earn one. Use them when they help.

Two standing constraints still apply, and they are the owner's, not defaults:

* **Ask before spawning subagents or a workflow, every time, and state the
  worst-case agent count in the question.** Approving a *shape* is not approving
  a *size*. This is a standing rule and the approval above does not lift it —
  what it lifts is any doubt about whether workflows are welcome at all.
* **Report after each task and wait for the green signal** before starting the
  next one.

**Where a workflow genuinely earns its cost on this codebase:**

* Sweeping the whole conformance corpus for rules whose directory does not cover
  every clause — the **D-035 shape**, and the highest-value sweep available
  right now, because that defect proves the shape exists and nothing has looked
  for others.
* Auditing a new IR form or value-producing expression against all four
  analyses at once (`drops.rs`, `borrows.rs`, `regions.rs`, `verify.rs`) —
  these fail closed, and several past defects were found by asking rather than
  by a test failing.
* Adversarially verifying a claimed defect before it reaches `DEFECTS.md`.
  Three claimed specification contradictions turned out to be misreadings, each
  dissolved by a sequencing, scoping or generation reading; a refuter pass is
  exactly the right instrument for that.
* Reading a large normative Part against the implementation, clause by clause,
  before building — `RefCell` spans `[CELL-5]`..`[CELL-10]`, `[CELL-6a]`,
  `[TYP-15]`, `[PRF-1]` and the region machinery.

**Where it does not:** a single coherent edit across several crates, which is
most implementation work here. `Cell[T]` touched five crates and was one change;
splitting it across agents would have cost more than it saved.

### 0.12 Open issues carried forward — verified against the repository

**Open defects — three.**

| # | What | Rule | Status |
|---|---|---|---|
| **D-030** | a `drop` body may move a field out of `mut self`, which the rule forbids; the field is then dropped again after `drop` returns | `[DRP-5]`, `[EXP-6]` | **open.** Filed rather than fixed: reaching it needs a `drop` body that moves, and nothing in the corpus does |
| **D-038** | implicit `String`→`str` coercion is rejected (`E2020`), though `[SPN-1]` lists it; the explicit `as_str()` spelling works | `[SPN-1]` | **open.** Fails closed (sound direction); needs the coercion arm in typeck. Found by task 2 |
| **D-040** | a method call defeating disjoint-field access reports `E3022`, where `[DIA-7a]` keys shape B8 to `E3025` — registered, shape-mapped, emitted by nothing | `[BRW-4]`, `[DIA-7a]` | **open.** Needs call provenance threaded to the reporter. Found by task 2 |

**Open deviations — six, in `docs/DEVIATIONS.md`.**

| # | What | Status |
|---|---|---|
| **D1** | `[RNG-5a1]`'s generated operator impls are a type-checker rule instead; no impls exist | open; not observable until operator interfaces exist. ADR-016 |
| **D2** | **`[CLO-6]`'s `owned f: fn(A) -> R` is refused, not implemented** | **open.** This is the live residual of the closure work — see the note below |
| **D3** | `extern class` parses and is then refused by name (the C++ importer is Phase 7) | open. ERR-037 |
| **D4** | `E9012` is registered and never emitted | open |
| **D5** | a `mut` parameter whose type is itself a borrow — `[FN-1]`'s literal reading versus Part VII §7's own worked example | **unratified; awaiting an owner decision.** Complying with the letter makes the document's own example uncompilable, so neither side moves. ERR-041, ADR-017 |
| **D6** | `L3011` is registered and never emitted; no guard exists yet to trigger it | **open.** Filed by task 1 of the handoff list; closed when `RefCell` (task 3) emits it from guard-liveness tracking. `[CELL-7]` |

**`[CLO-3]` / ADR-018 is CLOSED, not open.** The owner ruled that the rule
stands and the compiler catches up — *"Do not change the Ember spec to
accommodate the current compiler implementation."* `fn(A) -> R` in parameter
position is now an implicit generic bounded by `Callable`, monomorphised per
argument type; a capturing lambda is an anonymous struct of its `[CLO-2]`
captures. `tests/conformance/CLO-1/accept_a_capturing_closure.em` passes. **What
remains open is `[CLO-6]`'s `owned f`, which is deviation D2**, refused by name
rather than mis-compiled: consumption is a property of the bound, not of the
closure's type, and until a call can consume its callee, treating `CallableOnce`
as `Callable` would permit the second call the rule forbids.

**Open errata — awaiting the owner.**

| # | What | Status |
|---|---|---|
| **ERR-042** | nine rule ids are cited and defined by no rule — `[TYP-26]`, `[IFC-2]`, `[HND-2]`, `[GPU-7]`, and the whole `[IDE-1]`/`[IDE-2]`/`[IDE-5]`/`[IDE-7]`/`[IDE-10]` family. Part XX cites five IDE rules on one line and defines none | **open — reported to the owner.** Rule-id bookkeeping |
| **ERR-043** | `UnsafeCell` is named once in 5,526 lines, as *the* primitive a package uses for unchecked interior mutability, and no rule defines it | **open — reported to the owner.** See §0.13 |
| **ERR-029, the `[RNG-8]` part** | `[RNG-8]`'s text opens mid-sentence on an ellipsis and the missing opening survives in no copy of the document, so it cannot be restored, only guessed | **open.** The other nine items in ERR-029 are decided |

**Compiler debt — `docs/BACKLOG.md`.** `RT-GEN-1` (generate `ember_rt.h`/`.c`
from the symbol prefix), `LNT-CFG-1` (`[MAN-3]`'s `[lints]` configuration —
`[LT-1b]`'s `L3014` is an opt-in lint with nowhere to opt in), `TST-6-1`
(Appendix A's fixture as `compile-pass`), and the two new ones,
**`CELL-DEF-1`** and **`CELL-SYNC-1`**. `LT-REG-1` is struck through — done.

**Phase 2 coverage still outstanding**, from `COLD-START.md` §4: `OWN` 6/8,
`BRW` 7/9, `LT` 5/10, `DRP` 3/6, `CELL` 4/12, and **`[DIA-7..10]` with
`tests/ui/` snapshots at 0/5 — which is a Phase 2 *exit* criterion**
(COLD-START §6).

#### Bookkeeping inconsistencies found while writing this hand-off

These are **documentation** discrepancies, not compiler or specification
defects. **Update (consistency pass, §0.16): all four are now resolved. The
descriptions below are preserved as found; each carries its resolution.**

1. **`[EFF-18]` versus Part X §1 on `Nondet`.** `spec-errata.md`'s summary row
   marks **ERR-028 "decided"** and its body says the enumeration "gains
   `Nondet`" — but `docs/spec/part-10-*.md` line 57 still reads
   `{Alloc, Sync, Lock, Io, Panic, Unsafe, FFI, Block, RuntimeCheck(k)}`,
   **without `Nondet`**, and `spec-amendments.md` records no amendment applying
   it. `COLD-START.md` §7 still lists it as open. So either the amendment was
   never applied or the entry's "Applied to" is aspirational. **Left untouched at
   the time: this is two normative statements disagreeing, which is the kind where neither
   side moves until the owner rules. Resolved in §0.16: ERR-028 confirmed as the
   decision (X.1 governs, no supersession found, no genuine semantic conflict —
   a stale omission); `[EFF-18]` now includes `Nondet` (E5, editorial repair);
   COLD-START §7 no longer lists it.**
2. **ERR-044's body header is stale.** Its summary row says *"decided by the
   owner, 2026-09-09 … Amendment S1"* and amendment S1 exists and is applied,
   but the section body still opens *"Status: open. Reported to the owner.
   Neither side has been moved."* The decision is real — §0.9 is what governs;
   the body header was not updated. **Resolved in §0.16: header now reads
   decided / S1 accepted / current `[TYP-15]`-`[LT-3]` rules in 0.8.4.**
3. **ERR-041 reads as open in `COLD-START.md` §7** but its errata row is
   **decided** (ADR-017). Both are true in different senses: the errata question
   is decided, and it holds **deviation D5** open pending an owner ruling. The
   live item is D5. **Resolved in §0.16: COLD-START §7 now titles the bullet D5
   (open deviation) with ERR-041 noted as decided via ADR-017.**
4. **`error_pages.py` is described as one of six gates but is not in CI.** It is
   green; it only runs when somebody runs it. **Resolved in §0.16: wired into
   `.github/workflows/ci.yml` as "error pages compile as documented"; docs and
   CI now agree on six enforced gates.**

### 0.13 Interior mutability — keep the three apart

* **`Cell[T]`** — interior mutability. Replaces the **whole value**, hands out
  **no reference**. Nothing has to be proved and there is **no runtime state**.
  Built.
* **`RefCell[T]`** — interior mutability. Mutates **through a reference**, so
  `[BRW-1]`'s question is asked **at run time** against a borrow counter
  (`[CELL-5]`..`[CELL-8]`). Not built.
* **`Arena`** — **a region allocator, not an interior-mutability primitive.**
  Its mutation happens to reach through a shared borrow, but what it must prove
  is a **region** rather than an alias, which `[ARN-1]` does **statically**. Not
  built.

Amendment **A13** records that the three share **the implementation concern and
not the concept**. `[CELL-2]` says an implementation *may* share machinery
between them and nothing requires it to. **Do not build `RefCell` by
generalising `Cell`**, and do not describe `Arena` as a third interior-mutability
primitive.

What is common to all three: *"interior mutability never means the borrow
checker stops caring: the obligation moves — to a replacement that cannot alias,
to a counter, or to a region — and an implementation that satisfies any of the
three by exempting a type from `[BRW-1]` has not implemented it."* `Cell` is
sound here for exactly that reason: nothing escapes, so the write is not an
aliasing question at all, and the borrow checker was **not weakened**.

**`UnsafeCell` remains an unresolved owner-level question (ERR-043).** ADR-019
chose compiler-known types over building on it *because it is not available* —
it is named once and defined nowhere, and inventing the semantics of the one
construct that suspends `[BRW-1]` is the owner's call. ADR-019 carries an
explicit warning worth repeating: **the existence of `Builtin::CellSet` is not
evidence about what the language permits an arbitrary package to implement.**
Do not reach ERR-043, observe that the compiler already mutates through a shared
borrow for `Cell`, and conclude that the language therefore permits it. That is
the compiler-justifies-specification move by a longer road. **Do not invent
semantics for unresolved owner-level questions.**

### 0.14 The next task

**`RefCell[T]`, then `Arena`.** Block I's remaining two.

**Do not begin implementing them as part of reading this hand-off.** Report and
wait for the green signal, per the owner's standing rule.

**Why `RefCell` is the next significant milestone, and harder than `Cell` was.**
`Cell` was contained because nothing escaped it. `RefCell` is the opposite:
`[CELL-5]`'s `borrow` and `borrow_mut` hand out `Ref[T]` and `RefMut[T]`, and
`[CELL-7]` makes those **genuine view types** whose region borrows the cell and
whose `drop` releases the borrow state. So **`regions.rs`, `[TYP-15]`, borrow
checking, guard lifetime and escape behaviour all become load-bearing**, none of
which `Cell` touched. Add to that: `[CELL-9]` puts the borrow check in **every**
profile and forbids `exclusivity = "unchecked"` from reaching it; `[CELL-6a]`
forbids any profile making `try_borrow` infallible, because that would change
which branch of a `match` runs, which `[PRF-1]` forbids; `[CELL-5]`'s panic
message must name **the conflicting borrow's source location**, recorded in
debug *and* release; and `[CELL-7]`'s `L3011` — "`RefCell` guard held across a
call" — **is registered in `compiler/ember_diag/src/codes.rs` and emitted by
nothing**, so it is work this task closes. It is **not** blocked and **not**
opt-in: `[CELL-7]` says it *fires* when a guard is live across a call that could
re-enter the same cell. (An earlier revision of this section said it needed
`LNT-CFG-1` first — that entry is about `[LT-1b]`'s `L3014`, which is a
different lint and is the opt-in one.) `[CELL-10]` constrains the
*diagnostics*: shape B4
may suggest `RefCell` **only** when the conflicting accesses are provably not
simultaneous, and never as a first suggestion.

**What carries over from `Cell`:** the transparent-struct shape and the
`cells`-style side table; privacy as the mechanism for an unreachable field; and
`assert-c-order` for any rule that is about order.

**What does not, and it is an open question rather than a settled one:** whether
`RefCell[T]` may be `Copy`. `Cell` got `[CELL-4]` free because a struct's
`is_copy` reduces to its field's; a `RefCell` carrying a borrow counter would
get the same answer mechanically, and **copying the counter would fork the
borrow state** — two cells each believing they hold the only mutable borrow.
That reasoning is sound but it is **inference, not specification**: Part IX
states `[CELL-4]` for `Cell` and says nothing about `RefCell`'s `Copy`-ness, and
an earlier revision of this section asserted "`RefCell` is never `Copy`" as
though the document said so. **Escalate it** (§0.10's rule) rather than deciding
it while implementing: quote `[CELL-4]` and `[CELL-5]`, show the two-holders
program, and let the owner rule.

**Read before starting, in this order:**

1. `docs/spec-source/Ember_v0.8.4_Hardened_1.md` — the frozen language
   revision (Part IX §7 for `[CELL-*]`, Part IX §2 for `[ARN-*]`). For
   implementation, `docs/spec-source/ember-spec.md` governs on any difference
   — the two currently differ only by E5 (Part X, unrelated to `RefCell`); see
   §0.17 and do not treat the files as interchangeable.
2. `docs/HANDOFF.md` — this file, §0 first, then the Block I section
3. `docs/COLD-START.md` — §2's rules and §5's design notes
4. `docs/DECISIONS.md` — ADR-019 and ADR-020 especially
5. `docs/DEFECTS.md` — D-035 and D-030
6. `docs/BACKLOG.md` — the compiler-debt table

**Then inspect the existing infrastructure before writing anything:**

    compiler/ember_analysis/src/regions.rs   region variables, the constraint graph
    compiler/ember_analysis/src/borrows.rs   NLL, loans, elision, the [DIA-7] shapes
    compiler/ember_analysis/src/drops.rs     moves, drop flags, [OWN-4]'s loop shape
    compiler/ember_typeck/src/lib.rs         cell_of, cells, synth_cell_method — the compiler-known-type pattern
    compiler/ember_mir/src/lower.rs          lower_assign, lower_cell_store, lower_cell_into_inner, temp_unowned

**`RefCell` conformance must actually exercise the rules** (§0.3's lesson
applies in full). At least, each tied to its rule id: multiple `Ref`s;
`Ref` + `RefMut` refused; multiple `RefMut`s refused; runtime contention
panics with the conflicting borrow's source location (debug *and* release);
guard `drop` releasing borrow state; early return through a live guard;
nested borrows and nested scopes; returned `Ref` / `RefMut`; stored views and
region boundaries; non-escaping vs escaping views; static-region views
admitted where `[TYP-15]` allows; invalid long-lived storage refused with
`E3063`. Break every new case red once before trusting it.

### 0.15 The principles, in one place

Carried forward deliberately, because the reasoning matters more than the
inventory:

> **The specification is the contract.** It is never edited to make the compiler
> agree with it. Where the two disagree and the rule is sound, the compiler
> moves.
>
> **A compiler bug is not a reason to weaken the language.** An implementation
> gap goes in the backlog with an id, and the rule stays as written.
>
> **Where the document contradicts itself, neither side moves** until the owner
> rules. Do not pick the reading that matches what is already built.
>
> **A test named after a rule is not necessarily coverage of that rule.** Read
> the rule, count its clauses, and check each one has a case that can fail.
>
> **Minimal adversarial programs are the preferred instrument** for validating
> a semantic invariant — not "does the suite pass".
>
> **When the semantics are clear, fix the compiler. When the semantics are
> genuinely ambiguous, stop and ask.**
>
> **Do not let an editable or generated specification silently diverge from the
> frozen normative artifact.** If the working source runs ahead of the frozen
> cut, document exactly which artifact is authoritative, why they differ, and
> what the next agent must treat as truth — see §0.17.

### 0.16 Consistency pass — the four §0.12 items, resolved (no semantics weakened)

On top of `a2c0032`, committed herein as a doc/CI-only delta. No language rule changed
meaning; no compiler code moved; `RefCell[T]` not started. Hierarchy applied:
accepted owner decisions, then normative 0.8.4 spec, then amendments/ADRs, then
implementation, then tests/docs. Where docs conflicted with a settled decision,
the docs were synchronised.

1. **`[EFF-18]` / ERR-028 / `Nondet` — the only semantic reconciliation.**
   Sweep: `[EFF-18]`'s bullet listed nine effects without
   `Nondet`; Part X §1's opening plus its table already define
   ten with `Nondet` and cite `[DET-2]`; `[DET-1]`–`[DET-9]` and
   `@deterministic` all assume `Nondet`; ERR-028 records decided (ten members,
   X.1 governs, `[EFF-18]`'s own "does not remove an effect" sentence says it
   is not exclusive); front matter, `spec-amendments.md` (no applying
   amendment), change history (0.6.3 added `Nondet` to X.1 without revisiting
   `[EFF-18]`), and `git log` show no supersession. No genuine conflict — a
   stale omission, not two rules requiring different behaviour — so per the
   brief it was applied rather than escalated. Changed:
   `docs/spec-source/ember-spec.md` `[EFF-18]` full set now includes `Nondet`
   with marker `*(ERR-028 applied 2026-09-09; see docs/spec-amendments.md)*`;
   front matter "Deliberately not done" paragraph now records the prior
   disagreement in the past tense; `docs/spec-amendments.md` gains **E5**
   (`[EFF-18]`, class EDITORIAL REPAIR, no implementation change, authorised by
   ERR-028); `docs/spec/` regenerated via `split_spec.py` (no hand edit);
   COLD-START §7 `[EFF-18]` bullet removed. Dependent tables checked:
   X.1.1's four `RuntimeCheck` kinds are a different enumeration (correct as
   is); `[SIMD-5]`'s effect subset is intentional, untouched; the Part XXIII
   glossary's short `Effect` row (`Alloc, Sync, Panic, Unsafe, FFI, Block`)
   predates `Lock`/`Io`/`Nondet`/`RuntimeCheck` and is informative rather than
   normative — left as is and noted here rather than widened silently.
   Working copy `ember-spec.md` now runs one editorial repair ahead of the
   frozen `Ember_v0.8.4_Hardened_1.md` snapshot (which is left untouched for
   diffing); the Version header is unchanged.
2. **ERR-044 — header corrected, decision not reopened.**
   `docs/spec-errata.md` ERR-044 body header read "Status: open … Neither side
   has been moved" while its summary row, §0.9, amendment S1, and the entry's
   own resolution footer all record decided. Changed only the header to:
   decided by the owner 2026-09-09, S1 accepted, `[LT-3]` governs, current
   rules are `[TYP-15]`/`[LT-3]` in 0.8.4. No rule text touched.
3. **ERR-041 vs D5 — tracking distinguished.**
   Errata ERR-041 already decided (ADR-017); `DEVIATIONS.md` D5 already
   unratified/awaiting owner. Only `COLD-START.md` §7 titled the open bullet
   ERR-041. Changed §7 to title it **D5** (open deviation) with ERR-041 noted
   decided via ADR-017; removed the resolved `[EFF-18]` bullet in the same
   edit. No semantics touched.
4. **`error_pages.py` — promoted to a real CI gate.**
   Intended contract was already "gate": COLD-START §1 lists it among "6 gates
   green", it enforces normative `[DOC-1]`/`[DIA-6]`/`[PHIL-8a]`, and the only
   contrary evidence was its absence from `ci.yml`. Per the brief's preference,
   added `.github/workflows/ci.yml` step "error pages compile as documented"
   (`python tools/error_pages.py`) after the spec gate build, before the
   Appendix A check. Docs and CI now agree: six enforced gates. YAML validated;
   step order verified.

Validation: `hardening_check.py` (29 diffs, all declared), `split_spec.py
--check` (matches), `rule_index.py` (no new problems), `check_branding.py`
(no new names), `spec_check.py` (no new failures), `error_pages.py` (8 pages,
every fail/fix as documented), `spec_check.py --emit-appendix` (no drift),
`cargo test --workspace --locked` (**178 passed, 0 failed**). `cargo build`
was already green; the one `non_snake_case` test-target warning
(`from_is_contextual…From…`) is pre-existing.

Remaining unresolved, explicitly: **ERR-042** (nine cited-but-undefined rule
ids incl. `IDE-*`); **ERR-043** (`UnsafeCell` undefined; ADR-019 route
unaffected); **`[RNG-8]` tail of ERR-029** (truncated opening, cannot be
restored by guessing); **D-030** (`[DRP-5]` move out of `drop`); **D-038**
(implicit `String`→`str` coercion rejected); **D-040** (method conflicts lack
B8/`E3025`); **D1–D6**
(D5 unratified/awaiting owner; D2 `[CLO-6]` owned-`fn` refusal; D6 `L3011`
unemitted until `RefCell`; D1, D3, D4 as ledgered); compiler-debt `RT-GEN-1`, `LNT-CFG-1`, `TST-6-1`, `CELL-DEF-1`,
`CELL-SYNC-1`, `SPN-API-1`; Phase 2 coverage gaps per COLD-START §4 (13 new
conformance cases added by task 2 — fix-linked ones revert-verified, all
probe-verified). Not reopened or
renumbered here: D-018 and D-025 are **fixed** (see `DEFECTS.md`), `[CLO-3]` /
ADR-018 is **closed** with D2 as the live residual, and **no `[FFI-17]` open
issue exists in any ledger** — `DEFECTS.md`, `DEVIATIONS.md` and
`spec-errata.md` record none, so none is created here. (The `[FFI-17]`
numbered list's past collisions with its tables were settled by the owner's own
`NON-NORMATIVE` demotion under `[CAT-1]` — rules/tables govern — with only a
future owner revision's deletion outstanding per `spec-amendments.md` "Not
amended, and why". That is settled text, not an issue.) Next task remains
`RefCell[T]`, then `Arena` (§0.14); not started here.

### 0.17 Specification authority: frozen vs editable vs generated

**Language revision: 0.8.4. Hardened artifact: `Ember_v0.8.4_Hardened_1`.**
0.8.3 is the previous accepted version and stays accepted for compatibility
(`#! language "0.8.3"` compiles unchanged). 0.8.4 is current and additive over
0.8.3. S1 / ERR-044 is the sole semantic reason 0.8.4 exists. Hardening must
never become an excuse to redesign the language.

| Artifact | Role | Editable? |
|---|---|---|
| `docs/spec-source/as-received/Ember_v0.8.3_spec.md` | the owner's file (md5 `2bdffee6510b8668cf828185266efedb`) | **NEVER** |
| `docs/spec-source/ember-spec.md` | **working normative source — authoritative for implementation** | yes, by declared amendment only (one of the four hardening classes; `hardening_check.py` fails CI otherwise) |
| `docs/spec/` | generated split of `ember-spec.md` | **NEVER by hand** (`split_spec.py --check` fails CI) |
| `docs/spec-source/Ember_v0.8.4_Hardened_1.md` | frozen cut snapshot; the next hardening diffs against it | **NEVER** |
| `docs/spec-amendments.md` | ledger declaring every source-vs-owner difference | yes — it is the permission, not a description |

This assignment is verified, not assumed: `hardening_check.py` sets
`HARDENED = ember-spec.md`; `split_spec.py`'s own docstring calls
`ember-spec.md` "the normative document"; `spec_check.py` (`SPEC`) and
`rule_index.py` (`SPEC`) both read it. **No gate reads the frozen file.**
That is why the working source — not the frozen cut — is what the next agent
implements against.

**E5's standing.** E5 (`[EFF-18]` gains `Nondet`, §0.16) is an EDITORIAL
REPAIR authorised by the already-decided ERR-028: it changes no rule's
meaning, moves no implementation, and therefore forces neither a language bump
nor a new frozen cut. It folds into the next Hardened cut whenever one is
taken. Until then the working source **intentionally** runs exactly one repair
ahead of the frozen snapshot — the E5 enumeration entry plus the front-matter
tense note — and the `**Version:** 0.8.4_Hardened_1` header is deliberately
unchanged so the delta stays reviewable. This is documented here rather than
hidden: the hierarchy has not changed, the snapshot has not been redefined,
and no future agent should "fix" the discrepancy by editing the frozen file,
regenerating it silently, or declaring the files interchangeable.

**Rule: the frozen file, the working source, and the generated split are not
interchangeable.** For code, `ember-spec.md` (+ `docs/spec/`) governs; for
provenance (what Hardened_1 contained), the frozen file governs. On any
difference between them, the working source wins for implementation *if and
only if* the difference is a declared amendment — `hardening_check.py`
enforces exactly this. Anything else is a defect in the handoff, not a licence
to pick a reading.

---

### 0.18 Four factual errors in §0, corrected — and the task list

Written by the agent that authored §0, after re-auditing its own claims against
the repository at `c4fa15c`. **The four below were wrong in §0 as first
committed** (`a2c0032`) and survived the §0.16 pass because nobody had checked
them. They are corrected in place above; recorded here so that a reader who took
notes from the earlier text can find out what changed.

| # | Where | Said | Actually |
|---|---|---|---|
| 1 | §0.9 | the S1 conformance pair is in `tests/conformance/LT-3/` | **`tests/conformance/TYP-15/`**. `LT-3/` is a different directory holding one case, `accept_a_literal_in_a_view_struct.em` |
| 2 | §0.2 | "11 conformance cases" for `Cell` | **9**. The count included the two `OWN-5` cases from D-035 |
| 3 | §0.14 | `[CELL-7]`'s `L3011` "needs `LNT-CFG-1` first, or there is nowhere to opt in" | **False, and it mattered**: `LNT-CFG-1` is about `[LT-1b]`'s `L3014`. `[CELL-7]` says `L3011` *fires*; it is not opt-in and not blocked. It **is** registered in `compiler/ember_diag/src/codes.rs` and emitted by nothing — real `RefCell` work, and would have been skipped as blocked |
| 4 | §0.14 | "`RefCell` is **never `Copy`**", stated flat | **Inference, not specification.** Part IX says nothing about `RefCell`'s `Copy`-ness. The reasoning is sound; presenting it as the document's is the exact failure §0.8 exists to prevent |

Error 4 is the instructive one. It is the shape of both withdrawn amendments —
**a correct analysis written down as though the specification had said it** —
committed by the same agent that wrote §0.8 warning against it, in the same
document. The lesson is not "be careful": it is that **the discipline has to be
mechanical**, because judgement does not catch this. Before asserting a rule,
`grep` it and quote it. If the quote does not exist, say "inference" in the
sentence.

Errors 1 and 2 are the cheaper lesson: **a path or a count in prose is a claim,
and claims get checked.** Both were verified in under a minute with `ls` and
`find`, after being wrong in a committed document for two commits.

#### Standing rule this adds

> **Anything in a handoff that names a file, a directory, a count, a rule id or
> a symbol is checkable. Check it before writing it, and re-check it when you
> inherit it.** The `[EFF-18]` and `L3011` findings both came from re-checking
> an inherited claim rather than from new work.

---

## The task list — where to begin

`RefCell[T]` remains the next milestone (§0.14). These are ordered so that the
cheap verification that de-risks it comes first; **1 and 2 together are under a
day and 2 is what protects everything after it.**

**Ask before spawning subagents or a workflow, and state the worst-case agent
count (§0.11). Report after each task and wait for the green signal before
starting the next.**

### 1. Close `L3011`, or file it properly — half a day

`[CELL-7]`'s lint is registered and emitted by nothing. That is deviation D4's
shape (`E9012`, same), and D4 is already open. Decide deliberately: either
`RefCell` emits it as part of task 3, or it joins D4 in `DEVIATIONS.md` with a
reason. **Do not leave a third registered-and-unemitted code undocumented** —
that is how a registry stops meaning anything. This is listed first because it
is small and because §0.18 shows it was about to be skipped as blocked.

**Done — filed as D6.** `codes::L3011` is referenced nowhere in the compiler
(checked twice: once by enumeration, once by targeted grep — the only other
mention of the number is the `shapes.rs` catalogue mapping), and no guard
exists to observe, so emission now would be D1-shaped scaffolding. `D6` records
the rule verbatim, the reason, and the closure condition: task 3 emits it from
guard-liveness tracking when `Ref`/`RefMut` land. No spec file touched; no
semantics decided.

### 2. The D-035 sweep — one to two days, the best workflow candidate

D-035 existed because `tests/conformance/OWN-5/` covered one clause of a
two-clause rule and passed. **Nothing has looked for siblings.** Take the
ownership, drop, borrow, lifetime and view rule families and, for each, read the
rule, count its normative clauses, and check that a case exists which fails when
that clause is violated.

Start with the directories holding a **single** case, since one case can only
test one clause: `LT-1`, `LT-1a`, `LT-3`, `BRW-3`, `BRW-4`, `BRW-6`, `DRP-2`,
`OWN-7`, `SPN-3` — complete for these families as of `c4fa15c`. Then `OWN-1`,
`OWN-2`, `OWN-3`, `OWN-4` (two each). Six more single-case directories exist
outside these families (`CELL-11`, `GRM-21`, `MOD-1`, `MOD-3`, `RNG-8`,
`TYP-5`) and are out of scope here, not overlooked.

**Why before `RefCell` and not after:** `Ref`/`RefMut` are guards whose `drop`
releases the borrow state, so `RefCell`'s correctness rests directly on the drop
machinery D-035 was found in. Building guards on an unaudited drop path is how
a second D-035 gets buried under a feature. **Method:** probe with a minimal
program (§0.10); where the behaviour is invisible in output, `assert-c` or
`assert-c-order`; break every new case red once before trusting it.

**Done — ran solo, no subagents** (probes batch cleanly against one binary;
coordination would have cost more than it saved — same call as §0.11's Cell
note). Found two live defects and fixed both; filed two more; added 13 cases:

* **D-036 (fixed): `Array.push` of a moved value destroyed it twice** — the
  spill temporary kept its statement-end drop while the buffer took ownership
  (`[OWN-3]`). `tests/conformance/DRP-2/accept_array_elements_drop_in_index_order.em`
  prints 1, 2 and goes "1 2 1 2" red on revert.
* **D-037 (fixed): `as_str()` built a view with no borrow** — D-022's twin for
  strings; mutating while the `str` lived compiled and dangled.
  `tests/conformance/SPN-1/reject_mutating_around_a_live_str_view.em` fails as
  required and compiles on revert.
* **D-038 (open): implicit `String`→`str` coercion rejected** — fails closed;
  no case pins the rejection since the rule admits the program.
* **D-039 (fixed) / D-040 (open): reservation windows opened for user-local
  borrows** (two `ref mut` borrows reported B3/`E3021` instead of B1/`E3022`;
  now temp-only, with a case red-checked both ways). Method-call conflicts
  still lack B8/`E3025` — registered, shape-mapped, emitted by nothing — which
  needs call provenance at the reporter; no case pins that shape.
* **SPN-API-1 (debt): the span view-method surface** (`split_at`, `reborrow`,
  …) is refused by name and untracked until now; one case pins the message.
* **Nine more clause cases**, each probed first: DRP-2 tuple order, DRP-2 enum
  payload, OWN-4 every-path quantifier, BRW-6 shared-reborrow freeze, BRW-3
  `mut`-argument two-phase (+`[FN-1]` adhesiveness), BRW-4 nested disjointness,
  LT-1 rule-3 force, SPN-3 `MutSpan` move-only, OWN-7 derived-`Copy`.
* **Deliberately not cased:** LT-3 static-item stores (documented S1
  conservatism, fails closed — G1 probe); `CallIndirect` owned args (same
  erasure shape as D-036, dispatch machinery, noted for a future sweep, not
  probed); OWN-7 no-drop and OWN-1 owner-kinds (structural/non-observable).

Coverage claimed and checked corpus-wide, not per-directory: LT-1 rules 1–2,
LT-1a `E2031`/`E3062`, BRW-6 suspend/resume and the OWN-3 drop-flag path all
live in `run-pass/`/`compile-fail/` with tags — the D-035 lesson cuts both
ways, and the sweep mapped clauses to cases wherever they live.

### 3. `RefCell[T]` — the milestone

Scope, each tied to its rule id: `[CELL-5]` the one-word borrow counter,
`borrow`/`borrow_mut`, and the panic naming **the conflicting borrow's source
location in debug *and* release**; `[CELL-6]` `try_borrow`/`try_borrow_mut`
returning `Option`; `[CELL-6a]` — no profile may make them infallible, because
that changes which `match` arm runs, which `[PRF-1]` forbids; `[CELL-7]`
`Ref`/`RefMut` as **view types** with `[TYP-15]` applying and `drop` releasing
the state; `[CELL-8]` `!Sync` (blocked with `Cell`'s — `CELL-SYNC-1`);
`[CELL-9]` the check in **every** profile, and `exclusivity = "unchecked"` MUST
NOT reach it; `[CELL-10]` shape B4 may suggest `RefCell` **only** when the
accesses are provably not simultaneous, never first; `[CELL-11]` prelude.

**Escalate, do not decide: whether `RefCell[T]` may be `Copy`** (§0.18 item 4).

§0.14 has the read-first list, what carries over from `Cell`, and the
conformance expectations. **`[CELL-7]` is why this is the real milestone** —
guards escape, so `regions.rs`, `[TYP-15]`, borrow checking and guard lifetime
all become load-bearing in a way `Cell` never touched.

### 4. `Arena` — after `RefCell`

`[ARN-*]`. **Not a third interior-mutability primitive** (§0.13): it is a region
allocator, and what it must prove is a region rather than an alias, statically.
Do not build it by generalising either of the other two.

### Not yet, and why

* **`[DIA-7..10]` + `tests/ui/` snapshots** — a Phase 2 *exit* criterion at
  0/5, so it must happen, but after block I: three past defects were the
  compiler rejecting the right program under the wrong *shape*, and `RefCell`
  will add shapes. Writing the snapshots first means writing them twice.
* **`D-030`** — small and open, but it needs a `drop` body that moves and
  nothing in the corpus has one. Fold it into task 2, which is already reading
  the drop rules.
* **`CELL-DEF-1` / `CELL-SYNC-1`** — blocked on `Default` and on
  `Send`/`Sync`/threads. Not startable.

---

## READ `docs/COLD-START.md` FIRST

> **Everything from here down is the phase plan and the historical record.**
> §0 above is the current snapshot and governs wherever the two differ. This
> part is kept because the reasoning behind a decision is worth more than the
> decision, and several sections below are the only place a particular trap or
> design argument is written down.

It carries the current state, the rules that govern edits to the specification,
the next task, and the traps. **This file is the phase plan and is partly
historical** — several of its sections describe work that has since landed, and
its own "Start here" is marked superseded. Trust `COLD-START.md` and §0 where
they disagree.

`docs/spec-source/ember-spec.md` is **v0.8.4_Hardened_1**, split into
`docs/spec/`. The owner's file is `as-received/Ember_v0.8.3_spec.md` and is
**never edited**; every difference between the two is declared in
`docs/spec-amendments.md` and `tools/hardening_check.py` fails CI otherwise.

An earlier model — applying errata as edits to the source — was **reverted on
the owner's instruction**. The specification is the contract and is never
altered to make the compiler agree with it. Where the document contradicts
itself, neither side moves until the owner rules. `COLD-START.md` §2 has the
whole of it, and it is worth reading before touching anything under
`docs/spec-source/`.

### What arrived

Seven revisions at once — 0.6, 0.6.2, 0.6.3, 0.7.1, 0.7.2, 0.8, 0.8.1, 0.8.2b,
0.8.2c, 0.8.3 — the repository having been on 0.5. **833 rule ids, 677 stated,
205 diagnostic codes.** A new Part XVIII (Hot Reload); everything after it
renumbered by one, so the old Part XVIII (Compiler Architecture) is now XIX,
XIX (Toolchain) is XX, XX (Implementation Plan) is XXI, XXI (RageV) is XXII,
and XXII (Open Questions) is XXIII.

**Checked before anything was changed: no v0.5 rule id is absent from v0.8.3.**
The set difference is empty in that direction. Nothing already implemented was
invalidated — the diffs over Parts II–VII are 3, 22, 92, 12, 31 and 2 lines,
and Part IV's 92 are §IV.2a's range types in their entirety.

**Contracts are gone.** 0.6.2 removed `@requires`, `@ensures`, `@invariant`,
`@decreases`, `@verified`, `@assume`, the `[CTR-*]`/`[PRV-*]` rules, the proof
manifest and the solver (`OQ-27`). `docs/RFC-v0.6.md` argued for taking them
from Aegis; that argument was heard and declined. It is **history, not a
plan** — kept because eight of its eleven features did ship and it records why.

### Two questions left open for the owner

Both are Phase 5/7 and nothing before them depends on the answer. Neither was
decided silently; provisional readings are in `docs/spec-errata.md` and
`docs/DECISIONS.md`, and the grammar is **not** changed until the owner rules.

* **ERR-036** — Part III §2's `fn_header` has no `extern` prefix, so
  `pub extern "C" fn on_update(…)` — which XVI.10 writes and `[FFI-26]`,
  `[FFI-28]`, `[FFI-31b]` and `[HR-21]` all depend on — parses under no
  production. Provisional: the prefix belongs on `fn_header` (ADR-014).
* **ERR-037** — Part III §2 has no `extern class` production, though
  `[FFI-39]` spends forty lines specifying one. **No provisional reading
  taken**: there is no smallest-change answer, and writing a production would
  decide the shape of the C++ inheritance surface silently.

Both blocks sit in `[TST-7]`'s baseline, which is where a gap the gate exists
to expose belongs.

### Where the work is

**Phase 0 and Phase 1 complete; Phase 2 in progress.** Against `[CONF-2]` —
Ember Core is "Parts II–VII and XIII […] and range types" — what landed on
2026-09-09:

| What | Rules |
|---|---|
| `yield` a v1 keyword (49, not 48); `gen` contextual | `[LEX-15b]`, ERR-025 |
| `range_clause`, `gen fn`, `yield` parse | `[GRM-8d]`, `[GRM-21]`, `[GRM-22]` |
| 62 diagnostic codes registered; the registry is exhaustive again | `[DIA-6a]` |
| **Range and domain types**, all of §IV.2a except `[RNG-4]` | `[RNG-1..10]`, `[TYP-5]` |
| `std` is a resolvable package with three modules | `[MOD-1]`, `[STD-1]` |
| `Self` resolves in every signature | Part IV §8 |
| interface collection is whole-program | `[IFC-1]`, `[MOD-4]` |
| field visibility, `pub(read)`, the memberwise constructor's | `[MOD-2]`, `[MOD-7]`, `[STR-1]` |
| the conformance suite is **run**, and its baseline ratchets | `[TST-4]`, `[TST-4a]`, `[TST-4c]` |

**Eight defects found and fixed in the same period** (D-014..D-021 in
`docs/DEFECTS.md`), every one of them something the compiler let a program do
that the specification forbids. Two are worth knowing about before touching
anything:

* interfaces were collected **per module**, immediately before that module's
  `implements`, so a module checked first could not implement an interface
  declared later. `[MOD-4]` allows import cycles inside a package, so no load
  order would have worked;
* **field visibility was not enforced at all**, and fixing it turned
  `tests/run-pass/modules_across_files.em` red — a test that had been green on
  the strength of the gap.

### What is next, in order

`docs/MIGRATION-0.8.3.md` §4 is the full route. The head of it:

1. `[RNG-4]`'s range tracking (D-018 — **fixed** per `DEFECTS.md`; what remains
   is precision later-work, not an open defect — half of it waits for a generic
   `min`/`max` over a numeric bound).
2. Closures (`[CLO-*]`), `Span`/`MutSpan` (`[SPN-*]`), `Cell`/`RefCell`
   (`[CELL-*]`), `Arena` (`[ARN-*]`), `assert_disjoint` (`[DSJ-*]`) — the rest
   of Phase 2's list.
3. The diagnostic shape catalogue with `tests/ui/` snapshots and `[PHIL-8a]`'s
   before-and-after verification, which is a **Phase 2 exit criterion** and has
   never been run.
4. `docs/errors/EXXXX.md` for every code emitted, per `[DOC-1]`.


## Where the specification lives

**Superseded by v0.8.3 above.** This section records the three-way split,
which is unchanged and is the reason a revision can be applied without losing
a ruling.

| Path | What it is |
|---|---|
| `docs/spec-source/as-received/Ember_v0.8.3_spec.md` | byte-identical to what the owner sent. **Never edit it.** It is what the next revision is diffed against. `Ember_v0.5_spec.md` beside it is the previous one, kept for the same reason |
| `docs/spec-source/ember-spec.md` | the **normative** document: as-received plus the errata rulings in `docs/spec-errata.md` |
| `docs/spec/` | generated from it by `tools/split_spec.py`. **Never hand-edit.** `tools/split_spec.py --check` fails CI if anyone does |

That three-way split exists because v0.5's own ground rule 3 records how 0.3
silently reverted four owner rulings: it was authored from an unpatched copy.
Keeping the pristine file *and* the errata means either side is reconstructible.

### What v0.5 changed under work that was already finished

Two of the four rulings the owner had made were **reversed** by v0.5:

- `let` is **fully reserved**, not contextual (`OQ-26`, reverses ERR-004).
- `return`, `break` and `continue` are **expressions** of type `!`, not
  statements (`OQ-14`, reverses ERR-008). This is what makes
  `Circle(r) => return PI * r * r` parse.

The other two (`E0102`/`E0103`/`E0104`, and a dangling doc comment being
silent) were carried into the document by the owner. `1f32` and `;` were
decided the way the implementation had already guessed.

### Seven defects in v0.5 itself, found and fixed before any of it reached code

`docs/spec-errata.md` ERR-009 … ERR-016. **Six of the seven had one cause**:
0.4's and 0.5's amendments were *appended* to each Part instead of
*substituted into* the rule they amend. Two signatures, both now checked by
`tools/rule_index.py`:

- a rule id stated twice (`[EFF-12]`, `[PAR-2]`);
- a rule whose body begins with `…` (`[MOD-6]`, `[CLO-2]`, `[FFI-6]`).

**`[MOD-6]` and `[CLO-2]` had lost text that survives in no copy of v0.5.** It
was restored from the committed split of v0.2 — which would have been destroyed
had `docs/spec/` been regenerated before the diff was read. That is the reason
to diff first and regenerate second, every time.

The seventh (ERR-016) is the standard-interface list in Part IV §8: every one
of its twenty declarations put its members on the header line, which
`interface_decl` has never admitted, and six also used `;` as a separator,
which `OQ-25` had just outlawed. Rewritten indented.

`type` was in both keyword lists (ERR-009). It is now a v1 keyword and the
reserved set has **49** entries, not 47.

## Applying a new specification version

The procedure that worked for v0.5, in order:

1. **Copy the owner's file to `docs/spec-source/as-received/` untouched**, then
   copy it again to `docs/spec-source/ember-spec.md` as the working normative
   copy.
2. **Split it to a scratch directory** with `tools/split_spec.py` and **diff
   each part against the committed one** before changing anything. The diff is
   the specification of the work; read all of it before applying any of it.
   Do not regenerate `docs/spec/` until this is done — it is the only copy of
   whatever the new document deleted.
3. **Apply errata to `ember-spec.md`** with a script that asserts each
   replacement matched exactly once, then regenerate `docs/spec/`.
4. **Record every deliberate difference** in `docs/spec-errata.md`.
5. **Run `python tools/rule_index.py`** — duplicate rule ids, orphaned
   amendments, codes missing from the registry, registry entries citing rules
   that do not exist.
6. **Watch the line endings.** `.gitattributes` pins LF and the owner's file is
   LF; `split_spec.py` now writes LF explicitly, because Python's text mode was
   turning it into CRLF on Windows and leaving the tree mixed.

## Start here — SUPERSEDED

> **This section is history, kept for the record. Read
> `docs/COLD-START.md` instead.** Everything below that it calls "not
> started" is built: the NLL borrow checker (block E) with real region
> variables, and closures including capturing ones under `[CLO-3]`. Leaving a
> stale "start here" in place is worse than having none, because a cold start
> reads exactly this paragraph and acts on it.

**Phases 0 and 1 of nine were complete; Phase 2 is in progress and is roughly a
third done.** Phase 2's blocks A to C — moves, drops and drop flags, generics,
and iterators — are done, and so is the core of block G: `unsafe` blocks, raw
pointers and the memory builtins. Block D is writable but not written. **The
NLL borrow checker (block E) is the largest single piece left in the project
and is not started.** Closures are not started.

**v0.5 reopened Phase 0.** `[TOOL-1]`..`[TOOL-4]` add milestone **M0, "the
first hour"**, as a *Phase 0 exit criterion*: on a clean machine with no C
toolchain, `install → ember new hello → cd hello → ember run` must print
`hello, world` in under five minutes and at most six typed commands, on Windows
and Linux. `[TOOL-2]` makes a bundled Clang and `lld` a committed deliverable,
not a conditional one. Neither is built.

**Phase 2 grew again** (Part XX §2): besides `Cell`/`RefCell` and the borrow
diagnostics the 2026-09-08 amendment added, v0.5 puts `assert_disjoint` /
`assume_disjoint` with proof-carrying returns inside it, and `[LT-7]`'s
late-bound callback regions and `[TYP-15a]`'s `BorrowList`/`ViewList` land
*inside* the borrow checker rather than beside it. Block E got bigger.

### Conformance work done 2026-09-08, after v0.5 landed

None of this advanced a phase: it brought the finished phases into line with
the new document. All of it is green.

| What | Rules |
|---|---|
| 49 keywords; `let` and `type` fully reserved | `[LEX-15]`, `[LEX-15a]` |
| `E0005` names the version that will take the word | `[LEX-14a]` |
| `type` aliases at item level, resolved to a fixpoint so one alias may name another | `[LEX-15a]` |
| `return`/`break`/`continue` are expressions; `E0107` when one is an operand | `[GRM-16]` |
| `E0105` for `;` between statements, `E0109` for stray `owned`, `E2036` for an irrefutable condition | `[GRM-18]`, `[GRM-15]`, `[GRM-19]` |
| statement attributes: the four permitted, only on a compound statement, three only on a `for` | `[ATT-2]`..`[ATT-4]` |
| `E2035` — a condition must be `bool`, with the fix chosen by the operand's type | `[CTL-0]` |
| `W2015` — a defaulted float literal written with more precision than `f32` holds | `[LEX-17a]` |
| `E0006` — the `#! language` directive is checked against the supported set | `[MOD-6]` |
| 34 codes named by v0.5 added to the registry; it is exhaustive again | `[DIA-6a]` |
| `E0006` — `#! language` is checked; `--syntax-only` and flags before the file | `[MOD-6]`, `[CLI-9]` |
| every MIR statement and terminator must carry a source span, checked by the verifier | `[CG-C-8]` |
| `from` made contextual so `interface From[T]` can declare `fn from(…)` | `[LEX-15]`, ERR-017 |
| the `branding` module: the symbol prefix, CLI name, manifest and extensions are spelled once | `[RT-5]`, `[MAN-4]`, `[MNG-5]` |
| Appendix A generated from a fixture that is checked | `[TST-6]` |
| type arguments inside `name[…]` in expression position, with `E2172`/`E2173` | `[GRM-8a]`..`[GRM-8c]` |
| `tools/rule_index.py`, `tools/spec_check.py`, `tools/check_branding.py`, `tools/split_spec.py --check`, CI workflow | `[XXII.4]`, `[TST-4]`, `[TST-7]`, `[RT-5]`, Phase 0 |

**Three defects found by writing the tests, not by reading:**

- **A `##` comment inside a function body deleted the statement under it.** The
  parser returned "no statement" for the comment, and the caller treated that
  as a parse failure and skipped to the next line. A comment must never change
  what a program does (ERR-007); this one removed code.
- **`let` was dropped by the formatter**, so `fmt` turned an immutable field
  into a mutable one and `[FMT-1]`'s round trip was false for every `let`.
- **Doc comments were dropped by the formatter entirely.** No test file had one
  until now, so nothing caught it.

**Two more defects came out of `tools/spec_check.py`** once it existed, both
now recorded: `interface From[T]` could not be declared because `from` was a
reserved word (ERR-017 — the owner ruled it contextual, and set the general
rule: a reserved word that blocks a name the standard library must declare
becomes contextual, never renamed and never `r#`), and two `assert` examples
were written without parentheses (ERR-018).

**The `[TST-7]` gate's current state: 14 of 38 fenced `ember` blocks parse.**
The other 24 are fragments — bare statements at file level, which `[GRM-2]`
forbids — and are baselined in `tools/spec_check_baseline.json`. Shrinking that
number is what the gate exists to drive.

**The branding refactor is done and was proved by construction.** Hard-coded
occurrences went from 96 to 13, and the emitted C for all 17 test programs is
byte-for-byte identical before and after — which is also what caught the first
attempt, where `{RT}` in a string passed to `line()` rather than `format!`
emitted the braces literally and no test would have noticed. The 13 that remain
are fixture file names inside tests, baselined; the runtime's own two files are
exempt for the reason ERR-019 records.

**Still open from v0.5:** `[GRM-17]`/`[LEX-6a]` closures inside brackets, which
wait on closures themselves (Phase 2 block F); and shrinking `[TST-7]`'s
24-block baseline, which needs the grammar gaps those fragments expose. With
those, the conformance catch-up is done and Phase 2 resumes at block D or E.

**`docs/spec-source/Ember_v0.6_spec.md` is the full v0.6 specification** (new,
2026-09-09): the whole normative document, v0.5 with the v0.6 rules substituted
into the Parts they belong to rather than appended beside them. It is **not yet
normative** — `docs/spec-source/ember-spec.md` is still v0.5 and `docs/spec/` is
still generated from it. When the owner rules, the procedure at the top of this
file applies: copy to `as-received/`, diff each part, then regenerate.

**`docs/RFC-v0.6.md` is the rationale behind it** (new, 2026-09-09), written after
reading the owner's friend's Aegis design specification alongside the owner's own
v0.6 RFC. Eleven features taken from Aegis, stated in Ember's grammar and rule
numbering, amending v0.5 by rule id rather than restating it. **Nothing in it is
normative until the owner rules**, and §10 carries the three decisions that block
everything else: how contracts are spelled, how a range type is declared, and
what arithmetic a contract expression uses.

**`docs/DEFECTS.md` is the defect ledger** (new, 2026-09-09). Every defect and
whether it is closed, with how the fix was verified. Defects used to be recorded
in prose here, spread across the per-block sections, where "found" and "fixed"
read the same — a reader could not tell which were still open. Add a row when
you fix one. A defect in the *specification* still goes to
`docs/spec-errata.md`; the ledger points at the errata id.

`docs/LIBRARIES.md` records what ships in `std` and what the ecosystem should
gain once the phases are done, with the reasoning; `docs/BACKLOG.md` carries
the same list as 31 numbered tasks, `LIB-1`..`LIB-31`, each with the phase it
is gated behind and what finishing it means. Both at the owner's request, and
nothing in either starts before Phase 8.

Read `docs/spec/` (the specification, split by part) and `docs/DECISIONS.md`
before touching anything. `docs/spec-errata.md` lists **twenty-four** entries.
ERR-001..ERR-008 are closed or carried by v0.5; ERR-009..ERR-022 came out of
applying it; ERR-023 and ERR-024 came out of the region work on 2026-09-09,
where the document's own wording admitted the reading the compiler took. **Three of those are withdrawn or half-withdrawn, and the reason
is worth reading before adding another:** ERR-014, ERR-019 and ERR-022 recorded
contradictions that were not there, and ERR-020 rewrote a requirement to match
what the compiler can do today.

**The pattern to avoid: "the implementation does not do this yet" is not "the
specification is wrong."** Each of those three read a rule as self-contradictory
when a careful reading resolved it — the C-header diagram describes one
pipeline stage and the overlay another; `[RT-5]`'s header is *generated*, so
literal identifiers in it cost no source file anything; `[DIA-7a]` sits under
`[DIA-7]`, which scopes it. Compiler debt now goes in `docs/BACKLOG.md`'s
compiler-debt table instead: `RT-GEN-1`, `LT-REG-1`, `LNT-CFG-1`, `TST-6-1`.

**Before recording a defect, quote both halves and read them together.** The
owner caught all three, in a document this session had already read twice.

| | |
|---|---|
| Repository | `https://github.com/Insomniac-Coder/ember.git` |
| Pushed | `11a7922` on `origin/main`. `main` and `phase-1-core-language` are identical and both pushed; the working tree is clean |
| Working branch | `phase-1-core-language`, now identical to `main`. The owner merged it on 2026-09-08; work from here can go on `main` or a new branch |
| Tests | `cargo test --workspace` → **161 passed, 0 failed**; 49 `.em` programs under `tests/`. The count of Rust tests does not move when `.em` files are added: one `#[test]` walks a whole directory |
| Build | warning-free; the emitted C is warning-free under `clang -Wall -Wextra` and MSVC `/W3` |

## How to check everything

Four gates, all in `.github/workflows/ci.yml`. Run them before and after any
change to the specification or the diagnostics:

```
cargo test --workspace
python tools/rule_index.py        # duplicate rule ids, orphaned amendments, code registry
python tools/spec_check.py        # every ember block in Parts I-XVII and Appendix A parses
python tools/check_branding.py    # no file spells the project's names but ember_branding
python tools/split_spec.py --check docs/spec-source/ember-spec.md docs/spec
```

Three carry a baseline of known gaps and fail only on new ones:
`rule_index_baseline.json` (rules with no conformance directory, codes with no
error page), `spec_check_baseline.json` (23 blocks that are fragments), and
`check_branding_baseline.json` (13 fixture file names in tests). **Both are
keyed by content, not by line number** — an edit elsewhere in the document used
to invalidate the whole spec-check baseline, and a gate that cries wolf on an
unrelated edit is worse than no gate.

`cargo` is not on `PATH` in this environment's bash; use
`export PATH="$HOME/.cargo/bin:$PATH"`. `clang` lives in
`C:/Program Files/LLVM/bin`.

## Hard constraint

**Never read, build or modify anything in `Code/RageV`.** Three conditions must
all hold before engine code may even be raised as a question (owner,
2026-09-07):

1. all nine phases complete — not Phase 5, when the C FFI first makes it
   technically possible;
2. the owner has inspected the work themselves and given an explicit green
   flag;
3. any engine work then happens on a separate RageV branch named
   `RageV_Ember`, never the working branch.

The specification's Part XXI is a *description* of a possible integration, not
a work queue.

## Working with this owner

- **Explain in plain words, and lead with the consequence, not the mechanism.**
  Several long explanations were rejected outright during Phase 0. What landed
  was a table of "you write X / it used to do Y / it now does Z".
- **Show, do not describe.** Running the compiler and pasting its real output
  settled arguments that four written explanations did not.
- Report at block boundaries, not per file.

## Phase 1 — where it stands

Seven blocks, dependency-ordered. All are done.

| | Block | State |
|---|---|---|
| **A** | numeric semantics `[TYP-4..10]`, `Assert` lowering, definite-init | **done** |
| **B** | tuples, fixed arrays, enums, `match` with exhaustiveness | **done** |
| **C** | `for` over ranges, loop `else`, labelled break, `with`, `defer` | **done** |
| **D** | interfaces, operator interfaces, method resolution, `extend`, visibility | **done** |
| **E** | `String`/`Array`, f-strings, `Option`/`Result`, `?` | **done** |
| **F** | modules across files, `const`, `static` | **done** |
| **G** | formatter (`[FMT-1]`) | **done** |

### Block A, in detail

- `[TYP-8]` — `debug` panics on overflow, `release`/`shipping` wrap.
  `@overflow(panic|wrap)` overrides per function. `/` and `%` by zero always
  panic whatever the policy, as does `T.MIN / -1`.
- `[TYP-10]` — a shift amount at or past the type's width panics under `panic`
  and is masked under `wrap`.
- `Terminator::Assert` in MIR, carrying the span of the operation, reaching
  `ember_panic_*`. Checked arithmetic goes through `ember_ck_*` in
  `ember_rt.h`: `__builtin_*_overflow` on clang and gcc, exact widening
  fallbacks for MSVC.
- New crate **`ember_analysis`** with definite initialisation — the
  `Uninit | Init | Maybe` lattice of Part XVIII §4.6, reported as `E3050`.
  The NLL borrow checker (§4.7), exclusivity (§4.8), drop elaboration (§4.9)
  and effects (§4.10) belong in this crate too.
- MIR statements and terminators now carry spans (`Stmt { kind, span }`,
  `BasicBlock::terminator_span`).
- The test harness handles `tests/compile-fail/` and `tests/run-fail/`.

**Deferred, deliberately:** `@overflow(saturate)` is rejected with a clear
diagnostic rather than silently treated as `wrap`. It needs each type's bounds
as constants, and the backend cannot yet render a signed minimum without
tripping C's `-2147483648` parsing quirk (that literal is `-(2147483648)`,
which overflows `int`). Emit `(-MAX - 1)` when this is picked up.

### Block B, part one: tuples and fixed arrays

Done end to end — written, run, and checked against the emitted C.

- **C has no tuple, and a bare C array cannot be assigned, passed or returned
  by value**, but an Ember tuple and an Ember `[T; N]` are ordinary values
  (Part IV.3). Both are therefore emitted as a **generated struct** with the
  same layout: `struct em_tup_i32_f32 { int32_t _0; float _1; };`,
  `struct em_arr_i32_4 { int32_t _0[4]; };`. C's value semantics then apply
  with no change to any copy, argument or return path in the backend. The
  alternative — bare arrays plus `memcpy` and a hidden out-parameter for
  returns — is more work *and* a worse result, and it still needs generated
  structs for tuples.
- `plan_types` in the C backend orders every type definition so that a type is
  defined after everything it holds by value, and names the generated ones
  from how they print (`[[i32; 2]; 2]` → `em_arr_i32_2_2`), made unique
  against every other name. A reference or pointer is not a containment edge,
  because everything is forward-declared.
- **Indexing is bounds-checked** (Part IV.3). The comparison and its
  `Assert` are built during MIR lowering, not in the backend, so the check is
  visible to every MIR analysis — the same choice `[TYP-8]` made. Block A had
  already built `AssertKind::Bounds` and `ember_panic_bounds`; nothing had
  generated them until now.
- `Rvalue::Repeat` keeps `[value; count]` as one statement rather than `count`
  operands, and the backend emits a `for` loop. `[0; 4096]` would otherwise be
  four thousand entries in the IR and in the C.
- **`t.0.1` did not parse**: the lexer reads `0.1` as one float literal, so
  every chained tuple index was a syntax error. `eat_tuple_index_pair` splits
  the token in the parser, reading the text from the source rather than from
  the `f64` (which no longer tells `.1` from `.10`). Three levels work, because
  `1.1.0` arrives as a float and then a separate `.0`.
- An array length must be an integer literal for now (`E2131`); Part IV.3 makes
  `N` a const generic, which needs const generics and `const` items.

**Not done, and deliberately:** Part IV.3's coercion of `[T; N]` to
`Span[T]`/`MutSpan[T]` at coercion sites. `Span` is a standard-library type and
arrives in block E.

### Block B, part two: enums and `match`

- **A unit-only enum is a `typedef` of its repr integer** (`[ENM-3]`), so it
  costs nothing and `as` to an integer is the identity. **A payload enum is
  `{tag, union}`** (`[TYP-12]`), the union holding one anonymous struct per
  variant that has fields. `@repr(u8)` names the tag type; without it the
  compiler picks the smallest of `u8`/`u16`/`u32` (or the signed ones, when a
  discriminant is negative) that holds every value, which is what Part IV.3's
  "`i32`-sized-or-smaller, chosen by the compiler" means.
- `SwitchInt`'s case values became **`i128`, not `u128`** — a negative
  discriminant rendered through `u128` prints a huge positive number into the
  emitted `switch`.
- **Exhaustiveness and redundancy are one algorithm** (Maranget's usefulness
  test, `compiler/ember_typeck/src/usefulness.rs`). An arm is `W2091` when it
  is not useful against the arms above it, and the `match` is `E2090` when a
  bare `_` still would be — so "not covered" and the witnesses it lists cannot
  disagree. A guarded arm covers nothing, because its guard may fail.
- **`match` lowering has two shapes.** When every arm names a different
  variant, with no guard and payloads only bound, the whole thing is one
  `SwitchInt` on a tag read once — a real `switch` in the C. Anything else
  (guards, wildcards, two arms on one variant, a non-enum scrutinee) falls
  back to a chain of per-arm tests, which handles every pattern form at one
  test per arm. Both were measured against the emitted C, not assumed.
- **Every alternative of an `|` pattern binds the same locals.** Checking each
  alternative separately would declare a new local per alternative, and the
  body would read whichever was declared last — so the first alternative's
  bindings are recorded and the rest reuse them (`or_bindings` in the checker).
- **Three defects found and fixed on the way**, all pre-existing:
  - `println(10.0)` printed `1e+01`. `write_shortest_f64` took the first
    *precision* that round-trips, and `%.1g` of 10.0 is `"1e+01"`, which does.
    It now keeps the shortest *text*, so `10` wins and `1e+20` still beats
    twenty-one digits.
  - `n = match d:` with indented arms did not parse: the match consumed the
    line's end along with the block's `Dedent`, and the enclosing statement
    then demanded a newline that was gone.
  - An unused pattern binding tripped `-Wunused-but-set-variable`, breaking
    `[CG-C-1]`. Locals that are written and never read are now discarded once
    with `(void)`.

**Not done:** slice patterns and range patterns (both parse, both are reported
as unsupported); `from_repr` (`[ENM-3]`) needs `Option`, which is block E;
`[TYP-13]`'s niche optimisation, which Phase 3 gives something to optimise.

### Block C: `for`, loop `else`, labels, `with`, `defer`

- **`for i in a..b` is a counted loop and nothing else** (`[CTL-3]`). It is not
  desugared through `Iterator` — that needs interfaces — so `hir::Stmt::ForRange`
  carries the two ends directly and MIR builds the counter. The emitted C holds
  two `int32_t` and a `bool`, which is what `[CTL-3a]` asks for; the test
  asserts no range struct appears.
- **The continue target is not the loop head.** A counted loop's `continue`
  must run the increment or it spins forever, so the loop stack records where
  `continue` goes separately from where `break` goes.
- **Loop `else` needs no flag** (`[CTL-4]`). Falling out of the condition goes
  to the `else` block; `break` jumps to a second block past it. The two exits
  are different basic blocks, so nothing is tested at run time.
- **A label is resolved to a depth while checking**, so MIR never sees a name —
  `break outer` becomes `break 1`, and the lowering indexes its loop stack.
- **`defer` runs on every way out** (`[CTL-7]`, `[CTL-8]`): the end of the
  block, a `return`, and a `break` or `continue` that leaves the scope. The
  pending list is *not* shortened when a `return` emits it, because another
  branch of the same block still has to run the same blocks on its own way out.
  Each loop records how many defers were pending when it opened, which is what
  `break` and `continue` unwind to.
- `[CTL-7]`'s restriction is checked: `return` inside a `defer` is `E2160`, and
  the loops outside are hidden while checking one, so `break` cannot reach them.

**Not done:** `for` over anything but a range (needs `Iterator`, so block D or
E); `a..` with no end; destructors at `with`-block exit, which is what makes
`with lock.acquire():` a guard — until drop elaboration exists, `with` binds
for the block and no more. **The inclusive form stops short of the type's
maximum**: `0..=i32.MAX` would wrap at the increment, and a checked step would
cost a branch in every counted loop. Worth a decision when `for` is revisited.

## Crate state

A note on the specification: **Appendix A's `match` example is not what the
parser accepts.** It writes `Circle(r) => return PI * r * r`, but `return` is a
statement and `=>` takes an expression; the statement form is `Circle(r):` and
an indented block. Worth an erratum.

| Crate | State |
|---|---|
| `ember_span` | `FileId`, `Span`, `SourceMap`, `Symbol` interner |
| `ember_diag` | model, renderer matching XIX §6, JSON, 90-code registry |
| `ember_lexer` | all of Part II |
| `ember_ast` | full Part XVIII §2 tree, structural dump (prints doc comments) |
| `ember_parser` | **the whole v1 grammar** |
| `ember_types` | interner, C layout, `Copy`/`needs_drop`/`is_view`, `OverflowPolicy` |
| `ember_hir` | typed, desugared tree; scalar and struct cases populated |
| `ember_typeck` | bidirectional checking, literal defaulting, coercion sites, patterns, `match` exhaustiveness |
| `ember_mir` | CFG with spans, lowering, pruning, verifier, checked arithmetic |
| `ember_analysis` | definite initialisation |
| `ember_codegen_c` | MIR to C11, `#line`, shortest round-trip floats |
| `ember_build` | MSVC via captured `vcvars64`, clang, gcc |
| `ember_fmt` | `[FMT-1]`'s canonical printer, comments preserved |
| `ember_driver` | `build`, `run`, `check`, `fmt`, `explain`, `--emit`, `--json`; module loading |
| `runtime/ember_rt` | C11: alloc, panics, checked arithmetic, printers, embedding API |

## What the language can do today

Structs with C layout, memberwise constructors, field access, functions with
the three parameter modes, locals with inference and definite-init checking,
`if`/`elif`/`else`, `while`, `break`/`continue`, scalar arithmetic with
`[TYP-4]`'s no-implicit-conversion rule and `[TYP-8]`/`[TYP-10]`'s checks,
`[TYP-5]` widening at coercion sites, untyped-literal defaulting, short-circuit
`and`/`or`, casts, and `println` for every scalar and `str`.

## Load-bearing invariants

**`Span` offsets are always absolute byte offsets into the normalised file.**
`ember_span::normalise` strips a BOM and folds CRLF to LF before anything else
sees the text. `lex_range` (f-string interpolations) truncates the source at
`end` rather than slicing from `start`, precisely so spans stay absolute.

**The MIR builder tracks `current_span`**, set by `Builder::at` at the top of
`lower_into`, `lower_rvalue` and `lower_operand`. `push` and `terminate` both
read it. If a new lowering entry point forgets to call `at`, its statements
inherit the previous statement's span and diagnostics silently point at the
wrong line — which is exactly the bug that made `E3050` blame declarations.

**`referenced_blocks` in the C backend must agree exactly with
`emit_terminator`'s fallthrough rule.** A jump to the next block in order emits
no `goto`, so it must not mark that block referenced. If they drift, the C
either has an unreferenced label (a warning, breaking `[CG-C-1]`) or a `goto`
to a label that was never emitted (a hard error).

**MIR lowering creates a fresh block after every `return`, `break` and
`continue`.** Most are unreachable and `prune_unreachable` removes them before
codegen. Without it the C is full of dead labels.

**The parser's cascade limit is per *region*, not per file** (`[AST-2]`). If a
recovery path fails to advance the cursor, the limit silently swallows every
later diagnostic — so every recovery loop checks
`if self.pos == before { self.bump() }`.

**The `Symbol` interner leaks, on purpose.** `Box::leak` buys a `&'static str`
with no unsafe code and no lifetime plumbing. A compiler process interns a
bounded set of names and exits.

**`workspace.lints.rust` denies `unsafe_code`.**

**`ember check` must run the MIR analyses.** Part XIX §1 defines it as
"type-check + borrow-check without codegen". It returned right after type
checking at first, so definite-init never ran and every `compile-fail` test
passed vacuously.

## Traps paid for

**Writing Rust or C source containing `\n`, `\\`, `\"` or a line-continuation
backslash through a bash heredoc into a Python string destroys the escapes.**
They become real newlines and tabs, and the damage is invisible in a diff — it
looks like the file was always that way. This cost four separate rewrites
(`scan_escape` twice, the `ember_rt.h` macros, a `replace` call in the C
backend). **Write such files with the Write or Edit tool directly, never
through a shell.** If a script is genuinely needed, write the script to a file
with Write and then run it.

**A panicking program reported exit code 0.** Windows delivers `abort()` as
`0xC0000409`, which arrives as a large negative `i32`; `clamp(0, 255)` turned
it into a clean exit. Anything not representable as `u8` is now a plain
failure. Every `run-fail` test would otherwise have passed vacuously.

**An unrecognised `#$` key in a test is silently ignored.** A `run-fail` test
written with `#$ panic:` instead of `#$ panics:` passes without ever checking
the panic — the same vacuous pass as the exit-code trap above. And only
`milestones_pass` asserts that it found any files; `run-pass`, `run-fail` and
`compile-fail` all pass on an empty or missing directory. **After adding a
test, break it on purpose once and watch the suite go red.** A `compile-fail`
expectation is a substring of the rendered message, not the registry title.

**A statement that writes a place, but is not `Assign`, was invisible to the
move analysis.** `[TYP-8]`'s checked arithmetic lowers to
`StmtKind::CheckedBinaryOp`, which `drops.rs`'s `step` did not handle, so the
destination was never marked live and the next read of it was reported as a
use after move. Every integer `bigger = cap * 2` bound to a local hit it;
floats did not, because they take the plain `Assign` path, and nothing in the
suite wrote that shape until block G's tests did. **When a new MIR statement
writes a place, `drops.rs` has to learn it** — the analysis fails closed, and
failing closed here means rejecting a correct program.


**Part II §4's keyword table has 47 entries, not 46.** `ember_lexer::token`
asserts the count so the table and the enum cannot drift.

**`Range ..` is a prefix of `Range ..=`.** Substring counts over the AST dump
over-count; match to end of line.

**`cl.exe` cannot find its own headers without `INCLUDE` and `LIB`, and no flag
substitutes.** `ember_build` runs `vcvars64.bat` through `cmd /c … && set` and
captures the environment. A capture missing `INCLUDE` means the batch file did
not really run.

**`clang.exe` is not on `PATH`** even with LLVM installed;
`Toolchain::find_on_path` falls back to `C:\Program Files\LLVM\bin`.

### Block D: methods, interfaces, `extend`, operators

- **`mut` parameters did not work at all before this block.** `fn bump(mut n:
  i32)` was compiled by value, so `bump(x)` left `x` alone — parsed, stored in
  the signature, and ignored by every later stage. A `mut` parameter is now a
  `ref mut T` inside the function (which is what Part V's mode table says it
  is), every mention of its name reads through it, and the call site passes the
  address. `mut self` is the same mechanism, so it had to be fixed before any
  method could mutate its receiver.
- **A method is a function whose first parameter is the receiver.** There is no
  vtable and no dynamic dispatch: `recv.m(args)` resolves at compile time to
  one symbol, `em_<Type>_<method>`. `dyn` is what needs vtables, and it is not
  in this block.
- **`[TYP-24]` is enforced through one table**, `methods: (Ty, Symbol) ->
  entry`, which records whether an entry came from an interface. An inherent
  method registered later replaces an interface one; two interfaces offering a
  name is `E2070` at the call site, not at the declaration, because it is only
  ambiguous when someone calls it.
- **Required-method checks run after all collection**, not while walking each
  type: a type may declare `implements I` in its header and define the methods
  in a later `extend` block. Checking in place reported methods as missing that
  were defined ten lines further down.
- **An interface default body becomes one real method per implementing type**,
  registered during collection so that a call in any function body resolves,
  and checked afterwards with `self` bound to the concrete type.
- `[TYP-21]` — an operator whose left operand has the matching method calls it:
  `a + b` becomes `a.add(b)`. The check is on the method's presence, not on a
  declared `Add` interface, so `extend Vec2 implements Add:` and a plain
  `extend Vec2:` both work.
- An unused parameter now gets the same `(void)` discard an unused local does.
  `fn sides(self) -> i32: return 4` ignores its receiver, which is ordinary
  Ember and `-Wunused-parameter` under `[CG-C-1]`.

**Not done:** `dyn I` and vtables (`[TYP-22]`); generics, so a generic
interface such as `Add[Rhs]` is only usable at one concrete type; associated
types and consts (`[IFC-4]`); `[TYP-20]`'s orphan rule and `[IFC-2]`'s
cross-package restriction, which need modules — block F; `pub`/`pub(read)`
visibility, which is unenforceable while everything is one module. `a += b`
still expands to `a = a + b` rather than looking for `add_assign`.

A note on the specification, now ERR-008: **Appendix A's `match` example did
not parse.** It wrote `Circle(r) => return PI * r * r`, but Part III's grammar
makes `return` a statement while `[GRM-10]`'s `=>` takes an expression — and
Part IV §2 gives `return` the type `!`, siding with the appendix. The owner
ruled the grammar wins (2026-09-08); the appendix is corrected and the entry
records what would change if that is ever revisited.

### Block E: `Option`, `Result`, `?`, `Array`, `String`, f-strings

- **`Option[T]` and `Result[T, E]` are synthesised payload enums**, one per set
  of type arguments, named `Option_i32` and so on. They reuse all of block B —
  the `{tag, union}` layout, `match`, and exhaustiveness — so nothing new was
  needed to pattern-match them. `Some`/`None`/`Ok`/`Err` take the expected type
  when there is one; a bare `None` with no expectation is `E2060`, because
  nothing says which `Option` it is.
- **`?` becomes a two-arm `match`**, one arm yielding the payload and one
  returning the failure — which is why HIR lets a `match`'s arms mix a value
  and a block even though `[GRM-10]` forbids that in source.
- `[ERR-2]`'s `F.from(e)` conversion is **not** applied: the failure types must
  match exactly, because `From` needs generics. The diagnostic says so.
- **`Array[T]` and `String` are one runtime buffer** — pointer, length,
  capacity — with the element size passed at each call. That is what lets a
  single `ember_vec_push` serve every element type without generics, which is
  exactly what Part XX.1 asks for ("a compiler-known type temporarily"). `String`
  is `Vec { elem: u8 }`, so it prints as `String` and shares every growth path.
- **`push` needed a spill.** `ember_vec_push` copies through a pointer, and
  `&10` is not C, so the pushed value is lowered into a local first.
- Indexing an `Array[T]` is bounds-checked against the runtime length, using
  the same `AssertKind::Bounds` a fixed array uses — its `len` operand was
  already an operand rather than a constant, so nothing had to change.
- **f-strings build a `String`**, appending each piece through
  `ember_fmt_*`, chosen from the value's type exactly as `println`'s printer
  is. A format spec (`{x:.2}`) is reported as unsupported rather than ignored.
- `for i in 0..xs.len()` now works: either end of a range may be an untyped
  literal, and the other end says what it should be.

**Not done:** small-string optimisation (`String` is always heap); `Array`'s
`pop`, `insert`, `remove`, iteration by `for x in xs` (that needs `Iterator`);
freeing — **nothing calls `ember_vec_free`, so every `Array` and `String`
leaks.** Drop elaboration is Phase 2's, and this is the first type that needs
it; `ember_vec_free` exists and is ready for it.

### Block F, half: `const` and `static`

`const NAME: T = literal` and `static NAME: T = literal` both become a value
substituted where the name is used. `[STA-2]` says there are no runtime
initialisers, so a non-literal is `E2130` — which is what Part XX.1's
"comptime-initialised only via literal for now" asks for. A `const` may be an
array length, which closes the gap `E2131` left in block B.

`static mut` is rejected: `[STA-1]` requires `unsafe` for any access to one.
With no mutation and no runtime initialiser, a `static` and a `const` behave
identically here; the stable address `static` promises is not observable until
references to globals exist.

### Block F, the rest: modules across files

- **Every declared name is stored qualified** — `math.ops.Point` — and each
  module carries a map from what it may write to what that means. Two modules
  may each declare a `helper`; the mangled C symbols carry the module, so
  nothing collides.
- **Names are declared for every module before any import is bound**, because
  `[MOD-4]` allows cycles inside a package: an import may name an item in a
  module that has not been walked yet.
- `[MOD-1]` maps `a.b.c` to `a/b/c.em`, or to `a/b/c/mod.em` when the directory
  has submodules. The root is the directory of the file named on the command
  line, until `ember.toml` is read.
- `[MOD-2]` is enforced: importing a private item is `E1020`.
- **`import a.b.ops` then `ops.add(x)` parses as a method call**, because the
  parser cannot know `ops` is a module — the same shape `Shape.Circle(1.0)`
  has. Both are settled in the checker.
- **A `DefId` is no longer a position in `Program::functions`.** Functions are
  gathered module by module, then methods, then interface defaults, so the
  lookup searches by identity. It is linear; if it ever shows up in a profile,
  a map is the fix.
- Functions are **no longer emitted `static`**. Modules share one translation
  unit, so a private helper another module never calls was an unused `static`
  function — a warning, and `[CG-C-1]` forbids those.

### Block G: the formatter

`ember fmt [--check|--write]`, in `compiler/ember_fmt`.

- **Comments were not the obstacle they looked like.** `[II.7]` keeps them out
  of the token stream, but Phase 0's lexer already records each one's span and
  whether it had a line to itself — written for exactly this. The printer
  flushes them in span order as it passes, so a comment above an item stays
  with that item.
- **The blank line comes before the comment**, not after: separation first,
  then whatever introduces the next item. Getting that backwards detaches every
  comment from what it describes, which is what the first version did.
- `[FMT-1]`'s two guarantees — `fmt(fmt(x)) == fmt(x)` and `parse(fmt(x)) ≡
  parse(x)` — are a test over the whole corpus, not an assertion in a comment.
- **Anything the printer has no rule for is copied from the source.** A lambda,
  an f-string, a `match` expression: reproducing one from the tree risks
  changing what it means, and copying keeps parse-equivalence true for the
  whole language rather than only the part with rules. **That catch-all is not
  as safe as it reads**, and it cost two defects later in Phase 2: it trims
  each copied line, so a statement holding a block — `unsafe:` — came out with
  its body at the parent's indentation. A new statement or item kind needs its
  own arm, not the catch-all.
- **The printer knew nothing about type parameters** until Phase 2 block G's
  tests were written: `struct Pair[A, B]` printed as `struct Pair`. Generics
  arrived in Phase 2, after this block, and nothing re-checked the formatter
  against them, so `[FMT-1]`'s round trip was false for every generic
  declaration in the corpus. `generics()` now prints them for `fn`, `struct`,
  `enum`, `interface`, `extend` and `type`.

**Not done:** width-based breaking inside expressions. A long call or condition
stays on one line; only a long parameter list breaks. `[FMT-1]` asks for
conditions broken after `and`/`or` and trailing commas in multi-line element
lists, which needs a proper Wadler-style layout algebra rather than the
string-building printer this is.

## Phase 2 — where it stands

Part XX's Phase 2 is ownership: generics, drops, the borrow checker, closures
and arenas. Split the same way Phase 1 was:

| | Block | State |
|---|---|---|
| **A** | moves, `Copy`, drop elaboration, drop flags, `E3040` | **done** |
| **B** | generics with bounds, monomorphisation, associated types, generic structs | **done** |
| **C** | `Iterator` and `for` over anything | **done; the adaptor set is not written** |
| D | `Array`, `Span`, `MutSpan`, `Box`, `Map` written in Ember | **unblocked; not written** |
| E | the NLL borrow checker (§4.7), two-phase borrows, `@view` | **started** — see below |
| F | closures (`[CLO-*]`), `Callable`, `fn(A)->R` parameters | not started |
| **G**, core | `unsafe:` blocks, `*T`/`*mut T`, `alloc`/`free`/`read`/`write`/`size_of` | **done** |
| G, the rest | arenas (`[ARN-*]`), `MaybeUninit`, `transmute`, pointer arithmetic, inline asm | not started |
| H | the §XIX.6.1 shape catalogue, the classifier, `ember explain --borrow` | not started |
| I | `Cell`, `RefCell`, `Ref`/`RefMut` (`[CELL-*]`), `std.cell` | not started — **added by the 2026-09-08 amendment** |

Blocks D and G were planned the other way round. G's core was pulled forward
because D could not be written without it, and the rest of G — arenas,
`MaybeUninit`, `transmute` — turned out not to be needed for D at all.
Block I did not exist until the amendment.

### Block E: the borrow checker

**References had to become expressible first.** `ref place` / `ref mut place`
parsed since Phase 1 and the type checker reported them as unsupported, so the
language could not state a borrow at all and there was nothing to check.
Everything below the type checker was already there — HIR's `ExprKind::Ref`,
MIR's `Rvalue::Ref`, `T*` in the C — so it was one arm plus `[TYP-14]` for
shared references.

**What `compiler/ember_analysis/src/borrows.rs` does now**, against §4.7's
seven steps: exact backward liveness over the CFG (step 3); loans collected
from every `Rvalue::Ref` (step 2, partially); loan scope as "the borrower local
is live" (step 4); `[BRW-1]` over overlapping places (step 5); and `[DIA-3]`'s
two labels plus the later-use line (step 7).

**The approximation is gone: `compiler/ember_analysis/src/regions.rs` is real
region variables and a constraint graph** (`LT-REG-1`, done 2026-09-09). A
region is the set of points where a borrow must be valid, and for a borrow that
stays in the local it was created in that is exactly where the local is live —
which is why the approximation held for as long as it did. It stopped being
true the moment the reference moved, and **three programs that write through a
dangling reference compiled in silence**: a copy of the reference, a reborrow,
and a call handing the reference back.

One region variable per view-typed local (§4.7 step 1) and one per borrow
expression; one edge per assignment, in the direction the data travels. The two
things the checker wants are the two directions of that one graph — **points**
travel backwards along it (`points(from) ⊇ points(to)`, which answers "is this
loan still live here") and **provenance** forwards (which answers "which
parameter did this reference come from", for `[LT-1]` and `[LT-1a]`). Calls
join it through the callee's elision: a function returning a view ties its
result to every view-typed argument, so the caller holds the borrow for as long
as it holds the result.

Constraints are **not location-sensitive**, which §4.7 settles deliberately
("Polonius-style location-sensitive reasoning is not required for v1"). The
imprecision that costs needs a reference local to be *re-seated*, and Ember has
no syntax for it: `r = ref mut m` writes *through* `r` (`[TYP-14]`). Every
reference local is assigned exactly once.

**And `[DIA-3]`'s "later used here" label exists now, because regions are what
can produce it.** The label was missing entirely and the help line named the
local the borrow was first written into — which, once a loan can travel, is
routinely the wrong one. The region's *holders* are the locals it reaches, and
the one to name is the holder whose next read comes after the conflict, because
that read is the reason the borrow has not ended.

**`E3021`–`E3027` had to be allocated.** `[DIA-7a]` keys every `E3xxx` code to
a diagnostic shape and forbids emitting one that is absent from its table — and
seven shapes, including `[BRW-1]` itself, have no code anywhere in v0.5. See
ERR-021.

**The checker's first real catch was a test this session had just written.**
`tests/run-pass/references.em` ended with a reborrow, which held the first
borrow live across every access above it. The program was wrong, not the
checker.

**`[BRW-3]` two-phase borrows are in**, which is what makes
`xs.push(xs.len())` legal — the example the rule is written around. A mutable
borrow taken for a call is *reserved* where it is created and *activated* at
the call, and behaves as shared in between. The reservation window is a set of
points rather than a start and an end, because evaluating an argument that is
itself a call ends the block: the reservation and its activation are never
adjacent, and the first attempt at this looked only within one block and did
nothing at all.

**`[BRW-4]` disjoint fields work** and fall out of prefix overlap: two distinct
fields never conflict, the same field does, and reading the whole struct while
a field is borrowed does. Places are named as the programmer wrote them —
`p.x`, not MIR's `p.0` — by walking the type alongside the projections, and the
help line never names a compiler temporary.

**`E3060` is in and milestone M2 exists.** Returning a reference to a local
compiled in silence before this — a dangling pointer in safe code. A borrow of
a *parameter* may be returned, because the caller owns what it points at; a
borrow of a local may not. Two things had to change to make the check fire at
all: the return slot is live at every `Return` (without that the borrow looks
dead exactly where it matters), and `[TYP-14]`'s auto-deref now consults the
expected type, since a reference that always derefs cannot be returned or
passed on.

M2 is written over `ref i32` rather than the specification's `Array[i32]` and
`xs[0]`, which needs `Index` to return a `ref` — block D. The rule under test
is the same, and the file says so.

**`[BRW-7]` is in.** A borrow of a moved place is `E3050`, shape O6, separate
from `[OWN-3]`'s use-after-move: taking an address does not consume, so a
borrow is not in the move analysis's reads, but a reference into memory whose
owner gave it away dangles just the same.

**`[DIA-7]`'s classifier is in, and designed into the pass rather than bolted
on** — which is what the rule requires, because the classifier needs the
borrow checker's own loan and region tables and cannot be reconstructed from
rendered text afterwards. `compiler/ember_diag/src/shapes.rs` holds
§XIX.6.1's twenty-six shapes and `[DIA-7a]`'s code-to-shape table; the borrow
checker emits through `emit_classified`, which records anything it cannot
place; the driver writes `target/<profile>/unclassified-borrow-errors.log` and
CI fails if the file exists. **Zero unclassified borrow errors across the
corpus today**, which is one of Phase 2's exit criteria met early.

A test asserts every ownership code has a shape, and it found one that does
not: `E3100`, `[UNS-1]`'s "requires an `unsafe` block", which sits in the
ownership range and is not an ownership error. `[DIA-7]` and `[DIA-7a]`
disagree about the classifier's scope; ERR-022 records that `[DIA-7]`'s is the
real one, and the exception list is one entry long.

**`[TYP-14]`'s `@view` requirement is in.** A struct carrying a borrow *is* a
view type whatever it says, and omitting the attribute is `E2030`. The
attribute is documentation, and the reason it is mandatory is that without it a
reader has to check every field's type to learn the struct may not be stored.
**`[TYP-15]`'s storage restriction is in for the cases that need no regions**:
a `static` and a container element have no bounding region at all, so no
analysis can make them work, and both are `E3063` with shape B12's mandated
help — an owned copy with its per-element cost named, or an index or handle
with the container it indexes named. A view escaping through a *return* is
`[LT-1]`'s job and needs the region graph.

**`[LT-1a]`'s `@borrows` is validated.** It was parsed, listed in Part III §7's
attribute table, and read by nobody, so a function could name a parameter that
does not exist, or one that is not view-typed, or carry the attribute with no
view-typed return, and compile. All three are `E2031` now, with the valid
parameters listed when a name matches none. It is a public contract — `[VER-2]`
makes widening it a breaking change — so a typo in it is worth catching loudly.

**And the body half is in (2026-09-09), which is what regions bought.** The
attribute was checked as a *signature* and the body was free to contradict it:
`@borrows(a)` on a function that returned `b` compiled, and the caller then went
on using `b` while holding a reference into it. `E3062` (shape B6) is emitted
when the returned view's provenance includes a parameter elision did not tie it
to — `@borrows` naming a different one, or `[LT-1]` rule 1's borrowing receiver
where the result points into an argument instead. The attribute's parameter
*positions* now ride the signature into HIR and MIR, so the borrow checker
reads a contract rather than re-parsing an attribute.

**A hole in `E3060` fell out of the same work: a borrow of a parameter passed
by value.** The escape check exempted every parameter, on the reasoning that
the caller owns what a parameter points at. That is true of a view-typed one
and false of a copy: `fn peek(self) -> ref i32: return ref self.n` returns a
pointer into a frame that is about to end, and it compiled. The exemption is
now for view-typed parameters only.

### What block E still needs, in order

1. **`E3064`, two independent regions in one view struct.** It is registered
   and keyed and nothing emits it, and **no case that reaches it has been
   found**: `[LT-2]` says a struct built from several references takes the
   *intersection* of their regions, so the compiler narrows rather than
   refusing. The rest of `[LT-2]` is done and tested (2026-09-09) — a view
   struct carries its region through a return, through `@borrows`, and through
   a call, because the aggregate that builds it is an edge in the constraint
   graph like any other. Before writing a diagnostic for `E3064`, find the
   program that needs it.
2. **`[BRW-4]` through method calls.** Disjoint fields work by prefix overlap;
   shape B8 — "a method takes all of `self`" — needs the call to know which
   fields it touches.
3. **`[LT-1b]`'s `L3014`**, which needs `[MAN-3]`'s `[lints]` configuration
   first (`LNT-CFG-1`): an opt-in lint has nowhere to be opted into.
4. **The remaining shapes.** `E3023`–`E3027` are registered and keyed and
   nothing emits them yet; each waits on the construct it describes — aliased
   value mutation, self-referential structs, closures, `mut` arguments.
5. **`[LT-7]`'s callback regions** wait on closures (block F).

**Where the borrow checker lives:** `compiler/ember_analysis/src/borrows.rs`,
run from the driver after drop elaboration so the drops it sees are the ones
that will exist. The module header states the approximation against §4.7's
seven steps; read it before extending it.
with a constraint graph, which is what `[LT-1]`, `[LT-2]` and `[LT-7]` need and
what the liveness approximation cannot do; and the shape classifier
(`[DIA-7]`, block H), which has to be designed into this pass rather than
bolted on.

### Block A: moves and drops

**The `Array`/`String` leak is closed, and measured rather than asserted:** the
runtime's own allocation counter reports 204 allocations and 204 frees over
`tests/run-pass/drops_and_moves.em`, with nothing live at exit.

- `StmtKind::Drop { place, flag }` is inserted by lowering at every way out of
  a scope — the end of the block, a `return`, and a `break` or `continue` that
  leaves it — in reverse declaration order (`[DRP-2]`), after that scope's
  `defer` blocks (`[CTL-8]`).
- `compiler/ember_analysis/src/drops.rs` then runs the `Live | Moved | Maybe`
  dataflow: a drop of a moved value becomes a `Nop`, and a drop of a `Maybe`
  value gets `[OWN-3]`'s **drop flag**, a hidden `bool` set where the value is
  stored and cleared where it is moved away. A conditional move is never an
  error.
- **A value can arrive from a call**, whose destination is written by a
  terminator rather than a statement — `xs = Array()` is one — so the flag is
  set at the top of the block control reaches next. Missing that leaked exactly
  the values that were never moved.
- **`[FN-1]`'s borrow is the default, and reading a place for a borrowed
  argument must not consume it.** `xs.len()` was moving `xs` away, because a
  non-`Copy` place always read as a move; the drop was then removed as
  "already moved" and the buffer leaked. `lower_operand_borrowed` is the fix.
  **`owned` now consumes**, as of 2026-09-08: the callee's parameter modes are
  read at the call site, and an `owned` argument lowers to a move rather than a
  borrow. Until then every argument was borrowed, so `owned` parsed, reached
  the signature, and silently did nothing — the trap this file warns about,
  found by a `[BRW-7]` test that would not fire.
- The drop glue is written straight into the C rather than as a synthesised
  function: a struct drops its fields in reverse, an enum switches on its tag
  and drops the active variant's payload, an array drops its elements.
- **The overwrite was the third occasion and it was never built** (D-035,
  found 2026-09-09 while starting block I, fixed the same day). `[OWN-2]` names
  three moments a value dies: its scope ends, it is **overwritten**, or it is a
  temporary at the end of its statement. Lowering had `emit_drops_from` for the
  first and `emit_statement_temps` for the third — nothing at all for the
  second. `r = R(1)` followed by `r = R(2)` ran `R(1)`'s destructor zero times,
  and assigning a fresh `Array[i32]` over a live one emitted **one**
  `ember_vec_free` for two buffers.

  `lower_assign` is the fix, and it is deliberately three lines of policy
  rather than a new analysis: evaluate the new value into a temporary, push
  `Drop`, store. Everything about *which* of those drops actually runs is left
  to the elaboration above — a local is `Moved` immediately after
  `StorageLive`, so the drop inserted at its initialising assignment is deleted
  and a first initialisation stays free; a conditionally-moved local gets the
  same drop flag as anywhere else. ADR-020 records why the temporary is
  unavoidable (a value arriving from a call is written by a terminator, so
  there is no point between "evaluated" and "stored" to push a drop into) and
  why lowering must not try to decide liveness itself.

  **What let it survive this long is worth more than the fix.**
  `tests/conformance/OWN-5/` existed and passed. Its one case tested the
  parenthetical — that the new value is evaluated before the store — and was
  written as `x = grow(x)`, which **moves** `x` into the call, so nothing is
  ever live at the store and the main clause could not fire. A rule stated in a
  main clause and a parenthetical needs a case for each; a directory with the
  rule's name in it is not coverage of the rule.

### Blocks B and C: generics, bounds, associated types, `for`

- **A generic body is checked once, with its parameters opaque** (`TyKind::Param`),
  which is what `[TYP-17]` demands: only what the bounds provide is available,
  and there is no duck typing. `a.area()` inside `fn f[T]` is an error even
  when every caller happens to pass something with an `area` — and the
  diagnostic names the bound that would fix it (`help: add the bound: T: Shape`).
- **Nothing is emitted for the generic body itself.** Each call instantiates
  it: the arguments are unified against the declared parameter types
  (`[TYP-18]`), the bounds are checked, and the body is re-checked with the
  parameters bound to the concrete types, producing its own `DefId` and symbol
  (`[MONO-1]`). Instantiations are cached by `(DefId, args)` and drained from a
  queue, because instantiating one can reach another.
- Errors from an instantiation go to a **discarded sink** — the generic body
  was already checked once, and reporting the same mistake per instantiation
  would bury it.
- `f[i32](x)` parses as a call on an index, which is where explicit
  instantiation is picked up. `[TYP-18]` requires it when no argument mentions
  the parameter, and that is `E2060` with the shape to write.
- **`[IFC-4]` associated types** are `TyKind::Assoc { name }`, in scope while an
  interface declaration is read and resolved once the receiver is concrete
  through `assoc_values[(type, name)]`. That is what lets `Iterator` say
  `fn next(mut self) -> Option[Item]` and an implementation say `type Item = i32`.
- **`[CTL-1]`'s `for`** now has three shapes: a range is still a counted loop
  (`[CTL-3]`); an `Array[T]` is a counted loop over its indices, so iterating a
  collection also costs no iterator object; anything else is driven through
  `next()`. Exhaustion ends the loop through its condition rather than a
  `break`, so `[CTL-4]`'s `else` still tells the two apart.

**Not done in B and C:** const generics (`E1010` with a clear message); generic
structs and enums — only functions are parameterised, so `Pair[A, B]` is not
yet writable; generic interfaces (`Add[Rhs]`); the `Iterator` adaptor set
(`map`, `filter`, …), which needs closures — block F.

### Block I: `Cell[T]` — done 2026-09-09. `RefCell` and `Arena` are not.

`Cell[T]` is **a transparent one-field struct**, interned once per `T` by
`cell_of`, with `cells: HashMap<StructId, Ty>` on the `Checker` telling method
dispatch that this struct's `set` is a builtin. ADR-019 is why it is
compiler-known and not written in Ember on `UnsafeCell`: the document names
`UnsafeCell` once and defines it nowhere, and inventing the one construct that
suspends `[BRW-1]` is the owner's call, not the compiler's (ERR-043).

**A struct rather than a `TyKind` is what makes `[CELL-4]` free.** `is_copy` on
a struct is already `derives_copy && !has_drop && every field Copy`, and
`needs_drop` is already `has_drop || any field needs it` — so with
`derives_copy` set and `has_drop` clear, both questions reduce to the same
question about `T`. "`Copy` when `T: Copy` … move-only for a non-`Copy` `T`,
and `Drop` iff `T` is" has **no code behind it at all**, and three conformance
cases confirm each half. It is also what `[CELL-2]`'s "no overhead relative to
a plain field" means in practice: the emitted C is a struct field access.

**What is built, and what is not.** `Cell()`, `set`, `replace`, `into_inner`,
`get` (`T: Copy`) and `update`'s `T: Copy` arm. `take` needs a `Default`
interface, and there is none anywhere in the compiler — `CELL-DEF-1`. `[CELL-3]`'s
`!Sync` needs `Send`/`Sync`/threads — `CELL-SYNC-1`. Both report or are filed by
name; neither rule is softened, and `reject_take_needs_default.em` pins the
message so `take` never reads as a typo.

**`[CELL-1]`'s ordering rule is the whole point of the type, and it is the
opposite of `[OWN-5]`.** "`set` and `replace` MUST store the new value before
dropping the old one", because a drop runs user code that can re-enter the same
cell and read it while it is torn. An ordinary assignment does the reverse
(D-035, fixed the same day). That is why `set` is lowered itself rather than as
an assignment: `lower_cell_store` moves the old value to a slot, stores, and
only then drops. ADR-020's last paragraph is the note not to generalise either
rule to the other.

**`into_inner` needed one trick worth remembering.** It takes `owned self`, so
the payload leaves and the cell must not be dropped behind it. A move out of a
*field* is a partial move and `[OWN-3]`'s analysis does not track one, so the
receiver would still look live and be dropped at scope end — freeing the value
the caller now holds. Moving the **whole cell** into a slot first is what marks
the receiver moved, and the slot is `temp_unowned` so nothing drops it either.
Exactly one `ember_vec_free` in the emitted program, checked.

**`[CELL-2]` rests on ordinary privacy, not on a clever name.** The payload is a
private field whose `declaring_module` is `usize::MAX`, which no real module can
be, so `check_field_visible` refuses it everywhere — read, write and `ref` all
`E1020`. An *unspellable* field name (`$value`) was tried first and is wrong:
the backend writes field names into the C verbatim, `$` in an identifier is a
compiler extension, and `[CG-C-1]` says the emitted C must not depend on one. It
compiled clean under `-Wall -Wextra`; only `-pedantic` said so. **Read the
emitted C for anything that puts a new name into it.**

**The harness gained `assert-c-order`**, and it was needed rather than
convenient. `assert-c`'s needle is one line of literal text, so it can say what
the backend emitted but not in what order — and both `[CELL-1]` and `[OWN-5]`
are entirely about order, invisible in output, because seeing the difference
needs a destructor that re-enters the value being replaced and no safe Ember
program can build that back-pointer (a `static` holding a cell is refused:
`[STA-1]` requires a comptime-evaluable initialiser and the compiler narrows
that to a literal, so `static COUNTER: Cell[i32] = Cell(0)` is `E2130`).
`#$ assert-c-order: "a" then "b"` asserts
both are present and in that order. Verified by flipping the store and the drop
in `lower_cell_store` and watching the case go red with the right message.

**Anchor an order assertion on text that does not move.** The first needles were
`_1.value = _4` and `em_Bag_drop(&_5)`, which depend on MIR local numbering.
`.value = ` and `em_Bag_drop(&` do not — and the `&` is load-bearing: without
it, `em_Bag_drop(` matches the *prototype* at the top of the file, which
precedes everything and would make the assertion vacuously true.

**A cosmetic thing left alone.** A compiler-known generic shows in diagnostics
under its interned name — `Cell_i32`, and `Option_i32` has always done the same.
Fixing it means changing how `Option` and `Result` display and re-baselining the
cases that quote them, so it is not folded into this.

### Block G's core, and block D unblocked

`Array`, `Span`, `MutSpan`, `Box` and `Map` could not be written in Ember
without raw pointers and `unsafe`. Those now exist, so D is writable. What D
was worth in the meantime was already delivered: `for x in xs` over an
`Array[T]` works, and it is a counted loop rather than an iterator object.

**`unsafe:` is a block, and `[UNS-1]` is enforced.** Using a raw pointer, or
any of the memory builtins, outside one is `E3100`, whose `help` names the
fix. The typechecker carries an `in_unsafe` flag; `[UNS-2]` holds — nothing
inside an `unsafe` block skips type checking, bounds checking or the move
analysis.

**The builtins are `alloc`, `free`, `read`, `write` and `size_of`**, each
generic in the pointee type. MIR lowering picks `arg_ty` per builtin rather
than from the first argument, which was a real bug: `alloc`'s first argument
is a count, so taking the type from it made every allocation `usize`-sized.

**Generic structs came with it.** Only generic *functions* existed before, and
D needs `Buffer[T]`-shaped types. `StructDef` gained an `origin` — the generic
it was instantiated from, and the arguments used — because without it
`Buffer[T]` in a method signature would not unify with the `Buffer[i32]` at
the call site. `unify` and `substitute_ty` both consult it.

**D is proved unblocked, not assumed.** A growable collection is written
entirely in Ember — allocate, grow by doubling, copy across, index, free — and
runs correctly, with the runtime's allocation counter reporting 3 allocations,
3 frees and nothing live at exit.

**Three defects, all found by writing the tests rather than by reading the
code.** None was visible while the demo programs ran:

1. **Methods on a generic struct were silently dropped.** Collection kept the
   fields and ignored every other member, so `Pair[A, B]` with a `fn left`
   produced an instantiation with no `left`, and the call site said
   `Pair_i32_f32 has no method named left` — an error at a distance from the
   cause. `GenericStruct` now stores each method resolved once with the
   parameters opaque, and `instantiate_struct` substitutes and registers them,
   queueing the bodies on `pending_methods` the way generic functions queue on
   `pending`. This mattered more than it looked: `Array[T].push` is a method,
   so block D was not really unblocked without it.
2. **The formatter dropped every type parameter list.** `struct Pair[A, B]`
   printed as `struct Pair`, and `fn make[T](...)` as `fn make(...)`. The
   formatter was written in Phase 1, before generics existed, and nothing had
   re-checked it since. `[FMT-1]`'s `parse(fmt(x)) ≡ parse(x)` was false for
   every generic declaration in the corpus.
3. **The formatter flattened `unsafe:` blocks.** `StmtKind::Unsafe` had no arm,
   so the catch-all copied the text through with each line trimmed, which put
   the body at the parent's indentation.

The third defect is the general shape of the second: the formatter's catch-all
looks safe but is not — it preserves text and destroys structure. Any new
statement or item kind needs an arm.

**Not done in G:** arenas (`[ARN-*]`), `MaybeUninit`, `transmute`, pointer
arithmetic (`offset`/`add`/`sub`), and inline assembly (`[UNS-6]`). None of
them is needed for D. `L3010 unsafe block larger than necessary` (`[UNS-3]`)
is a lint, and no lint pass exists yet.

**One limit worth knowing.** A generic struct's method body is checked only
when something instantiates it — there is no opaque `Buffer[T]` for `self` to
have, so unlike a generic function it is never checked once with the
parameters left abstract. A method nobody instantiates is never type-checked,
which is C++'s behaviour rather than Rust's. The first instantiation of each
method reports; later ones are quiet, so one mistake is one error.

**Still open from blocks A to D:** partial moves (moving one field out of a struct)
are not tracked — only whole locals; `mem.take`/`replace`/`swap`/`forget`
(`[OWN-6]`); `[OWN-4]`'s `E3041` for a loop that moves a value declared
outside it; and user-defined `drop` methods are recognised (`has_drop`) but
their bodies are not yet called by the glue.

**What Phase 1 left open, and where each belongs:**

- **`dyn` and vtables** (`[TYP-22]`), which block D deliberately left out.
- **`[T; N]` coercing to `Span[T]`**, and **`for` over anything but a range** —
  both need `Iterator`, which block D's interface machinery can express but
  nothing has written.
- `[ERR-2]`'s `F.from(e)` conversion for `?`, which needs `From`, which needs
  generics.
- `@overflow(saturate)` (block A), `[TYP-13]`'s niche optimisation, slice and
  range patterns, `from_repr`, small-string optimisation, `a += b` looking for
  `add_assign`, and width-based breaking in the formatter. Each is written up
  in its block's section above.
- **`pub` visibility is enforced across modules but not within one.** Nothing
  checks `pub(package)` yet, because there is one package.

## The 2026-09-08 spec amendment

The owner replaced Parts IX, XI, XV, XIX, XX and XXII wholesale with the v0.2
memory-safety update. Two things were added, and both are Phase 2 work:

**1. `Cell` and `RefCell` (`[CELL-1..11]`, new Part IX §7).** The escape hatch
for value types, matching what classes already give the object world. `Cell[T]`
requires `T: Copy`, hands out no reference, and therefore needs no runtime
check at all — `get` is a load, `set` is a store. `RefCell[T]` keeps a
one-word borrow counter and panics on conflict, naming the line of the
conflicting borrow. Both are `!Sync`. Neither is in the prelude (`[CELL-11]`),
so reaching for one is a visible import — that is open question 9, awaiting the
owner. They live in `std.cell`, a new module row in Part XV.

**2. A mandatory borrow-diagnostic catalogue (`[DIA-7..10]`, new §XIX.6.1).**
Sixteen error shapes — O1–O4, B1–B10, X1, R1 — each with a *required* `help`
line naming a concrete API. `[DIA-7]` makes classification mandatory: an
ownership or borrow error the classifier cannot place is written to
`target/<profile>/unclassified-borrow-errors.log`, and CI fails if the
conformance suite produces even one. `[DIA-9]` forbids suggesting `unsafe`,
`Cell`, `RefCell`, `Shared` or `clone()` first when a structural fix exists;
`clone()` leads only for O1. `[DIA-8]` adds `ember explain --borrow
<file>:<line>`, which must be generated from the borrow checker's own loan and
region tables, not reconstructed after the fact.

**What that does to the plan.** Phase 2's exit criteria in §XX.2 now also
require all `[CELL-*]` and `[DIA-7..10]` tests, a `tests/ui/borrow/<shape>/`
snapshot for each of the sixteen shapes, and **zero unclassified borrow errors
across the whole corpus**. `[DIA-7]` is the constraint that matters: it has to
be designed into the borrow checker (block E) rather than bolted on in block H,
because the classifier needs the loan and region tables that E builds. Building
E without it means building E twice.

`L3011 RefCell guard held across a call` joined the lint list, and the error
registry's `E1000–E1499` row now names `E1050`/`E1051` — both already exist.

**Three things in the update were deliberately not taken**, all because they
predate owner decisions already recorded in `docs/spec-errata.md`: it reverts
test annotations from `#$` to `#!` in Part XIX §5 and in the §XX.3 milestones
(ERR-006), and it carries Appendix A's `match` example back to the `=>` form
that does not parse (ERR-008). It also drops `[TST-0]` and the `assert-c`
annotation, which came in with ERR-006. Diffing the owner's file against
`docs/spec/` will show exactly those differences and nothing else.

## Environment


- Rust 1.98.1 stable, MSVC toolchain. Installed 2026-09-07 via winget.
- LLVM 22.1.8 at `C:\Program Files\LLVM`; `libclang.dll` in its `bin`. Not
  needed until Phase 5, but installed so that phase does not stall.
- MSVC 14.44 under
  `C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools`.
- `cmake` and `ninja` exist only under the Visual Studio install, not on
  `PATH` — the same trap RageV's build has.
- `cargo` is on `PATH` only in shells started after the install; PowerShell
  calls in this project prepend the machine and user `Path` values.
