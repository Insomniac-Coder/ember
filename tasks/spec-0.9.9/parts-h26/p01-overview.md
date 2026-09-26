---

# Part I — Language Overview

## I.1 One-paragraph description

Ember is a statically typed, ahead-of-time compiled language with Python's indentation syntax and
Python's everyday vocabulary — list, map and set literals, f-strings, comprehensions, generators,
keyword arguments, `in`, `is None`, chained comparisons — compiled to native code with no garbage
collector. Values have ownership; borrowing is checked by the compiler and never written at call
sites; lifetimes are inferred and never named. Classes are reference-counted objects with
deterministic destruction. There is no null, no exceptions, and no undefined behaviour outside
`unsafe`. Performance-critical code uses value types, views, structure-of-arrays containers, arenas
and enforceable contracts (`@noalloc`, `@nosync`). C headers are imported directly. The reference
compiler emits C11.

## I.2 The three tiers

| Tier | How you enter it | What the compiler guarantees | Typical use |
|---|---|---|---|
| **Safe** (default) | nothing | memory safety, null safety, bounds safety, lifetime safety, data-race freedom, deterministic destruction | applications, gameplay, tools |
| **Contract** | `@noalloc`, `@nosync`, `@noblock`, `@nopanic(explicit)`, `@static_safe`, `@deterministic` | everything in Safe, plus the declared contract, checked at compile time | hot loops, real-time and lock-step code |
| **Unsafe** | `unsafe:` block, `unsafe fn`, `unsafe extern` | type checking and diagnostics only; the programmer upholds `[UNS-4]` | allocators, FFI shims, intrinsics |

* `[TIER-1]` *(changed in 0.9.9)* Safe code MUST NOT invoke an operation with an unverifiable precondition unless an
  `unsafe` boundary in the same package discharges it. There are exactly three such boundaries: an
  `unsafe:` block or `unsafe fn` lexically enclosing the call; an `unsafe extern` declaration
  (`[FFI-10]`); an `unsafe overlay` (`[GRM-35]`, `[FFI-2]`). No attribute, profile, manifest key or contract
  introduces a fourth.

## I.3 Principles the language is held to

* `[PHIL-1]` Two syntactically identical declarations in the same context have identical storage,
  lifetime and synchronisation behaviour.
* `[PHIL-2]` *(changed in 0.9.9)* No heap allocation happens that the source does not show. The
  allocating forms are: constructing a type documented to allocate (`class` instances, `Array`,
  `String`, `Map`, `Set`, `Box`, `Shared`); a list, map or set literal or comprehension in a position
  that produces a heap collection; a string literal in a position that produces a `String`
  (`[TXT-9]`); an f-string; a callable value whose captures exceed its inline capacity (`[CLO-10]`);
  and container growth. Each carries the `Alloc` effect and is listed by `ember inspect --alloc`.
* `[PHIL-3]` No implicit copy of a non-`Copy` value occurs. A copy of a `Copy` value is its bits, and
  retains each counted handle in it (`[OWN-7]`); no copy runs any other code.
* `[PHIL-4]` No implicit synchronisation occurs except through types documented to synchronise
  (`Mutex`, `RwLock`, `Atomic`, channels, `Sync` class handles).
* `[PHIL-5]` Every safety check the compiler removes, it removes because it proved the check
  unnecessary. Declining to look is not a proof.
* `[PHIL-6]` Every expensive or dangerous conversion at an FFI boundary is explicit in source or
  reported by the compiler.
* `[PHIL-7]` Panics are for programmer errors. Recoverable failures use `Result`.
* `[PHIL-8]` **The enforcement ladder.** Ember guarantees memory safety, lifetime safety and data-race
  freedom for Safe code. It proves a property statically where practical; where it cannot but the
  property can be enforced safely at run time, it emits a check; an operation that neither can make
  safe requires `unsafe`. A program is rejected only when no safe enforcement exists **and** the
  intent is expressible some other way — and the diagnostic MUST name that way (`[DIA-7]`).
