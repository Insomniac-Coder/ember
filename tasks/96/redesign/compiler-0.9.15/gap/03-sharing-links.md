# Gap map 03: sharing and links (RC, WK, CELL, OBJ, BCK, HND)

Area 3 of the 0.9.15_Hardened_1 gap map (`tasks/96/redesign/PLAN-0.9.15-compiler.md`, job 3). Read-only:
nothing was built or run. Every claim about the compiler cites the file and line it was read at, in the
tree at `ab2b7b76`.

**How the old tests were classified.** All 157 files in the 41 folders of these families were read.
120 were read by hand (every BCK, HND, OBJ, RC and WK file, and CELL-1 to CELL-4). The 36 RefCell files
in CELL-5 to CELL-12 and CELL-6a were classified by a script: it counted `RefCell` and `borrow` in each
and printed the first expectation. Three of them were then read by hand: two in CELL-10 and
`CELL-11/accept_cell_is_in_the_prelude.em`. A test counts as a `respell` when its expectation (its code,
line, stdout and count assertions) stays the same and only the spellings change. Changed message wording
counts as a respell, because the runner matches a substring (`compiler/ember_driver/tests/milestones.rs:334-341`).

**Design tests** were counted by a script (`target/gap-0.9.15/03/design_counts.py`) from each file's
`#$ rules:` line. 347 of the 841 tests carry only their design rule's main spec ID, so the exact-ID
counts are low. The rows below also name, by file, the tests that will move to these rules when their
IDs are refined.

---

## 0. What exists today, and what becomes compiler-built

None of the new words exist yet. A search of `ember_lexer`, `ember_parser`, `ember_ast`, `ember_hir`
and `ember_typeck` for `weakref`, `shares`, `thread_shared`, `sharedref` and `exists as` finds nothing.
Today's model is the old one: every class handle is `Copy`, and every copy is counted.

| today | where | what it does | in 0.9.15 |
|---|---|---|---|
| class handle is `Copy`; a copy retains, a drop releases | `ember_types/src/lib.rs:2119-2122` (`is_copy` true), `:2658-2659` (`needs_drop`); C retain glue `ember_codegen_c/src/lib.rs:8340-8395`, `count_function` `:8220-8228`; evaluator `ember_interp/src/object.rs:165-200` | an implicit +1 on every copy of a class handle | a class object has one owner; counts only where a line says `shares`/`thread_shared`/`weakref` (area 1 makes the class not `Copy`, A1-9; this area builds the counts) |
| `Shared[T]`, `SyncShared[T]` | compiler-made struct: `ember_typeck/src/lib.rs:19473-19510` (`counted_of`), name dispatch `:15094-15100`, re-instancing `:48134-48135`, `:34198-34203`, `:38310-38350`, `get`/`get_mut` `:50225-50240`, `:50610-50614`; `ember_types/src/lib.rs:1825-1830`, `:2307-2321`, `:2653`; `Builtin::SharedNew` `ember_hir/src/lib.rs:1982`; C `ember_codegen_c/src/lib.rs:10700-10711`, `:5415`; declaration `std/src/sync.em:104` | counted owner of a payload; `get_mut` checked at run time through the header's access word (`ember_analysis/src/borrows.rs:327-368`, `:1626-1648`) | **`shares T` / `shares mut T` / `thread_shared T`** as ownership kinds in the type; no run-time access check |
| `Weak[T]` | `ember_typeck/src/lib.rs:19542-19590` (`weak_of`, E2020 "`Weak` requires a class handle or `Shared[T]`"), `Weak(...)` `:38351-38360`, `Weak.empty()` `:37288-37292`, `:38871-38875`; `Builtin::WeakNew/WeakUpgrade/WeakEmpty` `ember_hir/src/lib.rs:1986-1991`; MIR `ember_mir/src/lower.rs:2906`, `:7061`; C `ember_codegen_c/src/lib.rs:10712-10734`, glue `:8395-8401`; runtime `runtime/ember_rt/src/ember_rt.c:3323-3373`, `:3401-3406` | a `Copy` weak handle; `upgrade()` gives a strong owner | **`weakref T`**, not `Copy`; opened with `if x exists as y:` (a header test; a count only for a `thread_shared` target) |
| `RefCell[T]`, `Ref`, `RefMut`, `borrow*` | `ember_typeck/src/lib.rs:19900-20060` (`refcell_of`, `ref_guard_of`), `:15127-15156`, `:46456`; `Builtin::RefCellBorrow*` `ember_hir/src/lib.rs:2194-2202`; C `ember_codegen_c/src/lib.rs:722`, `:5631-5850`, `:8712-8713`, `:10140`, `:13525`, `:14253`; checker `ember_analysis/src/borrows.rs:4057` (L3011), `:5350-5378` | run-time borrow flag | **removed**; `E1010` with CELL-13's message |
| `Cell[T]` | `ember_typeck/src/lib.rs:19647-19690` (`cell_of`), methods `:46346-46356` and `synth_cell_method` `:50320-50490`; `Builtin::Cell*` `ember_hir/src/lib.rs:2145-2165`; checker exemption `ember_analysis/src/borrows.rs:1553-1559` | set through a read-only (`self`) receiver | stays compiler-known; **set only through a name that may change its holder** |
| object header, 24 bytes | `runtime/ember_rt/include/ember_rt.h:1418-1455` (from `templates/ember_rt.h.in:1419-1455`): `strong`, `weak`, `access`, `flags`, `ti*`; C class struct `ember_codegen_c/src/lib.rs:5516-5537` (header plus per-field access words); LLVM `ember_codegen_llvm/src/glue.rs:80-230`; evaluator `ember_interp/src/object.rs:1-60` | — | **8 bytes** (count, type index) or **12** (plus weak count), with no flags and no access words |
| count limit | inline `ember_rt.h:1527-1550` (plain), `:1915-1948` (Sync); out of line `ember_rt.c:3034-3040` "reference count overflow on X", `:3275-3321`; weak `:3323-3340` ("retain of invalid weak reference") | one-thread limit exists with the wrong message. The Sync limit tests after an add that may already have rolled over. The weak limit is a racy load then add | RC-1/RC-4/WK-11: an exact compare (one thread); `fetch_add` then a test of `prev >= 2^31-1` (`thread_shared`); 8-bit test counts |
| resurrection check, deinit flag | `ember_rt.c:3375-3399` (`EMBER_OBJ_DEINITIALISING`, panics), `object_try_strong_retain` `:3052-3071` | caught while running | OBJ-5 refuses it when compiling (E3016 through summaries); no flag |
| count elision | `ember_analysis/src/uncounted_handles.rs` (RC-3, RC-2c), `temporary_owners.rs`, `count_report.rs` (RC-6), `frame_objects.rs` (OPT-1) | elides implicit retains | rework for explicit counts |
| ODR-065 retained copies | `ember_mir/src/lower.rs:264-275`, `:7402-7446` (`keep_handle_alive`, `lower_borrowed_place`) | keeps a stored handle alive for a borrow or call | removed: RC-5 is checked when compiling (E3060/E3021) |
| owner cycles | `ember_analysis/src/cycles.rs:80-130` (`lint_strong_cycles`: L3001 warning, `inspect_ownership_graph`), called at `ember_driver/src/lib.rs:5224-5229`; `explain/inspect --cycle` `ember_driver/src/lib.rs:630-641`, `:989-1075` | warning only | **E3032 refusal** unless proved opened |
| leak report | `ember_rt.c:537-960` (`L3017` at `:735`); evaluator `ember_interp/src/leaks.rs` | reports leaked cycles | reports only what `unsafe` or foreign code kept |
| E3016 | `ember_analysis/src/drops.rs:477-690` | `self` stored from inside a `drop` body | plus everything the drop calls, and `shares this`/`shares mut this`/`thread_shared this` |
| `Pool`/`Handle` | `std/src/collections.em:1359-1466` | std code, generation compare | unchanged (HND-3 is a text change) |

**What becomes compiler-built:** the ownership kinds `shares`, `shares mut`, `thread_shared` and
`weakref` in `TyKind`, replacing the `Shared_*`/`Weak_*` structs; making and ending owners and links,
with inline count operations and limit compares; the `exists as` opening of a link; the 8/12-byte header
and the type-index table; E3032's loop proof; the parent-link proof and the walk's deferred-drop region;
and the checks on link views, when compiling. `Cell` stays the compiler-known struct it is. `RefCell`,
`Ref`, `RefMut`, `Weak.empty()`, `upgrade` and `SyncShared` are deleted.

---

## 1. Rule table

