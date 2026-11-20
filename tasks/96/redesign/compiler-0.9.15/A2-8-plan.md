# A2-8: kept views — `sharedref mut`, `ref mut` fields alone, class view fields with `from`, values that hold views (a plan, 2026-11-20)

The item is `gap/02-views-borrowing.md`'s A2-8 (M5's last area-2 item before A2-15). Rules, in
`docs/spec-source/Ember_v0.9.20_Hardened_14.md`: `[LT-46]`, `[LT-45]`, `[LT-47]` (callee side), `[LT-42]`, `[LT-26]`,
`[BRW-17]`, `[BRW-22]`, `[THR-8]`, `[BRW-1]`, with `[TYP-15]`, `[TYP-34]`, `[EXC-3]`, `[EXC-17]`, `[OWN-8]`. Tests written
ahead: `jobs/ahead/M5/` (BRW-17 x3, BRW-1, LT-18, BCK-4, BRW-12), `M6a/` (EXC-17, LT-47 x2), `M9/THR-15`,
`M11b/EFF-27` x2; the design set `run4/v3/conformance/R7` (14 files) and `R9` (6) in older syntax
(`players.push(Player())` with no `owns`), migrated only where a step names them.

## Where it stands (probes with `target/ci/ember.exe`, 2026-11-20)

- The parser takes both words (`ember_parser/src/decls.rs:2116`, `body.rs:1360` give `ast::LinkKind::SharedrefMut`);
  `E0114` is built (`tests/conformance/GRM-43/reject_sharedref_alone.em`). Typeck refuses the rest with `E0900`:
  `resolve_type` (`ember_typeck/src/lib.rs:15964`), `synth_link_expecting` (`:21806`), `type_from_expr` (`:43022`).
- A kept `ref mut` of a class element is the address of the list's slot (`_7 = &mut _1[0]`, then `(*_2.0).0`), so a
  push while `w` is used is `E3022` at the push. A second kept `ref mut p` is `E3022` with 0.9.10's text ("`p` is
  already mutably borrowed"); a write through another name while a kept `ref mut` field is used is `E3021`
  ("cannot be written while it is borrowed"; `[BRW-1]` says `E3022`); `c.at += 1` through a `ref mut i64` field is
  `E2020`.
