# The gap map after the helpers' merges (lead, 2026-10-27)

The gap map (`gap/01` to `gap/10`, `MILESTONES.md`) was made against `main` at `561143a1`, with the three
stopped helpers' branches unmerged. On the owner's word ("bring their work to main") they are now merged
on `main`, at the old spec, as milestones M-A to M-C planned. This note re-checks the 160 work items
against the merged compiler.

## 1. What is on `main` now

| Commit | What |
|---|---|
| `158cc9a0` | M-A: helper AK (ODR-281 part way: the compiler holds only what its later stages read) |
| `b8fed8ae` | M-B: helper AJ (ODR-278: every function and capture-free lambda has a type of its own) |
| `8fa95b5f` | M-C: helper AI (the LLVM backend's rest: hot-reload images, cdylib and staticlib, PGO, vectorcall, AAPCS64, ODR-274..277, D-911..D-916) |
| `80ad7bce` | D-917 (an instrumented test's entry points declared), D-918 (a Ninja build the same bytes in another folder under clang-cl) |
| `347eff57` | README: phase 9 at 23% |
| `6aad1828` | D-919 (`--pgo-use` finds `llvm-profdata` beside a Clang found on `PATH`) |
| `3971831f` | the ledger: D-911..D-916, and D-920 (open) |
| `93e897ac` | `[RT-5]`: the 105 hard-coded names the branding check found, spelled through `ember_branding` (most from AJ's and AI's code and tests); the same IR byte for byte |

The checks each merge passed are in the merge commits and in `tasks/96/helpers-stopped.md`. Since the
merges, 60 files of `compiler/`, `std/` and `runtime/` changed (ember_codegen_llvm 15, ember_driver 13,
ember_analysis 7, ember_codegen_c 6, ember_build 5, ember_parser 3, ember_mir 3, ember_types 2,
ember_interp 2, ember_typeck, ember_hir, ember_ast and `std/src` 1 each).

## 2. The line references

Every `file:line` in `gap/01` to `gap/10` reads `561143a1`'s file. They are not renumbered: read one with
`git show 561143a1:compiler/<path>`, or find the named function on `main`. Where a work item names a
function or a type, the name is the stable reference.

Checked by name on `main` (all present): `refcell_of`, `lower_refcell_borrow`, `check_refcell_call`,
`is_guard_loan`, `guarded_parent`, `state_member`, `refcell_for_guard`, `behind_dyn`, `access_call`,
`has_field_word`, `VirtualSlots`, `insert_shared_accesses_all` (A9-5); `callable_values`, `item_to_shape`,
`fn_value_of`, `Const::Fn` (A9-5, A9-7); `class_field_offsets_after` (A9-8: now
`ember_types/src/lib.rs:2503`, "HEAD 2434" in the map); `emit_class_exports`, `class_exports.rs` (A9-9);
`body_signature`, `Ext::Ref`, `restrict_views` (A9-10); `plain_function_values` (A9-11:
`ember_analysis/src/effects.rs:904`, unchanged); `tools/memory_curve.py` (A9-12).

## 3. Work items the merges change

The count stays 160 (40 L, 77 M, 43 S). M-A, M-B and M-C are done. What changes:

- **A9-4 (record the gates).**
  - (a) B9-curve reports and does not gate: already so. `the_compilers_speed_is_measured_...` asserts
    that only HR-9's ratio is bounded.
  - (b) The LLVM matrix ran at the AI merge and again after the branding fix:
    `EMBER_BACKEND=llvm EMBER_PROFILES=all` over every run test, 2152 of 2152; the driver's llvm 13,
    reload 51, cdylib 11 and staticlib 6 with it selected. From now on it runs in every milestone's full check: the lead's full-check script runs it after the MSVC suite.
  - (c) The list of checks red on this branch because they read the new spec stays M0's.
- **A6-18, A6-20, A6-21, A6-22 (hot reload, M13): both backends.**
  - What an image says of itself is decided once, by `ember_codegen_c::image`'s planner, and each
    backend writes it through its `Writer`: the C backend's, and the LLVM backend's
    (`ember_codegen_llvm/src/reload.rs`).
  - So every new schema kind (A6-18), fact record (A6-20), generator frame schema (A6-21) and `using`
    migration (A6-22) is written twice: add `ember_codegen_llvm/src/reload.rs` to each item's files.
  - The reload tests (now 51) run under both backends.
  - `gap/06` risk 5's "Main's LLVM backend has no reload images" no longer holds.
  - An image is built one C unit per module through Ninja (`ember_build::ninja::Program` with an
    `Image`). An IR unit has a rule of its own (`ll`), and both backends share the driver's
    `build_reload_program`.
- **A6-6 (the checked trampoline, M12).**
  - Both backends' trampolines change, as `gap/06` says.
  - The LLVM one on `main` is now AI's (with HR-42's depth counter).
- **A9-9 and A6-13 (class exports for `thread_shared` handles).**
  - `@sync` class exports are on `main` in both backends (C `class_exports.rs`, LLVM `ffi.rs`
    `emit_class_exports`).
  - The items stand as written.
- **A9-5, A9-6, A9-7, A9-10, A9-11, A9-12.** Stand as written; what they delete, adapt or build on is on
  `main` now.
- **D-920 (open; M2 or M5).**
  - `development` and `release` builds of `tests/firstweek/helped/21_particles.em` panic in the
    compiler: "callable-region metadata disagrees with the MIR body" in a `struct`'s two `mut self`
    methods (`Rng.next_float`, `Rng.range`).
  - The same before the merges.
  - The callable-region facts are what M2's whole-program summaries build on, so M2 either fixes D-920
    first or shows the summaries make it moot.
  - Either way, M2 adds a check that builds the `firstweek/helped` copies in an optimising profile.
- **D-917, D-918, D-919 (fixed).** No work item changes. D-918's rule:
  - the Ninja file spells the build's own files from its folder;
  - any later change to the image or library build keeps it (`ninja.rs`, `from_dir`).

## 4. The owner's answers (verdicts 102-106) and the milestones they unblock

| Question | Verdict | What it changes | Milestone |
|---|---|---|---|
| Q-A | 102: `owns` in a C contract | `[FFI-11]`'s word is `owns`; `owned` is refused everywhere (the respelling and the migrator) | M1, unblocked |
| Q-B | 103: (a) no `shares` of a one-owner place | A3-2 types `shares` of a list element, field or plain parameter as an error with a code of its own (a sibling of E2119) and a help naming the rewrite; the count test at every one-owner end (the cost D5 named) is not built | M4, unblocked |
| Q-C | 104: the parent may use what the task used after `h.join()` | `[THR-11]`, `[THR-13]`: a task's claims end at its join | M9 |
| Q-D | 105 (for now): a task's own total stops on overflow | `[THR-21]` reductions: each task's private total has the variable's type and checked adds; the join's adds are checked; THR-21's "always what adding one by one gives" is narrowed for whole numbers | M9 (Q-E still open) |
| Q-G | 106: a function type's contract above its `type` line | `[EFF-2]`'s sentence gets its grammar: `@noalloc` over `type Handler = fn()`; parse it in M1's grammar commit, check it in M11 | M1 (parse), M11 (check; Q-H still open) |

Still open: Q-E (M9), Q-F (M10: the owner wants (b) and (c) together "if it's safe and fast"; the numbers
are in `../measure-0.9.15/RESULTS.md` section 7 and the safety argument in `../measure-0.9.15/M3.md`
"Verdict 47's two ways"), Q-H (M11) and Q-I (M14), which the owner asked to have explained again.
