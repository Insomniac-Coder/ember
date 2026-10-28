---

# Part XII — Data-Oriented Programming

Data-oriented code keeps each field of many objects in its own contiguous array, walks those arrays
in tight loops, and lets the machine's vector units do several elements per instruction. Ember makes
that layout a one-word change and makes the loops provably free of aliasing, so they run at C speed
with the safety checks either proved away or hoisted out.

## XII.1 `SoA[T]`

```ember
from std.math import Vec3

struct Particle:
    pos: Vec3
    vel: Vec3
    life: f32

fn integrate(ref mut pos: MutSpan[Vec3], vel: Span[Vec3], dt: f32):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    ps = SoA[Particle].with_capacity(100_000)
    ps.push(Particle(pos=Vec3.ZERO, vel=Vec3(1.0, 0.0, 0.0), life=2.0))
    integrate(ref mut ps.pos, ps.vel, 0.016)  # two columns at once: never one place
    ps[0].life -= 0.016                     # writes one element of the `life` column
    p = ps.get(0)                           # Option[SoARef[Particle]]: names particle 0, no copy
    snapshot = copy of ps[0]                # a Particle of your own, gathered from the columns
```

* `[SOA-1]` *(changed in 0.9.9)* `SoA[T]` is a **compiler-known type constructor**, like `Cell`: for
  any struct `T` it is a growable container holding one column per field of `T`. No derive is needed.
  `T` MUST be a struct (`E2071` otherwise). A field marked `@soa(flatten)` whose type is a struct is
  split into one column per field of its own (`ps.pos.x`); otherwise a struct field is one column.
* `[SOA-2]` *(changed in 0.9.15)* `ps.f` names the column of field `f`: a view of that column's elements, a `MutSpan[F]`,
  which supports everything a `MutSpan` supports and converts to `Span[F]` where one is expected
  (`[SPN-3]`). Like every `MutSpan`, it writes the elements only through a name or argument that says
  `ref mut` (`d = ref mut ps.pos`; `integrate(ref mut ps.pos, ps.vel, dt)`), and only where `ps` itself
  is reached through a name that may change it (`E3027` otherwise); a plain name holding a column only
  reads it (`E3023`). Different columns
  are different places, as different fields are (`[BRW-4]`), so any number of them may be named or
  given to one call at once, changeable or not. A length-changing operation (`push`, `pop`,
  `swap_remove`, `retain`, `clear`) is a reshape of all of `ps`: it may move the block every column
  lives in, so it ends every view of a column (`E3060` at a later use), and it may not run while a loop
  walks `ps` or one of its columns (`E3020`).
* `[SOA-6]` *(new in 0.9.9)* *(changed in 0.9.15)* `ps[i]` is an **element proxy**: `SoARef[T]`, a read-only name of element
  `i`, and `ref mut ps[i]` an `SoAMut[T]`, a name of it that may change it. A proxy is a position: it
  finds element `i` again at each use, so adding to `ps` keeps it, and a removal, reorder or clear ends
  it (`E3060` at a later use). A proxy's field `ps[i].f` is a place in column `f` at index `i`; reading
  or writing it touches that column only, and no `T` is assembled. `ps.get(i) -> Option[SoARef[T]]`
  names element `i` as `ps[i]` does, with no copy, as `Array.get` does. `copy of ps[i]` gathers a `T`
  of your own (for `T: Clone`) by the proxy's `load()`, the method `copy of` runs here as it runs
  `clone` elsewhere; `load()` written at a use is `E1010` ("write `copy of ps[0]`"). `ps[i] = v`
  scatters a value. `for p in ps:` yields `SoARef[T]`, `for ref mut p in ps:` yields `SoAMut[T]`. While
  an `SoAMut[T]` is still used it is the only name of its element: a column named separately that
  reaches the element is refused (`E3021` to read it, `E3022` to change it); while an `SoARef[T]` is
  still used, a change of its element through another name is `E3021`.
* `[SOA-3]` `SoA[T]` provides `len`, `is_empty`, `push`, `pop`, `swap_remove`, `retain`, `clear`,
  `reserve`, `with_capacity`, and `sort_by_key` over any column; every operation keeps the columns
  aligned element for element.
* `[SOA-7]` *(new in 0.9.9)* All columns of one `SoA[T]` live in **one heap block**, each column
  starting at an address aligned to the larger of 64 bytes and its element type's alignment. Growth
  reallocates the block once. A column's base pointer is therefore suitable for aligned vector loads.
* `[SOA-4]` `ArenaSoA[T]` is the fixed-capacity, arena-backed form, under the rules of `[ARN-5]`.

## XII.2 SIMD

