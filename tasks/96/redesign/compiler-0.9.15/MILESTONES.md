# Milestones: today's compiler to spec 0.9.15_Hardened_1 (the lead's plan, for the owner's approval)

From the gap map (`gap/01` to `gap/09`, one report per area) and its critic (`gap/10-critic.md`).

## What the map found

- **Coverage.** All 521 changed rules have a row in exactly one area, and so do all 112 new or changed
  error codes. The work is 160 items: 40 large (a week or more), 77 medium (days) and 43 small.
- **The compiler today.** The rule rows of areas 1 to 8 say:
  - 7 are already built, and 59 changed only in wording;
  - 304 are partly built and 123 are not built;
  - 40 are removals.
  - The largest pieces that do not exist yet are:
    - the whole-program summary of what each function changes, reads and ends;
    - the call checks built on that summary;
    - the new object header (8 or 12 bytes, no flags, no access words);
    - the proof that a loop of owners is opened again (E3032);
    - the checked trampoline for C callbacks.
- **Today's borrow checker** (`borrows.rs`, 5,620 lines, and `regions.rs`, 2,827 lines) survives as the
  base. About 1,700 lines of run-time access insertion go, with `access.rs`, `loop_access.rs` and the
  access words in every backend and the runtime.
- **Old tests.** 1,954 files were classified one by one:
  - keep 490, respell 1,016, rewrite 296, retire 152;
  - the rest of the 3,758 are in folders kept whole.

  A migrator does the respelling. Rewrites and retires land with the milestone that changes their rule.
- **The stopped helpers.** They merge cleanly at the old spec, on `main`, before the rework:
  - AK: 0 conflicts.
  - AJ: 0 conflicts.
  - AI: 14 conflict hunks in 5 files.

  After the first rework milestone, their old-spec proofs cannot be repeated.

## How every milestone lands

Each milestone is complete when:
- the compiler, the std library and the tests change together;
- every MUST of the rules it touches is built;
- the full check is green (quick check, `cargo test --workspace --profile ci`, the gates, WSL gcc);
- the laptop speed gate passes, where speed can change.

Then it is committed and pushed. Each milestone also does two things with the tests:
- the design tests it makes pass move from `run4/v5/conformance/` into `tests/conformance/<first spec ID>/`,
  so the suite stays green;
- the old tests of the rules it changes are respelled, rewritten or retired, as the map says.

**Agents.** Whether helpers work on a milestone, and how many, is a separate small plan for the owner,
milestone by milestone. M-A to M0 need no agents.

## The order

