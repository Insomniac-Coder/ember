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

There is no fourth tier. `[TIER-1]` Safe code MUST NOT invoke an operation with an unverifiable precondition unless an `unsafe` boundary in the same package discharges it. There are exactly three such boundaries: (1) an `unsafe:` block or `unsafe fn` lexically enclosing the call; (2) an `unsafe extern` declaration (`[FFI-10]`); (3) an `unsafe overlay` declaration (`[FFI-2]`). No other construct may make an unverified assertion, and no attribute, profile or contract may introduce a fourth.

## I.3 Design rules the compiler is held to

* `[PHIL-1]` Two syntactically identical declarations in the same context have identical storage, lifetime and synchronisation behaviour.
* `[PHIL-2]` No implicit heap allocation occurs except by constructing a type that is documented to allocate (`class` instances, `Array`, `String`, `Map`, `Box`, `Shared`, closures that escape).
* `[PHIL-3]` No implicit copy of a non-`Copy` value occurs. Copies of `Copy` values are bitwise.
* `[PHIL-4]` No implicit synchronisation occurs except through types documented to synchronise (`Mutex`, `Atomic`, channels, `Sync` class handles).
* `[PHIL-5]` Every safety check the compiler removes, it removes because it **proved** the check unnecessary. Declining to look is not a proof: an analysis that inspects only part of the program state that could invalidate a property has not established it, and removing the check on that basis is the same violation as changing a result (`[EXC-3]`, `[EFF-15]`). Optimisation never changes observable behaviour of safe code, and no profile, optimisation level or elision pass changes whether a program is accepted.
* `[PHIL-6]` Any expensive or dangerous conversion at an FFI boundary is either explicit in source or reported by the compiler.
* `[PHIL-7]` Panics are for programmer errors. Recoverable failures use `Result`.
* `[PHIL-8]` **The enforcement ladder.** Ember MUST guarantee memory safety, lifetime safety and data-race freedom for Safe code. The compiler SHOULD prove a required property statically whenever practical. Where static proof is unavailable but the property can be enforced safely at runtime, Ember MAY emit a runtime check instead of rejecting the program. An operation that can be made safe by neither static proof nor runtime enforcement MUST require an explicit `unsafe` boundary. Rejecting a program is correct only when no safe enforcement exists **and** the operation is expressible some other way — in which case the diagnostic MUST name that way (`[DIA-7]`).
* `[PHIL-8a]` **The ladder is enforced on every revision.** Every error code that rejects a program a previous language version accepted MUST have a diagnostic shape in §XIX.6.1 or §XIX.6.2 whose mandated `help`, applied literally to the rejected program, produces a program that compiles. `tools/rule_index.py` MUST verify this mechanically over `tests/ui/`: for each shape, the recorded "before" program fails with that code and the "after" program — obtained by applying the mandated fix — compiles. A shape whose fix does not compile is a defect of the same kind as `[DIA-11]`'s.

`[PHIL-8]` is the sentence that distinguishes Ember from a language whose only safety mechanism is static proof. It is a rule about **how** a guarantee is met, not about *which* guarantees hold: the guarantee list never shrinks, and no profile, contract or attribute weakens it outside `unsafe`.

## I.4 Static proof and runtime enforcement

Safe code obtains its guarantees from two mechanisms. Which one applies is decided per property and per program point by the compiler, never by the programmer — but it is always **visible** (`[EFF-9]`, `ember inspect --safety`) and always **restrictable** (`@static_safe`, `[EFF-12]`).

| Guarantee | Statically proven when | Runtime-enforced when | Mechanism and cost |
|---|---|---|---|
| No use-after-free, no dangling reference | always, for `ref`/`Span`/view types | never | borrow checker; zero cost |
| Value aliasing (`ref mut` XOR `ref`) | always, for value types | never | borrow checker; zero cost |
| Object aliasing (long-term access through a class handle) | accesses go through the same handle local and are statically ordered (`[EXC-3]`) | handles differ, or a handle is loaded from memory | header `access_state` word; ~2 ns per access pair |
| Interior mutability of a value type | never (the programmer opted in) | always | `Cell` (no check needed) / `RefCell` borrow state (`[CELL-5]`) |
| Index in bounds | index derived from the container's own length or a proven range | otherwise | compare + branch, predictable |
| Resource handle is live | never (generation is runtime data) | always outside `shipping` | generation compare (`[GPU-1]`, `[HND-1]`) |
| Object lifetime (class instances) | escape analysis promotes to the stack (`[OPT-1]`) | otherwise | reference counts, elided per `[RC-2/3]` |
| No data race | always, via `Send`/`Sync` | never — synchronisation is always explicit (`Mutex`, `Atomic`) | type system; zero cost |

`[PHIL-9]` **Why class instances can carry dynamic checks and value types cannot.** A class instance already has a 24-byte header (`[OBJ-1]`) holding its reference counts and type information; one more word carries access state at no layout cost, and the header is invisible to foreign code because handles are always passed as opaque pointers. A `struct` has no header by design: `[TYP-11]` guarantees C-compatible layout so that every plain struct can cross an FFI boundary unchanged, and `size_of`, `align_of` and field offsets are exactly what a C compiler would produce. Adding hidden runtime state to value types would change `size_of[T]()`, break `@layout(c)`, invalidate every `[FFI-5]` layout assertion, and make `SoA` columns and GPU-uploaded buffers no longer bit-identical to their C counterparts. The dichotomy is therefore forced by the FFI and layout guarantees, not by a judgement that one kind of code deserves more freedom than the other. Value types that genuinely need aliased mutation opt into a header-carrying type explicitly: `RefCell[T]` (Part IX §7) or a `class`.

## I.5 A complete small program

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

