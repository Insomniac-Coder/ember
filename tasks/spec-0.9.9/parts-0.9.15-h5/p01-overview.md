---

# Part I — Language Overview

## I.1 One-paragraph description

Ember is a statically typed, ahead-of-time compiled language with Python's indentation syntax and
Python's everyday vocabulary — list, map and set literals, f-strings, comprehensions, generators,
keyword arguments, `in`, `is None`, chained comparisons — compiled to native code with no garbage
collector. Every value has one owner unless a line says `shares`; a name that points into what someone
else owns is a view, and the word in front of a name says whether it may change what it names
(`ref mut`) or takes it over (`owns`); a call says `ref mut` before an argument the callee may change.
What every function changes and reads is worked out by the compiler from its code, and all of it is
checked when compiling: nothing about ownership, views or aliasing is checked while the program runs,
and lifetimes are inferred and never named. Class objects live in their own blocks, with identity and
deterministic destruction; several owners are counted (`shares`), and a link back is a `weakref`. There
is no null, no exceptions, and no undefined behaviour outside `unsafe`. Performance-critical code uses
value types, views, structure-of-arrays containers, arenas and enforceable contracts (`@noalloc`,
`@nosync`). C headers are imported directly. The reference compiler emits C11; LLVM is its planned
second backend (`[CG-LL-1]`).

## I.2 The three tiers

| Tier | How you enter it | What the compiler guarantees | Typical use |
|---|---|---|---|
| **Safe** (default) | nothing | memory safety, null safety, bounds safety, lifetime safety, data-race freedom, deterministic destruction, no leak through a loop of owners | applications, gameplay, tools |
| **Contract** | `@noalloc`, `@nosync`, `@noblock`, `@nopanic(explicit)`, `@no_alias_checks`, `@deterministic` | everything in Safe, plus the declared contract, checked at compile time | hot loops, real-time and lock-step code |
| **Unsafe** | an `unsafe` boundary (`[TIER-1]`) | type checking and diagnostics only; the programmer upholds `[UNS-4]` | allocators, FFI shims, intrinsics |

* `[TIER-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* Anything that can break a safety promise is
  possible only inside an `unsafe` boundary. Safe code MUST NOT invoke an operation with an unverifiable
  precondition unless an `unsafe` boundary in the same package discharges it. The boundaries are each
  written with the word `unsafe`: an `unsafe:` block lexically enclosing the operation, or the
  `unsafe fn` whose body encloses it, outside the body's `safe:` blocks (`[UNS-12]`); an
  `unsafe extern` declaration (`[FFI-10]`); an `unsafe overlay` (`[GRM-35]`, `[FFI-2]`); an
  `unsafe import c` of C source compiled into the program (`[FFI-51]`); an export that names its own
  symbol, `@unsafe(export("name"))` (`[FFI-26]`); and an `unsafe extend` (a `Send` declared by hand, an
  `unsafe interface` implemented, `[THR-8]`). No other attribute, and no profile, manifest key or
  contract, introduces another.

## I.3 Principles the language is held to

* `[PHIL-1]` Two syntactically identical declarations in the same context have identical storage,
  lifetime and synchronisation behaviour.
* `[PHIL-2]` *(changed in 0.9.9)* *(changed in 0.9.15)* No heap allocation happens that the source does
  not show. The allocating forms are: constructing a type documented to allocate (class objects,
  `Array`, `String`, `Map`, `Set`); a value for an `owns T` place (`owns Mesh(...)`, `[OWN-10]`); a value
  made shared (`shares x`, `shares mut x`, `thread_shared x`, `[RC-1]`); `copy of x` of a value that
  owns memory (`[OWN-8]`); a list, map or set literal or comprehension in a position that produces a
  heap collection; a string literal in a position that produces a `String` (`[TXT-9]`); an f-string; a
  function value whose state exceeds its inline capacity (`[CLO-10]`); and container growth. Each
  carries the `Alloc` effect and is listed by `ember inspect --alloc`.
* `[PHIL-3]` *(changed in 0.9.15)* No implicit copy of a non-`Copy` value occurs: a copy is written
  `copy of x` (`[OWN-8]`). A copy of a `Copy` value is its bits, and a `Copy` type holds no owner and no
  link (`[OWN-7]`); no copy runs any other code.
* `[PHIL-4]` *(changed in 0.9.15)* No implicit synchronisation occurs except through types documented to
  synchronise (`Mutex`, `RwLock`, `Atomic`, channels, `guarded by` fields) and the counts of
  `thread_shared` values (`[RC-4]`).
* `[PHIL-5]` Every safety check the compiler removes, it removes because it proved the check
  unnecessary. Declining to look is not a proof.
* `[PHIL-6]` Every expensive or dangerous conversion at an FFI boundary is explicit in source or
  reported by the compiler.
* `[PHIL-7]` Panics are for programmer errors. Recoverable failures use `Result`.
* `[PHIL-8]` *(changed in 0.9.15)* **The enforcement ladder.** Ember guarantees memory safety, lifetime
  safety and data-race freedom for Safe code, and proves them when compiling: ownership, views, changes
  through several names, lifetimes and threads are checked by the compiler, and nothing is paid for
  them while the program runs. A check runs while the program runs only where the property is data the
  program computes as it runs: an index in bounds, an integer that may overflow, whether a `weakref`
  link's target still lives, a downcast, a count at its limit, a resource handle's generation, a foreign
  callback's token, and a disjointness test the program writes itself (`assert_disjoint`); each has a
  row in §X.4. A program whose safety the compiler cannot prove is rejected, and the diagnostic MUST name
  the line that expresses the same intent in a way it can prove (`[DIA-25]`); an operation that neither
  can make safe requires `unsafe`.
* `[PHIL-8a]` Every rejection shape in §XVII.6 has a mandated `help` that, applied literally to the
  rejected program, produces a program that compiles. `tests/ui/` holds a before/after pair per shape.
* `[PHIL-9]` *(changed in 0.9.15)* Class objects and shared values carry a header holding their counts
  and their type (`[OBJ-1]`), and nothing else; value types have no hidden state, because `[TYP-11]`
  gives every plain struct C layout. A value that is to be changed through several names says so where
  it is made or held: several owners (`shares mut`, `[RC-7]`), several changeable links
  (`sharedref mut`, `[LT-46]`), or, between threads, `thread_shared` with a lock or an `Atomic`.
* `[PHIL-12]` *(new in 0.9.9)* **No silent acceptance.** Every construct a program writes — an
  attribute, an import, a directive, a contract, a manifest key, a statement form — either has exactly
  the effect this specification gives it, or the program is rejected. An implementation that has not
  yet built a construct rejects it with `E0900 not supported yet by this compiler`, naming the construct
  and the rule; a construct this specification does not define is `E0901`. Accepting a construct and
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
* `[PHIL-14]` *(new in 0.9.9)* *(changed in 0.9.10)* **Python spelling, Python meaning.** Where Ember accepts a spelling that Python also
  has — `/`, `//`, `%`, `**`, `a < b < c`, `x in xs`, `x is None`, `[1, 2]`, `{"k": v}`, f-strings,
  `for … else`, keyword arguments — it gives it Python's meaning, restricted to Ember's types. Where
  Ember cannot give Python's meaning at C speed, it rejects the spelling with a fix-it rather than
  giving it a different meaning silently. Integer `/` is the example: rejected, with `//` and a float
  conversion as the fixes (`[TYP-28]`). Sequence repetition (`s * n`, `[x] * n`) is rejected likewise,
  with `s.repeat(n)` and `[x for _ in 0..n]` as the fixes (`[DIA-21]`).
