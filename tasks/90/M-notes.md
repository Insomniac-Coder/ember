# Helper M (90% push, 2026-10-19): ECS queries, systems and commands — notes

Job sheet: `[ECS-3]`..`[ECS-6]` (Part XII.3), with what they need: the type-checker defect that
stopped helper D's query (hidden projection parameters of a tuple from a generic tuple
extension), the component derive, and `E7020`. Branch `helper-M`. Each section below is one
feature, written like an ADR: what was built, the decisions and why, the tests, what is left.

## 1. Hidden parameters of a concrete instance from a generic tuple extension (D-407)

**Before.** `Query[(Mut[Pos], Vel)]` and `fn(mut Query[Q], A)` were each made as two instances
that differed only in a hidden projection parameter (`Q.State`), "expected `X`, found `X`"
(tasks/85/D-notes.md §8). Three separate causes, each fixed generally in `ember_typeck`:

1. **A signature read before the implementations.** `fn system(mut q: Query[(Mut[Pos], Vel)])`
   is read in `collect`, before `collect_methods` records `extend Pos implements Param` and before
   any tuple has taken `extend[A: P, B: P] (A, B) implements P`; the instance's `Q.State` was
   `<error>` and it was named by that. Now a generic struct's *concrete* instance (written
   arguments free of parameters) is one instance per written arguments (`concrete_instances`):
   its hidden arguments are what those arguments' types say. One made before the implementations
   are known whose hidden arguments cannot be read yet is registered and named, with no fields,
   and filled in once they are (`early_instances`, `complete_early_instances`, right after
   `collect_methods`); its name is updated to the one its hidden arguments give.
2. **A tuple takes its extensions only at a first method call.** `project` now gives a built-in
   generic instance (a tuple, an `Option`, an `Array`…) the extensions it lacks when a projection
   of it is asked and the implementations are known, and completes an early instance first
   (`project` wraps the old `project_now`). Narrowed to built-in instances on purpose: applying a
   generic struct's extensions from inside `project` broke `Map`'s blanket `Index[Q]` methods.