```ember
from std.simd import f32x8

fn scale(xs: Span[f32], ref mut out: MutSpan[f32], k: f32):
    assert(xs.len() == out.len() and xs.len() % 8 == 0)
    kv = f32x8.splat(k)
    for i in (0..xs.len()).step_by(8):
        v = f32x8.load(xs[i..i + 8])
        (v * kv).store(ref mut out[i..i + 8])
```

* `[SIMD-1]` `std.simd` provides vector types `f32x4`, `f32x8`, `f32x16`, `f64x2`, `f64x4`, `f64x8`,
  the signed and unsigned integer vectors of 8 to 64-bit lanes, and mask types. They are `Copy` value
  types aligned to their size. On the C backend they lower to the target's intrinsics through
  `ember_simd.h`, with a scalar fallback on targets without them; on the LLVM backend, to LLVM's
  vector types (`[CG-LL-2]`). `f32x8.LANES` is the lane count;
  `simd.native_lanes[T]()` is the target's preferred count, a compile-time constant.
* `[SIMD-8]` *(new in 0.9.9)* Vector operators are lane-wise and follow the scalar rules: integer
  lanes panic on overflow (`[TYP-8]`) unless the function is `@overflow(wrap)` or the code calls
  `wrapping_add` and friends; float lanes are IEEE with no contraction (`[TYP-9]`).
* `[SIMD-4]` Horizontal operations (`reduce_add`, `reduce_min`, `reduce_max`), shuffles with a
  constant lane list, gathers, scatters, `fma`, and approximations named `_approx` (`rsqrt_approx`,
  `rcp_approx`) are provided.
* `[SIMD-9]` *(new in 0.9.9)* *(changed in 0.9.15)* SIMD memory operations are bounds-checked like indexing. `V.load(s)` and
  `v.store(s)` require `s.len() >= V.LANES`; `V.gather(s, idx)` and `v.scatter(s, idx)` check every
  lane's index against `s.len()`; each failure is a `Bounds` panic naming the lane. A store and a
  scatter write the caller's elements, so their span is a `ref mut` parameter, and the call says so:
  `v.store(ref mut s)`, `v.scatter(ref mut s, idx)` (`E3028` without it). The unchecked forms
  (`load_unchecked`, `store_unchecked`, `gather_unchecked`, `scatter_unchecked`) are `unsafe fn`. A
  `load` or `store` at a loop's induction variable is covered by the entry test of `[OPT-2]`.
* `[SIMD-2]` `@simd` on a `for` loop is a request and a report: the compiler tries to vectorise the
  loop and `--emit-optimization-report` says whether it did and why not. It never changes meaning.
  `@simd(assert)` makes failure `E4020`.
* `[SIMD-3]` *(changed in 0.9.9)* *(changed in 0.9.15)* Alias facts given to the backend (`restrict` on the C backend, `noalias` on the LLVM backend) MUST be derived, never assumed. Two views
  are marked disjoint only when (a) the compiler related them to one owner and proved their ranges
  disjoint; (b) they come from owners proved distinct in the same function; (c) they are results of
  `split_at`, `chunks_mut` or distinct `SoA` columns; (d) they carry a fact established by
  `assert_disjoint` (`[DSJ-1]`), which stops the program when they overlap, or asserted by
  `unsafe assume_disjoint`; or (e) one of them is a `ref mut` parameter, a `MutSpan` one included:
  every call is checked, when compiling, to give it a place the function reaches by no other name while
  the parameter is used, so every `ref mut` parameter is compiled with `noalias` (a call that names one
  whole variable twice is compiled against its own copy of the function, in which the two names are
  one). A view built by `unsafe` code or received through FFI may alias until (d) is applied. A view
  reached through a link (a `shares`, `shares mut`, `thread_shared` or `sharedref mut` link, or a view
  kept in a struct or a list), and `this`, get an alias fact from (a)–(c) only when the compiler proves
  that no other path the loop uses reaches the same object (the two have different types, one is an
  object made in this function, or the loop is inside `if a is not b:`), because a write through
  another link may change what it views. `this` is never compiled with `noalias`: a method that changes
  its object may also reach it through a link. Two views being live at once is not a proof, and neither
  is a plain parameter's word.
