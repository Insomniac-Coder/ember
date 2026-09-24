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

fn integrate(pos: MutSpan[Vec3], vel: Span[Vec3], dt: f32):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    ps = SoA[Particle].with_capacity(100_000)
    ps.push(Particle(pos=Vec3.ZERO, vel=Vec3(1.0, 0.0, 0.0), life=2.0))
    integrate(ps.pos, ps.vel, 0.016)        # two columns borrowed at once: disjoint places
    ps[0].life -= 0.016                     # writes one element of the `life` column
    p = ps.get(0)                           # Option[Particle]: gathers a copy
```

* `[SOA-1]` *(changed in 0.9.9)* `SoA[T]` is a **compiler-known type constructor**, like `Cell`: for
  any struct `T` it is a growable container holding one column per field of `T`. No derive is needed.
  `T` MUST be a struct (`E2071` otherwise). A field marked `@soa(flatten)` whose type is a struct is
  split into one column per field of its own (`ps.pos.x`); otherwise a struct field is one column.
* `[SOA-2]` `ps.f` names the column of field `f`. It is a place of type `MutSpan[F]` where `ps` is
  mutable and `Span[F]` otherwise, and it supports everything those views support. Columns are
  **disjoint places** under `[BRW-4]`, so any number of different columns may be borrowed at once,
  mutably or not. A length-changing operation (`push`, `swap_remove`, `clear`) borrows all of `ps`.
* `[SOA-6]` *(new in 0.9.9)* `ps[i]` is an **element proxy**: `SoARef[T]` where `ps` is read-only here
  and `SoAMut[T]` where it is mutable. A proxy's field `ps[i].f` is a place in column `f` at index `i`;
  reading or writing it touches that column only, and no `T` is assembled. `ps.get(i) -> Option[T]`
  and `ps[i].load() -> T` gather a value (for `T: Clone`); `ps[i] = v` scatters one. `for p in ps:`
  yields `SoARef[T]`, `for p in ps.iter_mut():` yields `SoAMut[T]`. A `SoAMut[T]` borrows all of `ps`
  mutably for the proxy's life and a `SoARef[T]` borrows it shared, so a column borrowed separately
  conflicts with a live proxy.
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

fn scale(xs: Span[f32], out: MutSpan[f32], k: f32):
    assert(xs.len() == out.len() and xs.len() % 8 == 0)
    kv = f32x8.splat(k)
    for i in (0..xs.len()).step_by(8):
        v = f32x8.load(xs[i..i + 8])
        (v * kv).store(out[i..i + 8])
```

* `[SIMD-1]` `std.simd` provides vector types `f32x4`, `f32x8`, `f32x16`, `f64x2`, `f64x4`, `f64x8`,
  the signed and unsigned integer vectors of 8 to 64-bit lanes, and mask types. They are `Copy` value
  types aligned to their size. On the C backend they lower to the target's intrinsics through
  `ember_simd.h`, with a scalar fallback on targets without them. `f32x8.LANES` is the lane count;
  `simd.native_lanes[T]()` is the target's preferred count, a compile-time constant.
* `[SIMD-8]` *(new in 0.9.9)* Vector operators are lane-wise and follow the scalar rules: integer
  lanes panic on overflow (`[TYP-8]`) unless the function is `@overflow(wrap)` or the code calls
  `wrapping_add` and friends; float lanes are IEEE with no contraction (`[TYP-9]`).
* `[SIMD-4]` Horizontal operations (`reduce_add`, `reduce_min`, `reduce_max`), shuffles with a
  constant lane list, gathers, scatters, `fma`, and approximations named `_approx` (`rsqrt_approx`,
  `rcp_approx`) are provided.
* `[SIMD-9]` *(new in 0.9.9)* SIMD memory operations are bounds-checked like indexing. `V.load(s)` and
  `v.store(s)` require `s.len() >= V.LANES`; `V.gather(s, idx)` and `v.scatter(s, idx)` check every
  lane's index against `s.len()`; each failure is a `Bounds` panic naming the lane. The unchecked forms
  (`load_unchecked`, `store_unchecked`, `gather_unchecked`, `scatter_unchecked`) are `unsafe fn`. A
  `load` or `store` at a loop's induction variable is covered by the entry test of `[OPT-2]`.
* `[SIMD-2]` `@simd` on a `for` loop is a request and a report: the compiler tries to vectorise the
  loop and `--emit-optimization-report` says whether it did and why not. It never changes meaning.
  `@simd(assert)` makes failure `E4020`.
* `[SIMD-3]` *(changed in 0.9.9)* Alias facts given to the backend (`restrict`) MUST be derived, never assumed. Two views
  are marked disjoint only when (a) the borrow checker related them to one owner and proved their
  ranges disjoint; (b) they come from owners proved distinct in the same function; (c) they are
  results of `split_at`, `chunks_mut` or distinct `SoA` columns; or (d) they carry a fact established
  by `assert_disjoint` (`[DSJ-1]`) or asserted by `unsafe assume_disjoint`. A view built by `unsafe`
  code or received through FFI may alias until (d) is applied. A view reached through a class handle
  or a `Shared` gets an alias fact from (a)–(c) only when the loop contains no call that is not
  inlined, no call through a callable value, no virtual or interface dispatch and no access through
  another handle (the conditions of `[EXC-3]`), because an unchecked write through another handle may
  change what it views (`[EXC-17]`). Two views being live at once is not a proof, and neither are
  parameter modes.
