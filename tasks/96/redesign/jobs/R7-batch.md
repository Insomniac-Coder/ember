# Round 7: the first lane jobs (the owner, 2026-11-13: "use 2-3 helper agents at max"; the new way of working, "yea")

**How work is organised from now on:** `docs/AUTOPILOT.md` §3, "How the work is organised, to finish the language
faster". In short:
- a job finishes **whole rules**: every clause built, a test for each clause, credited in the same job;
- the Opus deputy works one milestone ahead (tests from the spec, every spec question in one cut) and reviews;
- **rolling slots**: each job is collected when it ends and its slot gets the next job at once;
- three **lanes**, each with a notes file in `tasks/96/redesign/jobs/lanes/`, and its own worktree and `target/`
  kept between jobs.

**Why rules:** of the target's rules, 594 are unfinished (116,328 of 134,174 words). 497 of them already have tests,
but they are rules 0.9.15 or 0.9.20 reworked, so a rule counts only once its new behaviour is checked clause by
clause and credited in `docs/phase-status.json`'s `built_for_0915` (`tools/phase_progress.py`, condition 3). The
list: `python tools/phase_progress.py` prints each phase's unfinished rules.

Sizes: **D2 (Opus deputy) 11, then M5 ahead; H3 (Opus, lane b) 29 rules; W3 (Sonnet, lane c) 30 rules.** The lead
takes lane a (M4's slice C on `lead-m4`) and collects, reviews, tests and merges each job himself.

## The rules for every lane job

The rules of `N-batch.md`, "Rules for every job", apply, with these:
- **Your lane's worktree is fixed**, and you work only in it (absolute paths in every command): lane b is
  `C:\Users\ism19\Code\ember\.claude\worktrees\lane-b` (branch `lane-b`), lane c is `…\lane-c` (branch `lane-c`).
  Its `target/` stays when you finish (the owner's yes, 2026-11-13). Never touch the main checkout
  `C:\Users\ism19\Code\ember` or another lane's worktree.
- **Start:** `git -C <lane> merge --ff-only main` (or `git reset --hard main` when the lead says so), and check that
  `git log -1` shows the base the lead gives you.
- **Read your lane's notes first** (`tasks/96/redesign/jobs/lanes/<lane>.md`); **update them last** (what you found
  where, traps, what is done), in your last commit.
- `CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4`.
- **A task is a rule.** For each rule of your list:
  1. read the rule in the target (`docs/spec-source/development-target.json` pins it) and split it into its
     clauses: each sentence that says what is accepted, refused (with its code and message), or what runs;
  2. write a test for each clause from the spec's words (`tests/conformance/<RULE>/…`, `#$ rules:` naming it),
     before changing the compiler;
  3. make the compiler pass them, and fix any older test of the rule that pins what the target changed (the tests
     follow the spec);
  4. add the rule to `built_for_0915` in the same commit, one commit per rule.
  A clause the compiler cannot build yet because it needs another milestone (say which) is reported, and the rule
  is not credited. Never credit a rule on a probe, a docstring or a partial test.
- **Spec questions:** report them with the rule, the program, what the compiler does and the readings, as code. Do
  not decide them; carry on with the next rule. (Sonnet never rules one; Opus helpers send theirs to the lead.)
- New code goes in your lane's own module files (the notes list them), not appended to
  `ember_typeck/src/lib.rs`.
- Commit with `--author="Gunslinger <ism1988@live.com>"`. Write no `Box` anywhere: one owner in a block of its own
  is `owns T`.
- **The quick check's compiler:** `tasks/impl-0.9.9/annotations.py` runs `target/debug/ember.exe` unless `EMBER`
  names another binary: set `EMBER` to the binary you just built.
- Before you finish: the quick check over your folders, `cargo test -p` of every crate you touched, `rule_index`,
  `error_pages` and `check_branding`; `python tools/phase_progress.py` shows your credits.

**Files nobody else touches:** lane a's (M4, `lead-m4`): the `Link` kinds, `Shared`, `SyncShared`, `Weak`,
`synth_link`, `read_link_through`, `open_link`, `SharedNew`/`WeakNew`/`WeakUpgrade`/`WeakEmpty`/`LinkMake`/
`LinkTarget`/`LinkOpen`, `uncounted_handles.rs`, `temporary_owners.rs`, class objects' copy and retain (A1-9),
`into_raw()` (A5-4), and A6-5, A6-9, A6-10, FFI-35a. The deputy's: `tasks/spec-0.9.9/`, `docs/spec-source/`,
`tasks/96/redesign/compiler-0.9.15/decisions.md`, `docs/AUDIT-0.9.20.md`, `docs/OWNER-QUEUE.md`.

## D2 (Opus deputy, a reader; no builds): Hardened_14, then M5 ahead

1. The 11 findings of `review.md`'s round 6, ruled, and the cut `0.9.20_Hardened_14` (done: N176 to N181, merged
   `c167f08d`).