* `[SIMD-5]` *(changed in 0.9.9)* **Vectorisable form**, which `@simd(assert)` checks, is computed by
  Ember and never delegated to the C compiler. A loop is in vectorisable form when: its trip count is
  known before entry; every memory access is to a local or to a view at a unit-stride affine index of
  the induction variable; every written place is accessed at one constant offset (`[PAR-2]`(b)); every
  written view is disjoint from every other accessed view by `[SIMD-3]`; every call in the body is
  to a function `[CG-C-3]` places in the inline header, read through as `[PAR-2b]` says, whatever
  the C compiler then does (ODR-062); the body has none of `Alloc`, `Sync`, `Block`, `Io`, `FFI`,
  `Panic(Explicit)`; no integer division or remainder in the body is by a divisor that is not a
  constant, nor, at 64 bits or more, by a constant that is not a power of two, since no vector
  instruction set does either and such a loop runs one iteration at a time (ODR-088); every
  `RuntimeCheck(Bounds)` has been removed by `[OPT-2]`; every
  `RuntimeCheck(Arithmetic)` is removed, is one `[SIMD-7]` permits grouping (grouping is then
  required), or belongs to a running total `[SIMD-7]` proves safe block by block; the loop has one exit; and every floating-point reduction is declared by `@parallel(reduce=…)` or the function is `@fastmath`.
  `E4020` names the first clause that fails, and for a remaining check its reason (`[EFF-11]`).
* `[SIMD-7]` *(new in 0.9.9)* **Grouped overflow checks.** In a loop whose body has none of `Sync`,
  `Io`, `FFI`, `Unsafe` and `Block`, the compiler MAY check integer overflow once per group of
  iterations (a vector's width times the unroll factor) instead of once per operation, and in a loop in
  vectorisable form (`[SIMD-5]`) it MUST: it computes the group with wrapping arithmetic, accumulates the lanes' overflow bits, and panics at the end of the
  group, reporting the source location of the first overflowing iteration. This is not observable:
  the body's writes go only to memory this thread owns, and the process aborts before anything reads
  them. Bounds checks are never grouped. An integer **running total**, a local the loop changes only
  by `total = total ± x` (`total += xs[i]`), is not grouped: each partial sum waits for the one
  before, so grouping its checks would only add work (ODR-086). It is vectorised under checked
  arithmetic only when no partial sum can overflow in any grouping, which the compiler proves in one
  of two ways. From the element and accumulator widths, versioning the loop on the trip count as
  `[OPT-2]` does: a sum of 32-bit elements into an `int` is safe for fewer than 2³¹ elements. Or, for
  a signed total of *w* ≥ 16 bits, at run time over each block of 64 iterations: the total lies in
  [−2^(w−2), 2^(w−2)) at the block's start and every value the block adds or subtracts lies in
  [−2^(w−9), 2^(w−9)), so no partial sum can leave the type in any order, and the block runs without
  per-operation checks. A block that fails the test runs checked, and so does the rest of the loop. A
  running total proved safe neither way keeps a check on every operation, and its loop is not in
  vectorisable form.
* `[SIMD-6]` The C compiler's own vectorisation report is corroborating evidence only.
  `--emit-optimization-report` prints it labelled as the host compiler's opinion, after Ember's
  vectorisable-form verdict; its absence or disagreement is never an error.

## XII.3 ECS (`std.ecs`)

The entity-component system is an ordinary library, EnTT's design (EnTT 3.x: `entt::registry`, its
views, groups, signals and snapshots) written in Ember: a `Registry` holds entities and one pool per
component type, and a program walks the entities that have a set of components through a view. Any
type is a component, with no registration. There is no scheduler: the library runs nothing on its own,
and the compiler gives it no support of its own.

```ember
from std.ecs import Registry
from std.math import Vec3

@derive(Copy)
struct Position:
    value: Vec3

@derive(Copy)
struct Velocity:
    value: Vec3

@noalloc
fn integrate(ref mut reg: Registry, dt: f32):
    for e, ref mut pos, ref mut vel in reg.view_mut[(Position, Velocity)]():
        pos.value += vel.value * dt

fn main():
    reg = Registry()
    e = reg.create()
    reg.emplace(e, Position(value=Vec3.ZERO))
    reg.emplace(e, Velocity(value=Vec3(1.0, 0.0, 0.0)))
    integrate(ref mut reg, 0.016)
```

* `[ECS-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* `Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit
  generation (`Handle` with `Split20`, `[HND-2]`), EnTT's default `entt::entity`; `Entity.NULL` is all
  ones (`entt::null`). The compact split is kept so entity ids match engines that use it; general `Pool`
  handles are 64-bit (`[HND-1]`).
* `[ECS-2]` *(changed in 0.9.15)* Each component type is stored in a pool of its own, a sparse set: a
  dense `Array[T]` of the values, the entities in dense order, and a sparse index by entity
  (`basic_sparse_set` with `basic_storage`); removal is swap-and-pop. A pool is one contiguous array, not
  paged: no Ember name can keep a view into a pool across a change that may move it (§VII), so a stable
  address is never needed, and one array walks at least as fast. A component type's identity is the
  64-bit FNV-1a hash of its fully qualified name (`ecs.type_hash[T]()`), stable across builds and
  modules. The registry keeps its pools as `pools: Map[u64, owns dyn AnyPool]`: each pool has one owner,
  with no count, and `reg.storage[T]()` reaches its own pool as a place,
  `pool = ref mut cast this.pools[ecs.type_hash[T]()] as Pool[T]` (`[TYP-44]`, `[DSP-4]`).
