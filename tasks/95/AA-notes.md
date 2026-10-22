# Helper AA — language rules not yet done, 2026-10-22

Branch `helper-AA`, from `main` at `f00f2d61`. One commit per piece.

## Done (tests)

* **ODR-209 / D-852 `[GRM-17]`** — a lambda's `:` body on its own line is one simple statement
  that ends at the statement's newline: `h = fn(x): total += x` parses, and the newline is the
  enclosing statement's (`ember_parser`, `parse_lambda`).
  `tests/conformance/GRM-17/accept_a_same_line_lambda_body_ends_at_the_newline.em` (three
  profiles): an assignment, a `return` body, a call body, and one in a loop writing one capture
  while reading another (the defect below).

* **D-854 `[TYP-36]`** — `Array[T]` and `[T; N]` are `Ord` lexicographically when `T` is (the
  first elements that differ decide; a prefix is the smaller): `<`, `cmp`, `sort`, `sorted`,
  `min`/`max` and a `T: Ord` bound take them. `cmp` goes to std's `cmp_elements` over two views,
  as `==` goes to `eq_elements`. `TYP-36/accept_arrays_order_lexicographically.em` (three
  profiles; floats by totalOrder inside), `TYP-36/reject_an_array_whose_element_is_not_ord.em`.
* **D-860 `[MOD-5]`** — a program's own `struct int` (`float`, `str`) is what the name means in
  its types, constructions and constants (`int.ONE`); literals stay `i64`.
  `MOD-5/accept_a_programs_int_float_and_str_shadow_the_prelude.em` (three profiles).
