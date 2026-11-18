# M4: shared owners and links, one owner for class objects, `copy of` (the lead's work plan, 2026-11-11)

The items, from `MILESTONES.md`:
- A3-2, A3-3;
- A1-9;
- A7-10's rest;
- A1-5;
- A5-8, A5-9;
- A1-10 and A1-11 (`mem.forget` gone, `into_raw()` only in `unsafe`);
- A6-5, A6-9, A6-10;
- HEAP-3 to HEAP-7.

Q-B is answered (verdict 103: only a whole value is shared). The design decisions are `gap/03-sharing-links.md`
§3, D1 to D10. This plan says what lands in which order on today's tree, and what waits for M6b's header.

## Who does what

- **The Opus helper (round 5):** A1-10 (`mem.forget` removed), A6-5 (`ForeignBox[T]` becomes `owns T` of a
  foreign type), A1-11's `owns` half and A6-10 (`Retained` only for callbacks). This is slice F2 of
  `gap/06`, which touches none of the files below.
- **The lead:** everything else, in the slices below. Each slice lands on `lead-m4` with the quick check
  and the Rust suite green. The branch goes to `main` when the class objects' one owner (slice C) and the
  words that make a second owner (slice B) are both in. Before that, `main` cannot say what a program
  that shares a class object must now write.

## What exists

- **Class handles.** A class handle is `Copy`, and every copy is counted: an implicit +1 on copy, a -1 at
  each end.
- **The compiler's shared types.** `Shared[T]`, `SyncShared[T]` and `Weak[T]` are compiler structs:
  `counted_of`, `weak_of`, `Builtin::SharedNew`, `WeakNew`, `WeakUpgrade` and `WeakEmpty`.
- **The header** is 24 bytes, with `strong`, `weak`, `access`, `flags` and `ti`.
- **The count limits** (A3-1) are built: the compare before the +1, `fetch_add` then the test of the
  previous count, and both stop texts.
- **The parser** reads `shares`, `shares mut`, `thread_shared`, `weakref` and `sharedref` before a type and
  before a value, and refuses each with `E0900`. `exists as` opens an `Option` (`[GRM-44]`).

## Slices

