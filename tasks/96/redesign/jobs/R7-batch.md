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

1. The 11 findings of `review.md`'s round 6, ruled, and the cut `0.9.20_Hardened_14` (running).
2. Then, one milestone ahead: M5's rules (`MILESTONES.md`, M5: views inside a function, loops, spans, kept views,
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
  the conflict waits for D2's ruling (the lead tells you).
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