The table has 54 rows, every H.41 row of these families. Status counts: built 1, partly 31, not built
13, remove 7, text only 2.

Abbreviations:
- **k/rs/rw/rt**: keep, respell, rewrite and retire counts for the rule's folder. The non-keep file names
  are listed in §1b.
- **"—"**: the rule has no folder.
- **Design tests**: exact `#$ rules:` matches; "→" names the tests that will move to the rule when the
  rule map is refined.

| rule | H.41 | what the spec now requires (one line) | today in the compiler | status | what must change | old tests | design tests |
|---|---|---|---|---|---|---|---|
| OBJ-1 | changed | header 8 bytes (u32 count, u32 type index), 12 with a weak count only for families some `weakref` may name; no flags, no access words | 24 bytes with `access`, `flags` and a `ti` pointer, `ember_rt.h:1425-1455`; access words in each class level, `ember_codegen_c/src/lib.rs:5522-5537`, `ember_codegen_llvm/src/glue.rs:80-230` | not built | new header (D1-D3), whole-program weakref-reachability per class family, type-index table, ABI bump (VER-4) | — | 0 |
| OBJ-2 | changed | an owner, link or view of an object is one pointer; interface tables through the type index; bare interface in a type position is E2173 | one pointer already (`ember_types/src/lib.rs:1606`); interface table through `ti` (`ember_rt.h:1400`, `:1491-1503`); a bare interface is a class handle (old OBJ-2) | partly | itables through the type index; E2173 for a bare interface (area 4/7, IFC/TYP-45) | — | 0 |
| OBJ-3 | changed | weak count starts at 1 for all owners; the object ends at count 0; freed when weak reaches 0 (at once where no weakref may reach) | as written, but through the `DEINITIALISING` flag (`ember_rt.c:3375-3399`, `:3342-3373`) | partly | "ended" is count 0 (D2); no weak word where none may reach | — | 0 |
| OBJ-4 | changed | objects from the runtime allocator, or the manager the class names with `using` | `ember_obj_new` (`ember_rt.c:3140`); no `using` on classes | partly | `using` on a class (area 1, ALC-1); allocation takes the type index | k 1 | 0 |
| OBJ-5 | changed | no owner of the object being ended is made in its `drop` or anything it calls (E3016); `weakref this` allowed; nothing checked at run time | E3016 inside the drop body only (`ember_analysis/src/drops.rs:477-690`); run-time resurrection panic (`ember_rt.c:3388`, `:3394`) | partly | E3016 through summaries and for `shares this`/`shares mut this`/`thread_shared this`; remove the run-time check (A3-6) | — | 0 (→ none exists: write one) |
| RC-1 | changed | `shares`/`shares mut`/`thread_shared x` make a counted owner in a block with the header; one owner is not counted; compare with the max before each +1, stop with "too many owners of a `Player` (4,294,967,295)" | no word; every class copy counted; limit with the message "reference count overflow on X" (`ember_rt.c:3034-3040`) | not built | ownership kinds in types (A3-2), counts only there (A3-3), limit and message (A3-1); Q2 | — | 33 (R3 4, R7 1, R8 4, R9 17, R10 1, R11 1, R15 5) |
| RC-7 | added | changed by `shares mut` owners and the first owner; a `shares` owner only reads (E3023); `thread_shared` only through locks (E7014); no access words | `Shared.get_mut()` checked at run time (`borrows.rs:327-368`) | not built | `shares mut` kind; E3023 from area 2's BRW-15; E7014 from area 5 | — | 0 (→ R9 accept/reject_a_change_through_a_shares(_mut)_owner, accept/reject_a_field_change_through_a_shares(_mut)_field, run_two_shares_mut_owners_*) |
| RC-2 | changed | no count op in RC-2a..e, tested by counting ops in the emitted C | `assert-c-count` tests on `ember_retain_plain` exist | partly | the count functions get their new names; assertions follow | — | 0 |
| RC-2a | changed | a shared value passed to a non-`owns` parameter: no count | a borrowed parameter takes no retain | partly | the `x: shares T` parameter form (needs A3-2) | — | 0 |
| RC-2b | changed | a view of a shared value or object (part, loop name, name opened from a `shares` link, parameter): no count, except RC-5 point 2 | a handle read within one expression is not retained (`count_report.rs` header; codegen) | partly | the set of views is wider: every view, not "one expression" | rw 1 | 0 |
| RC-2c | changed | a +1 immediately followed by a -1 of the same count, no call or store between | `uncounted_handles.rs:20-30` (D-506), wider than the text | partly | apply to explicit `shares` pairs | rw 1 | 0 |
| RC-2d | changed | an owner handed over (`b = a`, `owns` argument, `return a`, field from temporary) moves, no count | field from temporary moves (D-505); `b = a` copies and retains | partly | with area 1's hand-over | rs 1 | 0 |
| RC-2e | changed | loop names over a list of `shares` owners are views unless the body makes an owner (`shares p`) | the loop yields borrowed handles; implicit retain at `owned`/capture/push/return boundaries | partly | the count only at an explicit `shares p` | rw 24 | 0 |
| RC-3 | changed | further removals only when meaning is kept; no move of a `drop`, an `exists as` outcome or a foreign release | `uncounted_handles.rs`, `temporary_owners.rs` | partly | re-target to explicit counts; `exists as` replaces `Weak.upgrade` in the proof | rs 5, rw 8 | 0 |
| RC-4 | changed | one-thread counts plain; `thread_shared` counts (owners and weak) atomic, one `fetch_add`, stop when prev ≥ 2,147,483,647 | Sync counts atomic, chosen by a `ti->flags` test on every op (`ember_rt.h:1933-1948`); limit tested after the add rolls over (`:1915-1921`); weak: a racy load then add (`ember_rt.c:3323-3340`) | partly | static choice by type (D6); half-range limit; weak limit; with area 5 | — | 0 (→ R10 run_a_view_of_a_thread_shared_object_through_a_link_holds_an_atomic_count) |
| RC-5 | changed | a view through a link must not outlive its object: E3060 after a line or call that may end an object of its type; E3021 across a call that may change it; one atomic count only for a `thread_shared` object opened from a `weakref` | ODR-065 retained copies (`ember_mir/src/lower.rs:264-275`, `:7402-7446`); no type-level "may end" check | not built | A3-7 (with area 2's BRW-17 and area 4's summaries) | rs 3, rw 3, rt 1 | 5 (R10 2, R11 3) (→ R10's 10 BRW-17 tests) |
| RC-6 | changed | `ember inspect` lists each count op surviving in a loop, with the reason | `ember_analysis/src/count_report.rs` (`--counts`) | partly | count only explicit owners and links; rename "retain/release" | rw 1 | 0 |
| WK-1 | changed | no collector; a loop of owners allowed only where proved opened, otherwise refused | documented leak plus the L3001 lint (`cycles.rs:1-10`) | not built | E3032 (A3-9) | — | 0 |
| WK-11 | changed | `weakref T`: not `Copy`; `weakref x`, `weakref this.keep`; one weak count per link, limit-checked; making a link is not a change of the target | `Weak[T]` is `Copy` (`ember_typeck/src/lib.rs:19561-19563`); counts; `Weak.empty()`; limit message wrong | partly | `weakref` kind, `None` for empty, no `Copy`, limit (A3-1, A3-2) | rs 12, rw 1, rt 4 | 27 (R3 4, R5 1, R9 1, R10 2, R11 17, R13 1, R15 1) |
| WK-16 | added | `weakref` of a part is E2119; using a link unopened, or storing without `weakref`, is E2020 | no E2119 in `ember_diag/src/codes.rs` | not built | A3-2 | — | 4 (R11 4) (→ R11 reject_a_link_used_as_its_target_without_opening, reject_an_object_stored_into_a_link_without_weakref) |
| WK-12 | changed | opened with `exists as`: one header test; `thread_shared` target: a CAS loop +1 while the count is above 0, held for the view's life | `upgrade()` returns a strong owner, +1 even on one thread (`ember_rt.c:3052-3071`, `:3401-3406`) | partly | an opening is a test with no count; CAS only for `thread_shared` (GRM-44 parsing is area 7) | rs 5 | 0 (→ R11 run_an_empty_link_opens_to_its_else_branch, accept_a_link_opened_before_its_target_is_used, reject_upgrade_is_not_ember; R10 run_a_view_opened_from_a_link_needs_no_protection_*) |
| WK-2 | changed | ended at count 0 whatever links remain | as written (`ember_rt.c:3287-3307`) | built | none (the representation moves with OBJ-1) | — | 0 (→ R11 run_a_second_link_finds_its_target_gone_*) |
| WK-3 | changed | ended before its `drop` runs; links opened during the drop (its own or its children's) find it ended; `this.parent = None` allowed | `DEINITIALISING` flag makes `upgrade` fail (`ember_rt.c:3055-3057`, `:3383`) | partly | count set to 0 before the drop (D2) | — | 0 (→ R11 run_a_childs_drop_finds_its_parent_ended, run_a_link_opened_during_its_targets_drop_finds_it_ended) |
| WK-13 | changed | a `weakref` to a `shares` value names the same block | `Weak[Shared[T]]` does | partly | spelling only, through the new kinds | rs 1 | 0 |
| WK-14 | changed | `weakref`/`shares` never convert to or from C++ `weak_ptr`/`shared_ptr` | no conversion exists; C++ wrapper types in `std/src/ffi.em:431`, `:483` | text only | ffi.em's comments respelled (area 6/8) | rt 2 | 0 |
| WK-17 | added | a parent-link read with no header test where the package proves the link follows ownership and nothing ends such an object meanwhile | nothing | not built | A3-11 | — | 0 (→ R11 accept_a_parent_link_set_by_the_program, run_a_proved_parent_walk_prints_the_path, run_a_parent_link_that_does_not_follow_ownership_*, run_a_walk_follows_a_node_moved_*) |
| WK-18 | added | a walk across a call that may change nodes keeps a `weakref` and tests it each step; stops at the first ended parent; deferred-drop region where proved | nothing | not built | A3-11 (Q3) | — | 0 (→ R11 accept_a_walk_with_views_*, reject_a_walk_with_views_across_*, reject_a_visit_function_that_reads_*, run_a_walk_over_nodes_with_a_drop_method_*, run_a_walk_whose_visit_may_end_nodes_*) |
| WK-5 | changed | owner-edge graph after substitution: `shares`/`shares mut`/`thread_shared` fields and collections, function-value state, channel ends; not weakref, views, raw pointers; `let` fields never close a loop | class-field strong edges incl. one-owner fields and `Shared` (`cycles.rs:222-260`); no function values, channels or `let` rule | partly | A3-9 | rs 1, rw 3 | 0 |
| WK-6 | changed | a line that closes a loop of owners is E3032 unless proved opened before its objects lose their last outside owner | warning L3001 at the declaration (`cycles.rs:580-620`) | not built | A3-9 (Q1) | rs 1, rw 1 | 0 (→ R11 accept_a_cycle_of_shares_owners_is_warned, R9 run_a_tree_child_with_two_owners_* (both still expect retired L3001)) |
| WK-7 | changed | proof: same function and callees, or the owning object's always-called `clear()`; refused: run-time-dependent opening, loop handed out of reach; also through function values, `thread_shared`, channels | "conservative lint" (`cycles.rs:1-10`) | not built | A3-9 | rs 1, rw 1 | 0 |
| WK-15 | changed | debug `run`/`test` report objects alive at exit; in Safe Ember only `unsafe` or foreign code can cause one | debug report on by default (`ember_driver/src/lib.rs:747-748`; `ember_rt.c:960`) | partly | text and edges; test via `into_raw()` (area 6) | rw 1 | 0 |
| WK-8 | changed | report lists each group of leaked objects that own each other, classes, fields, link to make `weakref` | SCC report (`ember_rt.c:900-950`) | partly | owner edges per WK-5; no "predicted by L3001" | rw 4 | 0 |
| WK-4 | changed | per leaked object on a loop, the shortest loop as `Type.field` edges and the link to make `weakref` (L3017) | `ember_rt.c:563-735` | partly | text; edges | rw 1 | 0 |
| WK-9 | changed | `explain --cycle` and `inspect --cycle` on the graph of owners | `ember_driver/src/lib.rs:630-641`, `:1004-1075` on `inspect_ownership_graph` | partly | use A3-9's graph | rw 1 | 0 |
| HND-3 | changed | retired slot's memory stays with the pool until it ends (a bounded cost) | retired slots stay in `slots` and are never pushed to `free` (`std/src/collections.em:1456-1466`) | text only | none | k 2 | 0 |
| CELL-1 | changed | `Cell` API as before; setting is a change of the holder, through a name that may change it (E3023); store before drop | API built (`ember_typeck/src/lib.rs:50320-50490`); store-before-drop built (`ember_hir/src/lib.rs:2133-2145`); set through `self`, a shared borrow | partly | the receiver of `set`/`replace`/`take`/`update` must be changeable (A3-5) | k 2, rs 4, rw 2 | 0 (→ R4 accept_a_ref_mut_shares_mut_parameter_sets_a_cell, reject_a_plain_shares_mut_parameter_sets_a_cell) |
| CELL-2 | changed | no view of its contents, no run-time check; `get` a load, `set` a store with the count changes of any owner or link inside | built (private field, `ember_typeck/src/lib.rs:19610-19646` doc) | partly | with CELL-1's receiver rule; checker exemption `borrows.rs:1553` goes | k 2, rs 2, rw 1 | 0 |
| CELL-3 | changed | `Cell` not `ThreadShared`; `Send` if `T` is | not Sync (`ember_types/src/lib.rs:1872-1880`) | partly | the trait name and E7002 text (area 5) | rs 1, rw 2 | 0 |
| CELL-5 | removed | (RefCell's counter and panic) | `refcell_of`, `Builtin::RefCellBorrow*` | remove | delete (A3-5) | rt 6 | 0 |
| CELL-6 | removed | (`try_borrow*`) | `Builtin::RefCellTryBorrow*` `ember_hir/src/lib.rs:2200-2202` | remove | delete | rt 4 | 0 |
| CELL-7 | removed | (guards, L3011) | `ref_guard_of`; L3011 `borrows.rs:4057` | remove | delete; L3011 retired (area 7) | rt 10 | 0 |
| CELL-8 | removed | (RefCell not Sync) | `ember_types/src/lib.rs:1876` | remove | delete | rt 3 | 0 |
| CELL-9 | removed | (check in every profile) | `AssertKind::RefCellBorrow` in the C backend | remove | delete | rt 2 | 0 |
| CELL-10 | removed | (diagnostics suggest RefCell last) | help text in the borrow diagnostics | remove | delete the RefCell help | rt 7 | 0 |
| CELL-12 | removed | (RefCell never `Copy`) | `refcell_of` `derives_copy: false` | remove | delete | rt 2 | 0 |
| CELL-13 | added | nothing checked at run time; no value has a borrow flag or access word; `RefCell`, `Ref`, `RefMut`, `borrow`, `borrow_mut` are E1010 with the given message | RefCell exists; access words (EXC-19) | not built | A3-5; access words go with area 4 | — | 0 (→ R9 reject_refcell_is_not_ember) |
| BCK-1 | changed | views: read-only or changeable; a **position** (list element by index through owning locals) or an **address**; values carry their views | loans and regions (`borrows.rs:54-80`, `regions.rs`); no position/address split | partly | A3-8 (with area 2) | — | 0 |
| BCK-2 | changed | live where used later; a view whose last use is a call argument is not live across the call; the callee's summary checks it | NLL liveness built (`borrows.rs:1-30`) | partly | the call rule and summaries (A3-8) | rs 2 | 0 |
| BCK-3 | changed | conflicts: E3021, E3022, E3020; reshape ends an address, removal ends a position, push does not; "may be one" through links within a class family and for unproved indexes | overlap within one local only (`borrows.rs:5143-5176`); links left to run-time EXC checks | partly | A3-8 | — | 0 |
| BCK-4 | changed | storage end (incl. the owner handed over, replaced or ended) with a live view is E3060 at the later use; kept or returned past its life is E3063 | E3060 at scope end (`borrows.rs:3754-3906` `check_escapes`) | partly | the owner's replacement and hand-over as storage end; E3061 folded into E3063 | k 1, rs 1 | 0 |
| BCK-5 | changed | reserved then activated; at the start, arguments are checked against each other (E3022/E3021) unless proved one whole variable (MONO-10) | two-phase built (`borrows.rs:64-73`, `:4245-4260`) | partly | the argument cross-check and MONO-10 (area 1/2) | — | 0 |
| BCK-6 | changed | required acceptances, plus a list pushed to while a position of it is still used | three acceptances built (old BCK-6 tests pass) | partly | the position case (A3-8) | rs 1 | 0 |
| BCK-7 | changed | a place through a link is in another object; a view through a link lives only while nothing may end an object of its type (E3060), not across a call that may change one (E3021); only a `thread_shared` weakref view holds a count | old text: through a handle is governed by run-time EXC checks | not built | A3-7 | rw 2 | 0 |
| BCK-8 | added | per-function summaries (change, read, end with call chains; list touches; view-param use after end; aliases of `ref mut`; parent-link setting; locks), least fixed point; E3031 | effects over the call graph (`ember_analysis/src/effects.rs:1-12`), dynamic calls "every effect"; callable region summaries (`borrows.rs:2763-2930`) | not built | the summary record is area 4's (EFF-24); this area adds the bits in D7 and consumes them | — | 0 |

### 1a. Folders of unchanged rules

These folders were classified too, since they share this area's code:
- CELL-4: keep 1, respell 2.
- HND-1: keep 2.
- HND-2: keep 1.
- CELL-6a: retire 1.
- CELL-11: keep 1, retire 1.

CELL-6a and CELL-11 are not rules in either spec.

**Total over the 157 files:** keep 13, respell 43, rewrite 58, retire 43.

### 1b. Old tests, every non-keep named

Common respellings, used below:
- `self` → `this`; `fn drop(mut self)` → `fn drop():`; `fn m(mut self)` / `fn m(self)` → `fn m():`;
  `owned` → `owns`; a `mut xs` parameter → `ref mut xs`, with `ref mut` at the call;
  `r = pick(...)` → `r = ref mut pick(ref mut ...)`; `fn default()` in an `extend` → `static fn default()`.
- `Weak[C]` → `weakref C`; `Weak[Shared[T]]` → `weakref T`; `Weak(x)` → `weakref x`;
  `Weak[...].empty()` → `None`; `match w.upgrade(): Some(o) ... None` → `if w exists as o: ... else:`;
  copying a link → `weakref link`.
- `Shared(v)` → `shares v`; `.get()` and `.get_mut()` → nothing.
- A bare interface type → `owns dyn I`, with `owns` on the making line.
- `assert-c` needles that name `ember_retain_plain` / `ember_weak_retain` → the new count functions' names.

- **BCK-2** (respell 2): `accept_a_returned_loan_dies_at_its_last_use.em` (`ref mut` parameter and call, `r = ref mut pick(...)`), `reject_a_returned_loan_used_after_a_write.em` (same, plus E3022's new wording).
- **BCK-4** (respell 1): `reject_a_loan_outliving_its_place.em` (E3060's wording).
- **BCK-6** (respell 1): `accept_the_three_required_acceptances.em` (`ref mut xs` parameters and calls).
- **BCK-7** (rewrite 2): `accept_accesses_through_a_handle_while_a_field_is_borrowed.em` (`m = n` now hands the node over: needs `m = shares n`); `reject_a_handle_repointed_while_a_field_is_borrowed.em` (E3080 is retired; replacing `n` ends what `r` views: E3060 at `println(r[0])`).
- **RC-2b** (rewrite 1): `accept_a_handle_read_within_one_expression_is_not_retained.em` (no counts in a one-owner program; rewrite over `shares` owners).
- **RC-2c** (rewrite 1): `accept_a_copy_ending_while_its_source_holds_the_object_is_not_counted.em` (`b = a` hands over; rewrite with `b = shares a`).
- **RC-2d** (respell 1): `accept_a_temporary_stored_into_a_field_is_moved.em` (drop/this, count names).
- **RC-2e** (rewrite 24), all of them, because each asserts implicit retains of class or `Shared` copies:
  - `accept_iterator_destructured_class_handles_owned_parameters_retain.em`
  - `accept_iterator_destructured_shared_handle_owned_capture_retains.em`
  - `accept_loop_borrows_class_handles.em`
  - `accept_loop_class_handle_borrowed_capture_auto_derefs_method.em`
  - `accept_loop_class_handle_direct_owned_capture_retains.em`
  - `accept_loop_class_handle_owned_capture_retains.em`
  - `accept_loop_class_handle_owned_class_interface_parameter_retains.em`
  - `accept_loop_class_handle_owned_indirect_parameter_retains.em`
  - `accept_loop_class_handle_owned_interface_parameter_retains.em`
  - `accept_loop_class_handle_owned_parameter_retains.em`
  - `accept_loop_class_handle_owned_virtual_parameter_retains.em`
  - `accept_loop_class_handle_publication_retains.em`
  - `accept_loop_class_handle_return_retains.em`
  - `accept_loop_destructured_class_handles_owned_parameters_retain.em`
  - `accept_loop_destructured_shared_handle_owned_capture_retains.em`
  - `accept_loop_shared_handle_borrowed_capture_elides_retain.em`
  - `accept_loop_shared_handle_owned_capture_retains.em`
  - `accept_loop_shared_handle_owned_parameter_retains.em`
  - `accept_loop_shared_handle_publication_retains.em`
  - `accept_loop_shared_handle_return_retains.em`
  - `accept_loop_struct_containing_class_handle_owned_parameter_retains.em`
  - `accept_loop_struct_containing_shared_handle_owned_parameter_retains.em`
  - `accept_value_iterator_destructured_class_handles_owned_parameters_transfer.em`
  - `reject_loop_shared_handle_borrowed_capture_get_mut.em` (becomes E3023)

  The new form: a loop over `Array[shares Token]` (no count), and the body's `shares token` given to an `owns` parameter, a capture or a push (one count each). The interface, class-interface, virtual and indirect variants become duplicates, because the count is written at the call. Retire them at the rewrite unless the dispatch path needs its own test.
- **RC-3** (respell 5):
  - `accept_an_object_dies_at_its_last_owners_end.em`
  - `accept_fresh_base_upcasts_transfer_to_array_owners.em`
  - `accept_fresh_temporaries_transfer_to_array_owners.em` (`Array[Value]` → `Array[owns dyn Value]`, `push(owns Token(i))`)
  - `accept_keep_alive_marks_a_handle_use.em`
  - `accept_transferred_array_owners_preserve_weak_lifetime.em` (`weakref tokens[0]`)

  **RC-3** (rewrite 8):
  - `accept_a_handle_copied_from_a_list_uses_the_lists_count.em` (`t = nodes[i]` is now a view: no count at all)
  - `accept_a_handle_outlives_its_list_being_cleared.em` (now E3060 unless the list holds `shares` owners and `t = shares nodes[0]`)
  - `accept_a_handle_outlives_its_list_cleared_by_a_call.em` (same)
  - `accept_a_handle_outlives_its_list_element_being_replaced.em` (same)
  - `accept_a_handle_repointed_by_a_mut_parameter_keeps_its_count.em` (FN-9's re-pointing handle is gone; `ref mut h` writes the caller's place)
  - `accept_fresh_sync_temporaries_keep_atomic_counts.em` (`@sync` gone: `thread_shared`)
  - `accept_named_owners_survive_their_array_being_cleared.em` (`push(held)` now hands over: needs `shares`)
  - `accept_owned_calls_preserve_the_callers_temporary_drop.em` (the hand-over ends the object inside the callee; the stdout order changes)
- **RC-5** (respell 3):
  - `accept_an_inherited_generic_view_getter_on_a_derived_local.em`
  - `accept_an_inherited_view_getter_on_a_derived_local_borrows_the_local.em`
  - `accept_an_inherited_view_getter_through_self_or_a_reference.em`

  **RC-5** (rewrite 3):
  - `reject_a_borrow_through_a_stored_handle_past_its_statement.em` (`q = p` hands over; with `shares mut` it becomes E3060 at the later use)
  - `reject_a_derived_local_viewed_through_a_reference_is_not_replaced.em` (BCK-3: replacing `d` ends the address `v`, so E3060 at `println(v)` instead of E3021 at the write)
  - `reject_a_derived_local_viewed_through_an_inherited_getter_is_not_replaced.em` (same)

  **RC-5** (retire 1): `accept_a_handle_stored_in_an_object_is_kept_for_a_call.em` (ODR-065's retained copy for a call is removed; the refusal it becomes is BRW-21/E3031's)
- **RC-6** (rewrite 1): `accept_count_operations_left_in_loops_are_listed.em` (`nodes.push(node)` in a loop now hands over in round 1: needs `shares node`).
- **WK-11** (respell 12):
  - `accept_class_weak_field_owned_argument_copy.em`
  - `accept_class_weak_field_reassignment.em`
  - `accept_class_weak_field_return_copy.em`
  - `accept_generic_base_shared_weak_field.em`
  - `accept_generic_class_shared_weak_field.em`
  - `accept_loop_borrows_weak_handles.em` (`copies.push(weakref weak)` keeps its 2/2 counts)
  - `accept_loop_shared_weak_borrowed_capture_elides_retain.em`
  - `accept_loop_weak_handle_borrowed_capture_elides_retain.em`
  - `accept_shared_weak_class_field_argument_copy.em`
  - `accept_shared_weak_class_field_reassignment.em`
  - `accept_shared_weak_class_field_return_copy.em`
  - `accept_shared_weak_field_borrowed_argument_elides_copy.em`

  **WK-11** (rewrite 1): `accept_empty_shared_weak_copy_and_upgrade.em` (an empty link is `None` and makes no count; its 1/2 count assertions change)

  **WK-11** (retire 4), which test `Weak.empty()`'s type inference (D-401/D-434), a removed API:
  - `accept_weak_empty_at_an_option_place.em`
  - `accept_weak_empty_takes_the_type_it_is_used_as.em`
  - `reject_weak_empty_at_a_place_of_another_type.em`
  - `reject_weak_empty_with_no_type_to_take.em`
- **WK-12** (respell 5):
  - `accept_class_weak_upgrade_regression.em`
  - `accept_expired_class_weak_field_return_copy.em`
  - `accept_expired_shared_weak_class_field_return_copy.em`
  - `accept_live_shared_weak_upgrade.em`
  - `accept_shared_weak_class_field_upgrade.em`
- **WK-13** (respell 1): `accept_shared_weak_self_edge_does_not_keep_payload_alive.em` (`node = shares mut Node(...)`, `node.weak = weakref node`).
- **WK-14** (retire 2): `accept_distinct_weak_owner_forms.em`, `reject_weak_owner_interconversion.em` (the two `Weak` forms are gone; `weakref T` names its target type).
- **WK-15** (rewrite 1): `accept_a_debug_run_reports_a_leaked_cycle_by_default.em` (that cycle is now impossible: `Some(a)` hands `a` over, and a `shares` loop is E3032. Make the leak with `into_raw()` in `unsafe:`).
- **WK-4** (rewrite 1): `accept_each_leaked_object_is_named_with_its_shortest_cycle.em` (same; a loop left unopened because its owner was given to `into_raw()`).
- **WK-5** (respell 1): `accept_weak_class_handle_is_nonowning.em`.

  **WK-5** (rewrite 3), each to an E3032 pair or to "no diagnostic":
  - `warning_direct_strong_class_cycle.em`
  - `warning_mixed_cycle_suggests_first_weakable_edge.em`
  - `warning_shared_class_cycle.em`
- **WK-6** (respell 1): `accept_weak_self_field_breaks_cycle.em`.

  **WK-6** (rewrite 1): `warning_direct_strong_self_cycle.em`.
- **WK-7** (respell 1): `accept_generic_cycle_with_weak_back_reference.em`.

  **WK-7** (rewrite 1): `warning_generic_class_cycle_after_substitution.em`.
- **WK-8** (rewrite 4), leaks made only through `unsafe` or foreign code:
  - `accept_runtime_shared_self_cycle.em`
  - `accept_runtime_strong_self_cycle.em`
  - `accept_runtime_three_object_cycle.em`
  - `accept_runtime_two_independent_cycles.em`
- **WK-9** (rewrite 1): `accept_cycle_explanation.em` (one-owner fields are not owner edges; make them `shares mut`; drop the L3001 line).
- **CELL-1** (respell 4):
  - `accept_cell_method_through_shared_ref.em` (plain parameter)
  - `accept_set_stores_before_it_drops.em`
  - `accept_take_uses_default_and_returns_the_old_value.em`
  - `accept_update_uses_default_for_a_move_only_value.em`

  **CELL-1** (rewrite 2), which now need a `ref mut` receiver:
  - `accept_cell_replace_through_shared_ref.em`
  - `accept_cell_update_through_shared_ref.em`
- **CELL-2** (respell 2): `accept_a_cell_shows_a_copy_of_its_value.em`, `reject_a_cell_lending_its_value_to_a_written_fmt.em`.

  **CELL-2** (rewrite 1): `accept_cell_mutation_through_shared_ref.em` (needs `ref mut`).
- **CELL-3** (respell 1): `reject_a_static_cell.em` (E7002's text: "not `ThreadShared`").

  **CELL-3** (rewrite 2):
  - `accept_a_cell_field_of_a_class_that_is_not_sync.em` (`alias = counter` hands over; needs `shares mut`)
  - `reject_a_cell_field_of_a_sync_class_names_only_sync.em` (E7001 is retired with `@sync`; becomes a `thread_shared` holder of a `Cell`)
- **CELL-4** (respell 2): `accept_drop_follows_the_payload.em`, `reject_a_non_copy_payload_makes_it_move_only.em` (E3040's wording).
- **Retire 36**, every RefCell test:
  - CELL-5: `accept_a_refcell_class_field_borrows`, `accept_multiple_shared_borrows`, `run_fail_a_refcell_class_field_counts_its_borrows`, `run_fail_multiple_mut_panics_with_location`, `run_fail_mut_then_shared_panics_with_location`, `run_fail_shared_then_mut_panics_with_location`
  - CELL-6: `accept_try_borrow_mut_none_on_contention`, `accept_try_borrow_none_on_contention`, `accept_try_borrow_of_a_class_field`, `accept_try_borrow_some_when_free`
  - CELL-6a: `accept_try_borrows_return_none_in_every_profile`
  - CELL-7: `accept_early_return_releases_guard`, `accept_guard_drop_releases_for_later_borrow`, `accept_local_binding_holds_guard`, `accept_mutation_through_shared_borrow`, `accept_nested_borrows_and_scopes`, `accept_returned_guard_borrowing_a_borrowed_param`, `accept_returned_shared_guard_from_mut_param`, `reject_returned_guard_borrowing_an_owned_param`, `reject_stored_guard_in_static`, `warning_guard_live_across_call_l3011`
  - CELL-8: `accept_a_refcell_field_of_a_class_that_is_not_sync`, `reject_a_refcell_field_of_a_sync_class_names_only_sync`, `reject_a_static_refcell`
  - CELL-9: `accept_the_counter_is_one_machine_word`, `run_fail_the_check_remains_in_every_profile`
  - CELL-10: `accept_a_single_owner_uses_a_mut_parameter`, `reject_mutating_a_borrowed_container_parameter`, `reject_passing_a_borrowed_parameter_as_mut`, `reject_refcell_for_simultaneous_mutable_borrows`, `reject_taking_a_mutable_view_of_a_borrowed_parameter`, `reject_taking_ref_mut_to_a_borrowed_parameter`, `reject_writing_through_a_borrowed_value_parameter`
  - CELL-11: `accept_refcell_in_prelude`
  - CELL-12: `accept_move_transfers_borrow_state`, `reject_use_after_move_is_not_copy`

  CELL-10's six E3023/E3022 programs are still valid refusals under BRW-15. Area 2 may re-home them if BRW-15's folder lacks those shapes. Their `not-help: RefCell` and `help:` lines go.

**Kept as written (13):**
- `BCK-4/accept_a_loan_ending_with_its_place.em`
- HND-1 ×2, HND-2 ×1, HND-3 ×2
- `OBJ-4/accept_objects_are_made_by_the_runtime_allocator.em`: its needle `ember_obj_new(&em_ti_5Point)` holds only if allocation keeps taking the type record's address (D3)
- `CELL-1/accept_the_unconditional_api_and_get.em`, `CELL-1/reject_take_needs_default.em` (E2140 keeps its meaning)
- `CELL-2/accept_a_cell_costs_no_more_than_a_field.em`, `CELL-2/reject_the_contents_are_not_reachable.em`
- `CELL-4/accept_copy_follows_the_payload.em`
- `CELL-11/accept_cell_is_in_the_prelude.em`

---

## 2. Work items

The slices are ordered. Each slice leaves the compiler consistent: built, its tests green, nothing
half-removed. Sizes are S (under a day), M (days) and L (a week or more).

### Slice 1: count limits on today's header (independent; can land first)

**A3-1 Count limit compares and their stop texts** (S)
- Changes:
  - `runtime/ember_rt/templates/ember_rt.h.in` (`ember_retain_plain/_sync` around `:1527-1550` and `:1915-1948`; a new inline weak +1);
  - `templates/ember_rt.c.in` (the messages at `:3034-3040`, `object_panic_count_overflow`; weak `:3323-3340`);
  - `runtime/ember_rt/tests/count_overflow.c` and `sync_counts.c`;
  - the evaluator's `ember_interp/src/object.rs:165-200`.
- What it builds:
  - one-thread: compare before the add;
  - Sync: one `fetch_add`, then stop on `prev >= 2,147,483,647`;
  - weak counts the same way;
  - texts "too many owners of a `X` (4,294,967,295)" and "too many `weakref` links to a `X` (…)", with grouped digits and the class's display name from its type information.
- Completes: the limit halves of RC-1, RC-4 and WK-11, with RT-7, RT-8 and RT-10 (area 7's rows).
- Tests: runtime C tests. The three design count tests (R9 2, R11 1) pass once A3-2 and A3-4 land.
- Depends: none.

### Slice 2: `Cell` and the end of `RefCell` (lands with area 2's E3023 for changes through a read-only name)

**A3-5 `Cell` set through a changeable name; RefCell deleted** (M)
- Changes in `ember_typeck/src/lib.rs`:
  - `synth_cell_method` `:50320-50490`: `set`/`replace`/`take`/`update` need a receiver that may change its holder, else E3023;
  - delete `refcell_of`/`ref_guard_of` `:19900-20060` and the name dispatch `:15127-15156`, `:46456`, `:1717`, `:7961`;
  - the E1010 messages of CELL-13 for `RefCell`/`Ref`/`RefMut`/`borrow`/`borrow_mut`.
- Other deletions:
  - `ember_hir/src/lib.rs:2194-2202` and `ember_mir` `AssertKind::RefCellBorrow`;
  - `ember_codegen_c/src/lib.rs:722`, `:5631-5850`, `:8712`, `:10140`, `:13525`, `:14253`;
  - the LLVM and evaluator counterparts;
  - `ember_analysis/src/borrows.rs:4057` (L3011), `:5350-5378`, and the `inside_a_cell` exemption `:1553-1559`.
- Completes: CELL-1, CELL-2, CELL-5..10, CELL-12, CELL-13, and CELL-3 with area 5's trait name.
- Tests:
  - Old: CELL keep 6, respell 9, rewrite 5, retire 36.
  - Design: R9 `reject_refcell_is_not_ember`; R4's two Cell tests once `shares mut` exists (slice 3).
- Depends:
  - area 2 (BRW-15's E3023 machinery);
  - area 7 (retire L3011 in `ember_diag/src/codes.rs:456`, the [COST-5] labels, the error pages);
  - area 8 (the 47 `.em` files using RefCell, outside these folders).

### Slice 3: ownership words for sharing and links

This slice is one milestone with area 1's A1-9 (class objects are not `Copy`) and area 7's words.
Neither can land alone: once a class is not `Copy`, every program that shares a class handle needs
`shares`.

**A3-2 `shares`, `shares mut`, `thread_shared`, `weakref` as kinds in the type; `exists as` for links** (L)
- `ember_types`: a `TyKind` variant `Link { kind: Shares | SharesMut | ThreadShared | WeakRef, target }`
  (D4), not `Copy`, needing drop, one pointer wide. Replace the `Shared`/`SyncShared`/`Weak` cases at
  `ember_types/src/lib.rs:1825-1835`, `:2307-2321`, `:2653`, `:3518`.
- `ember_typeck`:
  - the words before a value and in a type;
  - `shares mut x` from a read-only name is E3023;
  - `weakref` of a part is E2119; an unopened use, or a store without `weakref`, is E2020 (WK-16);
  - `copy of` a link is E2116 (with area 1's OWN-8);
  - `None` as an empty link;
  - opening with `exists as` (parsing is area 7's GRM-44);
  - delete `counted_of`/`weak_of` and their callers (`:19473-19590`, `:15094-15126`, `:34198`, `:37288`, `:38310-38360`, `:38871`, `:48133-48142`, `:50225-50240`, `:50610`);
  - E1010 messages for `Shared`, `SyncShared`, `Weak`, `upgrade`.
- HIR and MIR: replace `SharedNew/WeakNew/WeakUpgrade/WeakEmpty` (`ember_hir/src/lib.rs:1982-1991`;
  `ember_mir/src/lower.rs:2906`, `:3532`, `:7061`) with make-owner, make-link and open-link.
- Backends and evaluator: C (`ember_codegen_c/src/lib.rs:10700-10734`, `:8382-8401`, `:5415-5460`), LLVM
  (`glue.rs`, `builtins.rs`, `func.rs`) and the evaluator.
- What it emits:
  - +1 with the limit (A3-1) at each `shares`/`weakref`, and -1 at each end;
  - a header test at each opening;
  - for `thread_shared`: atomic counts, the CAS opening and the view's count pair (RC-5 point 2);
  - a block with the header for a non-class value made shared, decided at the declaration of the local that is shared (D5).
- Completes: RC-1, RC-7 (with area 2's E3023 and area 5's E7014), WK-11, WK-12, WK-13, WK-16, WK-2, WK-14.
- Tests:
  - Design, by name, not run: R3 8, R9 about 14, R10 2, R11 about 16, R13 1. The rest wait on A3-7/A3-9/A3-11.
  - Old: respell WK-11 12, WK-12 5, WK-13 1, WK-5/6/7 accept 3, RC-3 weak 1; rewrite WK-11 1; retire WK-11 4, WK-14 2.
- Depends:
  - area 1 (A1-9, one-owner classes; OWN-13; `owns`);
  - area 7 (lexer and parser for the words, `exists as`, E2119 in the registry, E1010 texts);
  - area 5 (`thread_shared` type rules, E7014/E7015/E7017);
  - area 2 (E3023);
  - A3-1;
  - Q2 (sharing a part or a view).

**A3-3 Count operations only where an owner or link is made or ended** (M)
- What changes:
  - the class arm of the retain glue goes (`ember_codegen_c/src/lib.rs:8382-8390`, `count_function` `:8220-8228`; LLVM and evaluator likewise);
  - an owner's end is a decrement-and-test where the object may have another owner, a direct end otherwise (D5), and the "ended" store before the drop for a weakref-reachable family (D2);
  - `uncounted_handles.rs` and `temporary_owners.rs` re-targeted to explicit `shares` (RC-3, RC-2c);
  - `count_report.rs` counts explicit operations only (RC-6);
  - the `Option` and `drop` glue for links.
- Completes: RC-2, RC-2a..e, RC-3, RC-6.
- Tests:
  - Old: RC respell 5 (RC-2d 1, RC-3 4), rewrite 35 (RC-2b 1, RC-2c 1, RC-2e 24, RC-3 8, RC-6 1).
  - Design: R9 `run_two_shares_mut_owners_change_one_counter_with_no_run_time_check`, `run_a_shared_object_ends_with_its_last_owner`.
- Depends: A3-2; area 1 A1-9 (which also edits `uncounted_handles.rs`/`temporary_owners.rs`, so agree on one owner of those files).

### Slice 4: the 8/12-byte header (after area 4 removes the run-time exclusivity checks)

**A3-4 New object header and type-index table** (L)
- Runtime:
  - the header becomes `{count, type, [weak]}`, D1;
  - `ember_type_info` reached by index, D3, with the `field_desc`/`access_offset` tables removed (`ember_rt.h:1382-1411`);
  - `ember_obj_new*` take the type;
  - the `access`, `flags`, `PINNED`, `FRAME` and `DEINITIALISING` words and macros go;
  - the debug object tracking for the leak report moves to a side table;
  - `EMBER_COUNT_BITS` for TST-31;
  - VER-4 bump.
- Compiler:
  - a whole-program pass after type checking finds every class family and shared type some `weakref` may name (OBJ-1's 12-byte case);
  - C class struct `ember_codegen_c/src/lib.rs:5516-5537`, type info `:4939-4960`, payload offset `:5359`, `:8335`;
  - LLVM `glue.rs:80-230`; evaluator `object.rs`;
  - frame objects without the `FRAME` flag (D10; `frame_objects.rs`, `ember_codegen_c/src/lib.rs:8844`);
  - the driver flag behind `#$ count_bits: 8`.
- Completes: OBJ-1, OBJ-2 (the itables half), OBJ-3, OBJ-4, and WK-3's representation; with PHIL-9, RT-3, VER-4 (other areas' rows) and HR-12 (area 6: 16 bytes more in reload builds).
- Tests:
  - Design: R9 `run_fail_too_many_owners_of_one_player_stop_the_program`, `run_real_programs_keep_32_bit_counts`; R11 `run_fail_too_many_weakref_links_to_one_player_stop_the_program`.
  - Old: OBJ-4 (its needle).
- Depends:
  - area 4 (the access words gone: `ember_codegen_c/src/lib.rs:7521`, `:8085`, `:8165-8330`, `ember_rt.h:1953-1972`, `borrows.rs:327-1300`);
  - area 6 (HR-12's live list, the users of `EMBER_OBJ_PINNED`, `class_exports.rs`);
  - area 2 (OPT-1);
  - area 7 (TST-31 runner key);
  - A3-2.

**A3-6 Ended objects: count 0, and OBJ-5 checked when compiling** (M)
- Runtime: the object is ended by setting its count to 0 before its `drop` (WK-3), and openings test the
  count.
- Until summaries exist, a one-line test "count still 0 after the drop" stays in the out-of-line end
  path. It needs no flag, and it keeps OBJ-5 safe between slices.
- In slice 5, E3016 is extended through summaries, and to `shares this`, `shares mut this` and
  `thread_shared this` (`ember_analysis/src/drops.rs:477-690`), and the run-time test is deleted.
- Completes: OBJ-5, WK-3, WK-2.
- Tests: R11 `run_a_childs_drop_finds_its_parent_ended`, `run_a_link_opened_during_its_targets_drop_finds_it_ended`. Write an E3016 test (none of the 841 has one).
- Depends: A3-4; area 4's summaries (D7) for the second half.

### Slice 5: checking views through links (one milestone with area 2's checker and area 4's summaries)

**A3-8 The checking algorithm of XVIII.4** (L)
- What it builds:
  - position and address views (BCK-1);
  - a view whose last use is a call argument is not live across the call (BCK-2);
  - "may be one" through links within a class family (with EXC-3) and for unproved indexes (verdict 98) (BCK-3);
  - the owner's replacement or hand-over as a storage end, and E3063 (BCK-4);
  - the argument cross-check and MONO-10 (BCK-5);
  - the position acceptance (BCK-6);
  - each call checked by its callee's summary, E3031 (BCK-8).
- Changes: `ember_analysis/src/borrows.rs` (`Loan` `:54-80`, `overlaps` `:5143-5176`, `check_body` `:3460`), `regions.rs`.
- This is the same code as area 2's BRW items. **One implementation; the critic assigns it to one area.**
- Completes: BCK-1..6, BCK-8 (with area 4's summary record).
- Tests: old BCK respell 4; design R8 (area 2's).
- Depends: area 2, area 4 (EFF-1, EFF-24).

**A3-7 Views through links, and removal of ODR-065** (L)
- What it builds:
  - RC-5 point 1: E3060 after any line or call that may end an object of the view's type;
  - E3021 for a read-only view across a call that may change one;
  - DIA-28's choice between the two;
  - point 2's atomic pair for a `thread_shared` weakref view;
  - BCK-7.
- Deletes `ember_mir/src/lower.rs:264-275`, `:7402-7446` (`keep_handle_alive`, write-backs) and the RC-5 retained copies.
- Completes: RC-5, BCK-7.
- Tests:
  - Design: R10 12, R11 3 (RC-5).
  - Old: RC-5 respell 3, rewrite 3, retire 1; BCK-7 rewrite 2.
- Depends: A3-8; area 4's summary bits "may end T" and "may change T" (D7); area 2 (BRW-17).

### Slice 6: loops of owners

**A3-9 E3032: a loop of owners refused unless proved opened** (L)
- Replaces `ember_analysis/src/cycles.rs` with D8's owner-edge graph:
  - `shares`/`shares mut`/`thread_shared` fields and collections;
  - function-value state from area 4's possible-target sets;
  - channel ends (area 5);
  - `let` fields excluded; one-owner fields are not edges.
- Adds the closing-line check with WK-7's proofs: the same function and its callees by summary, or an
  owning object's always-called `clear()`.
- E3032 with B17's message. L3001 removed (`ember_driver/src/lib.rs:5224-5229`; codes registry, area 7).
- Completes: WK-1, WK-5, WK-6, WK-7.
- Tests:
  - Old: WK-5/6/7 rewrite 5.
  - Design: **none of the 841 tests E3032.** Write the B17 pair, a callee that opens the loop, `clear()`
    by the owning object, a refused run-time-dependent opening, a loop returned out of reach, a `let`
    field, function-value state, `thread_shared`, and a channel.
  - Remove the L3001 line from R11 `accept_a_cycle_of_shares_owners_is_warned` and R9 `run_a_tree_child_with_two_owners_changed_through_either`.
- Depends: A3-2; area 4 (targets and summaries, D7's "opens"); area 5 (channels); **Q1**.

**A3-10 Leak report and the cycle commands** (M)
- The runtime report (`ember_rt.c:537-960`) on the new header's side table. Its edges are D8's
  (emitted by the compiler's edge enumerators). It no longer says "predicted by L3001"; L3017's text
  stays.
- `explain --cycle` and `inspect --cycle` (`ember_driver/src/lib.rs:630-641`, `:1004-1075`) read A3-9's graph.
- Completes: WK-15, WK-8, WK-4, WK-9 (with area 7's CLI-18).
- Tests: old WK-15 1, WK-4 1, WK-8 4, WK-9 1, all rewritten as `unsafe` `into_raw()` leaks.
- Depends: A3-4, A3-9; area 6 (`into_raw()` only in `unsafe`, verdict 91).

### Slice 7: parent links and walks

**A3-11 Parent-link proof, the walk with a link, the deferred-drop region** (L)
- WK-17 condition 1 is a package-wide pass over every line that gives the parent field a target, every
  take-out of such an object, whole-list hand-overs, single ownership, no `drop` of its own, and no
  other package changing the field (EFF-4). Condition 2 comes from summaries. A proved read is
  compiled as a load and a test for no link.
- WK-18:
  - the walk's E3021 (from A3-7);
  - the deferred-drop region: a per-thread depth counter and deferred list (D9), with the recursive
    "ended" marking on the out-of-line end path;
  - region-local links with no weak count; the first link keeps its count.
- Completes: WK-17, WK-18.
- Tests: design R11, 9 named in the table. **Add `assert-c` checks**: the design tests only say in
  comments that the header test and the counts are absent.
- Depends: A3-3, A3-6, A3-7; area 4 (the summary bit "sets the parent link before reading it", BCK-8);
  **Q3**.

**Nothing to build:** HND-3 and WK-14 (text only); std's `Pool` spellings are area 8's.

---

## 3. Shared decisions (proposed)

**D1. Header layout.**
- 64-bit, non-reload:
  - offset 0: `count` u32 (owners);
  - offset 4: `type` u32 (index);
  - offset 8: `weak` u32, only in a class family (all classes under one root) or a shared value type
    that some `weakref` in the whole program may name;
  - payload at `round_up(8 or 12, align)`.
- Under `#$ count_bits: 8` (`EMBER_COUNT_BITS=8`), both counts are `uint8_t` at the same offsets
  (padding keeps `type` at 4), stored and compared as the real ones.
- Reload builds append HR-12's 16 bytes.
- Cost against C: 8 bytes per object, the same as a C++ polymorphic object's vptr; +4 (often +8 with
  alignment) only where a `weakref` may reach.
- Whole-program is sound today because Ember compiles every package from source: no package interface
  files exist yet (a search for an interface-file mechanism in `compiler/` finds none).

**D2. "Ended" is count 0, with no flag.**
- When the last owner ends an object of a weakref-reachable family, the count is stored 0 before its
  `drop`. A one-owner object holds 1 from allocation; the store is folded into the allocation's
  initialisation, so it is not a count op.
- An opening is `count != 0`. A `thread_shared` opening is a CAS +1 while the count is above 0.
- Openings during the `drop` see 0 (WK-3). Nothing counts again, so no resurrection is possible once
  E3016 holds.

**D3. The type index.**
- A u32 index into a program-wide array of fixed-stride type records. An object's record is
  `base + idx * stride` (a shift-add); `base` is a link-time constant, or one hoisted load in a dylib or
  reload build.
- Put the first virtual slots inline in the record, so a virtual call is two dependent loads
  (index, slot), as C++'s vptr then slot.
- Cost against C++: one shift-add per dispatch, plus one more load past the inline slots. Measure it in
  the measuring phase.
- `ember_obj_new` keeps taking the record's address, so the OBJ-4 test's needle survives.

**D4. Ownership kinds are in the type, not in wrapper structs.**
- `Link{Shares, SharesMut, ThreadShared, WeakRef}` over the target type.
- Every kind is one pointer: to the object for a class, to `{header, value}` for any other value.
  `None` is null (OBJ-2).
- `Class(id)` alone means one owner (area 1).
- One representation lets C, LLVM and the evaluator share the count and opening code.

**D5. Where an owner's end is counted.**
- An owner's end is a decrement-and-test (inline; the end itself out of line) exactly where the object
  may have another owner:
  - its type is `shares`/`shares mut`/`thread_shared`;
  - or it is a local that a line of its function shares;
  - or, depending on Q2, a place a callee may share by its summary.
- Everywhere else the end is direct, with D2's store where a link may look.
- A non-class value that is shared lives in a block from its local's declaration: the compiler sees the
  `shares` line in the same function, so nothing moves at run time.
- Cost: Rust `Rc`'s, only where sharing exists.

**D6. Count operations are chosen when compiling, by the kind in the type.**
- One-thread: `if (c == MAX) stop(); c += 1` and `if (--c == 0) end()`.
- `thread_shared`: `prev = fetch_add(relaxed); if (prev >= 0x7FFFFFFF) stop()`, and an acq_rel subtract.
- The run-time `ti->flags` test on every operation goes (`ember_rt.h:1933-1948`).
- **Reply to area 5's decision 1**, which asks this area to place a "thread_shared bit" in the 8-byte
  header: OBJ-1 forbids flags. Give `thread_shared T` objects a type index of their own instead: a
  second record that points at the same type information, used only for a family the whole program
  makes both plain and `thread_shared`.
  - The kind is then part of the object's type, not a flag.
  - It is read only where the static type cannot say: a method making an owner or link of `this`, an
    opening of a `weakref` in a mixed family, and the leak report.
  - Elsewhere it costs nothing.
  - It keeps RC-4's exact one-thread limit, which an always-atomic fallback would break by stopping at
    2^31-1.

**D7. Summary bits this area needs from area 4's summary record (EFF-24).** All are unions over a call
through a function value, `dyn` or virtual method's possible targets:
- may end an object of type T (RC-5 point 1, WK-17 condition 2);
- may change T (RC-5, WK-18);
- may make an owner of parameter k or of `this` (OBJ-5, D5);
- sets the parent link of an object it is given before reading it (WK-17, BCK-8);
- opens the owner edge `T.f` (WK-7: "the functions it calls", and `clear()`).

**D8. One owner-edge graph** (WK-5), built once after typing and generic substitution. It is consumed by
E3032, the leak report's edge enumerators and `explain`/`inspect --cycle` (CLI-18: the same graph for
both).

**D9. Deferred-drop region.**
- One per-thread depth counter: +1 at the walk's entry, -1 at its exit.
- An end on the out-of-line path while the depth is above 0 marks the object, and everything it owns,
  ended (count 0), and pushes its block on a per-thread list.
- The exit at depth 0 frees the listed blocks through the normal weak release, so a link outside the
  region still keeps a block.
- Cost: nothing per step; a test of the depth only on the end path.

**D10. Stack-promoted objects (OPT-1) lose the `FRAME` flag.** Escape analysis already proves every
owner and link of a frame object is a local of its function. So every end site of one is known when
compiling, and uses an end that runs the `drop` and frees nothing. Area 2 owns OPT-1; this area owns
the header.

---

## 4. Questions for the owner

**Q1. When does a line "close a loop of owners" (E3032)?**
- `[WK-6]`: "A line that closes a loop of owners (`a.next = Some(shares mut b)` after `b.next` names
  `a`) is refused, unless the compiler proves that the loop is opened again before its objects lose
  their last owner from outside".
- `[PHIL-13]` and XVIII.4 require every implementation to accept the same programs, but the spec does
  not say when a line counts as closing a loop.
- Example: `p.friends.push(shares q)` with `p` and `q` both taken from one `Array[shares mut Person]`.
  Whether it closes a loop is known only while running.
- Option (a): a line that stores an owner of a type that can reach the holder's type through owner edges
  is treated as closing a loop. It is refused unless the compiler proves the stored value cannot reach
  the holder, or that the loop is opened. The proofs that the value cannot reach the holder would need
  listing: a value made in this function whose owner fields are still empty, as in R9's
  `root.children.push(shares mut child)`, and others. This refuses most friend graphs.
- Option (b): refuse only lines proved to close a loop. That misses loops, which then leak, against
  WK-15's "nothing leaks through a loop of owners".
- Blocks A3-9, and writing E3032's tests. The proposal is (a), with the list of proofs given in the
  spec.

**Q2. May `shares` be made from a place that is neither a local nor already a counted owner?**
- `[RC-1]`: "A value made shared lives in a block of its own with the header of `[OBJ-1]` (a class
  object already does)". `[OWN-13]` allows `shares x` before any value, and lets an owner be made from a
  plain parameter.
- The cases: a list element (`shares mut players[1]` of an `Array[Player]`), a field
  (`shares this.items`), or a plain parameter (`fn keep(p: Player): LIST.push(shares p)`).
- For a class object it can be represented, but every end of such a one-owner place (the list's element
  drop, the caller's owner) must then test the count.
- For a non-class value held inline (`shares this.items`, or a view parameter of an `Array`) there is no
  block to count. Building it would mean changing the holder's layout everywhere in the program.
- Evidence:
  - the 842 design programs share only locals, values made on the line, and existing owners;
  - the help in R7 `reject_removing_a_player_a_kept_link_points_at.em` rewrites the list to
    `[shares mut Player(), ...]` rather than share a one-owner element.
- Options:
  - (a) refuse it (a new code, a sibling of E2119);
  - (b) allow it for class objects only;
  - (c) allow it everywhere.
- Blocks A3-2's typing of `shares` and D5's cost.

**Q3. In WK-18's deferred-drop region, when do the drops of what the ended node owns run?**
- `[WK-18]`: "an object whose last owner ends during the walk is set aside whole, it and everything it
  owns marked ended at once, and their memory is freed when the walk ends".
- The condition only says the node's class has no `drop` of its own. A node may own objects of other
  classes that have one, for example a `log: Logger` field whose `drop` prints.
- Do those drops run when the node is set aside, so only memory waits, or when the walk ends? The order
  of their output differs.
- The alternative of refusing the region when any type a node owns has a `drop` is a narrower proof
  than the MUST as written.
- Blocks A3-11.

---

## 5. Risks

**The size of the switch.**
- Classes are counted handles in every layer: types, MIR, the elision passes, both backends, the
  evaluator and the runtime.
- 647 test files declare a class, and 236 use `Weak`/`Shared`/`SyncShared`/`RefCell` (counted with
  grep over `tests/`). Inline Ember in Rust tests adds more.
- Slice 3 (with area 1's A1-9) is the riskiest milestone. It cannot be split: making classes not `Copy`
  without `shares` breaks every sharing program.

**Dependencies.**
- The header (A3-4) depends on area 4 deleting the run-time exclusivity machinery first: access words in
  the class layout, `ember_access_*` and `ember_object_*`, and `borrows.rs:327-1300`.
- Until then the 24-byte header must stay. A3-1, A3-2 and A3-3 are written to work on it.

**Unknowns.**
- E3032's analysis (Q1), and WK-17's package-wide proof, have no prototype and no measurement.
- D3's dispatch cost and D1's sizes are not measured. They are jobs M1 and M2 of the measuring phase.

**Defects found in the design tests and design text.**
- Two design tests expect the retired `L3001`: R11 `accept_a_cycle_of_shares_owners_is_warned.em` and
  R9 `run_a_tree_child_with_two_owners_changed_through_either.em`.
- R11 `run_fail_too_many_weakref_links_to_one_player_stop_the_program.em` says round 256 stops the
  program. With OBJ-3's weak count starting at 1, round 255 does. The expectation (the panic text, no
  stdout) passes either way; fix the comment.
- No design test covers E3032, E3016, RC-2*, RC-6, WK-15/8/4/9 or HND.
- The WK-17 and WK-18 tests check the absence of header tests and count moves only in comments, with no
  `assert-c`.
- `run4/v5/rules.md` section 2 says the deferred-drop region is "gone", while R11 and the spec's WK-18
  keep it. This report follows the spec.

**Overlap with other areas.**
- BCK is the algorithm behind area 2's BRW rules. RC-5 and BCK-7 are BRW-17's mechanism, and BCK-8 is
  area 4's summary record.
- A3-7 and A3-8 must not be built twice. The critic should give each to one area.
- Area 1's A1-9 also rewrites `uncounted_handles.rs` and `temporary_owners.rs`.

**Not verified.**
- Nothing was run, so the respell classifications assume the new message texts still contain each test's
  substring. They are written fresh by area 7, so some respells may become wording edits.
- The 36 RefCell retirements were classified by a script; three files were read by hand.
- The design-test counts "by name" for slice 3 are estimates from file names and headers.