2. Then, one milestone ahead (a pilot first: A2-2 and A2-3, up to 8 rules, tests in `tasks/96/redesign/jobs/ahead/M5/`): M5's rules (`MILESTONES.md`, M5: views inside a function, loops, spans, kept views,
   generators): their clauses, a test per clause written from the spec alone, and every spec question gathered
   into one cut. And reviews of finished lane jobs when the lead asks.

## H3 (Opus), lane b, job 1: the C interface's safety words and contracts (29 rules)

`gap/06-ffi-hot-reload.md` (A6-2, A6-3, A6-11, A6-12, A6-16) and `gap/05-threads-unsafe.md` (A5-2, A5-3, A5-5) give
the files and the old tests for most of these.

- **`unsafe import c`** (A6-2): `[BLD-FFI-1]`, `[BLD-FFI-1a]`, `[BLD-FFI-3]`, `[FFI-29]`, `[FFI-29b]`, `[FFI-51]`.
- **`@unsafe(export(...))`, `@unsafe(export_table(...))`** (A6-3): `[FFI-25]`, `[FFI-26]`, `[FFI-33]`, `[FFI-33a]`,
  `[FFI-33c]`.
- **`[FFI-33b]`** with the lead's option (c) of round 6: the checker wraps a `returns_owned` call in a builtin that
  carries the call's line, and a named local's end passes its name to the release; both backends, no ABI change.
- **C function pointers** (A6-11): `[FFI-56]`, `[FFI-8]`; **`char` and promoted variadic arguments** (A6-12):
  `[FFI-31b]`.