* `[PHIL-8a]` Every rejection shape in §XVII.6 has a mandated `help` that, applied literally to the
  rejected program, produces a program that compiles. `tests/ui/` holds a before/after pair per shape.
* `[PHIL-9]` Class instances carry a header and can therefore carry dynamic checks; value types
  have no hidden state, because `[TYP-11]` gives every plain struct C layout. Value types that need
  aliased mutation opt into a checked type explicitly (`Cell`, `RefCell`, or a `class`).
* `[PHIL-12]` *(new in 0.9.9)* **No silent acceptance.** Every construct a program writes — an attribute, an import,
  a directive, a contract, a manifest key, a statement form — either has exactly the effect this
  specification gives it, or the program is rejected. An implementation that has not yet built a
  construct rejects it with `E0900 not supported yet by this compiler`, naming the construct and the
  rule; a construct this specification does not define is `E0901`. Accepting a construct and
  ignoring it is a defect of the worst class, because the programmer believes the effect holds.
* `[PHIL-13]` *(new in 0.9.9)* **One meaning per program.** No build profile, compiler flag, manifest key or
  optimisation level changes what a Safe program computes, which branch it takes, whether it panics,
  or whether it is accepted, and none turns off a check Safe Ember's guarantees need. What a build
  accepts depends on its semantic inputs alone — the language version, the target and its layout, the
  resolved dependencies and the library layers selected; a build may also stop for its policy
  (warnings made errors, `[MAN-3]`) or an exhausted budget (`[CT-3]`), each reported as that and never
  as a language error. `@realtime` means the declaring package's set (`[EFF-19]`), in every caller
  (ODR-061). Profiles change only speed, debug information, and the amount of detail
  in diagnostics and panic messages (`[PRF-1]`). `debug_assert` is the one check a profile turns off; it
  may not have side effects, so it cannot change what a program whose assertions hold computes.
* `[PHIL-14]` *(new in 0.9.9)* **Python spelling, Python meaning.** Where Ember accepts a spelling that Python also
  has — `/`, `//`, `%`, `**`, `a < b < c`, `x in xs`, `x is None`, `[1, 2]`, `{"k": v}`, f-strings,
  `for … else`, keyword arguments — it gives it Python's meaning, restricted to Ember's types. Where
  Ember cannot give Python's meaning at C speed, it rejects the spelling with a fix-it rather than
  giving it a different meaning silently. Integer `/` is the example: rejected, with `//` and a float
  conversion as the fixes (`[TYP-28]`).
* `[PHIL-15]` *(new in 0.9.9)* **Costs are named.** Every cost the language can insert without the programmer writing
  it — a bounds check, an overflow check, a reference-count operation, a dynamic exclusivity check, an
  indirect call, an allocation — has a row in §X.4 and is reported per site by `ember inspect`.

`[PHIL-8]` is what distinguishes Ember from a language whose only safety mechanism is static proof:
it is a rule about **how** a guarantee is met, never about **whether**.

## I.3a The Safe Ember invariant

* `[PHIL-10]` *(changed in 0.9.9)* A program containing no `unsafe` block, no `unsafe fn` and no
  false foreign contract (`[FFI-1]`) cannot perform an invalid ownership operation, a use-after-free,
  a double release, a data race, an out-of-bounds access, an invalid range-type construction
  (`[RNG-9]`), a read of uninitialised memory, an access through an invalid reference, or an
  arithmetic operation whose result C would leave undefined. **This holds in every build profile and
  under every manifest setting**; no configuration disables a check that this invariant relies on.
* `[PHIL-11]` What remains possible, and is therefore not a defect of the guarantee: resource
  exhaustion (a panic, `[ALC-4]`); deadlock; a logical race between correctly synchronised
  operations; a reference cycle leaking (`[WK-1]`); a bug in foreign code reached through a correctly
  declared boundary; a hardware fault; deliberate termination through `panic` or `abort`; and any
  violated obligation inside `unsafe` or an inaccurate `asserted` foreign fact.