3. **Substitution forgot hidden arguments.** `map_type_children` rebuilt an instance from its
   written arguments only (`StructDef::origin`), so `make[Q]()`'s `Query[Q, make's Q.State]` kept
   `make`'s own hidden parameter inside `run`. Each instance's hidden arguments are now kept
   (`struct_hidden`) and substituted with the written ones; a hidden argument the map leaves alone
   is recomputed from the new written arguments, as before.

Test: `tests/run-pass/a_tuple_instance_takes_its_hidden_parameters_from_a_tuple_extension.em`
(helper D's shape; each of the three made it fail).

## 2. Two more type-checker fixes the query needed

- **A chained projection in a generic struct (`Q.Ops.View`)** was `E2040` "its bounds declare
  none": generic structs are collected before interfaces, so `Q.Ops`'s bounds were still empty.
  Such a chain now waits for its base, as the base waits for its interface
  (`close_projection_bounds` closes them in order); after interfaces are collected
  (`interfaces_collected`) an empty base is an error as before.
- **`is_generic` (ember_types) looked only at fields.** An instance over a parameter whose
  parameter no field shows was taken as concrete: a phantom one (`Not[T]`, `[TYP-35]`), and
  `SoA[T]`, whose columns are known only per instance. So `struct W[T]: values: ref SoA[T]` was
  checked against the unsubstituted `ref SoA[T]` ("expected `ref SoA[T]`, found
  `ref SoA[Vel]`", a defect older than this work), and `Not[T]`'s `Ops` was asked to implement
  `ParamOps` with `T` open. A struct, enum or class instance is now generic when an argument of its
  `origin` is.

## 3. The query (`[ECS-3]`)

**Design.** `std/src/ecs.em`. Everything is ordinary Ember; the only compiler pieces are the
derive (which already existed for `[ECS-7]`) and §6.

- `Mut[T]` and `Not[T]` are `@derive(Copy)` structs with a phantom parameter (`[TYP-35]`).
- **Two interfaces.** `QueryParam` is what a query's parameter *is* as written; its one associated
  type `Ops: ParamOps` names the library type that does the work. `ParamOps` has the associated
  types `Item` (what `iter_mut` yields), `ReadItem` (what `iter` yields), `Fetch` (the storages
  held for a run), `View` (their contents borrowed), `Reads`, `Writes` (§5), and `fetch`, `view`,
  `driver`, `get`, `get_mut`, `accesses`. Why two: a component `T` must be a parameter itself, and
  a blanket `extend[T: Component] T implements QueryParam` overlaps `Option[P]`'s, `Mut[T]`'s and
  the tuples' (`E2041`, no negative reasoning), so the derive writes one per component; the split
  keeps what the derive writes to one line (`type Ops = std.ecs.Read[T]`) and every piece of logic
  in the library.
- **Implementations.** A component `T`: `Read[T]` (`ref T`) or, for `@component(layout=soa)`,
  `ReadSoa[T]` (`SoARef[T]`, the `[SOA-6]` proxy, per the owner's answer). `Mut[T]`: `T.Write`,
  which the derive also writes into `Component` (`Write[T]`: `ref mut T` from `iter_mut`, `ref T`
  from `iter`; `WriteSoa[T]`: `SoAMut[T]`/`SoARef[T]`). `Option[P]`: `Maybe[P.Ops]`, every entity
  matches, `Option` of `P`'s item. `Not[T]`: `Without[T]`, entities with a `T` do not match, item
  `()`. `Entity`: the entity itself, matching every entity (needed to name an entity to
  `q.commands().destroy(e)`; not in the rule's list, an addition of the library). Tuples of two to
  eight parameters (generic tuple extensions, written by a generator kept in the session scratchpad;
  one block per arity).
- **Not's item is `()`.** "the library maps each parameter to its item type (`ref T`, `ref mut T`,
  `Option[ref T]`, nothing)": an associated type per parameter cannot remove a tuple slot, so
  "nothing" is the unit type and `for a, b, c, _ in q.iter()` names it. Owner question 3.
- **Smallest storage drives.** `driver` gives each required parameter's dense entity list (`None`
  for `Option`, `Not`, `Entity`); a tuple keeps the shortest (`shorter`). The iterator walks it and
  `get`s each parameter for the entity (one sparse probe each: the sparse slot and the dense
  entity compared, as EnTT's `contains` does). A query with no required parameter panics when
  iterated (owner question 4).
- **`iter` / `iter_mut`.** `iter(self)` borrows the query shared and yields `ReadItem`s;
  `iter_mut(mut self)` yields `Item`s and needs `mut q`; both are `Iterator`s.
- **How the storages are held.** `World.run` takes each storage's class handle (`fetch`, making an
  empty storage if no entity has had the component), then `view` borrows its `entities`, `sparse`
  and `dense` fields through a `ref` to that handle: a span of each, `MutSpan` for a written
  component. By `[EXC-18]` each is a dynamic access held by the view until its last use: so the
  run holds exactly the storages its query names, writes for `Mut` and reads for the rest, with one
  check each per run and none per item (`[EXC-14]`). `view` takes the handle by `ref`: a class
  handle passed borrowed is "passed as a copy", and a view of its fields could not be returned.
- **Parameter passing in `ParamOps`.** `get` takes its view by `ref` (by value, an `Option`
  parameter's item borrowed the copy of its view: defect 4 below); `driver` by value and `get_mut`
  by `mut` place (by `ref` they tied the driving list, and `mut` refs needed a place).
- **Systems take `mut q`.** `run`'s parameter is `fn(mut Query[Q], A)`, and a callable's parameter
  modes are part of its type (`[FN-6a]`), so a read-only system is written `mut q` too, as the
  spec's own `integrate` is. A function taking `q: Query[…]` borrowed is still a query user, with
  `iter` only.
- **`@noalloc`.** The worked example's `integrate` is `@noalloc` and passes: nothing in the
  iteration allocates (`Commands` does, and `E4001` reports a `@noalloc` system that uses it).

**Tests** (`tests/conformance/ECS-3/`): `accept_the_worked_example_runs_as_written` (Part XII.3's
example verbatim, `@noalloc`), `accept_option_and_not_filter_and_items_follow_each_parameter` (the
rule's `(A, Mut[B], Option[C], Not[D])` shape), `accept_the_smallest_required_storage_drives_the_query`,
`accept_an_soa_component_yields_element_proxies`, `reject_writing_through_an_item_iter_yields`,
`reject_writing_a_component_the_query_reads`, `reject_iter_mut_needs_a_mut_query`.

**Cost.** Per entity: one sparse probe per parameter that has a storage (two loads, a compare),
the iterator's bounds check on the driving list and its overflow-checked counter. Not measured
against C++ (an EnTT-style view): the house rules forbid running programs outside the harness;
the benchmark is for the coordinator. Possible later gains: skip the driving parameter's own probe
(its slot is the index), count with a range instead of a checked `+= 1`.

## 4. The derive (`[ECS-7]` machinery, for `[ECS-3]`)

`@derive(Component)` (`derives.rs`, `component`) now writes, beside the name, hash and layout,
`type Read = std.ecs.Read[T]` and `type Write = std.ecs.Write[T]` (`ReadSoa`/`WriteSoa` for
`layout=soa`) in `extend T implements Component`, and a second block
`extend T implements std.ecs.QueryParam: type Ops = std.ecs.Read[T]`.

**Defect fixed with it: two programs with different components in one folder stopped with an
`[LT-40]` internal error** ("cached callable-region metadata disagrees with the current verified
summary for `std.ecs`"). `std.ecs.components()` is written from each program's components
(`[ECS-7]`), but `std.ecs`'s interface-cache key was its source text alone. The key now takes the
program's component registry (`ember_typeck::component_registry_key`, used by the driver's
`module_interface_inputs`). Test: `programs_with_different_components_share_a_folder` in
`compiler/ember_driver/tests/milestones.rs` (it failed before). Probe: check two programs, one
with `@derive(Component) struct A` and one with `A` and `B`, in one folder.

## 5. Systems: `run`, read and write sets, `run_parallel`, `E7020` (`[ECS-4]`)

- **Read and write sets** are associated types of `ParamOps`: `Reads` and `Writes`, a component
  or a tuple of them (`()` for none): `Read[T]` reads `T`, `Write[T]` writes `T`, `Maybe[O]` is
  `O`'s, `Not` and `Entity` touch nothing, a tuple is its elements'. So they are compile-time
  constants of the query's type, computed by the library. `accesses(out)` gives the same at run time
  (each component's hash and name, and whether it is written).
- **`world.run(system, arg)`** fetches and views `Q`'s storages, runs the system with its query
  and `arg`, and applies the query's `Commands`. It takes exactly one argument after the system
  (the spec's own `world.run(integrate, 0.016)`): a tuple for several, `()` for none (owner
  question 2). The run borrows exactly the storages its query names (§3): a system given the world
  as its argument may add a component its query does not name, and writing one it holds panics.
- **`world.run_parallel(systems: Array[System])`** gets each system's accesses, and before any runs
  compares each pair: one writing a component the other reads or writes panics, naming both
  systems and the component (`systems `a` and `b` cannot run at once: `a` writes `Pos`, which the
  other uses`). Then it runs them **one after another** (owner question 1).
- **`System`** is a system as a value: a struct of its `name` and two owned callable values (its
  accesses and its run), so systems over different queries are one type. `std.ecs.system(f, "f")`
  makes one. A generic subclass per query was tried first: a generic class naming `Query[Q]`
  (which has hidden parameters) made two instances again (classes get no hidden parameters), and a
  callable parameter cannot be stored in a field (it is a bound, `[CLO-3]`); an owned lambda
  stored through a struct literal can.
- **The compiler's one piece of ECS support** (`ecs_system_of`, `check_system_conflicts` in
  `ember_typeck`): where a `std.ecs.System` is wanted, a function over a `Query` written by name
  is the system `std.ecs.system(f, "f")`, named as written. That is what lets the spec's literal
  `[s1, s2, …]` (functions of different types) type-check at all, and an `Array[System]` be written
  `[movement, regen]`. In `world.run_parallel`'s own array literal, the systems' sets (the
  library's `Ops.Reads`/`Ops.Writes`, read by `project`, tuples flattened) are compared: a
  conflict is `E7020`, at the later system, naming both and the component, with the first marked.
  Part XII.3 says the ECS "has no compiler support of its own"; `E7020` cannot come from a library,
  so this is owner question 5. `docs/errors/E7020.md` is new.
- **Workaround in the library**: `system` calls `accesses_of[Q](out)` rather than
  `Q.Ops.accesses(out)` inside its lambda (defect 2 below).

**Tests** (`tests/conformance/ECS-4/`): `accept_a_run_borrows_only_the_storages_its_query_names`,
`run_fail_a_system_writes_a_storage_its_query_holds`,
`accept_run_parallel_runs_systems_whose_sets_do_not_conflict`,
`reject_systems_that_conflict_in_one_run_parallel` (`E7020`, two conflicts),
`run_fail_systems_passed_as_values_conflict`; and the worked example.

**Left (`docs/phase-status.json` lists `ECS-4` unfinished):** the systems of `run_parallel` do not
run at once, and `run` takes one argument, not `args…`. Why parallel running is not a small step:
the world's storages are classes, neither `Send` nor `Sync` (`[THR-1]`), a system value is an
owned callable value (not `Send`, ODR-102), and two readers of one storage would update its
non-atomic access words from two threads. A design that works within the rules, for the owner to
choose (question 1): create every system's views on the calling thread (their dynamic accesses
begin and end there), and run the bodies as jobs of one `jobs.scope()` borrowing those views,
which are `Send` when the components are; this needs the literal's systems kept unerased (the
compiler would call a generic `run_parallel_N[Q1, …, QN](f1, …, fN)`), `Commands` made `Send`
(a `@sync` buffer behind a `Mutex`), and for systems passed as values either nested scopes (each
system's frame opening a scope around the next) or running them in turn.

## 6. Commands (`[ECS-5]`)

`Commands` is a class holding the changes in order (`Array[Command]`; `Command` is an open class
with `DestroyCommand`, `AddCommand[T]`, `RemoveCommand[T]`, each applying itself). A query holds
one; `q.commands()` gives the handle (`destroy(e)`, `add(e, value)`, `remove[T](e)`, `len()`), and
`World.run` applies them in order after the system returns (`World.apply`). Adding to an entity
an earlier command destroyed panics, as `World.add` does for a dead entity.

`q.commands()` works inside `for … in q.iter()`; inside `iter_mut` the loop borrows the whole
query mutably (a method of a type of another package borrows all of `self`, `[BRW-10]`), so a
system takes `cmds = q.commands()` before that loop. Changing the world directly while a query
iterates (the world passed as the argument) panics with the exclusivity violation of the storage
the query holds: that is what the buffer avoids.

**Tests:** `ECS-5/accept_commands_apply_after_the_system_returns` (destroy, add, remove from
`iter`; remove from `iter_mut` through a buffer taken first; nothing changes while the system
runs), `ECS-5/run_fail_destroying_through_the_world_while_a_query_iterates`.

## 7. Iteration order (`[ECS-6]`)

Built by the storage (insertion order, swap-remove holes) and the query walking its driving
storage in dense order. **Test:** `ECS-6/accept_iteration_order_is_insertion_order_with_swap_remove_holes`
(removals, an add at the end, the same operations in a second world, destroys).

## Rules now done (each with its test)

| Rule | Status | Tests |
|---|---|---|
| `[ECS-3]` | done | `tests/conformance/ECS-3/` (7 files) |
| `[ECS-4]` | tested, **unfinished** (parallel running, `args…`) | `tests/conformance/ECS-4/` (5 files), the worked example |
| `[ECS-5]` | done | `tests/conformance/ECS-5/` (2 files) |
| `[ECS-6]` | done | `tests/conformance/ECS-6/` (1 file) |

`python tools/phase_progress.py`: phase 6 48/56 rules, 3425/3707 words (92%); `ECS-3`, `ECS-5`,
`ECS-6` (133 words) counted, `ECS-4` (84 words) not.

## Defects found (not numbered; the coordinator numbers them)

Fixed (sections 1, 2, 4): the three D-407 causes; the chained projection in a generic struct;
`is_generic` ignoring an instance's arguments (phantom parameters, `SoA[T]`); `std.ecs`'s
interface-cache key.

Not fixed (outside the job; each has a probe):

1. **An interface method whose signature names a `Map` instance breaks `Map`'s own generic
   methods.** Probe: `interface Ops: type Fetch; fn fetch(mut storages: Map[u64, int]) -> Fetch`,
   one implementation, an empty `main` — five `E2040` "`Q` does not implement
   `std.collections.AsKey[Q]`" inside `std/src/collections.em` (`get_or`, `index`, `index_mut`,
   `index_set`, `contains_key`…). Present before this branch. `std.ecs` passes the `World` instead.
2. **A lambda in a generic function calling an associated function through a projection stops
   the compiler.** Probe: `fn make[P: HasOp]() -> Holder: return Holder(f=owned fn(x: int) =>
   P.O.apply(x))` with `interface HasOp: type O: Op`: `panicked at ember_hir/src/lib.rs:63: every
   DefId names a function that was checked` (MIR lowering). The same call in the generic function
   itself, or through a generic helper, works. `std.ecs.system` uses a helper.
3. **Two generic structs that name each other are `E1010`.** Probe: `struct A[T]: n: T; fn b(mut
   self) -> B[T]: return B[T](a=ref mut self)` and `struct B[T]: a: ref mut A[T]`: "cannot find
   type `A`" on `B`'s field (collecting `A` collects `B` on demand from `A`'s signature, and `B`'s
   field then names `A`, which is half collected). Extensions are the workaround (std writes its
   iterators' constructors in `extend` blocks).
4. **A generic method's view parameter whose type is a generic struct over an opaque associated
   type is a place, not a view, in its instances.** Probe: `struct Maybe[O: Ops]: inner: O.View`
   with `extend[O: Ops] Maybe[O] implements Ops: fn get(view: Maybe[O]) -> Option[ref int]:
   return O.get(view.inner)`, and `fn first(m: Maybe[Col]) -> Option[ref int]: return
   Maybe[Col].get(m)` with `Col` holding a `Span[int]`: `E3060` "`m` is a view passed as a copy; a
   reference to the view itself points into this frame". `Maybe[Col]` is a view, so by `[LT-1]`
   (ODR-048) `get`'s result may borrow what it views. `std.ecs` passes such views by `ref`.

## Owner questions

1. **`run_parallel` runs its systems one after another.** `[ECS-4]`: "`world.run_parallel([s1, s2,
   …])` runs systems at once when their sets do not conflict". Running them on threads needs the
   design in §5 (views made on the calling thread, bodies as jobs of one scope; `Commands` made
   `Send`; the literal's systems kept unerased by the compiler; for systems passed as values, nested
   scopes or running in turn). Options: (a) build that design; (b) rule that running in turn
   meets "at once" (no program can tell the difference, as the sets do not conflict), so `ECS-4`
   is done as built; (c) parallel for literals only, values in turn. Built meanwhile: in turn.
2. **`run(system, args…)` without variadic parameters.** Built: one argument after the system
   (`world.run(integrate, 0.016)` as the spec writes it); several are a tuple
   (`fn step(mut q: Query[…], ctx: (f32, int))`), none is `()` (`fn tick(mut q: Query[…], _: ())`,
   `world.run(tick, ())`). Options: (a) keep that and write it in `[ECS-4]`; (b) a separate
   `run0(system)` for none; (c) a language feature (variadic generic callables). A system for
   `run_parallel` takes only its query, which matches nothing `run` takes: with (a), such a
   system runs alone as `world.run_parallel([s])`.
3. **What `Not[D]` yields.** The rule maps it to "nothing"; built as `()`, so the item tuple has a
   `()` slot (`for a, b, c, _ in …`). Dropping the slot would need type-level computation the
   language does not have. Confirm `()`.
4. **A query with no required parameter** (`Query[(Entity, Option[A])]`): the rule drives by "the
   smallest required storage" and there is none. Built: iterating it panics ("a query requires at
   least one component"). Options: (a) that; (b) iterate every live entity (the world would keep
   its live list); (c) a compile-time error (a code would be needed).
5. **"It has no compiler support of its own" vs `E7020`.** Part XII.3 says the ECS is an ordinary
   library, but `[ECS-4]` makes a conflict in `run_parallel([s1, s2])` a compile-time error and
   the literal holds functions of different types. Built: the compiler turns a function over a
   `Query`, written by name where a `std.ecs.System` is wanted, into `std.ecs.system(f, "f")`, and
   compares the systems of `run_parallel`'s own literal using the library's `Ops.Reads`/`Ops.Writes`
   (`E7020`). Options: (a) keep it and say so in Part XII.3; (b) a general mechanism instead (for
   example, a library declaring that a function item converts to one of its types); (c) drop
   `E7020` and the literal form, systems written `system(f, "f")` and checked at run time only.
6. **`Entity` as a query parameter** (yields the entity; not in `[ECS-3]`'s list) and **`Option[P]`
   over any parameter** (`Option[Mut[T]]` yields `Option[ref mut T]`): additions the library makes
   so that `q.commands().destroy(e)` has an `e`. Confirm, or name another way to get the entity.