* `[PHIL-15]` *(new in 0.9.9)* *(changed in 0.9.15)* **Costs are named.** Every cost the language can
  insert without the programmer writing it — a bounds check, an overflow check, a count operation and
  its limit compare, a `weakref` link's header test, a type-id compare, an indirect call, an
  allocation — has a row in §X.4 and is reported per site by `ember inspect`.
* `[PHIL-16]` *(new in 0.9.15)* **The words say what the code does.** Whether a name owns its value,
  copies it, takes it over or views it, and whether it may change what it names, is written by a word on
  the line that makes the name (`owns`, `copy of`, `shares`, `ref mut`, `weakref`); a name with no word
  takes a stated default that never changes anything, never aliases silently and never copies silently
  (`[BRW-12]`, `[BRW-15]`). A keyword does the same for every type. A rule that would make a program
  compile but do something other than what its words say is not part of Ember: such a program is
  rejected, and the message names what was refused in the program's own names, says why in one line,
  and gives the exact line to write, which compiles (`[DIA-25]`). A fact the compiler works out from a
  function's body never changes what a line does: it only decides whether a line compiles, or removes a
  test that would always pass (`[EFF-1]`).

`[PHIL-8]` is a rule about **how** a guarantee is met, never about **whether**: what can be proved when
compiling is proved then, at no cost while the program runs, and a refusal always names the line to
write instead.

## I.3a The Safe Ember invariant

* `[PHIL-10]` *(changed in 0.9.9)* *(changed in 0.9.15)* A program containing no `unsafe` boundary
  (`[TIER-1]`) and no false foreign contract (`[FFI-1]`) cannot perform an invalid ownership operation,
  a use-after-free, a double release, a data race, an out-of-bounds access, an invalid range-type
  construction (`[RNG-9]`), a read of uninitialised memory, an access through an invalid view, or an
  arithmetic operation whose result C would leave undefined; it cannot end a value without its clean-up
  (`[OWN-2]`) or leak objects through a loop of owners (`[WK-6]`). **This holds in every build profile
  and under every manifest setting**; no configuration disables a check that this invariant relies on.
* `[PHIL-11]` *(changed in 0.9.15)* What remains possible, and is therefore not a defect of the
  guarantee: resource exhaustion (a panic, `[ALC-4]`), a count reaching its limit included (`[RC-1]`);
  deadlock; a logical race between correctly synchronised operations; a bug in foreign code reached
  through a correctly declared boundary; a hardware fault; deliberate termination through `panic` or
  `abort`; and any violated obligation inside `unsafe` or an inaccurate `asserted` foreign fact.

