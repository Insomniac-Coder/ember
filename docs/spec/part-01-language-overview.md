# Part I — Language Overview

## I.1 One-paragraph description

Ember is a statically typed, ahead-of-time compiled systems language with Python-style indentation syntax. Values have ownership; borrowing is checked by the compiler and inferred at call sites. Classes are reference-counted objects with deterministic destruction. There is no garbage collector, no exceptions, no null, no undefined behaviour outside `unsafe`. Performance-critical code uses value types, views, structure-of-arrays containers, arenas, SIMD and parallel loops, and can declare enforceable contracts (`@noalloc`, `@nosync`). C is imported directly from headers; C++ is bound through generated `extern "C"` thunks. The toolchain compiles Ember to C11 (bootstrap and portability backend) or LLVM IR.

## I.2 The three tiers

Every Ember program is written in one of three tiers, and the tier of any function is visible from its declaration:

| Tier | How you enter it | What the compiler guarantees | Typical use |
|---|---|---|---|
| **Safe** (default) | Nothing | Memory safety, null safety, bounds safety, lifetime safety, data-race freedom, deterministic destruction | Gameplay, editor, tools, orchestration |
| **Contract** | `@noalloc`, `@nosync`, `@simd`, `@parallel`, explicit `Arena`, `SoA`, `Span` | Everything in Safe, plus the declared performance contract is enforced at compile time | Culling, ECS systems, particle updates, command generation |
| **Unsafe** | `unsafe:` block / `unsafe fn` / `unsafe extern` | Type checking and diagnostics only; the programmer upholds the invariants listed in `[UNS-*]` | Backends, FFI shims, allocators, intrinsics |

There is no fourth tier. `[TIER-1]` Safe code MUST NOT be able to invoke an operation with an unverifiable precondition without an `unsafe` block lexically enclosing the call.

## I.3 Design rules the compiler is held to

* `[PHIL-1]` Two syntactically identical declarations in the same context have identical storage, lifetime and synchronisation behaviour.
* `[PHIL-2]` No implicit heap allocation occurs except by constructing a type that is documented to allocate (`class` instances, `Array`, `String`, `Map`, `Box`, `Shared`, closures that escape).
* `[PHIL-3]` No implicit copy of a non-`Copy` value occurs. Copies of `Copy` values are bitwise.
* `[PHIL-4]` No implicit synchronisation occurs except through types documented to synchronise (`Mutex`, `Atomic`, channels, `Sync` class handles).
* `[PHIL-5]` Every safety check the compiler removes, it removes because it proved the check unnecessary. Optimisation never changes observable behaviour of safe code.
* `[PHIL-6]` Any expensive or dangerous conversion at an FFI boundary is either explicit in source or reported by the compiler.
* `[PHIL-7]` Panics are for programmer errors. Recoverable failures use `Result`.

## I.4 A complete small program

```ember
import std.io
from std.math import Vec3, sqrt

## A particle in a fountain simulation. Plain value type; 28 bytes.
@derive(Copy, Debug)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

## Reference type with identity; handles are reference counted.
class Emitter:
    name: String
    particles: Array[Particle]
    spawn_rate: f32 = 100.0

    fn init(mut self, name: String):
        self.name = name
        self.particles = Array[Particle]()

    fn spawn(mut self, count: usize):
        for i in 0..count:
            self.particles.push(Particle(
                position=Vec3.ZERO,
                velocity=Vec3(0, 9.8, 0),
                lifetime=2.0))

## Contract tier: proven not to allocate, vectorisable.
@noalloc
@simd
fn integrate(mut particles: MutSpan[Particle], dt: f32):
    for p in particles.iter_mut():
        p.velocity.y -= 9.81 * dt
        p.position += p.velocity * dt
        p.lifetime -= dt

fn main() -> Result[void, io.Error]:
    fountain = Emitter("fountain")
    fountain.spawn(10_000)

    for frame in 0..600:
        integrate(fountain.particles.as_mut_span(), 1.0 / 60.0)
        fountain.particles.retain(fn(p) => p.lifetime > 0.0)

    io.println(f"{fountain.name}: {fountain.particles.len()} alive")
    return Ok(())
```

Everything in this program is specified precisely in the parts that follow.

---