* `[ECS-3]` *(changed in 0.9.15)* **The registry.** `Registry` (EnTT's `entt::registry`) is a class:
  `create() -> Entity` and `create_at(hint: Entity) -> Entity`; `destroy(e) -> u32` (the entity's next
  generation); `valid(e)`, `current(e)`; `clear()` and `clear_of[T]()`; `emplace(e, owns value: T) -> ref
  mut T`, `emplace_or_replace(e, owns value: T) -> ref mut T`, `replace(e, owns value: T) -> ref mut T`
  (a stop when `e` has no `T`), `patch[T](e, f: fn(ref mut T)) -> ref mut T`, `insert(entities:
  Span[Entity], value: T)` for a `T: Clone`; `remove[T](e) -> int` (0 when absent) and `erase[T](e)` (a
  stop when absent); `get[T](e) -> ref T` and `get_mut[T](e) -> ref mut T`, `try_get[T](e) -> Option[ref
  T]` and `try_get_mut[T](e) -> Option[ref mut T]`, `get_or_emplace(e, owns value: T) -> ref mut T`;
  `all_of[(A, B)](e)` and `any_of[(A, B)](e)`; `orphan(e)`; `storage[T]() -> ref mut Storage[T]`, which a
  `for e, ref mut value in reg.storage[T]()` walks. Several types are one tuple type (`[(A, B)]`), since
  Ember has neither variadic generics nor overloading (`[TYP-19]`). A view a method gives points into the
  registry (`[LT-1]`), so the registry is not changed through another name while it is used (§VII).
* `[ECS-4]` *(changed in 0.9.15)* **Views.** `reg.view[(A, B)]()` walks the entities that have every
  listed component and gives each item read-only (`for e, a, b in reg.view[(A, B)]():`);
  `reg.view_mut[(A, B)]()` gives every item `ref mut` (`for e, ref mut a, ref mut b in
  reg.view_mut[(A, B)]():`), which changes the values in place. The items of one entity are values of
  different types, so they never name one place. `.exclude[(C,)]()` leaves out the entities that have a
  `C`. A view has `each(f)`, `get[T](e)`, `contains(e)`, `size_hint()`, `front()` and `back()`. A view
  walks its smallest pool and probes the others. The library implements its view interface for tuples
  of 1 to 8 types with a blanket implementation over the element types (`[GRM-34]`), so any type is a
  component.
* `[ECS-5]` *(changed in 0.9.15)* **Changing the registry while a view walks it.** A view walks the
  registry's pools, so while a `for` loop walks a view, adding, removing, creating or destroying through
  the registry is refused when compiling, as for any collection a loop walks (`[BRW-18]`, `E3020`: "`reg.destroy(e)`
  (line 6) changes `reg`, whose pools this loop walks (line 4); collect the entities in a list of your
  own: write `dead: Array[Entity] = []` before the loop, `dead.push(e)` in place of this line, and `for d
  in dead: reg.destroy(d)` after the loop"). The values themselves change through the view's `ref mut`
  items. EnTT allows some such changes through paged pools; Ember collects first and changes after.
* `[ECS-6]` Iteration order is insertion order with swap-remove holes: deterministic for a given
  sequence of operations.
* `[ECS-7]` *(changed in 0.9.15)* Any type is a component, with no registration and no derive. A tool
  that needs a list of component types reads the program's own types (Part XIV).
* `[ECS-8]` *(new in 0.9.15)* **Signals, sorting, context, groups, snapshots.** `on_construct[T]()`,
  `on_update[T]()` and `on_destroy[T]()` give a signal; `.connect(f)` and `.disconnect(f)` take a
  `f: fn(ref mut Registry, Entity)`, which the registry calls after the change is made, when no view of
  a pool is in use, so a listener that changes the registry is safe (the call check, `[BRW-21]`, holds
  for the library's own code). `sort[T](less)` and `sort_as[To, From]()` reorder a pool. `ctx()` keeps
  one value per type (`emplace`, `get`, `get_mut`, `find`, `contains`, `erase`). `group[(A, B)]()` packs
  the owned pools in one order. `Snapshot`, `SnapshotLoader` and `ContinuousLoader` save and load a
  registry through an archive given as a type parameter; `RuntimeView` walks pools chosen while the
  program runs; `Handle` pairs a registry and an entity.
* `[ECS-9]` *(new in 0.9.15)* **No scheduler.** The library runs no systems: a program calls its own
  functions on the registry. Work it runs at once goes through `std.thread` or `std.jobs` with the rules
  of Part XI, and two pieces of work run at once that may touch one pool, one changing it, are refused
  when compiling, worked out from every function the program can put there; nothing compares them while
  the program runs.