## I.4 Static proof and run-time enforcement

Which mechanism enforces a guarantee is set per property by this specification, never by the
programmer, and it is always visible (`ember inspect --safety`) and restrictable (`@no_alias_checks`,
`[EFF-12]`).

| Guarantee | Proven when compiling | Checked while running | Mechanism |
|---|---|---|---|
| No use-after-free, no dangling view | always, for views, positions and names opened from links (`[BRW-1]`, `[RC-5]`) | opening a `weakref` link tests its target's header (`[WK-12]`), except a parent-link read proved (`[WK-17]`) | the view rules, zero cost; the weak count |
| One name changes a place, or any number read it | always, for every type, class objects included (`[BRW-1]`, `[EXC-1]`) | never | the view rules; zero cost |
| Two elements of one list changed together | the positions are proved different (`if i != j:`, `j` from `i + 1..`, `[BRW-5]`); otherwise rejected | where the program writes `assert_disjoint` for two spans (`[DSJ-5]`) | the view rules; two compares the program writes |
| Changes through several owners or links | always (`[RC-7]`, `[LT-46]`) | never | the view and loop rules; zero cost |
| Index in bounds | the index comes from the container's own range, or a range fact (`[RNG-4]`) | otherwise | compare and branch |
| Integer arithmetic does not overflow | a range fact proves it | otherwise | overflow flag test (`[TYP-8]`) |
| Resource handle is live | never (generation is run-time data) | always, every profile | generation compare (`[HND-1]`) |
| Object lifetime | one owner ends it; escape analysis promotes to the stack (`[OPT-1]`) | a count for each `shares` and `thread_shared` owner, compared with its limit before each +1 (`[RC-1]`, `[RC-4]`) | owners; counts only where a line says `shares` |
| No leak through a loop of owners | always: a loop is proved opened again, or rejected (`[WK-6]`) | never | the graph of owners; zero cost |
| A downcast gives what it names | a kind test, `if s is Circle as c:`, runs its block only for a `Circle` (`[DSP-4]`) | each kind test and each `cast s as Circle` (`[DSP-6]`) | one type-id compare |
| No data race | always, via `Send`, `ThreadShared` and the thread rules (Part XI) | never | type system; zero cost |

## I.5 A complete small program

```ember
from std.math import Vec3

## A particle in a fountain simulation: a plain value, 28 bytes.
@derive(Copy)
struct Particle:
    position: Vec3
    velocity: Vec3
    lifetime: f32

## An object with identity, in its own block, with one owner.
class Emitter:
    name: String
    particles: Array[Particle] = []
    spawn_rate: f32 = 100.0

    fn init(owns name: String):
        this.name = name

    fn spawn(count: int):
        for _ in 0..count:
            this.particles.push(Particle(
                position=Vec3.ZERO,
                velocity=Vec3(0, 9.8, 0),
                lifetime=2.0))

## Contract tier: proven not to allocate.
@noalloc
fn integrate(ref mut particles: MutSpan[Particle], dt: f32):
    for ref mut p in particles:
        p.velocity.y -= 9.81 * dt
        p.position += p.velocity * dt
        p.lifetime -= dt

fn main():
    fountain = Emitter("fountain")
    fountain.spawn(10_000)
    for _ in 0..600:
        integrate(ref mut fountain.particles, 1.0 / 60.0)
        fountain.particles.retain(fn(p) => p.lifetime > 0.0)
    println(f"{fountain.name}: {fountain.particles.len()} alive")
```

*Note.* `Emitter("fountain")` passes a string literal where a `String` is expected; the literal
allocates the `String` there (`[TXT-9]`), and `owns name` takes it over. `particles: Array[Particle] =
[]` is an empty list literal in an `Array` position. `integrate(ref mut fountain.particles, ...)` says
at the call that `integrate` may change the particles; the list converts to a `MutSpan` (`[SPN-1]`),
and nothing is checked while the program runs (`[BRW-1]`). `1.0 / 60.0` takes `f32` from the parameter
it is passed to.

## I.6 Non-goals

Ember deliberately has none of these: a tracing garbage collector or a cycle collector (a loop of
owners is proved opened again or rejected, and a link back is a `weakref`, `[WK-6]`); exceptions or
unwinding (failures are values or panics, Part XIII); implicit numeric narrowing; function overloading
and C's untyped variadic functions outside `unsafe` (Ember's own take any number of typed values,
`[TYP-47]`); macros (`macro` is reserved);
named lifetimes, ever (`[LEX-22]`); an aliasing or exclusivity check while the program runs
(`[BRW-1]`); specialisation or higher-kinded types; dynamic or structural typing (`[TYP-40]`); a safety
check that depends on the profile (`[PRF-1]`); a correctness property that depends on whole-program
optimisation or LTO; a stable Ember-to-Ember ABI (the C ABI is the stable boundary, Part XVI); shader
compilation and `async` in this version (both reserved).