* `[SIMD-5]` *(changed in 0.9.9)* **Vectorisable form**, which `@simd(assert)` checks, is computed by
  Ember and never delegated to the C compiler. A loop is in vectorisable form when: its trip count is
  known before entry; every memory access is to a local or to a view at a unit-stride affine index of
  the induction variable; every written place is accessed at one constant offset (`[PAR-2]`(b)); every
  written view is disjoint from every other accessed view by `[SIMD-3]`; every call in the body is
  inlined (`[CG-C-3]`); the body has none of `Alloc`, `Sync`, `Block`, `Io`, `FFI`,
  `Panic(Explicit)`; every `RuntimeCheck(Bounds)` has been removed by `[OPT-2]`; every
  `RuntimeCheck(Arithmetic)` is removed, or is one `[SIMD-7]` permits grouping (grouping is then
  required); the loop has one exit; and every floating-point reduction is declared by `@parallel(reduce=…)` or the function is `@fastmath`.
  `E4020` names the first clause that fails, and for a remaining check its reason (`[EFF-11]`).
* `[SIMD-7]` *(new in 0.9.9)* **Grouped overflow checks.** In a loop whose body has none of `Sync`,
  `Io`, `FFI`, `Unsafe` and `Block`, the compiler MAY check integer overflow once per group of
  iterations (a vector's width times the unroll factor) instead of once per operation, and in a loop in
  vectorisable form (`[SIMD-5]`) it MUST: it computes the group with wrapping arithmetic, accumulates the lanes' overflow bits, and panics at the end of the
  group, reporting the source location of the first overflowing iteration. This is not observable:
  the body's writes go only to memory this thread owns, and the process aborts before anything reads
  them. Bounds checks are never grouped. An integer **reduction** (`total += xs[i]`) is vectorised
  under checked arithmetic only when no partial sum can overflow in any grouping; the compiler proves
  that from the element and accumulator widths, versioning the loop on the trip count as `[OPT-2]`
  does (a sum of 32-bit elements into an `int` is safe for fewer than 2³¹ elements).
* `[SIMD-6]` The C compiler's own vectorisation report is corroborating evidence only.
  `--emit-optimization-report` prints it labelled as the host compiler's opinion, after Ember's
  vectorisable-form verdict; its absence or disagreement is never an error.

## XII.3 ECS (`std.ecs`)

The entity-component system is an ordinary library built on `SoA`, generational handles and
compile-time reflection. It has no compiler support of its own.

```ember
from std.ecs import World, Query, Mut, Component
from std.math import Vec3

@derive(Copy, Component)
struct Position:
    value: Vec3

@derive(Copy, Component)
struct Velocity:
    value: Vec3

@noalloc
fn integrate(mut q: Query[(Mut[Position], Velocity)], dt: f32):
    for pos, vel in q.iter_mut():           # pos: ref mut Position, vel: ref Velocity
        pos.value += vel.value * dt

fn main():
    world = World()
    e = world.create()
    world.add(e, Position(Vec3.ZERO))
    world.add(e, Velocity(Vec3(1.0, 0.0, 0.0)))
    world.run(integrate, 0.016)
```

* `[ECS-1]` *(changed in 0.9.9)* `Entity` is a 32-bit generational handle: a 20-bit index and a 12-bit
  generation (`Handle` with `Split20`, `[HND-2]`); `Entity.NULL` is all ones. The compact split is kept
  so entity ids match engines that use it; general `Pool` handles are 64-bit (`[HND-1]`).
* `[ECS-2]` Each component type is stored as a sparse set: a dense `Array[T]` or `SoA[T]`
  (`@component(layout=soa)`; the default is `aos`), the entities in dense order, and a sparse index by
  entity. A component type's identity is the 64-bit FNV-1a hash of its fully qualified name
  (`ecs.type_hash[T]()`), stable across builds and modules.
* `[ECS-3]` *(changed in 0.9.9)* `Query[(A, Mut[B], Option[C], Not[D])]` is a view over a world's
  storages. `Mut[T]`, `Not[T]` and `Option[T]` in a query are **ordinary types**: `Mut` and `Not` are
  marker structs with a phantom parameter (`[TYP-35]`), and the library maps each parameter to its
  item type (`ref T`, `ref mut T`, `Option[ref T]`, nothing) through an interface with an associated
  type. A query iterates the smallest required storage and probes the others; `iter` borrows `q`
  shared and yields read-only items; `iter_mut` yields `ref mut` for `Mut` parameters and requires
  `mut q`. There is no access-mode generic kind in the
  language.
* `[ECS-4]` *(changed in 0.9.9)* A query's read and write sets are compile-time constants computed from
  its type. `world.run(system, args…)` borrows exactly those storages. `world.run_parallel([s1, s2, …])`
  runs systems at once when their sets do not conflict; a conflict between two systems known at
  compile time is `E7020`, naming both systems and the component. Systems passed as values, whose sets
  are not known at compile time, are checked against each other before any of them starts, and a
  conflict panics, naming both systems and the component.
* `[ECS-5]` Adding, removing and destroying during iteration go through a `Commands` buffer
  (`q.commands().destroy(e)`) that is applied after the system returns.
* `[ECS-6]` Iteration order is insertion order with swap-remove holes: deterministic for a given
  sequence of operations.
* `[ECS-7]` `@derive(Component)` registers the type in a compile-time component registry that
  serialisation and reflection-based tools read (Part XIV).