* **ODR-211 / D-856 `[LT-1a]`, `[CTL-1]`** — `@borrows` may stand on a result some instance makes
  a view (`Option[I.Item]`) and name a parameter or a `self.f` field some instance makes a view
  (a field of a type parameter's type, an `owned self` of type `Self`); each instance applies it
  where those are views and borrows nothing from the rest, and its body is checked per instance.
  An interface method's `@borrows(self)` is resolved against its copies' parameter list (`self`
  first). std declares it: `@borrows(self.inner)` on `next`/`next_back` of `Take`, `Skip`,
  `StepBy`, `Enumerate`, `Copied`, `Cloned`, `Filter`, `TakeWhile`, `SkipWhile`, `Inspect`, `Rev`;
  `(self.inner, self.f)` on `Map`, `FilterMap`; `(self.a, self.b)` on `Zip`; `(self.first,
  self.other)` on `Chain`; `@borrows(self.view)` on `QueryIter.next` (`ecs.em`).
  * `CTL-1/accept_an_adapters_items_borrow_what_it_reads.em` (three profiles): the D-856 loop,
    items of a chain of adapters kept across turns and after the loop, a generic `keep_last`.
  * `CTL-1/reject_writing_a_map_while_an_adapters_kept_item_lives.em` (`E3021` ×2).
  * `LT-1a/reject_borrows_names_a_generic_field_but_an_instance_returns_another.em`: `E3062` in
    the `T = str` instance only.
  * `ECS-3/accept_a_query_item_is_kept_across_turns.em` (shown to fail without the attribute).
* **ODR-210 `[STD-19]`** — the consumers take their callable over (`[F: fn(...)] owned f: F`):
  `fold`, `any`, `all`, `find`, `position`, `max_by_key`, `min_by_key`, `max_by`, `min_by`,
  `reduce`. The ones giving an item (`find`, `max_by_key`, `min_by_key`, `max_by`, `min_by`)
  declare `@borrows(self)` (ODR-211), so an item does not keep the predicate's captures borrowed.
  `reduce` and `fold` do not: their result may be the callable's. `STD-19/accept_the_consumers_take_their_callable_over.em`
  (three profiles): each consumer with a lambda that counts its calls.
* **ODR-207 `[STD-26]`** — `sorted` of an iterator that lends references (`m.keys()`) copies or
  clones the items (`items_array`, as a collection's elements are), so `sorted(m.keys())` is an
  `Array[K]` whose items index the map; with `key=`/`reverse=` too.
  `STD-26/accept_sorted_of_an_iterator_of_references_copies_or_clones.em` (three profiles).
  **First week**: 05's `E2040` (N9, `sorted(self.items.keys())`) is gone; its marks re-recorded
  (codes `E1010 E2020`, the `R1:N9` step dropped) after replaying rounds 2–4 with this compiler,
  and `helped/05_inventory.em` keeps the idiom as written (`sorted(self.items.keys())`). The rate
  did not move (0 of 30, 3 of 30), so the README has no new line.
* **ODR-174 `[VER-3]`** — `@deprecated(since, note)` (both text, by position or by name; any
  other form `E0104`) on any item and on a type's or an extension's method; `W2192` "use of a
  deprecated item" at each use outside the item's module, naming `since` and `note`: a call, a
  function value, a type (plain, generic, through a module), a construction, a constant, a
  `static`, a type alias, an enum (its paths and patterns), an interface (an `implements`
  clause, a bound, `dyn`), a method or an associated function. Once per use (a generic body is
  checked per instance; a closure probe's discarded diagnostics do not count). Error page
  `docs/errors/W2192.md` (with a module beside it); `tools/error_pages.py` and
  `tools/rule_index.py` pass.
  * `VER-3/warning_a_use_of_a_deprecated_item_outside_its_module.em`,
    `VER-3/warning_a_deprecated_interface_static_alias_or_associated_function.em`,
    `VER-3/accept_a_deprecated_item_used_in_its_own_module.em`,
    `VER-3/reject_a_deprecated_attribute_without_its_version_and_note.em`; support module
    `VER-3/support/old.em`.
* **D-850 `[CLO-6]`** — only `once fn` is single-call: an `owned f: fn(A) -> R` parameter is
  called any number of times (a loop was `E3041`), and a closure that moves a capture out given
  to one is `E3030` at the argument, whose help names `once fn` (the instance is no longer made,
  so the store inside it no longer repeats the error, `[DIA-14]`). An explicit bound
  `[F: once fn(A) -> R]` is a `once fn` bound. Tests changed to the rule:
  `CLO-6/reject_owned_callable_parameter_cannot_be_called_twice.em` became
  `accept_an_owned_callable_parameter_is_called_any_number_of_times.em` (three profiles: twice,
  in a `for`, in a `while`); `accept_once_closure_satisfies_owned_callable_parameter.em` became
  `reject_a_once_closure_given_to_an_owned_fn_parameter.em` (and the same closure given to a
  `once fn` parameter compiles); `accept_owned_callable_parameter_is_consumed_once.em`'s comment.
* **`[CLO-6a]`** — an owned `once fn(A) -> R` value: a type (`TyKind::Fn { once: true }`, never
  `Copy`) anywhere a type is written outside a parameter. It holds a closure that moves a capture
  out (`owned fn`), any `fn` value or function (a `fn` is a `once fn`, not the other way:
  `E2020`), and is called wherever it is held: a local, a struct's field, a `Box`, an element its
  collection's API hands over (`pop_front`, `pop`, `Option.take`). The call moves it out of its
  place, so the move rules refuse a second call (`E3040`), a call in a loop (`E3041`), through a
  borrowed parameter (`E3013`), out of an element in place (`E3011`) or a class field
  (`E3012`); given to a `fn(A) -> R` parameter it is `E3030` at the argument. The C side: each
  callable descriptor gains a `once` entry (a closure taking its environment whole is given it
  and only its allocation is freed; any other is called and then dropped), a `once fn` call goes
  through it, and one in a `Box` through a helper that moves the value out and frees the box
  first. `CLO-6a/accept_an_owned_once_fn_value_is_called_and_emptied.em` (three profiles,
  shown in `release` too), `CLO-6a/reject_a_once_fn_value_called_twice_or_from_a_borrowed_place.em`,
  `CLO-6a/reject_a_once_fn_value_where_fn_is_wanted.em`. `phase-status.json`: the `CLO-6a`
  entry removed. For the lead: `tools/phase_progress.py --check` now differs from the README
  (not mine to edit).
* **`[GRM-28]` (part) and `[CLO-2]`** — a closure inside a local function may call it, from any
  depth: it captures the local function's environment (under a name no program can write) through
  every closure between; one that captures nothing is called directly. This needed a lambda
  inside a lambda to capture the enclosing function's locals, which it could not (`E1010`): the
  enclosing lambda now captures them for it, shared or mutable as the inner one uses them. While
  a local function's captures are being found, a call to it from a closure inside it is a
  placeholder never run (those probe closures are emitted and never called).
  `GRM-28/accept_a_closure_inside_a_local_function_calls_it.em`,
  `CLO-2/accept_a_lambda_inside_a_lambda_captures_the_functions_locals.em` (three profiles each);
  `GRM-28/reject_the_local_function_forms_not_built.em` keeps the generic case only.
  `phase-status.json`: `GRM-28` says what is built.
* **ODR-194 `[HR-20]`** — `Retained.pin(handle, on_relocate: Option[extern "C" fn(ObjectPtr[C],
  ObjectPtr[C])] = None)`. Given a callback, a class object is recorded with it in the runtime
  (`ember_obj_pin_relocatable`) instead of pinned in its header, so a reload may move it; right
  after commit publishes the moves and before reclaim, the runtime calls each recorded callback
  once with the old and the new address (`rl_relocated`, the record unlocked during the calls)
  and the record follows the object. The token's end (`Retained` now has a `drop` and a
  `relocation` number) takes its record out. A `Box`'s value never moves; its callback is never
  called. `ObjectPtr` got its `==` ([FFI-23] says equal for one object's address; it was
  `E2040`).
  * `compiler/ember_driver/tests/reload.rs`, `a_pinned_object_with_on_relocate_moves_and_is_told`
    (the edit `a_pinned_object_is_never_moved` refuses applies, two moves, two calls, old ≠ new);
    `HR-20/accept_a_token_with_on_relocate.em` (three profiles). `--test reload` 50/50; milestone
    `retained_tokens_let_c_keep_ember_objects` passes. `phase-status.json`: `HR-20` removed.
* **ODR-175 `[ALC-2]`, `[ALC-3]`** — `std.mem`: `pub struct Layout { size, align }`,
  `pub unsafe interface Allocator` (`allocate`, `deallocate`, `reallocate`, null on failure) and
  `Global`, the runtime's (`ember_try_alloc`, `ember_free`, a new `ember_try_realloc`). The grammar
  takes `unsafe interface`; implementing one is `unsafe`: a plain `extend`, a generic one or a
  type's header naming it is `E3100`, and `unsafe extend` writes a marker or an `unsafe interface`
  (`[TYP-43]` as amended; another interface is `E2040`). `[build] global_allocator =
  "module.NAME"` (a `[MAN-8]` `[build]` key now): the driver writes two `extern "C"` functions
  into that module, calling the static's `allocate`/`deallocate` through std's
  `global_allocate_through[A: Allocator]` (so a static of another type is `E2040`), and the entry
  point gives them to `ember_rt_init` as its allocation functions (`[FFI-27]`'s), so every
  allocation, the runtime's included, goes through the package's allocator. A library's manifest
  naming one, a module the package does not have, or a name that is not a static, is `E9001` at
  the key.
  * `ALC-2/accept_an_unsafe_extend_implements_allocator.em` (three profiles: `Global` and a
    counting allocator through one `A: Allocator` function),
    `ALC-2/reject_implementing_allocator_without_unsafe.em`;
    `ember_driver/tests/package.rs`, `a_binary_package_replaces_the_global_allocator` (a counting
    allocator over C's `malloc`: 50 strings, at least 50 blocks through it; the four refusals).
    `phase-status.json`: `ALC-3` removed.
* **`[JOB-5]`** — `jobs.local_arena() -> ScopedArena`: a scope of the running job's arena, one
  per thread (made on first use, `ember_job_local_arena`), which `job_run` rewinds to where it
  stood when the job began (a thread that helps while waiting runs nested jobs, each rewinding its
  own). The view is anchored at a hidden local of the calling function (declared where its body
  begins), so the region rules keep it and its allocations inside that call: returning one is
  `E3060` ("the job's arena view does not live long enough") with `E3061`, keeping one in a
  captured variable `E3063`; outside a job it panics. Effects `Alloc` and `Panic`.
  `JOB-5/accept_a_jobs_local_arena.em` (three profiles: four jobs, a helper taking a second view),
  `JOB-5/reject_an_allocation_escaping_the_call.em`, `JOB-5/run_fail_a_local_arena_outside_a_job.em`.
* **`[STD-14]`** (part) — `[STD-1]`'s contract (every function `@noalloc` unless its
  documentation says it allocates, `E4001`) now holds every module of `std`, not five: 128
  functions that allocated undocumented are documented, each saying what it allocates (an
  `override` inherits its base method's contract, `[EFF-8]`, so `Command.apply` says an override
  allocates). Of std's 55 panics that run: 7 were internal invariants (`Map`'s "a listed entry was
  removed") and are `unreachable()`; `Pool.insert`, `pool[h]` and `Pool.get_unchecked` now name
  their twins (`try_insert`, `get`); 9 are settled by a rule (ODR-089's adapters, `[ECS-4]`'s
  conflict, `ItemCount`'s overflow); 10 are a caller's bug with no twin in std yet (question 2),
  which keep `[STD-14]` in `phase-status.json`.
  `ember_driver/tests/milestones.rs`, `std_panics_name_their_twins_and_allocations_are_documented`
  (a program importing every std module checks clean; every panic names a twin std declares, or
  is listed settled or open, and every listed one still panics); `std_functions_are_noalloc_unless_documented`
  now expects `std.thread` held too.

## Added to the job by the coordinator (owner answers, binding)

* ODR-218 `[ECS-4]` — `world.run_parallel([…])` runs systems whose sets do not conflict at once,
  on threads; an owned callable value is `Send` when everything it captures is; a component used
  by `run_parallel` must be `Send`.
* ODR-236 — a comprehension may take its list over (`[s for s in owned shapes if …]`, moving the
  chosen items) or copy them (`s.clone()`); first-week 16 re-marked after.
* ODR-237 — `Box[T]` is `Clone` when `T` is (recursive types' derived clone a function);
  `interface Shape: Clone:` requires implementers be `Clone`, and `Box[dyn Shape]` is `Clone`
  through a copy entry in the method table. Not `Eq` for `Box`.

## Not done

## Compiler changes

* `ember_parser::parse_lambda`: the same-line `:` body is parsed as one simple statement without
  its newline.
* `ember_typeck` (assignment): on a closure's capture-discovery pass a captured assignment target
  is a placeholder, not a place; the right side is still read for the captures it names.

* `ember_typeck`: `implements` (`Ord` of `Array`/`[T; N]` by the element), `provides_cmp`,
  `synth_cmp_of` (sequences to `cmp_elements`). std: `extend[T: Ord] Span[T]: cmp_elements`.
* `ember_typeck`: `scalar_in_scope` (the scalar table unless the program declares a type of a
  shadowable prelude name), used by type positions, `T.CONST` and type names in expressions;
  `root_qualified` keys the root module's `int`, `float`, `str` as `root.<name>` (as D-305 did
  for `Box`), so `names_program_type` sees them.
* `ember_types::TypeTable::may_be_view`. `ember_typeck`: `check_borrows_attribute` (results and
  parameters some instance makes views; `borrows_in_copy` while an interface default is read for
  one type: keeps the named sources this type makes views), `receiver_view_field` (fields of a
  type parameter's type), `method_signature_inner` (an interface method's attribute against
  `self` first).
* `ember_diag`: `W2192`. `ember_typeck`: `Deprecation`, `read_deprecation`,
  `record_member_deprecations` (in `declare_names`), `note_item_use` / `warn_deprecated` at the
  resolution sites, `method_deprecation` read in `check_method_visible`; the attribute table marks
  `@deprecated` built (`@allow` stays `E0900`).
* `ember_typeck`: `CallableBound.once` from the written `once fn` and a new `owned` (the
  parameter's mode, which ODR-189's storing reads through `consumed_callable_parameter`);
  `is_owned_callable_param` (the consuming call) is `once fn` only;
  `reject_once_closures_for_callable_bounds` returns whether it refused, and the call is then
  not instantiated.
* `ember_types`: `TyKind::Fn.once` (display, canonical name and spelling `once fn`, unify).
  `ember_typeck`: `once fn` types outside parameters; coercions (`fn` → `once fn`, a once
  closure into a `once fn`, an owned `once fn` parameter only into a `once fn`); a `once fn`
  call consumes its callee (`synth_indirect_call`); `through_callable_ref` reads through a `Box`;
  `reject_once_closures_for_callable_bounds` refuses a `once fn` value at a `fn` bound.
  `ember_mir`: a consumed callee in a `Box` moves the box. `ember_codegen_c`: the descriptor's
  `once` entry (after `captures`, so the runtime's reload view of the prefix is unchanged), the
  `_once` thunks, `ArrayHelper::OnceBoxCall`.
* runtime (templates, regenerated): `ember_obj_pin_relocatable`, `ember_obj_unpin_relocatable`,
  `rl_relocated`. `ember_hir`: `Builtin::RetainedUnpin`; `RetainedPin` takes the callback and
  gives the record's number. std (`ffi.em`): `Retained.pin`'s `on_relocate`, `Retained.drop`,
  `ObjectPtr`'s `Eq`.
* `ember_ast`: `InterfaceDecl.is_unsafe`; `ember_parser` (`unsafe interface`); `ember_fmt`.
  `ember_typeck`: `unsafe_interfaces` (in `declare_names`), `collect_implements_in`
  (`written_unsafe`), the generic extension's check, `collect_markers`. `ember_hir`: `RuntimeFn`
  `GlobalAllocate`, `GlobalDeallocate`, `GlobalReallocate`. Runtime: `ember_try_realloc`.
  `ember_driver`: `inject_global_allocator`, the `[build]` key. `ember_branding`:
  `global_allocator_glue`. `ember_codegen_c`: `cfg.alloc`/`cfg.free` in the entry point.
* `W2192` moved to the `Types` range, as the lead asked (`numbers_fall_in_their_subsystem_range`).
* `[JOB-5]`: `ember_hir::Builtin::JobLocalArena`; `ember_typeck` `synth_jobs_local_arena`,
  `arena_anchors`/`declare_arena_anchors` (functions and lambdas); `ember_types::JOB_ARENA_ANCHOR`
  (the anchor's name, which diagnostics show as the call); `ember_analysis` (the builtin borrows its
  argument, as `ArenaScope`; `E3060`'s wording for the anchor; `stores.rs` names it); runtime
  `ember_job_local_arena` and the rewind in `job_run`; std `jobs.local_arena`.

* `[STD-14]`: `std_contracts` holds every `std` module (it held `core`, `mem`, `math`, `simd`,
  `arena`).

## Defects found

* **AA1 — fixed.** A block-bodied lambda that assigns a capture and reads another
  (`f = fn(x: int):` / `seen += k`) was `E1010` "cannot find `k`": on the discovery pass the
  target `seen` is a placeholder, the assignment stopped at "not a place" before reading the
  right side, so `k` was never recorded as a capture. Covered by the GRM-17 case above.
* **AA2 — fixed.** A lambda inside a lambda could not capture a local of the enclosing function
  (`g = fn(y) => x * y + k` inside `f`: "cannot find `k`").
* **AA3 — fixed.** A capture named `env` resolved to the closure's own environment parameter
  (`env = 5; f = fn() => env + 1` was "`+` cannot be applied to `lambda`").
* **AA4 — not fixed.** `map(fn(w: ref String) => w)` (no result type written) reads the reference
  through and moves the `String` out of the borrowed parameter (`E3013`); `-> ref String` written
  compiles. A lambda body's type with nothing expected should be the parameter's, `ref String`.
* **AA5 — fixed.** `ObjectPtr[C]` had no `==`, though `[FFI-23]` makes two equal when they are
  the address of one object.

## Questions

1. **What may a global allocator allocate from?** ODR-175 makes `Global` "the runtime's", and the
   runtime allocates through the package's global allocator once one is named, so an allocator
   that delegates to `Global()` (a counting or tracing one) recurses until the stack overflows.
   Built: `Global` is the runtime's as configured (Rust's `Global`), and the test's allocator gets
   its memory from C's `malloc`. Options: (a) as built, documented; (b) a `std.mem.System`, the
   platform's allocator the runtime uses by default, for a global allocator to wrap (Rust's
   `System`); (c) `Global` is always the platform's, and the configured allocator is reached only
   through the runtime's own allocations. **Recommend (b)**: a decorating allocator is the common
   case, and (c) would make `Array[T, Global]` bypass the package's allocator.
2. **The twins `[STD-14]` asks for that std does not have.** A caller's bug panics "and has a
   non-panicking twin"; these have none: `range(...).stepped(0)`, `BitSet.insert` of a negative,
   `Entity.new` out of range, `World.add` to a dead entity, `CppShared.get` of an empty pointer,
   `Mat2`/`Mat3`/`Mat4` `col`/`row` out of range, `Quat.normalize` and `Plane.normalize` of zero,
   `str.split` on an empty separator. `[ERR-13]` says "each panicking *accessor*", `[STD-14]` "a
   caller's bug", so whether a constructor or an argument check needs one is open too. Options:
   (a) a twin for each, named by std's habits (`try_col`/`try_row`, `try_get`, `try_new`,
   `normalize_or_identity` for `Quat`, `try_normalize` for `Plane`, `try_insert` for `BitSet`);
   (b) twins for the accessors only (`col`, `row`, `CppShared.get`), the argument checks settled
   as ODR-089 settled the adapters'. **Recommend (b)**, with (a)'s names for the three accessors.
