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
* `[SOA-5]` `columns_mut(f0, f1, …)` takes **field-name arguments**: identifiers resolved against `T`'s fields at compile time rather than as expressions. It is the only construct with this argument kind; a name that is not a field of `T` is `E2020`. This is an argument *kind*, not overloading (XXIII.2 unaffected).

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
* `[SIMD-3]` Alias information passed to the backend MUST be **derived, never assumed**. Two views MAY be marked `restrict` (C backend) or `noalias` (LLVM) only when one of: (a) the borrow checker related them to the same owner and proved their ranges disjoint; (b) they derive from owners the compiler proved distinct in the same function; (c) they are results of `split_at_mut`, `chunks_mut`, `columns_mut`, or are distinct `SoA` columns of one container; (d) they carry the disjointness fact established by `mem.assert_disjoint`/`assert_disjoint_all` (`[DSJ-3]`) or asserted by `unsafe assume_disjoint` (`[DSJ-7]`). A view whose base pointer entered the function through an FFI contract, through `Span.from_raw_parts`, or through any `unsafe` construction is **may-alias** by default and MUST receive no annotation until (d) is applied to it. **The mere fact that two views are simultaneously live is not a proof and MUST NOT be used as one**, and alias facts MUST NOT be derived from parameter modes: two view parameters of one function are disjoint only where (a)–(d) established it in the caller and the fact travelled with the value. This is what makes `@simd` work without user annotations in cases (a), (c) and (d), and what makes `assert_disjoint` necessary otherwise.
* `[SIMD-4]` Horizontal ops (`reduce_add`, `reduce_max`), shuffles (`shuffle[...]` const-generic lane lists), gathers/scatters, `fma`, `rsqrt`/`rcp` approximations (explicitly named `_approx`) are provided.
* `[SIMD-5]` **Vectorisable form.** On every backend, `@simd(assert)` is checked against a compiler-computed property called *vectorisable form*, never against the host compiler's decision. A loop is in vectorisable form iff all of: its trip count is computable before entry; every memory access in the body is to a view at a unit-stride affine index of the induction variable, or to a local; every written place is accessed — reads included — through index expressions that all share **one** constant offset (`[PAR-2]`(b)); every written view is proven disjoint from every other accessed view per `[SIMD-3]`; the body contains no call not inlined by `[CG-C-3]` or §4.12; the body carries none of `Alloc`, `Sync`, `Block`, `FFI`, `Panic(Explicit)` (`[EFF-16]`), or any `RuntimeCheck(k)` remaining after `[OPT-2]`; the body has exactly one exit; and every floating-point reduction is absent, declared by `@parallel(reduce=…)`, or the function is `@fastmath`. `E4020` is reported when a `@simd(assert)` loop is not in vectorisable form, and the diagnostic MUST name the first violated clause and, where the clause is a `RuntimeCheck`, its `[EFF-11]` reason code.
* `[SIMD-6]` The host compiler's own vectorisation report is **corroborating evidence only**. `ember build --emit-optimization-report` MUST print it when the toolchain produces one and MUST label it as the host compiler's opinion, distinct from the vectorisable-form verdict. Its absence, its format, and its disagreement with the verdict are never errors. Part X §3's `ember inspect` line reports the vectorisable-form verdict first and the host report second. `[DSP-5]` is amended the same way: a devirtualisation performed by the compiler is reported; one performed by the host linker under LTO is not required to be.

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
* `[ECS-7]` `@derive(Component)` registers the type in a comptime component registry used by serialisation and by the editor bridge (Part XXII).

---