## I.4 Static proof and run-time enforcement

Which mechanism enforces a guarantee is decided per property and per program point by the compiler,
never by the programmer — but it is always visible (`ember inspect --safety`) and restrictable
(`@static_safe`, `[EFF-12]`).

| Guarantee | Statically proven when | Checked at run time when | Mechanism |
|---|---|---|---|
| No use-after-free, no dangling reference | always, for references and views | never | borrow checker; zero cost |
| Value aliasing (`ref mut` xor `ref`) | always, for value types | never | borrow checker; zero cost |
| Object aliasing (long-term access through a class handle) | the accesses go through one handle local and are statically ordered (`[EXC-3]`) | otherwise | the field's access word (`[EXC-19]`); a load, a compare and a store |
| Interior mutability of a value | never (the programmer chose it) | always | `Cell` (no check needed) / `RefCell` borrow counter |
| Index in bounds | the index comes from the container's own range, or a range fact (`[RNG-4]`) | otherwise | compare and branch |
| Integer arithmetic does not overflow | a range fact proves it | otherwise | overflow flag test (`[TYP-8]`) |
| Resource handle is live | never (generation is run-time data) | always, every profile | generation compare (`[HND-1]`) |
| Object lifetime | escape analysis promotes to the stack (`[OPT-1]`) | otherwise | reference counts, elided per `[RC-2]` |
| No data race | always, via `Send`/`Sync` | never | type system; zero cost |

## I.5 A complete small program

```ember
from std.math import Vec3

## A particle in a fountain simulation: a plain value, 28 bytes.
@derive(Copy)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

## A reference type with identity; handles are reference counted.
class Emitter:
    name: String
    particles: Array[Particle] = []
    spawn_rate: f32 = 100.0

    fn init(self, name: String):
        self.name = name

    fn spawn(self, count: int):
        for _ in 0..count:
            self.particles.push(Particle(
                position=Vec3.ZERO,
                velocity=Vec3(0, 9.8, 0),
                lifetime=2.0))

## Contract tier: proven not to allocate.
@noalloc
fn integrate(mut particles: MutSpan[Particle], dt: f32):
    for p in particles.iter_mut():
        p.velocity.y -= 9.81 * dt
        p.position += p.velocity * dt
        p.lifetime -= dt

fn main():
    fountain = Emitter("fountain")
    fountain.spawn(10_000)
    for _ in 0..600:
        integrate(fountain.particles, 1.0 / 60.0)
        fountain.particles.retain(fn(p) => p.lifetime > 0.0)
    println(f"{fountain.name}: {fountain.particles.len()} alive")
```

*Note.* `Emitter("fountain")` passes a string literal where a `String` is expected; the literal
allocates the `String` there (`[TXT-9]`). `particles: Array[Particle] = []` is an empty list literal
in an `Array` position. `integrate` receives the class field as a `MutSpan` through a checked write
access to the emitter's `particles` field (`[EXC-1]`, `[EXC-19]`). `1.0 / 60.0` takes `f32` from the parameter it is passed to.

## I.6 Non-goals

Ember deliberately has none of these: a tracing garbage collector or a cycle collector (cycles are
broken with `Weak` and reported, `[WK-15]`); exceptions or unwinding (failures are values or panics,
Part XIII); implicit numeric narrowing; function overloading and variadic functions, apart from the output
functions (`[TYP-26]`); macros (`macro` is reserved); named lifetimes, ever (`[LEX-22]`); specialisation or
higher-kinded types; dynamic or structural typing (`[TYP-40]`); a safety check that depends on the
profile (`[PRF-1]`); a correctness property that depends on whole-program optimisation or LTO; a stable
Ember-to-Ember ABI (the C ABI is the stable boundary, Part XVI); shader compilation and `async` in this
version (both reserved).

