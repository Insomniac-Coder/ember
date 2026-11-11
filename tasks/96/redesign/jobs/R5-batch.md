# Round 5: one Sonnet worker and the Opus helper, 10 to 12 tasks each

Sizes: **W1 (Sonnet) 10, H (Opus helper) 12** (22 tasks in all). The lead writes M4's core at the same
time: `[RC-1]`, `[WK-*]`, A3-2, the `shares`/`weakref` kinds.

The owner's rules for rounds:
- each agent gets 10 to 20 tasks;
- at most 2 to 3 agents run at once, the Opus helper included;
- the lead collects, tests and combines each batch;
- the laptop rests 10 minutes after each batch.

**The rules of `N-batch.md`, "Rules for every job", apply to every task here.**
- Use `CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4`: the push gate runs on the same machine.
- Branch from local `main`. Your worktree may start from `origin/main`: run `git reset --hard main`,
  then check that `git log` shows round 4's merges (`aa1fdab4`).
- Build in your worktree's own `target/`, and delete it at the end.
- Commit each task separately. Write no `Box` anywhere: one owner in a block of its own is `owns T`.

A task that cannot be done as the spec says is reported with its rule, the program, what the compiler
does and what the spec says. **Never leave files outside your worktree.**

**The lead's files now (do not touch):**
- `ember_types/src/lib.rs`'s `Shared`, `SyncShared` and `Weak` (`counted_of`, `weak_of` and their
  callers in `ember_typeck`);
- `ember_hir`'s and `ember_mir`'s `SharedNew`, `WeakNew`, `WeakUpgrade` and `WeakEmpty`;
- `uncounted_handles.rs` and `temporary_owners.rs`;
- class objects' copy and retain (A1-9).

## W1 (Sonnet): defects, messages to the rules (10 tasks)

1. **`E3025`**, still emitted though retired (`ember_analysis/src/borrows.rs`, about line 5109). Read the
   spec's table of retired codes (`p17a-codes.md` in `tasks/spec-0.9.9/parts/`, or the spec's "Retired
   codes" table) for what replaces it, and emit that, with the rule's message.
2. **`E3030`**, the same (`ember_typeck/src/lib.rs`, two sites, about lines 36788 and 41075).
3. **`E3061`**, the same (`borrows.rs`, about line 4117).
4. **`E3080`**, the same (`borrows.rs`, about line 4966).

   For tasks 1 to 4: the tests that expect the retired code follow the new one, and the retired code is
   one that `tools/rule_index.py` and `codes.rs` already list as retired.
5. **`[DIA-10]` shape B2's help** gives the line to write, not a category word (`borrows.rs`, about line
   5188): `for ref mut x in xs:`, a list of your own added after the loop, `retain` after the loop, or
   the call moved below the loop, as the rule and `[DIA-7a]`'s B2 row say. Then worker D's DIA-10
   test (`.claude/worktrees/agent-af0eb72abdb1e39be/tests/conformance/DIA-10/`), rewritten to the rule.
6. **`[OBJ-5]` `E3016`'s message** (`drops.rs`, `report_drop_self_escape`): "`drop` ends this `Player`;
   an owner made from it would point at a freed object; keep a copy of what you need:
   `NAMES.push(owns copy of this.name)`", with the class's name and the statement as written. Then
   CLS-7a's and OBJ-5's tests from the same folder, rewritten to the rule's text.
7. **`[CLS-9a]`:** a change through a `let` field (`this.items.push(x)`, a `drop` that clears a list)
   is `E2104`, as the rule says. Today only assignment to one is.
8. **`[TYP-3]` / `[LEX-*]`:** a char literal with a surrogate escape (`'\u{D800}'`) reports `E0009`
   twice (once on the literal, once on the escape). It is one mistake, so one error.
9. **`[DIA-7]`:** shape B4's other case, the rebinding (`new_x = …`), which round 3 left as it was
   (ODR-235). Read ODR-235 and the rule; build it if the rule gives the line, otherwise report it.
10. **`[TYP-15]`/`[TYP-15a]`:** `TYP-15/reject_a_borrowed_str_pushed_into_an_array.em` pins TYP-15's
    old reading, which TYP-15a contradicts. Read both rules. If the test is wrong, rewrite it to the
    rule and report what the compiler does. Do not change the region analysis: it is the lead's (M5).

## H (Opus helper): M4's foreign owners, from `gap/06-ffi-hot-reload.md` slice F2 and `gap/01-ownership.md` (12 tasks)

Read `tasks/96/redesign/compiler-0.9.15/gap/06-ffi-hot-reload.md` (A6-5, A6-10) and `gap/01-ownership.md`
(A1-10, A1-11) whole first. Their line numbers are of an older tree: find the code by its names.

1. **A1-10.** `mem.forget` is removed. A call is `E1010` with `[OWN-6]`'s message. Drop
   `Builtin::MemForget` from HIR, the checker, MIR, `drops.rs`, `effects.rs`, both backends and the
   evaluator.
2. **A1-10.** `std/src/mem.em`'s signatures say `ref mut` and `owns` as `[STD-*]`'s rules give them. The
   tests that use `mem.forget` are retired or rewritten (list each).
3. **A6-5.** `ForeignBox[T]` is removed: a foreign object Ember owns is an `owns T` of the foreign type.
   `adopt[T](h)` gives `owns T` (`std/src/ffi.em`). `ForeignBox` named anywhere is `E1010`, whose message
   gives what to write (`owns T`), as `Box`'s does (A1-6, `ember_typeck`'s `Box` refusal).
4. **A6-5.** The representation, per the gap file's D2. The end of an `owns T` of a foreign type calls
   its contract's destructor once and releases the adopt record (`[FFI-36b]`).
5. **A6-5.** Under `threads(creator)` the thread is checked in every profile (as D-973 did for
   callbacks). The stop names the type, the line that made the value and both threads (`[FFI-33b]`).
6. **A6-5.** The importer (`ember_driver/src/cimport.rs`) writes `owns T` where it writes
   `ForeignBox[T]` today, in every generated signature and overlay text.
7. **A6-5.** Both backends and the evaluator, through the existing block machinery.
8. **A6-5.** The `ForeignBox` tests respelled, and FFI-36a's tests.
9. **A1-11** (its `owns` half). `x.into_raw()` on an `owns` value is built in, `E3100` outside `unsafe:`,
   ends `x` and gives `*mut T`, with no drop. It replaces `ForeignBox.into_raw`. The class-object half
   waits for A1-9 (the lead's): report what it needs.
10. **A6-10.** `Retained` is only for callbacks. Remove `Retained.pin`, `retained_pin`, `retained_unpin`
    and `ObjectPtr` from tokens, with their builtins in HIR, both backends and the evaluator, and the
    `retained` object contract. Read `[FFI-23]` whole first.
11. **A6-10.** The tests: FFI-23's `Retained.pin` tests are rewritten to what the rule now gives (C++ keeps
    an object by hand-over) or retired; list each.
12. **Checks:**
    - the quick check over `tests/conformance/FFI-*/`, `tests/ffi`, `tests/std` and every folder you
      touched;
    - `cargo test -p` each crate you touched;
    - `cargo test -p ember_driver --test milestones` for the FFI tests by name (`-- ffi retained foreign`).

**Effort.** Opus at the lead's level: read each rule whole, and the gap file's plan. Report every choice
the spec does not force.
