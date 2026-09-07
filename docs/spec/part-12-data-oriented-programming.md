# Part XII — Data-Oriented Programming

## XII.1 `SoA[T]`

```ember
@derive(Copy, SoA)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

particles = SoA[Particle].with_capacity(100_000)
particles.push(Particle(...))                          # scatters fields into columns
pos = particles.position                               # MutSpan[Vec3] over the position column (mutable if `particles` is)
p   = particles[i]                                     # a *proxy* value of type SoA[Particle].Ref — field reads/writes hit columns
particles[i].lifetime -= dt                            # writes column `lifetime` at i; no Particle temporary
particles.get(i) -> Option[Particle]                   # gathers a copy

pos, vel = particles.columns_mut(position, velocity)   # two disjoint MutSpans in one call (compile-time field names)
```

* `[SOA-1]` `@derive(SoA)` on a struct `T` generates `SoA[T]` (a struct with one `Array[FieldType]` per field), the proxy types `SoA[T].Ref`/`RefMut`, `columns_mut`, and the `Iterable`/`IterableMut` impls yielding proxies. Nested `@derive(SoA)` structs are flattened recursively (`position.x` becomes its own column) when the field is marked `@soa(flatten)`; otherwise the nested struct is one column.
* `[SOA-2]` Column borrows are **disjoint places** for the borrow checker (`[BRW-4]` applies field-wise across columns), so two systems can hold `mut` borrows of different columns simultaneously.
* `[SOA-3]` `SoA[T]` provides `swap_remove`, `retain`, `sort_by_column`, `len`, `reserve`; element order is stable across all columns.
* `[SOA-4]` `ArenaSoA[T]` is the arena-backed view variant.

## XII.2 SIMD

```ember
from std.simd import f32x8, Mask8

a: f32x8 = f32x8.load(xs[i..i+8])                     # explicit vectors: f32x4/8/16, i32x4/8/16, u8x16/32, ...
b = a * f32x8.splat(2.0) + c
mask = a.gt(b)                                        # Mask8
r = mask.select(a, b)
r.store(mut out[i..i+8])
lanes = f32xN                                         # N = target-preferred width (comptime const)
```

* `[SIMD-1]` Vector types are plain `Copy` structs with `@align(width)`; on the C backend they lower to a per-compiler intrinsic shim (`ember_simd.h`: SSE/AVX/AVX-512/NEON via `immintrin.h`/`arm_neon.h`, with a scalar fallback); on LLVM to vector IR.
* `[SIMD-2]` `@simd` on a `for` loop is a **hint + diagnostic contract**: the compiler attempts vectorisation and, if `--emit-optimization-report` is on, reports success or the reason for failure (aliasing not provable, non-contiguous access, call in body, early exit). It never changes semantics. `@simd(assert)` upgrades failure to `E4020`.
* `[SIMD-3]` Alias information from the borrow checker is passed to the backend: two live `MutSpan`s are known disjoint; a `MutSpan` and any `Span` live at the same time are known disjoint (they could not otherwise coexist). The C backend emits `restrict` on the corresponding pointer locals; LLVM receives `noalias` metadata. This is what makes `@simd` work without user annotations.
* `[SIMD-4]` Horizontal ops (`reduce_add`, `reduce_max`), shuffles (`shuffle[...]` const-generic lane lists), gathers/scatters, `fma`, `rsqrt`/`rcp` approximations (explicitly named `_approx`) are provided.

## XII.3 ECS (`std.ecs`, a library)

The ECS is a library over `SoA`, `Pool`, and comptime reflection, designed to match RageV's existing storage shape (sparse set per component; `Entity` = 20-bit index + 12-bit generation) so that the two can share entity IDs across the FFI boundary.

```ember
@derive(Copy, Component)
struct Position:
    value: Vec3

@derive(Copy, Component)
struct Velocity:
    value: Vec3

@derive(Component)
struct Name:
    value: String

world = World()
e = world.create()
world.add(e, Position(Vec3.ZERO))
world.add(e, Velocity(Vec3(1, 0, 0)))

@noalloc
fn integrate(mut q: Query[(mut Position, Velocity)], dt: f32):
    for pos, vel in q:                                 # pos: ref mut Position, vel: ref Velocity
        pos.value += vel.value * dt

world.run(integrate, dt)                               # borrows the storages named by the query type
```

* `[ECS-1]` `Entity` is `Handle[EntityTag]` with the 20/12 split; `Entity.NULL` is all-ones (same as RageV's `Null`).
* `[ECS-2]` Storage per component type is a sparse set: `dense: SoA[T]` or `Array[T]` (chosen by `@component(layout=soa|aos)`, default AoS to match RageV), `entities: Array[Entity]`, `sparse: Array[u32]` indexed by entity index. Component type identity is a 64-bit FNV-1a hash of the fully-qualified type name (`std.ecs.type_hash[T]()`), the same scheme RageV's `TypeHash` uses, so IDs are stable across modules and DLLs.
* `[ECS-3]` `Query[(A, mut B, Option[C], Not[D])]` is a view type over the world; it iterates the smallest storage among its required components and probes the others. Iteration yields `ref`/`ref mut` tuples; component access is two loads and an index (`[ECS-2]` layout). `[ECS-4]` The query's read/write set is a comptime constant; `World.run` and `World.run_parallel([sys1, sys2])` use it for access-set scheduling (`[JOB-3]`) and reject conflicting systems at compile time when both are known (`E7020`).
* `[ECS-5]` Structural changes (`add`/`remove`/`destroy`) during iteration go through a `Commands` buffer applied after the system returns (`q.commands().destroy(e)`), which keeps iteration borrow-safe.
* `[ECS-6]` Iteration order is insertion order with swap-remove holes, i.e. deterministic for an untouched scene (RageV's requirement 3 in `ECS.h`).
* `[ECS-7]` `@derive(Component)` registers the type in a comptime component registry used by serialisation and by the editor bridge (Part XXI).

---

