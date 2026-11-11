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
- **Step 3, next:** the checker makes `Link` values for **class targets**, on today's counted handles.
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