| # | Milestone | Items (area ids) | Size | Blocked by an owner question |
|---|---|---|---|---|
| M-A | Merge AK (compiler memory), on `main`, at the old spec | A9-1 | S | — |
| M-B | Merge AJ (a type for every function, ODR-278) | A9-2 | M | — |
| M-C | Merge AI (LLVM backend: reload, libraries, ABI, PGO); record the gates | A9-3, A9-4 | 1 M, 1 S | — |
| M0 | The spec-tied checks read 0.9.15; independent pieces | A7-1, A7-2, A7-3; A3-1 + A7-17 (count limit compares, `#$ count_bits`); A5-1, A5-2, A5-3, A5-21 (unsafe, codes); A7-18; A6-12, A6-16 | 12 S | — |
| M1 | The 0.9.15 grammar, the spelling-only forms and the migrator, in one commit | A7-4 to A7-9, A7-15; A1-1; A2-1's spelling half; A8-6; the mechanical part of A8-9 and A8-10 | 5 L, 5 M, 2 S | Q-A |
| M2 | Whole-program facts, with nothing new refused | A4-1, A4-2 (with A2-9's borrow facts in it, C2), A4-3, A9-11; A1-17's first half (`static mut` as a place, Y2) | 3 L, 3 M | — |
| M3 | Ownership: hand-over, `owns T` replacing `Box`, parameter and call words, `this` | A1-2 to A1-4, A1-6 to A1-8, A1-12, A7-10's `owns` part; A6-1, A6-4, A6-17 | 2 L, 5 M, 3 S | — |
| M4 | Shared owners and links; class objects have one owner; `copy of` | A3-2, A3-3, A1-9, A7-10's rest, A1-5, A5-8, A5-9, A1-10 + A1-11 (`mem.forget` gone, `into_raw()` in `unsafe`), A6-5, A6-9, A6-10; HEAP-3..7 | 2 L, 9 M, 1 S | Q-B |
| M5 | Views inside a function, loops, spans, kept views, generators | A2-2 to A2-8, A2-15, A1-18, A1-19, A3-5 (`RefCell` ends), A5-6, A5-7; A7-23, A7-26, A7-27 (their TYP texts, C8) | 4 L, 8 M, 4 S | — |
| M6a | Calls checked by the summaries; one name for one variable; messages that name the chain | A4-4 to A4-7, A2-10, A2-13, A1-21, A1-22, A3-6, A3-7, A3-8, A7-14 | 5 L, 6 M, 1 S | — |
| M6b | The run-time exclusivity checks removed, and the new 8/12-byte header, in one layout change | A4-8, A3-4, A2-11 (`RefCell` loans only), A9-5, A9-8, A6-19, L3013 | 2 L, 3 M, 1 S | — |
| M6c | Alias facts: disjointness, `noalias` on every `ref mut` | A2-12, A2-14 (with A7-19 folded in, C3), A9-10 | 4 M | — |
| M7 | Loops of owners refused unless opened (E3032); parent links and the walk | A3-9, A3-10, A3-11 (with the measured salvages) | 2 L, 1 M | — |
| M8 | Function values: capture words, two words `{code, state}` | A4-9, A4-10, A9-6, A9-7, A4-17, A5-10, A6-7 | 2 L, 4 M, 1 S | — |
| M9 | Threads: `thread_shared` values, spawn, scoped tasks, reductions, locks, module variables | A5-11 to A5-20, A1-17's second half, A6-13, A6-14, A9-9 | 3 L, 7 M, 3 S | Q-C, Q-D, Q-E |
| M10 | Memory managers and arenas with `using` | A1-13 to A1-16, A7-11 (with the measured choice about the instance word) | 1 L, 3 M, 1 S | Q-F |
| M11 | Interface values, kind tests and casts, generic bounds, `init`, `@cached`, class rules, contracts | A4-11 to A4-16 (with A1-20 and A7-12's checker, C4), A7-13 | 2 L, 6 M, 1 S | Q-G, Q-H |
| M12 | The C interface: `unsafe import c`, `@unsafe(export)`, the checked trampoline, what a foreign call may run | A6-2, A6-3, A6-6, A6-8, A6-11, A6-15; A5-4, A5-5 | 2 L, 2 M, 4 S | — |
| M13 | Hot reload on the new model | A6-18, A6-20, A6-21, A6-22 | 2 L, 2 M | — |
| M14 | std.ecs as EnTT's design, and the rest of std's semantics | A8-1 to A8-5, A8-7, A8-8, A8-11, A8-12 | 1 L, 5 M, 3 S | Q-I |
| M15 | Close-out: stop texts, debugger, mangling, tools, Annex A compiles, the flat memory curve, std.gpu | A7-16, A7-20 to A7-22, A7-24, A7-25, A9-12, A5-22 | 2 L, 2 M, 4 S | — |

**Why this order.**
1. The merges come first. They are proven at the old spec, and every later milestone changes outputs.
2. M0 makes `main` green at the new target, so `main` can then move to this branch.
3. M1 refuses every removed spelling and migrates the repository in one commit, so nothing is accepted
   two ways. A feature's 0.9.10 forerunner (`Shared`, `Weak`, `RefCell`, `@sync`, `owned fn`,
   `mem.forget`) stays only until the milestone that builds its replacement. That milestone removes it
   and migrates its users. This is the owner's "nothing half-removed", applied per feature.
4. M2 builds the facts every check needs before any check uses them.
5. The run-time checks go only in M6b, after the compile-time checks that replace them exist (M6a). They
   go in the same layout change as the new header, so the class layout changes once.
6. Function values (M8), threads (M9) and the C interface (M12) follow the model they rest on.
7. The ECS (M14) needs the new forms of `owns dyn`. Its rewrite in today's spelling (A8-3) can start
   any time after M1 if helpers are free.

## The lead's rulings on the critic's findings

None of these is an owner verdict. Each follows the spec, a verdict or the measurements, and each is
recorded in `decisions.md`. The owner can overturn any of them.

- **Who owns which work.**
  - The run-time access machinery is deleted by A4-8.
  - The summary's facts are A4-2's; area 2 supplies its borrow transfer functions.
  - E3031 is A4-5's.
  - `noalias` is A2-14's.
  - Kind tests are A4-14's, with A7-7 for the parse.
  - `into_raw()` is A1-11's.
  - The words' type representation is A7-10's, in area 7's shape, which has the `using` slot.
  - Area 1 gives the words their meaning.
  - Every mechanical respell is A7-9's (C1 to C9).
- **The header (S-1).** No flags.
  - A `thread_shared` object is told by a twin type index, numbered inside its class's span, so a kind
    test stays one compare (S-2).
  - The pin of an object given to C moves to the low bit of the reload-only `live_prev` pointer.
  - A family that a reload extends falls back to the base-chain walk for kind tests. That happens in
    reload builds only.
- **The weak count (S-3).** Whether a family gets the 12-byte header is a whole-program fact, like the
  summaries. Interface records carry it, and a change rebuilds the modules it affects. Area 3's claim
  that no interface files exist was wrong.
- **`owns dyn I` (S-5).** Two words, `{data, vtable}`, with the type index in the vtable. A kind test is
  then one load and one compare, and no 8-byte header is added to every boxed struct (ECS pools,
  `Array[owns dyn Shape]`).
- **Interface schema (S-4).** The summary goes in the schema after AK's and AJ's bumps.
- **Cycles Y1 to Y7.** Broken as the critic proposes: A1-17 is split, A1-8 is only the words, and A3-4
  and A4-8 share one slice (M6b).
- **The 68 unclassified files.** Each joins the milestone of its area, as the critic's table says.

## Questions only the owner can answer

Each will be asked on its own, in plain words, with real code, when its milestone comes up (sooner if
the owner wants). The full text is in `questions.md`.

| id | blocks | the question in one line |
|---|---|---|
| Q-A | M1 | Inside `@ffi(...)` contracts, does `owned` stay the "C takes it" word (verdict 58 says so), while every other use of `owned` is refused? (Likely only a confirmation.) |
| Q-B | M4 | May `shares` be made from a list element, a field or a plain parameter (`shares mut players[1]`), from class objects only, or never? |
| Q-C | M9 | After `h.join()`, may the parent change what the task used, inside the same `with` block? |
| Q-D | M9 | An integer reduction whose partial totals overflow while the true total fits, or the reverse: what happens? |
| Q-E | M9 | Handing an object to another thread while a `weakref` on this thread points at it: refuse when compiling wherever it may happen, or one compare of the weak count at the hand-over? |
| Q-F | M10 | The instance word: keep it as written, or drop it by laying memory out so a block finds its arena (measured)? |
| Q-G | M11 | How is a contract such as `@noalloc` written on a function type, or is that sentence dropped? |
| Q-H | M11 | A private generic that writes some bounds: are the rest inferred, and what if two interfaces in scope both supply a method? |
| Q-I | M14 | One-element tuples `(A,)` for `registry.view[(A,)]()`, or another way to name one type? |

The other questions the areas raised are settled in `decisions.md`, from the spec's own text, a verdict
or the measurements: `from` on any field, the closed list of index proofs, `for i, ref mut x in
xs.enumerate():`, plain parameters across calls, E3032's exact rule (verdict 99), the region's drop
order, ODR-278 in the spec, codes, destructors, cdylib exports and the like.
