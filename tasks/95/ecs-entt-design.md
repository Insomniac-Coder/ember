# `std.ecs` as EnTT (ODR-279) — the design

The owner (2026-10-23): the ECS is to be like EnTT; it is replicated, not redesigned, so it needs no
approval. This file fixes the mapping before it is built and records each place Ember cannot copy
EnTT word for word, with the reason. EnTT here means EnTT 3.x (`entt::registry`, `entt::basic_view`,
`entt::basic_group`, `entt::sigh`/`sink`, `entt::snapshot`, `entt::handle`, `entt::runtime_view`).

## What stays (already EnTT's)

- `Entity`: 32 bits, a 20-bit index and a 12-bit version (EnTT's default `entt::entity`), `Entity.NULL`
  all ones (`entt::null`). `[ECS-1]` is unchanged.
- One sparse set per component type: a dense array of values, the entities in dense order, a sparse
  index by entity (EnTT's `basic_sparse_set` + `basic_storage`). Removal is swap-and-pop.

## The mapping

| EnTT | Ember | Notes |
|---|---|---|
| `entt::registry` | `Registry` (class) | replaces `World` |
| `create()`, `create(hint)` | `create()`, `create_at(hint)` | |
| `destroy(e)` | `destroy(e) -> u32` | the entity's next version, as EnTT returns it |
| `valid(e)`, `current(e)` | `valid(e)`, `current(e)` | |
| `clear()`, `clear<T>()` | `clear()`, `clear_of[T]()` | no overloading in Ember |
| `emplace<T>(e, args…)` | `emplace(e, value) -> ref mut T` | the value is built by the caller and moved in |
| `emplace_or_replace<T>` | `emplace_or_replace(e, value) -> ref mut T` | |
| `replace<T>(e, args…)` | `replace(e, value) -> ref mut T` | panics if `e` has no `T` |
| `patch<T>(e, f…)` | `patch[T](e, f: fn(mut T)) -> ref mut T` | `on_update` fires |
| `insert<T>(first, last, v)` | `insert(entities: Span[Entity], value: T)` | `T: Clone` |
| `remove<T…>(e)` | `remove[T](e) -> int` | one type a call; 0 when absent |
| `erase<T…>(e)` | `erase[T](e)` | panics when absent |
| `get<T>(e)` | `get[T](e) -> ref T`, `get_mut[T](e) -> ref mut T` | Ember says which access it wants |
| `try_get<T>(e)` | `try_get[T](e) -> Option[ref T]`, `try_get_mut` | |
| `get_or_emplace<T>(e, args…)` | `get_or_emplace(e, value) -> ref mut T` | |
| `all_of<T…>(e)`, `any_of<T…>(e)` | `all_of[(A, B)](e)`, `any_of[(A, B)](e)` | a tuple of types: no variadic generics (`[TYP-19]`) |
| `orphan(e)` | `orphan(e)` | |
| `storage<T>()` | `storage[T]() -> Storage[T]` | iterable: `for e, value in reg.storage[T]()` |
| `view<A, B>()` | `view[(A, B)]()` (read) / `view_mut[(A, B)]()` (write) | see below |
| `view<A>(exclude<C>)` | `view[(A,)]().exclude[(C,)]()` | |
| `view.each(f)`, `for (auto [e, a, b] : view.each())` | `view.each(f)`, `for e, a, b in view` | |
| `view.get<T>(e)`, `view.contains(e)`, `view.size_hint()`, `front()`, `back()` | the same names | |
| `group<A, B>(get<C>, exclude<D>)` | `group[(A, B)]()` … | owning groups: the owned pools packed in one order |
| `sort<T>(cmp)`, `sort<To, From>()` | `sort[T](less)`, `sort_as[To, From]()` | |
| `on_construct<T>()`, `on_update<T>()`, `on_destroy<T>()` | the same names, `.connect(f)`, `.disconnect(f)` | `f: fn(mut Registry, Entity)` |
| `ctx()` | `ctx()`: `emplace`, `get`, `get_mut`, `find`, `contains`, `erase` by type | |
| `entt::snapshot`, `snapshot_loader`, `continuous_loader` | `Snapshot`, `SnapshotLoader`, `ContinuousLoader` | archives are generic parameters |
| `entt::handle` | `Handle` | if a struct holding `ref mut Registry` is expressible; else methods on `Registry` |
| `entt::runtime_view` | `RuntimeView` | over type identities chosen at run time |
| no scheduler | none | `World.run`, `run_parallel`, `Query`, `Commands`, `System` go |

## Where Ember cannot copy EnTT word for word, and why

1. **Variadic type lists** (`view<A, B>`, `all_of<A, B>`): Ember has no variadic generics and no
   overloading (`[TYP-19]`), so several types are one tuple type, `view[(A, B)]`; the library
   implements its view interface for tuples of 1 to 8 elements with a blanket implementation over
   the element types (`[GRM-34]`), which is what lets **any type be a component** with no
   registration, as in EnTT.
2. **Per-type `const`** (`view<A, const B>`): Ember borrows the whole view either shared (`view`, every
   item `ref`) or mutably (`view_mut`, every item `ref mut`); the items of one entity are different
   types, so they never alias. EnTT's per-type `const` only documents intent there.
3. **Changing the registry while iterating** (EnTT allows removing the current entity's components
   while a view runs, and warns that adding to an iterated pool may invalidate it): a view borrows
   the registry, so the borrow rules refuse any structural change inside the loop, and a component
   reference never outlives a change that could move it (`[OWN-*]`). The pattern is to collect the
   entities, then change them. Values may be changed in place through `view_mut`.
4. **Paged storage** (EnTT pages its pools so a component's address survives insertion, because C++
   code keeps pointers): Ember code cannot keep a reference across an insertion, so a pool is one
   contiguous array, which iterates at least as fast.
5. **Signals' reentrancy**: a listener gets the registry mutably, as in EnTT; the library calls it
   only when it holds no reference into a pool, so a listener that changes the registry is memory
   safe.

## What goes

`World`, `Query`, `Mut`, `Not`, `QueryParam`, `ParamOps`, `Commands`, `System`, `World.run`,
`World.run_parallel`, the access sets and `E7020`, `@derive(Component)` (any type is a component),
the component registry `components()` (EnTT's equivalent is `entt::meta`, not part of this job), and
`@component(layout=soa)`: EnTT has no SoA pools; Ember's `SoA` stays a type of its own, and an SoA
pool can be added later as an extension.

## Order of work

1. `Registry`, `Storage[T]`, entities, `emplace`/`get`/`try_get`/`patch`/`replace`/`remove`/`erase`/
   `all_of`/`any_of`/`orphan`/`clear`, `storage[T]()`; views with `exclude`, `each`, `get`,
   `contains`, `size_hint`; the tests.
2. Signals, `sort`, `sort_as`, `ctx`.
3. Groups, snapshots, `handle`, runtime views.
4. The compiler's ECS support that no longer has a user (ODR-141's `System` coercion, `E7020`,
   `@derive(Component)`), removed once helper AJ's type-checker work is merged; the spec's XII.3
   rewritten at the next cut.
5. Measured against EnTT itself (speed and memory, the same program in C++ with EnTT), which needs
   EnTT's headers on this machine.