**A. The words in the type and before a value, on one thread.**
- **The type.** `TyKind::Link { kind, target }` (D4), with the kinds `Shares`, `SharesMut` and `WeakRef`.
  It is not `Copy`, it needs drop, and it is one pointer: to the object for a class target, or to a
  `{header, value}` block for any other value (today's `Shared[T]` block).
- **What it is called.** It is shown as `shares T`, `shares mut T` or `weakref T`, and mangled per
  `[MNG-6]`.
- **The words before a value** (`[OWN-13]`, `[RC-1]`):
  - `shares x` and `shares mut x` before a new value, a local of this function that owns its value, or
    a name that already is such an owner;
  - anywhere else it is `E2122`, with `[RC-1]`'s message;
  - `shares mut x` from a read-only name is `E3023`.
- **Links** (`[WK-11]`, `[WK-12]`):
  - `weakref x` makes a link;
  - `weakref` of an `owns` value is `E2119`, with the rule's message;
  - a `weakref` link may hold no target (`None`);
  - opening it with `exists as` gives a view while the target lives.
- **What runs.** Making an owner is a +1 with the limit; ending a `Link` owner is a decrement-and-test.
  Making a link is a weak +1; ending one is a weak -1. An opening is one test, `count != 0`. All of it
  is on today's header (its `strong` and `weak` words).
- **A non-class value made shared** lives in a block from the line that declares its local (D5). The
  local's type is the shared type through the whole function.
- **The backends.** C and LLVM, from one list of count operations, and the evaluator.
- **Tests:** the design tests R9 and R11 (one thread), and `[RC-1]`'s and `[WK-11]` to `[WK-13]`'s
  examples.

**B. `Shared`, `Weak` and `SyncShared` removed.**
- Naming one is `E1010`, whose message gives what to write (`[TYP-45]`): `shares T` or `shares mut T`,
  `weakref T`, `thread_shared T`. `.upgrade()` is `E1010`, with `exists as`.
- Migration edits, as A1-6 made. `ember __migrate --checker` rewrites the tests (TYP-22's 67
  `Weak[Shared[…]]` files among them).
- `counted_of`, `weak_of` and their builtins are deleted.

**C. Class objects have one owner (A1-9).**
- A class handle is not `Copy`. `=` and an `owns` argument hand the object over; the retain glue goes
  (A3-3), and so does ODR-065's store-back.
- `uncounted_handles.rs` and `temporary_owners.rs` are re-targeted to the counts the program writes
  (`[RC-3]`, `[RC-2c]`).
- `is` compares objects.
- The tests that keep two owners of one object without saying so are rewritten with `shares` (slice A).
  Their number is measured first, and the owner hears it before the rewrite.

**D. `copy of` (A1-5).**
- `copy of x` runs `Clone`. It is `E2116` for a link, a view, a type with no `Clone`, and a value whose
  views point into its own `let` field.
- `x.clone()` at a use is `E1010` with "write `copy of x`".
- A class object's `copy of` makes a new object, and a struct's link fields copy as more links.

**E. `thread_shared`** (A5-8, A5-9 and `[RC-4]`).
- Atomic counts, and the home and away counts of `[RC-4]` point 4.
- Opening a `weakref thread_shared T` holds a count for the life of the view (`[WK-12]`).
- `E7014`, `E7015` and `E7017`.

**Waits for M6b:** the 8/12-byte header (A3-4), "ended is count 0" with no flag (D2), and the type index
(D3). Until then the 24-byte header keeps its `strong` and `weak` words, and slice A writes the counts
there.

## Costs, to measure before each push

- **A one-owner class object** is no longer counted: the implicit +1 on each copy goes (slice C). This
  should make `counted_handles` faster than today's 1.133x of C++.
- **A `shares` owner** costs Rust's `Rc`: a compare and an add to make one, a subtract and a test to end
  one.
- **A `weakref` opening** is one load and one compare.
- **The bench:** `counted_handles` and `counted_sync_handles` on the laptop, before and after each slice.

## Progress (branch `lead-m4`)

- **Step 1** (`0f9b3baf`): the words are in the AST (`LinkKind`, `TypeKind::Link`, `ExprKind::Link`), and
  the parser builds them. The checker still gives `E0900`.
- **Step 2** (`979328a8`): `TyKind::Link { kind, target }` is in the type table:
  - layout, copy, drop, display, mangling, thread properties and the codec;
  - the cycle lint and the debug owner graph;
  - C: a class target's handle type, and the end of a link (`ember_release_*`, `ember_weak_release`).
- **Step 3** (`087ac833`): the checker makes `Link` values for **class targets**, on today's counted handles.
  - **Builtins.** Each takes its argument by reference (`ref x`), so that reading an owner never moves it:
    - `Builtin::LinkMake { kind, link }`: a +1, or a weak +1, and the same pointer (C: `(retain(*x), *x)`,
      as `WeakNew` does);
    - `Builtin::LinkTarget { link, class }`: a `shares C` read where a `C` is wanted, as a counted copy of
      the handle (+1). The temporary's end gives the -1, as today's handle copies do. This is a priced
      deviation until slice C, when a read is a view with no count.
    - `Builtin::LinkOpen { link, class }`: `if w exists as p:` gives `Option[C]` (`weak_upgrade`: a +1
      while the target lives). `[WK-12]`'s "one test and nothing else" waits for slice C's views and
      M6b's header.
  - **Refusals:**
    - `E2122` for `shares` of a list element, a field, a plain parameter or `this`, with `[RC-1]`'s text;
    - `E2119` for `weakref` of an `owns` value;
    - `E2020` for an unopened read of a `weakref` (`[WK-16]`);
    - `E0900` for a target that is not a class object (D5's block, in a later step) and for
      `thread_shared` (slice E).
  - **Everywhere a builtin goes:** effects, MIR, both backends and the evaluator.
- **Slice A's rest** (round 6, the Opus helper H2; R6-batch.md tasks 1 to 8):
  - **The refusals**, each with its rule's message where the rule gives one:
    - `E2122` (`[RC-1]`) for `shares`/`shares mut` of a list element (the list's own line, each element
      made with the word), a field (the field's line, made to share), a plain parameter (`owns p:
      shares T` and the word at each call) and `this` in any method (the method made a `static fn`
      that takes `owns me`);
    - `E3023` (`[OWN-13]`, `[RC-7]`) for `shares mut x` from a name that only reads (an owner made
      `shares`, a plain parameter of a `shares mut` type), and for a change through a `shares` owner
      (`s.hp = 5`, `other.push(owns 4)`), with the line that makes the owner `shares mut`;
    - `E2119` (`[WK-11]`, `[WK-16]`, `[OWN-13]`) for `weakref` of a new value, of a value held `owns`,
      of a part (a field held in its object, an element stored inline), of a local holding its value in
      place, of a value held `dyn I`, `owns dyn I` or `some I`, and the type `weakref dyn I` (the
      interface's lines made an `open class` with `virtual` methods, each implementing class derived
      from it);
    - `E2020` (`[WK-16]`, `[TYP-45]`) for a link used as its target unopened (a field, a method, a
      place that wants the target) and an object stored into a link place without `weakref`;
      `.upgrade()` on a link is `E1010` with `exists as`.
  - **A link may hold no target**: `None` where a `weakref T` is wanted (a field's default, a local, an
    argument) is a null link (`WeakEmpty`); re-pointing a link gives the old target's weak count back
    (the assignment's drop of the old link).
  - **The limits** (`[RT-7]`): `shares` and `weakref` go through the checked +1 of the runtime and the
    evaluator; the block's type information names the value by its own type ("too many owners of a
    `Counter`"). Tests with `#$ count_bits: 8` for both stops.
  - **Targets that are not class objects** (D5): `TyKind::Link` over any value; the block is `Shared[T]`'s
    (`counted_of`, made with the link type by `link_of`), and its one logical field (`Field(0)`, the
    pointer to the value, `TypeTable::link_payload`) is projected in every type walk and placed by C,
    LLVM and the evaluator as `Shared`'s was. `shares` before a new value makes the block
    (`SharedNew`); a local a line shares is made in its block on its own line (the checker's next pass
    of the body, `block_locals`), typed `shares mut T`; an `owns` parameter gets its block at the
    function's entry. An owner reads as its value in a field, a method's object, an index, an operand,
    a loop and an argument to a parameter that borrows; it is copied only where `=` copies, borrowed
    where a view is wanted, and never moved out of its block. An opening of a link to a value in a
    block gives `Some` of a `shares T` owner (read-only).
  - **Tests:** RC-1, RC-7, OWN-13, WK-11, WK-12, WK-13, WK-16 and RT-7, with fifteen of the design tests
    R9 and R11 (one thread) that slice A builds; C, LLVM and the evaluator agree.
  - **Not built here:** `shares`/`weakref` of an `owns` value or of a `dyn` value (`E0900`); a class
    local shared by a line keeps its class type (spec question); `shares mut T` to `shares T` and the
    class-family upcasts of `[TYP-5]` item 8 on links; the design tests that wait for `copy of` (slice
    D), the views of `[BRW-17]`/`[BRW-18]`, `[WK-17]`/`[WK-18]`'s walks and the cycle rule (A3-9).
- **Slice B** (round 6, H2; R6-batch.md tasks 9 to 13; `SyncShared` stays until slice E):
  - **`E1010`** (`[TYP-45]`) for `Shared[T]`, `Weak[T]`, `Shared(v)`, `Weak(x)` and `Weak.empty()`, each
    message giving what to write; `w.upgrade()` is `E1010` with `exists as` (`[WK-12]`); an owner's
    `.get()`/`.get_mut()` is `E2072` ("reads as a `T`; remove `.get()`").
  - **The migration** (`ember __migrate --checker`): `Shared[T]`/`Shared(v)` to `shares mut T`/`shares mut
    v`, `Weak[…]` to `weakref T`, `Weak(x)` to `weakref x`, `Weak.empty()` to `None`, `.get()` away, `if
    w.upgrade() exists as p:` to `if w exists as p:`, and `match w.upgrade():` with `Some(p)`/`None` arms
    to `if w exists as p:` and `else:`.
  - **The repository migrated**: the tests, `docs/book`, the error pages; the copies of a link or owner
    that a plain parameter or a class's field cannot give up (`Weak` and `Shared` were `Copy`) are
    written `weakref x`/`shares x` (another link or owner, as the copy was).
  - **Deleted**: `weak_of`, `Weak[T]`'s struct and every backend's and the analysis's code for it,
    `Builtin::WeakNew` and `WeakUpgrade`, `Shared`'s `get_mut`. `WeakEmpty` is `LinkNone`, a link's
    `None`. **What stays**: `counted_of`'s block (`Shared[T]`'s struct, `SharedNew`, its type information
    and its payload's place in every backend), which holds both a value made `shares` and `SyncShared[T]`
    (with atomic counts and its `get`).
  - **`ember inspect --alloc`** (`[SEL-1]`, `[PHIL-2]`): a link's declaration is reported by its row
    ("shares T, shares mut T", "thread_shared T", "weakref T"), and the allocation of a value made shared
    as "a value made shared". The reload test of `[HR-15]` (`weak_handles_follow_a_moved_instance`) is
    migrated and ignored: `[THR-2]` refuses a `weakref Particle` in a `static`, this compiler has no `static
    mut`, and the legal program needs `weakref thread_shared Particle` (slice E).
- **Slice C, measured only** (round 6, H2; R6-batch.md task 14; nothing rewritten):
  `count_handle_copies.py` counts the copies of a class handle the compiler keeps as a +1 that no line
  asks for (`ember_retain*((ember_obj_header*)x);` statements in the emitted C): **581 in 289 of the 2436
  programs** of the quick check's folders that build (compile-fail tests skipped; the 20 left out are
  parse-level tests), and **65 in 19 functions of `std`** (a generic function once, at its most):
  `std.ecs.World.add` 24, `std.ecs.aos_storage` 12, `std.ecs.run_staged` 4, `std.sync.open_channel` 4,
  `std.ecs.soa_storage` 3, and 1 or 2 in each of `Map.index_set`, `Map.push_new`, `Map.insert`,
  `Set.add`, `Query.commands`, `ecs.system`, `coroutine.wait_until`, `RemoveChange.apply`,
  `Without.fetch`, `World.destroy`, `World.remove`, `World.run_parallel`, `World.storage`,
  `Sender.clone`. By folder, copies (programs): tests/run-pass 65 (38), TYP-16 58 (26), RC-2e 37 (15),
  WK-11 31 (10), TYP-22 30 (20), RC-3 26 (12), EXC-18 22 (16), EXC-8 20 (6), FFI-39 18 (4), GRM-30 16
  (3), EXC-19 13 (9), DSP-3 12 (1), WK-12 11 (5), CTL-3 10 (1), ECS-4 9 (4), RC-5 9 (3), FN-9 8 (2),
  RT-10 8 (1), TYP-36 8 (3), CORO-11 5 (1), EXC-15 5 (2), CLS-7 4 (4), EXC-1 4 (4), EXC-6 4 (4), FFI-39e
  4 (3), OPT-2 4 (3), THR-10 4 (1), THR-13 4 (1), TYP-9 4 (2), WK-16 4 (1), WK-4 4 (1), CLO-10 3 (2),
  DRP-2 3 (1), DSP-1 3 (1), ECS-6 3 (1), EXC-10 3 (3), EXC-3 3 (3), FFI-17d 3 (3), GRM-19 3 (1), HEAP-10
  3 (1), HR-35 3 (1), OPT-1 3 (2), OWN-7 3 (2), WK-8 3 (3), tests/compile-pass 2 (1), BLD-FFI-5a 2 (1),
  CELL-5 2 (2), CLO-6 2 (1), CLS-2 2 (1), CLS-4 2 (1), DSP-2 2 (1), DSP-5 2 (1), ECS-2 2 (1), FFI-39c 2
  (1), FFI-39d 2 (2), FN-5 2 (1), GRM-13 2 (2), LT-1 2 (1), OWN-13 2 (1), PHIL-4 2 (1), RC-2c 2 (1), RC-6
  2 (1), RNG-4 2 (1), RT-7 2 (1), THR-11 2 (1), THR-2 2 (1), TYP-38 2 (1), WK-15 2 (1), WK-5 2 (1),
  tests/run-fail 2 (2), BCK-7 1 (1), CELL-3 1 (1), CELL-6 1 (1), CLO-5 1 (1), CORO-1 1 (1), CORO-12 1
  (1), ECS-5 1 (1), EFF-8 1 (1), EXC-2 1 (1), EXC-5 1 (1), EXC-7 1 (1), EXP-9 1 (1), GRM-1 1 (1), GRM-21
  1 (1), GRM-33 1 (1), HR-12 1 (1), LEX-15 1 (1), LNT-6 1 (1), MNG-4 1 (1), OBJ-4 1 (1), PHIL-1 1 (1),
  PHIL-3 1 (1), RC-1 1 (1), RFL-2 1 (1), SPN-1 1 (1), STR-5 1 (1), STR-6 1 (1), THR-1 1 (1), TYP-13 1
  (1).
- **Slice C, the lead's steps (2026-11-13, on branch `m4-slice-c-probe` in the lead worktree):**
  - **The probe:** `copy_with` gives `false` for `Class`/`ClassInterface` (`ember_types/src/lib.rs`): the quick check
    fails 262 programs (E3040 145+85, E3013 75+35, E3011 29+14, E3021 13+9; TYP-22 46, run-pass 31, EXC-18 16,
    RC-2e 12, EXC-19 9, RC-3 8, ECS-4 8, …). Every other program builds and runs as before, so the compiler already
    moves a handle that is not `Copy`.
  - **C2:** MIR records each store that hands a class object over (`m = n`), with the source's span, and `E3040`
    carries a migration there, `m = shares mut n` (the old meaning exactly: two counted owners that both change
    it); `ember __migrate --checker` applies it; a script then tries `shares` at each and keeps it where the program
    still checks.
  - **C3:** `[RC-1]` (Hardened_14): a class local a line shares takes `shares mut C` from its declaration, by a
    builtin that takes the one owner over with no count (`LinkAdopt`, an `Operand::Move`); a marked local whose
    declaration cannot make it (a loop's name) is `E0900`, never a second owner the type does not say.
  - **C4:** the tests and std migrated; a call that hands the object over `owns` and uses it again is rewritten by
    hand (its callee's parameter changes).
  - **C5:** `uncounted_handles.rs` and `temporary_owners.rs` re-targeted, `is` compares objects, ODR-065's store-back
    and the copy's retain glue for class handles go.
  - **C6:** the suites three ways, the bench; then `lead-m4` takes the branch.
  - **Found by C2 (2026-11-13): slice C needs `[BRW-12]` first.** After C2's migration 258 programs still fail, and
    most are reads of a class object out of a place: `x = list[0]` (E3011, "cannot move out of an element", 29),
    `x = p.child` through a view (E3013, 28 + 27), a part of a shared object (11). The copy used to hide them; under
    the spec none is a hand-over: `[BRW-12]` clause 2, "a name bound to a part … is a read-only view: never a
    hand-over, never a silent copy". `[BRW-12]` is M5's A2-3, and it holds for every type, not class objects alone
    (one rule, no exception by type). Tried on its own (`x = <part>` of a type that is not `Copy` binds a `ref`): the
    quick check then gives 3,344 "cannot write through a shared reference", the programs that write through a name
    bound to a part, which `[BRW-12]` clause 6 writes `ref mut x = <part>`. So the order is: `[BRW-12]` with its own
    checker migration (`ref mut` where the name is written through, a hand-over word where the part is moved
    out), then slice C on top. The experiment was reverted; C1 and C2 stay on `m4-slice-c-probe`.
- **Slice C and D, built (2026-11-19, on `m4-slice-c-probe`, with H4's A2-3 merged in):**
  - **C1-C3:** a class handle is not `Copy`; `m = n` hands the object over, and E3040 migrates a store to `m = shares mut n`;
    a class local (or `owns` parameter) a line shares is a `shares mut` owner from its declaration, by `LinkAdopt` (no count).
    A name no line gives a value of its own is `E0900` when the body settles (`unmade_owners`).
  - **C5:** an owner read where its object is wanted is the owner's own place: `TypeTable::link_field`, `Field(0)` of a link to a
    class is the same pointer; `LinkTarget` is gone, and no count is taken. A read-only class parameter takes the handle itself
    (`passed_by_address` is false for a class), so a plain parameter holding an object is a view (`is_part`). A class field holds
    no access word. `is` reads owners and leaves both operands in place. A class given to a parameter of its base or an interface is
    lent (`lending_upcast`). A view upcasts (`ref Child` to `ref Base`), an owner too. A generator's frame holds a lent object by a
    count of its own. Writing through an owner after it was handed over is E3040 (`handles` in the move paths).
  - **Still counted, until `[RC-5]`'s check when compiling (M6):** `keep_handle_alive`, the copy made for a call that reads a
    handle out of an object. `uncounted_handles.rs` and `temporary_owners.rs` stay until then.
  - **L3001** needs a counted owner in the loop: objects with one owner cannot form one.
  - **C4:** the tests follow (`shares`, `ref`, `owns`, `copy of`); those needing `shares dyn I` wait in `jobs/ahead/M11`. `std.ecs`:
    `Commands` are counted owners, `World.storage` gives a view.
  - **D:** `x.clone()` at a use is `E1010`, migrating to `copy of x` (120 sites); `@derive(Clone)` on a class makes a new object;
    `copy of` follows a type parameter's bounds and is one more view of a read-only view.
  - **Left in M4:** `TYP-36`'s row for owners (a `shares T` implements `Eq`, `Ord`, `Hash`, `Display`, `Debug` as `T` does), and
    **slice E**: `thread_shared`. These wait for E: ECS-4's two thread tests, THR-2, THR-11's scoped capture and THR-13.
- **Task 15, the suites** (round 6, H2): the quick check (failing 0; the touched folders on the C backend
  too), `cargo test -p` of every crate touched (`ember_types`, `ember_hir`, `ember_typeck`, `ember_mir`,
  `ember_analysis`, `ember_interp`, `ember_codegen_c`, `ember_codegen_llvm`, `ember_driver`; the reload
  test of `[HR-15]` ignored, above), `rule_index`, `error_pages` and `check_branding` green. The driver's
  suite found what the quick check does not read: eleven tests whose `assert-c` lines named the old
  `Weak` struct's C (`weak.value`, its `ember_obj_header* value`) or counted the +1 of a copy that is now
  a hand-over (the copy written `weakref x`/`shares mut x` where the test is about the copy); three
  examples of the specification that now check (IV.6a, Part VIII, VIII.5: their baseline entries go);
  and a test of `Shared.get_mut`'s elided access (HEAP-5's, retired with it), deleted with the dead path
  of `access.rs` that served it.