- **`no_callbacks`, recorded** (the walk analysis that uses it is A6-8's, later): `[FFI-11]`, `[FFI-11a]`,
  `[FFI-11b]`; E0104's help no longer lists `unsafe` among the contract words.
- **`ember tcb`** (A5-5): `[TCB-4]`.
- **A `unique_ptr`'s deleter as the one destructor** (N173): `[FFI-17a]`, `[FFI-36]`, `[FFI-36a]`; what `hide` does to
  the conflict is N180 (Hardened_14): a hidden declaration counts only where a wrapper calls it, every place is
  compared at once, and one `E5002` per type names every place.
- **`unsafe`'s rules**: `[UNS-2]`, `[UNS-3]`, `[UNS-4]`, `[UNS-5]`, `[UNS-10]`, `[UNS-10a]`, `[UNS-11]`. (`[UNS-1]`'s
  `into_raw()` is lane a's.)

## W3 (Sonnet), lane c, job 1: the grammar and the lexer (30 rules)

Parse-only tests are evidence for these two families (`phase_progress.py`'s first condition), and M1 built most of
the 0.9.15 grammar, so most rules need their clauses checked and tested more than new code.

- **`[GRM-*]`, all 27:** GRM-2, GRM-3, GRM-3a, GRM-4, GRM-5, GRM-8a, GRM-8b, GRM-13, GRM-15, GRM-17, GRM-19, GRM-27,
  GRM-28 (listed under `unfinished` in `phase-status.json`: find why), GRM-29, GRM-30, GRM-34, GRM-35, GRM-36, GRM-37,
  GRM-38, GRM-41, GRM-42, GRM-43, GRM-44, GRM-45, GRM-46, GRM-47.
- **`[LEX-*]`, the three largest:** LEX-15, LEX-23, LEX-26.

## Left for later lane jobs (each lands with its rule's job)

From round 6's reports, each now a part of a rule: `[DIA-2]`'s spelling done generally (lane c, Opus: commit
`29cc91df`'s whole-program scan is not the rule); `[MNG-1]`/`[MNG-6]`'s `StrView` under `std.core`; DIA-10 B5's and
B9's helps; `[TYP-26]`; `[TYP-15a]`; `[STD-26]`; `[FFI-10]`/`[FFI-21]` (needs A6-6, M8); E5060; E7008's other cases;
the `match owns jobs.pop():` help; `ArenaMap.in`; HR-16 and HR-20; D-861; `[DSP-3]`; CLI-7's credit.

## H4 (Opus), lane a, job 1: M5's A2-3, a name for a part is a view (`[BRW-12]` to `[BRW-15]`) (rolling slot, 2026-11-13)

Worktree `C:\Users\ism19\Code\ember\.claude\worktrees\lane-a` (branch `lane-a`, from `main`). It lands on `main` on
its own; M4's slice C (the lead, branch `m4-slice-c-probe`) builds on it. Read `M4-plan.md`, "Found by C2", and
`lanes/a.md` first: they hold the measurements.

1. **`[BRW-12]` clause 2:** a name bound to a part (a field, an element, a value read through a view) of a type that
   is not `Copy` is a read-only view: never a hand-over, never a silent copy; a part of a `Copy` type is copied; a
   name whose first line writes its type takes the part as that type says. The lead's experiment (both `let`
   paths in `ember_typeck`, a `part_as_view` wrapping the place in a `ref`) gave 3,344 "cannot write through a
   shared reference": the programs that write through such a name.
2. **`[BRW-12]` clause 6:** `x = ref mut <part>` names that very place and may change it (today `x = ref mut b.items;
   x.push(3)` is E3021); clause 7 (`ref mut` before a call that gives `MutSpan`s) works since W3's GRM-36.
3. **`[BRW-13]`:** `=` on a name that has a value: a view name is re-pointed (checked against its new target), a
   `ref mut` name writes its place.
4. **`[BRW-14]`:** parameters, loop, pattern and opened names keep their place: `=` on one is E3023 (B4's `new_n`
   is built, N174).
5. **`[BRW-15]`:** a plain name never changes what it names or leads to.
6. **The migration:** where a name bound to a part is written through, its line becomes `x = ref mut <part>`,
   which is what the line did before (it moved the part out, so a valid old program never used the part again):
   a checker migration (`Diagnostic::migrate_edits`, as `kept_by_copy` in `ember_analysis/src/drops.rs` does),
   applied with `ember __migrate --checker <file>` over `tests/`, `std/` and `docs/book/`. A name that is moved on
   (`take(owns x)`) is rewritten by hand, with `copy of` or by handing the part over where it is used.
7. **The tests:** the deputy's in `tasks/96/redesign/jobs/ahead/M5/BRW-12/` to `BRW-15/` (with `INDEX.md`'s rulings
   Q3 to Q5, Q12; its §2 lists the old tests the target contradicts). Move each into `tests/conformance/<RULE>/`
   when it passes; one that needs another milestone stays, with the milestone named.
8. Credit what is whole (the also-needs table says which clauses wait for M4 or M11).

## W4 (Sonnet), lane c, job 2: the block-lambda migration and the core language (26 tasks) (rolling slot)

1. **`[GRM-17]`:** a block after a lambda's `:` outside brackets is E0106 with a local function as the line to
   write. W3 built it and reverted it because 31 older tests use one (CLO-2, RC-2e, GRM-28, STD-19, CTL-1, CLS-2,
   ECS-4, JOB-5, CLO-15, …): rebuild it, migrate the 31 to local functions (the tests follow the spec), credit
   GRM-17 if whole.
2. **`[LEX-*]`, the rest:** LEX-1a, LEX-10, LEX-11, LEX-16, LEX-18, LEX-20, LEX-21.
3. **`[TXT-*]`:** TXT-2, TXT-3, TXT-4, TXT-6, TXT-8, TXT-9, TXT-10, TXT-11 (TXT-10's temporaries may need A1-18,
   M5: report).
4. **`[STR-*]`:** STR-1, STR-2, STR-3, STR-5, STR-6, STR-7.
5. **`[CTL-*]`:** CTL-3b, CTL-7, CTL-10, CTL-11 (CTL-1 and CTL-2 are M5's A2-6: not this job).

## W5 (Sonnet), lane c, job 3: the types of Part III, and LEX-10 (28 tasks) (rolling slot)

The rules, each a task: TYP-1, TYP-5, TYP-6, TYP-9, TYP-9b, TYP-10, TYP-12, TYP-13, TYP-14, TYP-17, TYP-19,
TYP-21, TYP-22, TYP-23, TYP-24, TYP-25, TYP-26, TYP-31, TYP-32, TYP-34, TYP-35, TYP-36, TYP-38, TYP-39, TYP-43,
TYP-44a, TYP-47; and LEX-10 first:

- **LEX-10, the lead's ruling (reading B, `review.md`, W4's question 3):** a line whose first characters after its
  indentation are `#!` is a directive, never a comment. So an indented `#! module overflow(wrap)` in a function body
  is `E0104` with the spec's message ("`#! module overflow(wrap)` is a directive, and directives come first; move it
  above `<the first import or item>`"). W4 built reading A (column 0 only) and an existing test pins A: change that
  test (the tests follow the ruling), build B, and credit LEX-10 again.

- Not this job (other lanes' milestones): TYP-15/15a (stored views, M5), TYP-44 (kind tests, M11), TYP-45/45a
  (owner words, lane a's M4 slice C), TYP-46/46a (memory managers).
- TYP-14: test what is built; the `shares`/`weakref`/`thread_shared` parts are M4 (E0900 today): report them.
- TYP-47 (any number of types and values) may be a large piece: build it if it fits, else test what is built and
  report what is missing, clause by clause.
- A class handle is still `Copy` on main (lane a changes that in M4 slice C): do not change `copy_with`.