- `class Player: name: StrView` checks clean (no `E2118`); `words: Array[StrView] from text` is `E0100` (`from` on a
  field does not parse). A view struct stored in a local list is `E3063` ("may not be stored in an `Array` unless it
  is `static`", `stores.rs`) before any return, though `[TYP-15]` lets a local list hold views.
- `[LT-47]`'s callee side: every view put into a `ref mut` view parameter is `E3063` with the old text ("a view of
  `names` is stored in `out`, which the caller owns"), a part of a view argument included; `best = mine` is reported
  on `best = other`. The `weakref` half of BRW-17's accept file checks clean today.

## The representation

`sharedref mut T` is, at compile time, a view in every rule (`[LT-45]`, `[TYP-34]`, `[BRW-17]`; `[LT-26]`:
"constructing, copying or storing it counts nothing") whose loans several holders and the owner share, and at run
time one pointer (`[LT-46]`: "costs nothing"). Four ways to carry it, sizes estimated in lines of Rust:

| | A. `LinkKind::SharedrefMut` (`TyKind::Link`) | B. `shared` on `TyKind::Ref` | C. a compiler-known wrapper struct |
|---|---|---|---|
| ember_types | a kind, `word()`; `link_payload`/`link_field`/`intern_link` (`lib.rs:888-958`) give a non-class target a `{header, value}` block, which this has not; `is_view`, `needs_drop` learn it | `Ref { mutable, shared, inner }`; `render`, `canonical_name`, `codec.rs`; one `thread_properties` arm (`:2083`) | nothing new: a struct as `soa_proxy_of` makes (`ember_typeck/src/lib.rs:57098`) |
| typeck | read-through exists for `shares mut`; view-ness is new everywhere a view is asked (`is_view`, `@view`, view structs, `E3063`) | the words go to the `RefOf` path; field access, methods, `+=`, plain-parameter read-through, `b = w.target` read-only, all `ref mut`'s | a proxy case at every `[TYP-14]` read-through site (SoA's: `lib.rs:15109`, `29551`, `34412`, `50775-50833`), `==`, printing, rendering |
| MIR | a link-making builtin as `weakref(copy _1[0])`: no `Rvalue::Ref`, so no loan | `Rvalue::Ref` as today; the loan reads its destination's type (`place_ty` in `collect_loans`), so `Rvalue::Ref` needs no flag | an aggregate of the inner pointer |
| borrows.rs | links make no loans today: a new loan source, then the shared rules; `insert_shared_accesses` (`:520`) must skip it | a `SharedMut` loan permission and the path rule (steps 4-5) | the inner field's loan is an exclusive `ref mut`: special-cased, then the shared rules |
| regions, stores | not a view: `view_region_paths` (`regions.rs:2607`), `stores.rs`, `is_view` learn it | `Ref { .. }` is a view everywhere already | inner `ref mut T`: a view but the slot's address; inner `*mut T`: the handle but no view |
| backends, interp | a class target is free (a link to a class is the object's pointer); a struct target must be the place's address in C, LLVM and the interpreter; every count and drop site skips it | a class target is the object's handle: 3 places in each of C, LLVM, interpreter (step 6) | a proxy deref in each |
| a missed site does | counts or frees through a pointer into a list's buffer: memory corruption at run time (`TyKind::Link` is matched ~120 times: typeck 40, ember_types 19, codegen_c 17, drops 9, cycles 8, interp 7, LLVM 6, effects 4) | behaves as `ref mut`: stricter, except the named exclusivity sites (risks 1-5) | a compile error, or a field read as a struct |
| size | ~1,150 | ~1,050, ~250 of it mechanical | ~1,300 |

D, a flag on `Rvalue::Ref` with the type left `ref mut T`, cannot work alone: a field `target: sharedref mut
Player`, an `Array[sharedref mut Player]` and a parameter have no `Rvalue`; the type must say it.

**Recommendation: B.** It carries what the rules say the thing is: a view (regions, stores, `E3063`, `@view`, kept-view
inference, coroutines and the interpreter's pointer come free) with one pointer's layout (`[LT-30]`). The differences
are few and nameable (the loan, the path rule, the exclusivity facts, `Send`, `copy of`, the class handle). `TyKind::Ref`
is written 572 times; 350 say `..` and are untouched, ~200 do not (~100 constructors in `intern`/`find`, ~100 patterns
in 22 files, 130 in typeck's `lib.rs`), and `cargo check` stops at each, so every site that must choose is found by
the compiler, not by grep. A site that defaults wrongly is a `ref mut`: stricter than the spec, never unsafe, except
the noalias and `Send` sites, which step 2 closes before any `sharedref mut` value exists. A's two audits run against
each other (counting off, view-ness on) and its miss corrupts memory; C re-implements read-through and still cannot
hold the region and the handle at once.

## The order

Each step lands alone, sound at every landing (a stricter answer is allowed in between, a weaker one never); the
bank's files move to `tests/conformance/` as they pass; one quick check before each merge.

1. **The type** (~250 lines, mostly mechanical; no behaviour change). `ember_types/src/lib.rs`: `TyKind::Ref { mutable,
   shared, inner }` (`shared` only with `mutable`, a `debug_assert!` in `intern`); `render`/`canonical_name` print
   `sharedref mut T`; `codec.rs` carries it; `thread_properties` gives `(false, false)` (an arm before the `Ref | Span`
   or-pattern); `TypeTable::is_sharedref(ty)` and `object_link(ty) -> Option<ClassId>` (a `sharedref mut` of a class).
   Constructors get `shared: false`; each pattern without `..` gets `..`, or `shared: false` where it means "alone".
   Tests: none new; the quick check unchanged is the proof.
2. **The words, as an exclusive view** (~200). typeck: `resolve_type`, `type_from_expr` and `synth_link_expecting`
   send `SharedrefMut` to the `RefOf` path (HIR `ExprKind::Ref { mutable: true }` with the shared type; `views.rs`
   `ViewKind::Ref` gains `shared`); `sharedref mut dyn I` keeps `E0900`; `ref mut p` given to a `sharedref mut` place, or
   the reverse, is a mismatch whose help is the other word (Q6); `copy of` a value holding one makes one more holder
   (`[OWN-8]`; the `E2116` arms, `lib.rs:50844-50866`, skip `shared`); `holds_one_thread_owner` (`:45233`) so `E7015`
   names the link; `scoped_task_refused` refuses a capture holding one (`[THR-8]`; M9's `[THR-13]` lets locks,
   `Atomic` and reductions through). Risks 1-5's sites land here. Borrows see an exclusive loan
   (`Rvalue::Ref { mutable: true }`): a second holder is `E3022`, sound and stricter. Tests: a new run-pass (one holder
   changes the player through `w.target`; `copy of w`), M9 THR-15's reject where M9's channel check is in, GRM-43 and
   R7 `reject_sharedref_without_mut` still.
3. **`ref mut` fields alone** (~120). borrows.rs `check_point`: a mutable borrow meeting a live mutable loan a struct's
   `ref mut` field holds (`regions.holders`, the path from `view_region_paths`) gets `[LT-46]`'s `E3022` text (the
   field, its last use, the new holder, the fix in both types and both lines); a write through another name while a
   mutable loan is live is `E3022` (the `_ =>` arm, `[BRW-1]`: "`E3022` to change it"); `[BRW-1]`'s `E3021` text for a
   read ("`boss` is the only name for `enemies[0]` until line 22, ..."; left by A2-4b, in here because the file needs
   it); typeck: `c.at += 1` reads through a `ref mut` field (`[TYP-14]`). Tests: M5 BRW-1/reject_a_ref_mut_name_is_alone,
   LT-18/reject_a_view_field_is_an_ordinary_view; R7 reject_two_kept_ref_mut_fields_of_one_player.
4. **Paths through a holder** (~250), loans still exclusive. `[BRW-17]`/`[EXC-3]`'s rule: a place whose path crosses a
   `sharedref mut` deref meets every loan and access whose place reaches a value of the target's type (a prefix of that
   type, `players[0]`, or another holder's deref), the field paths after them compared as `overlaps` does
   (`[EXC-19]`), unless proved different (the types, a fresh object, `if a is not b:`): `may_reach` beside `overlaps`
   (`borrows.rs:5706`). At a call, a `ref mut` or `owns` argument and an argument holding a `sharedref mut` whose
   target the first's type may own (`stores.rs` `owns_viewable`) are `E3022` unless proved different (`[BRW-1]`: "a link
   of the same type included"; `[BRW-21]`'s finer check is M6a's summaries). Tests: new rejects with holders as
   parameters (a write through `w.target` while `v = ref p.hp` is used: `E3021`; `f(ref mut players, w)`: `E3022`).
5. **Several holders** (~150). `facts.rs` `AccessPermission::SharedMut`, made in `collect_loans` (`borrows.rs:4403`) when
   the borrow's destination `is_sharedref`, `is_mut()` false: it meets reads, writes, other holders and the owner's
   `ref mut` without conflict, and a reshape or a hand-over of what it points into ends it, `E3060` at the holder's
   later use (the read-only path), a write that replaces the viewed place itself included (`players[0] =
   Player(hp=1)`, `reshapes_at`'s `Access::Write` arm). Accesses through holders stay step 4's. Tests: M5
   BRW-17/reject_adding_ends_a_view_of_an_inline_element, reject_removing_clearing_or_replacing_ends_class_object_views;
   M6a EXC-17 (with its fix, below); R7 accept_sharedref_mut_holders_and_the_owner_all_change_one_player,
   reject_a_push_moves_the_inline_players_a_kept_link_points_at, reject_removing_a_player_a_kept_link_points_at,
   reject_a_view_into_a_part_used_after_another_holder_reshapes_it,
   reject_a_push_through_one_holder_while_a_loop_walks_through_another (`E3020`, `[BRW-18]` through any holder); R9
   run_two_sharedref_mut_holders_and_the_owner_change_one_player; `[LT-42]`: a function value capturing a holder
   keeps its regions (`E3026`), one new test.
6. **A view of a class object is the object** (~200). An object link holds the object's handle, not the slot's
   address (`[BRW-17]`: "Adding to a list moves only what it stores inline"). MIR unchanged; C: `c_type` (`lib.rs:12195`,
   `struct Player*`), `rvalue`'s Ref arm (`:11520`, the handle, not `&slot`), `place_in`'s Deref arm (`:11338`, nothing);
   LLVM: `rvalue` (`func.rs:1848`, a load), `place` (`:943`, no load); interpreter: `rvalue` (`lib.rs:1002`), `place`
   (`:543`). Borrows: an object link's loan is not ended by an add (`positions::written_only_adds`' calls; Q2);
   removing, clearing, replacing end it. A whole write of the object through a holder, or a `ref mut` made from one
   given to a callee that may reshape `*p` (`summary.may_reshape(param, [Deref])`), is refused (Q3). Tests: M5
   BRW-17/accept_adding_keeps_views_of_class_objects; R7 accept_a_push_keeps_a_kept_link_to_an_object_in_its_own_block,
   accept_removing_after_the_kept_links_last_use, accept_reading_again_after_another_holder_reshapes; each run-pass on
   the C backend with MSVC, Clang and GCC (WSL), the LLVM backend and the interpreter.
7. **Values that hold views** (~150). A store into an element of a list a local owns is bounded by that local
   (`[TYP-15]`; `stores.rs` `judge`, regions' unbounded stores), not static-only; the list, and all that holds it, carry
   the views (`[BCK-1]`); kept or returned past them is `E3063` with `[LT-45]`'s text (`ws` holds views of `players`...,
   its help built from the program's names and lines). A list a call made carries its arguments' views (`[LT-1]`; M5
   INDEX Q15 (a)). Tests: M5 BRW-12/accept_a_value_a_call_made_is_owned_by_the_name,
   BCK-4/reject_a_view_kept_or_returned_past_its_target (now at the `return`), R7
   reject_a_sharedref_mut_holder_returned_past_its_target (with the spec's text).
8. **Class view fields with `from`** (~250). Parser (`decls.rs` fields): `T from <field>`, `from static` (`[GRM-46]`);
   the ast field and typeck's `FieldDef` carry it (a `let` field index or static); `E2118` for a class's view field
   without a usable `from` (not `let`, a fixed array, an inline struct: `[LT-45]`'s texts), `E3063` for a `from static`
   field given other text; `from static` on a struct's field leaves the struct no view (`[TYP-34]`). Regions: a view
   stored there is checked against `this.<field>`'s contents or static, wherever stored. Tests: R7
   reject_a_sharedref_mut_field_of_a_class_without_from, new files for each `E2118` text and the `E3063`. The four
   files below are fixed here.
9. **`[LT-47]`, the callee side** (~100). `stores.rs` `caller_place_error`'s check accepts another view argument, a
   part of one and program text, and gives `[LT-47]`'s `E3063` text at the store of a callee's local or temporary.
   Tests: M6a LT-47/accept_the_views_a_callee_may_put_in, reject_a_view_of_the_callees_own_local_put_in_through_ref_mut;
   BCK-4's `put_a_local` keeps its `E3063`.

Left after A2-8: `sharedref mut dyn I` (`[TYP-45a]`); `sharedref mut this` pinning a class to one thread (`[OWN-13]`,
M9); scoped tasks through a holder (`[THR-13]`, M9); the instance link (`[LT-4]`, M10); `@cached` (`[EFF-27]`, M11b);
`[BRW-21]`'s per-callee check (M6a); `[HR-15c]` (M13).

## Risks, soundness first: where a shared mutable view could break an exclusivity assumption

1. `kernels.rs` moves a loop into a body whose list parameters are `restrict` (`restrict_views`; the `Ref` tests at
   `:238`, `:857-873`, `:987`); two holders of one list given as two arguments would miscompile. Skip `shared`.
2. `loop_version.rs` `locate` and `mutable_referents` (`:589`, `:698-712`, `:2141`, `:2320`, `:4383`) place a write
   through a reference at its origin and treat `Ref { mutable: true }` referents as apart; a versioned loop would drop
   its overlap test. A `sharedref mut` referent locates as unknown.
3. `positions.rs:343-346`: a list reached through a link gives addresses, but the test is `TyKind::Link`; a list under a
   `sharedref mut` parameter's deref would become a position re-found after an add another holder may undo. Add it.
4. SoA: `soa_proxy_of` builds `Ref { mutable, inner: soa }`; `sharedref mut ps[i]` must reach neither an exclusive
   `SoAMut` nor a pointer into a column a push moves. `E0900` until Q4 is answered.
5. `Send`/`ThreadShared`: `thread_properties` makes `ref mut T` `Send` when `T` is; `scoped_task_refused`, the
   `JobScopeSubmit` and `@parallel` environments (`borrows.rs:300` bounds their views by the scope only): a holder in
   a task while the owner writes is a data race. Step 2 refuses all three.
6. `[BRW-22]` noalias is not emitted for `ref mut` parameters today (LLVM `Ext::Ref` is `nonnull dereferenceable`,
   `func.rs:102`; C's `restrict` is kernels' only); whoever builds it tests `!shared`, and `this` stays out.
7. Step 5 without step 4 is unsound (a write through `w2.target` while `v = ref w1.target.name` is used, in a callee
   given both); and without step 4's call rule a callee's `ref mut p` (alone, later noalias) and `w.target` may be one
   place. Hence the order: the path rule lands while loans are still exclusive.
8. The handle (step 6): `bump(ref mut w.target)` whose body does `p = Player(hp=3)` lowers to `drop((*_1)); (*_1) =
   move _2` (probe), which would free the object `players[0]` still holds. Refused in step 6. The same shape through
   `exists as ref mut t` checks clean today (M4's; not run here): run it under the interpreter in step 6.
9. Drops and counts: none, since a `Ref` needs no drop; `insert_shared_accesses`' words on class fields reached through
   a holder are the class's own (M6a removes them), as for a `ref mut` now.
10. Step 4's type rule refuses more than a loan-by-loan answer would (`v = ref players[1].hp` while `w` holds
    `players[0]`): stricter, not unsafe, and what `[BRW-17]`'s "unless the compiler knows they differ" lists.

## Spec questions (readings and a recommendation; none decided)

- **Q1** A local `t = sharedref mut enemies[i]`: a position (`[BRW-16]`, "a name for a list element made by
  indexing") or an address (`[BRW-17]`, a link)? Recommend address: a holder is copied into others as one more holder,
  and `[BRW-17]`'s own example is the kept one.
- **Q2** Which list changes end a kept view of a class object? `[BRW-17]` keeps it across `push`/`extend` and ends it
  at "removing, clearing or replacing"; `insert`, `sort`, `swap`, `reserve`? (a) only what may remove or replace an
  element ends it; (b) only `push` and `extend` keep it. Recommend (a): "moves only what it stores inline".
- **Q3** `=` of a whole value through a holder (`*` of `w.target`, or a `ref mut` made from it): (a) refused for every
  target; (b) a struct target's place is written, a class target is refused (a holder changes the object, not which
  object its owner holds). Recommend (b).
- **Q4** `sharedref mut ps[i]` of an `SoA[T]` (and of a span's element): (a) a shared proxy (the `SoA` and the index);
  (b) refused. Recommend (a) with A2-7's proxies, `E0900` until then.
- **Q5** `[BRW-17]` keeps "a kept view" of a class object across an add, so a kept `ref Player` too; today it is the
  slot's address and an add ends it (safe). (a) every kept view of a class object holds the handle (all `ref C`, one
  type, one layout); (b) only links. Recommend (a), as its own item after A2-8: it changes `ref C` everywhere.
- **Q6** The code for `ref mut p` given to a `sharedref mut` place (or the reverse): no code is given. Recommend
  `E2020`, as `[TYP-45]` gives a link of one kind given to a place of another.

## Tests that disagree with the spec

M6a `EXC-17/reject_a_number_written_through_another_name_while_viewed`, M9
`THR-15/reject_a_channel_of_a_type_that_is_not_send`, M11b `EFF-27/accept_a_link_to_the_whole_holder_and_the_clears_it_needs` and
`EFF-27/reject_a_link_into_what_a_cached_method_reads` declare `class Watcher` with `target: sharedref mut ...` and no
`from`: `E2118` by `[TYP-34]` ("a class is never a view type"), `[TYP-15]` and `[LT-45]`, which R7's
`reject_a_sharedref_mut_field_of_a_class_without_from` also says. Tests follow the spec (owner, 2026-11-01): write
`struct Watcher` in all four, in step 8 (with `struct`, EXC-17's file's other lines stand as written).
