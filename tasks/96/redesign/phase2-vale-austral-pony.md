# Phase 2, languages: Vale (in depth), Austral and Pony (briefly)

Written 2026-10-24 by the phase-2 "Vale / Austral / Pony" research agent, for the memory-model rework
(owner's directive 2026-10-23: C-like speed and memory, memory safety without Rust's annoying parts,
Python-like syntax). Annoyance ids are the shared A1 to A20; this report adds none.

How to read the verdicts. For each id and language:

- **removed**: the compile-time restriction is gone and nothing replaces it at run time;
- **moved to run time**: the compile-time restriction is gone and a run-time check (or a collector)
  now carries the safety, so the cost moves into every run;
- **kept**: the restriction is still there (sometimes in a different spelling, sometimes worse);
- **n/a**: the language has no such feature, so the question does not arise.

Sources are listed at the end. Every claim that comes from a source names it; a claim I could not
check against a source is marked UNVERIFIED. Code in Vale, Austral and Pony is my own, written in
each language's syntax as the sources show it; it was not compiled (no compiler for any of the three
is installed, and installing is not allowed here). One measurement was made on this laptop
(section 6); its program and raw output are in `target/redesign/phase2-vale-austral-pony/`.

## Summary

| | Vale (0.2 + the designed regions) | Valen / group borrowing (Vale's successor, 2025-2026) | Austral | Pony |
|---|---|---|---|---|
| Safety mechanism | one owner per object; every other reference is a generational reference checked on each use; regions remove the checks where data is frozen | one owner; references remember which part of the data they point into; a change invalidates only references *into* what changed | linear types (every linear value used exactly once) and Rust-like borrowing with named regions | tracing collector per actor; six reference capabilities stop data races |
| Removes (compile time, no run-time cost) | A10, A17 | A1, A3, A9 (one thread) | almost nothing | through the collector: A1, A2, A3, A5, A6, A7, A8, A9 inside an actor; A13, A14, A15 by design |
| Moves to run time | A1 to A9, A12 (one generation check per use of a non-owning reference) | A8 (planned: generational references or counting) | none found | everything memory-related moves to the collector |
| Keeps | A11 | A2, A4, partly A5 (functions declare what they change), A7, A11 | A1 to A9, A11 (worse), A16, A18, A20 | A10 (six capabilities), A11 for `iso` |
| Cost against C | published: 10.84% on one program (2021); my measurement: +23% for an in-order walk, +45% for random access through generational references, plus 8 bytes per reference | none published; the design claims zero run-time cost | none published; no run-time mechanism | published against Erlang and Akka only, never against C; no figure exists against C |
| Maturity | experimental; last release v0.2.0 pre-release; regions prototype never merged | experimental, unreleased, v0.1 announced for February | experimental; no release; last commit 2025-07-28 | shipped (0.77.1, 2026-10-22), pre-1.0, a few production users |

The result for Ember, in short (section 5 has the details):

1. **Take Valen's path-based invalidation** for local references (a smarter rule, kind b). It removes
   A1 and A3 with no run-time cost. Rust forbids two mutable references because they *might* free
   what the other points into; this rule forbids only the use of a reference after a change that
   could have freed what it points to.
2. **Take generational references, with counter generations rather than random ones**, only for references
   stored in fields (kind b). They remove the run-time count updates of today's class handles, the
   wrapper types and A8. My measurement puts their cost against C at 0.57 ns on each in-order use,
   4.6 ns on each random use, and 8 more bytes for each reference. A non-atomic reference count
   measured cheaper than that on this laptop.
3. **Take pages that never move for the ECS pools** (kind b), with a "stable" container whose
   growth does not invalidate element references. This is the remaining piece of A2/A4 for ODR-286.
4. **Take Vale's frozen-region idea in inferred form** (kind a for the spelling, b for the mechanism):
   a function that writes nothing drops every run-time check inside it, and nothing is written in
   the source.
5. **Take Pony's sendability rule spelled as "move it or freeze it"** for threads (kind a), with no
   Send/Sync traits visible.
6. **Leave out:** Pony's collector, which has no published C comparison, and its footprint was worse
   than the JVM's C4 collector in its own paper. Also leave out Austral's exactly-once rule and
   named regions, which add annoyance rather than remove it.

---

## 1. Vale

### 1.1 The model in plain words

Vale has two kinds of reference.

- An **owning reference** is the one place responsible for destroying the object. It is a plain
  8-byte pointer and needs no check, because while the owner holds it the object is alive
  (verdagon.dev/blog/single-ownership-without-borrow-checking-rc-gc, 2023-06-15).
- A **non-owning reference** (`&Ship`) may point anywhere, any number of them may exist, and you
  may change the object through any of them. There is no borrow checker for these references.
  Each one is a **generational reference**, made of a pointer and the generation number the object
  had when the reference was made. Each object carries its current generation. Every use of a
  non-owning reference compares the two numbers, and a mismatch stops the program
  (verdagon.dev/blog/generational-references, updated 2023-07-09: "Vale triggers a segmentation
  fault" on a mismatch).

```vale
struct Ship { hp int; }

func heal(s &Ship) { set s.hp = s.hp + 1; }      // generation check on s, then the write

exported func main() {
  ship = Ship(10);          // owning (no check when used through `ship`)
  a = &ship;                // non-owning, remembers ship's generation
  b = &ship;                // a second one: allowed, and both may write
  heal(a);
  heal(b);
  println(ship.hp);         // 12
}
```

What happens when the object is destroyed (generational-references article):

- **Counter generations** (the original design): freeing an object adds 1 to its generation, so
  every older reference now mismatches. A slot whose counter reaches its maximum is retired.
  Detection is deterministic.
- **Random generations** (the default in the 2023 update): each object gets a pseudo-random
  generation. Destruction overwrites it with zero, and memory can be reused for anything and given
  back to the operating system. Detection is then **probabilistic**: a stale reference passes when
  the new object's random number equals the old one. The article does not give the width of the
  random number, so the miss probability is UNVERIFIED. For the owner's "deterministic panic"
  requirement this variant is weaker than the counter.
- **Generation table** (an appendix, not built): each reference holds a 32-bit index and a 32-bit
  generation into a per-region table.

**Regions and region borrowing** (verdagon.dev/blog/zero-cost-borrowing-regions-overview,
2022-11-14; prototype in verdagon.dev/blog/first-regions-prototype, 2023-07-11). A region is a
group of data that can be frozen as a unit. While a region is frozen nothing in it can be
destroyed, so the generation checks on references into it can be dropped. A `pure` function sees
all of its arguments as one frozen region:

```vale
pure func totalHp(ships &List<Ship>) int {      // no generation check anywhere in here
  sum = 0;
  foreach s in ships { set sum = sum + s.hp; }
  return sum;
}

func copyHp<r'>(from &r'Ship, to &Ship) {       // `r'` = this argument is read-only
  set to.hp = from.hp;                          // check on `to` only
}
```

Regions are described as "an optimization hint that can be safely ignored": deleting `pure` and
`r'` changes speed, not meaning. **Isolates** (`'Cannon`) are sub-trees with nothing pointing in or
out except the owner. They can be frozen on their own (`cannon.imm`) or opened for writing
(`cannon.open`), and the compiler stops both happening at once. A "cell" (`''Cannon`) moves that
check to run time. Only immutable region borrowing was prototyped, on a branch, and the prototype
"hadn't been merged" (first-regions-prototype). Isolates, one-way isolation and region-based
concurrency are design text.

**The linear-aliasing model and own-lending** (verdagon.dev/blog/linear-types-borrowing,
2023-06-01). Any Vale program can also be written so that it has no generation checks at all. You
use only owning references and pass ownership into a function and back out ("own-lending"), much
as a C programmer passes a struct by value and gets it back:

```vale
func rollDice(rng Random) (int, Random) { ... }   // takes the generator, gives it back
[roll, set rng] = rollDice(rng);
```

The article says this brought the compiler's `--print_mem_overhead` count of checks to zero on its
sample program. It lists what this style cannot express: several long-lived references to one
object, intrusive structures, graphs, observers and callbacks, and back-references. For those you
go back to generational references or to indices into a central array. It proposes, but does not
build, an `inout` parameter spelling to hide the "take it and give it back" dance.

**Higher RAII** (verdagon.dev/blog/higher-raii-uses-linear-types, 2024-05-14; implemented, the
article says it "has become a central feature of Vale"). A struct can be declared `linear`. Such a
value cannot just go out of scope: one of the functions its author picked must destroy it, and
those functions may take arguments and return results.

```vale
linear struct Transaction { id int; }
func commit(t Transaction, db &DB) { ...; destruct t; }
func rollback(t Transaction, db &DB) { ...; destruct t; }
// forgetting to call commit or rollback on every path is a compile error
```

Its costs, from the same article, are that linearity spreads (a list of linear values is linear),
that library functions which drop elements need redesign, and that panics cannot run a linear
value's destroyer.

**Hybrid-generational memory** (verdagon.dev/blog/hybrid-generational-memory, 2021-01-02) is a
design that was never built. A local that uses an object several times sets a "tethered" bit on it.
While the bit is set the owner may not free the object, so the local's uses need no checks.

**The successor: Valen and group borrowing.** The author worked on the Mojo compiler from July 2024
to December 2025 (verdagon.dev home page). Since September 2026 he has been building Valen, an
"extremely experimental" language with Rust interop. Its compiler was made partly by moving Vale's
compiler to Rust, and **generational references are "temporarily disabled" in it**; group borrowing
is its safety mechanism (verdagon.dev/blog/golden-spike-reviving-vale-valen, 2026-09-17). Nick Smith
designed group borrowing for Mojo, and the design "hasn't been implemented yet" there
(verdagon.dev/blog/group-borrowing, 2025-08-28). Valen implements it, except for closures
(golden-spike; verdagon.dev/blog/valen-group-borrowing, 2026-10-11;
verdagon.dev/blog/reverse-borrow-checking, 2026-10-18). Its rules, from those posts:

1. Every value has one owner, so every object has one path (`world.entities[3].rings[0]`).
2. A reference remembers the path of what it points to.
3. Changing an object invalidates references *into its child groups*: the elements of a list it
   owns, the payload of a variant, anything behind a pointer it owns. References to the object
   itself, and to its inline fields, stay valid, because those cannot be freed by the change.
4. A function's signature says which paths it may change (`mut`). Calling it invalidates
   references into those paths in the caller.
5. Using a reference after it was invalidated is a compile error.

So "shared XOR mutable" becomes "no use after a change that could have freed it":

```valen
fn attack(a: &Entity mut, d: &Entity mut):   // a and d may be the same entity
    d.hp = d.hp - a.power
attack(world.entities[0], world.entities[1]) // accepted (Rust: E0499)
attack(world.entities[0], world.entities[0]) // accepted: an entity may attack itself

hp = &d.hp
d.take_damage(10)        // hp is an inline field of d: still valid
ring = &d.rings[0]
d.add_ring(Ring())       // may grow d.rings: ring is invalidated
print(ring.power)        // compile error
```

(The syntax is my paraphrase; the posts use several spellings, `ref[r]`, `&T mut`, `in x[] mut`.)

What it still forbids, from group-borrowing: copying between two places that might alias
(`p1[] = p2[]`; the post suggests a `distinct(x, y)` run-time check), owning cycles, and any use of
an element reference after a possibly-growing call. Graphs ("arbitrary object graphs") are left to
a later post, with generational references or counting described as run-time groups whose
references are never invalidated (valen-group-borrowing). A "type-stability exception", under which
type-stable data such as `my_vec.size` would not be invalidated, "isn't implemented in Valen yet". The
checker is about 3,600 lines, walks each function backwards, does not yet support `break`,
`continue` or closures, and its author offers a prize for a soundness flaw (reverse-borrow-checking).

### 1.2 A1 to A20 in Vale

Vale (generational references + regions) first, then the Valen column where group borrowing differs.

| id | Vale | how | Valen (group borrowing) |
|---|---|---|---|
| A1 whole-object borrow | moved to run time | any number of `&` to an object and its fields; each use pays a check unless in a frozen region | **removed** (compile time; inline fields survive changes to the owner) |
| A2 container invalidation | moved to run time | Vale 0.2 puts every non-primitive struct on the heap (generational-references: "only heap allocation for non-primitive objects"), so growing a list moves pointers, not objects, and element references survive. Removing an element destroys it and later uses halt. For inline elements, the single-ownership article says structs "share their container's key", so a reallocated array would make element references halt (UNVERIFIED for arrays) | **kept** (an element reference dies at a growing call; the reference to the list survives) |
| A3 two `&mut` into one container | moved to run time | `a = &ships[0]; b = &ships[1]` both writable | **removed** |
| A4 change while iterating | moved to run time | allowed; an element removed during the loop halts if used later. Iterator semantics on growth UNVERIFIED | kept |
| A5 returning references | moved to run time | `func first(l &List<Ship>) &Ship` with no annotation; a stale result halts. With regions, `r'` markers come back as optional speed hints | partly kept: the signature names the paths the function changes, and the result's path is inferred |
| A6 structs holding references | moved to run time | `struct Ship { dock &Dock; }`; `Ship<x'>` only for isolates | UNVERIFIED (posts do not cover it) |
| A7 self-referential | moved to run time | heap objects do not move when their owner moves | kept |
| A8 graphs, parents, cycles | moved to run time | children owned, parent as `&Node`; non-owning cycles are free; the ownership must still be a tree | deferred to run-time groups (planned) |
| A9 shared mutable state | moved to run time | no RefCell: share `&` and write; dangling use halts | **removed** within one thread |
| A10 ownership kinds | **removed** | two kinds only: owning and `&` (plus linear structs as an opt-in) | removed |
| A11 moves | kept | single ownership: use after move is a compile error; fewer clones because `&` can be stored | kept |
| A12 closures | moved to run time | a closure may hold `&` to anything; a stale capture halts (UNVERIFIED: no source read covers closures) | kept for now (closures not supported by the checker) |
| A13 trait objects, downcasting | n/a for the memory model | Vale has interfaces; downcasting not covered by the sources read (UNVERIFIED) | n/a |
| A14 async | n/a | no async; "concurrency strategy" is listed as open work (vale.dev/contribute via search) | n/a |
| A15 threads | design only | `parallel foreach`: everything made outside the block is read-only inside it, so threads share it with no locks; writes only through channels, mutexes, atomics (seamless-fearless-structured-concurrency, 2022-02-15: "a theoretical feature, and not implemented yet") | n/a |
| A16 generics friction | n/a | not part of the memory model | n/a |
| A17 borrow-error messages | **removed** in the base model (there are no borrow errors) | the regions prototype's errors are "very, very verbose" and some crash the compiler (first-regions-prototype) | unknown |
| A18 initialisation | n/a | not covered by the sources | n/a |
| A19 global mutable state | design only | every global lives in a mutex that contains an isolate (zero-cost-borrowing-regions-overview) | n/a |
| A20 unsafe and FFI | moved to the boundary | "Fearless FFI": safe and unsafe memory are separate, references across are "scrambled", data crosses by copying (vale.dev/memory-safe); how much is built is UNVERIFIED | Rust interop: a reference becomes `&mut` only when it is the sole path to the data for the call |

The ECS case that ODR-286 is about, in Vale:

```vale
struct Pool<T> { dense List<T>; }
struct Registry { pools HashMap<i64, Pool<Pos>>; }

func get(reg &Registry, i int) &Pos {        // no lifetime, no E3060
  pool = reg.pools.get(7).get();             // non-owning reference into the map
  return &pool.dense.get(i);                 // generational reference to the element
}

p = get(&reg, 1);
emplace(&reg, 3, Pos(0, 0));                 // allowed: no borrow checker
set p.x = p.x + 1;                           // checked: halts if the element was destroyed
```

The function compiles as written. The price is a check on every `p.x`, plus a heap object per
element (Vale 0.2) or a run-time halt when the array reallocates (inline elements, UNVERIFIED).
EnTT's paged storage would remove that halt: pages never move.

### 1.3 Costs

**Run-time cost per operation.**

- Owning references: none (a plain pointer).
- A non-owning reference costs one load of the object's generation and one compare-and-branch per
  use. The check reads the same cache line as the field, says the author (myth-zero-overhead,
  2023-06-19). The tethering bit in hybrid-generational memory would have cut repeated checks, but
  it was never built.
- Making or copying a reference has no count update (generational-references), unlike reference
  counting.
- Frozen regions cost nothing at run time. They cost compile time and code size, because the
  compiler generates separate read-only and mutable versions of functions
  (zero-cost-borrowing-regions-overview).

**Memory cost.** One generation word per allocation: 8 bytes in the 2023 article and 48 bits in
the 2021 design. A non-owning reference is 16 bytes instead of 8; Vale's non-owning references are
"128 bits" (single-ownership article). Owning references stay 8 bytes, and the article says most
pointers are owning.

**Published measurements** (all by Vale's author, one program each, machine and compiler mostly
not stated):

| What | Result | Source |
|---|---|---|
| BenchmarkRL terrain generator, whole program: unsafe (no checks) vs naive RC vs generational references, no regions, every object on the heap | unsafe 43.82 s; RC 54.90 s (+25.29%); GR 48.57 s (+10.84%) | generational-references (2021 numbers, republished 2023-07-09); machine not stated |
| the same figure, restated | "2% to 10.84%" overhead; the 2% has no stated source | zero-cost-borrowing-regions-overview |
| cellular-automata level generator: no checks at all vs bounds checks only vs Vale's safe mode with regions | no-checks build 1.18 ± 0.01 times faster than *both* the bounds-checked build and the safe build, so on this program generation checks cost nothing beyond the bounds checks once regions froze the data | first-regions-prototype, 2023-07-11; Razer Blade 15 (2018), Ubuntu 22.04, hyperfine under a cset shield |
| own-lending on a sample program | `--print_mem_overhead` reports zero checks; no timing | linear-types-borrowing |
| comparison figures quoted by the author for other languages (Java 89%, Go 183%, Swift 320%) | from an external article he cites, not his own runs; not checked here | zero-cost-borrowing-regions-overview |

Neither source states which allocator, generation width or check variant (counter or random) the
10.84% used beyond "resilient-v3" mode.

**My measurement on this laptop** (section 6): a C model of the dereference alone, with 4,194,304
separately allocated 16-byte objects, clang 22.1.8 `-O2`, on a pinned P-core and mains power, median
of 7 runs:

| access | plain C pointer | 16-byte reference, no check | generational reference (checked) | non-atomic RC take+drop per use |
|---|---|---|---|---|
| in order, 83,886,080 uses | 203.5 ms (2.43 ns/use) | 229.3 ms (+12.7%) | 251.1 ms (**+23.4%**, +0.57 ns/use) | 232.3 ms (+14.2%) |
| random, 16,777,216 uses | 173.6 ms (10.35 ns/use) | 204.0 ms (+17.5%) | 250.9 ms (**+44.5%**, +4.61 ns/use) | 207.1 ms (+19.3%) |
| peak memory | 168,928 KB | 201,696 KB | 201,696 KB | 168,932 KB |

What this shows:

1. About half the in-order cost, and 40% of the random cost, is the bigger layout alone: references
   twice the size, in an array twice as large.
2. The compare adds the rest: 0.26 ns per in-order use and 2.80 ns per random use. Why the compare costs
   that much in the random loop was not established here; it is UNVERIFIED whether it comes from the
   branch holding back the next independent loads, or from something else.
3. In this test a non-atomic count update per use was **cheaper** than the generation check. The test
   does not show what counting costs elsewhere: count updates when references are stored, copied and
   dropped across a real program, cache misses on objects not otherwise touched, atomics when shared
   between threads. Vale's own whole-program figure (25.29% for RC against 10.84% for generations)
   includes those.
4. Memory: +32,768 KB, which is exactly 4,194,304 references × 8 extra bytes. The 8-byte generation
   in each object cost nothing measurable, because the Windows heap gave the 16-byte and the 24-byte
   objects the same peak. That the heap rounds both sizes to one block size is my inference, not a
   measurement. With a tight pool allocator, the header would cost 8 bytes per object.

**What Vale cannot express, or makes slow.**

- Values stored inline inside containers: the 2023 prototype still put structs on the heap. An ECS
  pool of inline components is therefore either an array of pointers, which pays a pointer chase
  per element against C's dense array, or relies on unbuilt design.
- Threads: nothing built. Generations are plain integers, so sharing mutable objects between
  threads would need atomics or the unbuilt freeze design.
- Hard real-time code that cannot accept a halt: a dangling use is found only when it happens.
- Random generations: detection is probabilistic, as described in 1.1.

### 1.4 Maturity

Vale is **experimental**. The last release is v0.2.0, marked "Pre-release" on GitHub (its post,
"Vale 0.2 Released", is on verdagon.dev; the year is not shown on the release page). The roadmap
on vale.dev still calls 0.2 current and plans 0.6.1 for "early 2025" (seen through search, not
fetched directly). The regions prototype stayed on a branch. Its author worked on Mojo from
2024-07 to 2025-12, and in 2026 started Valen, which drops generational references for now. Vale
has no production users that I found. Group borrowing is a design (Mojo, 2025) and an unreleased
implementation (Valen, 2026, with v0.1 announced for February).

---

## 2. Austral

### 2.1 The model in plain words

Austral (Fernando Borretti; "Introducing Austral", borretti.me, 2022-12-28; the spec at
austral-lang.org/spec/spec.html) has no collector, no counts, no destructors and no run-time
safety mechanism. Types live in two "universes":

- **Free** types (integers, floats, records of free types, and references) can be used any number of
  times.
- **Linear** types (pointers, files, sockets, and anything that contains a linear type) must be
  used **exactly once**. Not using one is an error, and so is using one twice. A linear variable
  from outside an `if` must be used once in every branch or in none. A linear variable from outside
  a loop may not appear in the loop at all.

Destroying a value is an explicit call. To use a value without consuming it you borrow it in a
statement that names a region, and the reference cannot leave that statement:

```austral
let f: File := openFile("log.txt");
let f1: File := writeString(f, "first");     -- f is used up; f1 is the new handle
borrow f1 as r in R do                       -- read-only reference r: &[File, R]
    printSize(r);                            -- f1 itself may not be used in here
end;
borrow! f1 as w in W do                      -- writable reference w: &![File, W]
    truncate(w);
end;
closeFile(f1);                               -- forgetting this is a compile error
```

`&x` and `&!x` are anonymous borrows that cannot escape their statement. A function that takes a
reference names the region as a generic parameter, for example
`generic [R: Region] function Get_Filesystem(root: &[Root_Capability, R]): Filesystem;`. Reference
types are Free (`type Reference[T: Linear, R: Region]: Free;`). The spec does not say whether a
record may hold a reference. Code that touches raw memory or C must be in a module marked
`pragma Unsafe_Module;`, and the build can list every such module for audit. Capabilities such as
`Filesystem` are linear values that must be passed down explicitly.

### 2.2 A1 to A20 in Austral

| id | verdict | how |
|---|---|---|
| A1 | kept (worse) | borrowing `x` makes `x` unusable inside the borrow statement; paths through a reference (`x->y`) give references to fields, but the whole variable is locked |
| A2 | kept | a writable borrow of a buffer is the only way to change it; an element reference lives in a borrow statement on the same buffer. Growable containers are not in the spec |
| A3 | kept | one writable borrow of a variable at a time (same lock as A1) |
| A4 | kept (worse) | a linear variable from outside a loop cannot appear in its body at all; you borrow it outside the loop and use the reference |
| A5 | kept (worse than Rust) | regions are written: `generic [R: Region]` and `&[T, R]`. Whether a function may return a reference into an argument's region is not stated in the spec (UNVERIFIED) |
| A6 | kept / UNVERIFIED | the spec does not say whether records may hold references |
| A7 | kept | impossible without unsafe code |
| A8 | kept | no shared ownership in the safe language; graphs need indices (inference from the model, UNVERIFIED as library practice) |
| A9 | kept | no shared mutable state, no counting in the spec |
| A10 | removed, in a way | one rule (linear or free) instead of Box/Rc/RefCell; but everything that owns memory is linear |
| A11 | kept (worse) | every linear value must be used exactly once and destroyed by an explicit call; a handle is re-bound after every operation (`f1 := writeString(f, ...)`) |
| A12 | n/a | no closures; function pointers `Fn[...]` only, always Free |
| A13 | n/a | no dynamic dispatch in the spec; typeclasses resolve statically |
| A14 | n/a | no async ("no first-class async", Introducing Austral) |
| A15 | n/a | no concurrency model in the spec; it argues linear values are safe to hand between threads |
| A16 | kept | typeclass instances must be globally unique, with no overlap (the orphan-rule family) |
| A17 | n/a | the rules are few and local; no measured or quoted message quality found |
| A18 | kept | not addressed |
| A19 | kept (forbidden) | no global state at all (Introducing Austral's anti-feature list) |
| A20 | moved to module boundary | `pragma Unsafe_Module;` per module; the build lists every unsafe module |

### 2.3 Costs

At run time Austral adds nothing: no check, no count, no header. It compiles to C
("generated C should be compiled with `gcc -fwrapv`", per the GitHub README). Whether array indexing is
bounds-checked is UNVERIFIED. The cost moves into the program's shape instead. Handles are threaded
through every call, data structures become index-based, and every resource needs an explicit
destroy call. I found **no published benchmark** of Austral against C or anything else. It cannot
express closures, dynamic dispatch, shared mutable objects, global state or long-lived references
in safe code.

### 2.4 Maturity

Austral is **experimental**: a whole-program compiler in OCaml that emits C, with no separate
compilation, no tagged release, about 1.6k GitHub stars and no listed users (github.com/austral/austral).
The latest commit is 2025-07-28, "Fix Bugs in Buffer stdlib impl" (GitHub API, fetched 2026-10-24).

---

## 3. Pony

### 3.1 The model in plain words

Pony solves memory safety with a **tracing collector** and data races with **reference
capabilities**. Every reference carries one of six capabilities (tutorial.ponylang.io, "Reference
Capabilities" and "Guarantees"):

| capability | you may | others may | can be sent to another actor |
|---|---|---|---|
| `iso` | read, write | nothing, anywhere | yes |
| `trn` | read, write | read, in the same actor | no |
| `ref` | read, write | read and write, in the same actor | no |
| `val` | read | read, anywhere; nobody writes, ever | yes |
| `box` | read | the same actor may write | no |
| `tag` | compare identity, send messages | anything | yes |

Classes default to `ref`, which behaves like Python or Java objects inside one actor: any number of
aliases, all writable. An **actor** has its own heap and runs one message ("behaviour") at a time,
so `ref` aliases inside an actor can never race. Only `iso`, `val` and `tag` may cross between
actors. `consume x` moves a unique value out of a variable. `recover ... end` builds a value from
sendable inputs only, and so can lift its result to `iso` or `val`. Return types written as
`this->T` ("viewpoint adaptation") take the capability of the receiver.

```pony
class Player
  var hp: U32 = 10

actor Main
  new create(env: Env) =>
    let p = Player              // Player ref
    let cam = p                 // second alias, also writable
    cam.hp = cam.hp - 1         // fine: same actor, no borrow rules
    let msg: Array[U32] iso = recover Array[U32] end
    msg.push(1)                 // allowed on iso: argument is sendable
    worker.take(consume msg)    // msg is unusable afterwards
```

Memory is managed by **ORCA**. Each actor collects its own heap between behaviours, never during
one, with no stop-the-world step and no read or write barriers. Objects shared between actors are
tracked with deferred, weighted reference counts that travel in messages. When a message carries a
mutable object graph, the runtime traces it on send and on receive (Clebsch et al., "Orca: GC and
Type System Co-Design for Actor Languages", OOPSLA 2017; tutorial, Garbage Collection appendix).

### 3.2 A1 to A20 in Pony

| id | verdict | how |
|---|---|---|
| A1 | removed (by the collector) | `ref` aliases to an object and its fields, all writable, inside one actor |
| A2 | removed (by the collector) | an array of objects holds references; pushing reallocates the array, but the objects stay where they are and stay alive |
| A3 | removed | two writable `ref`s to two elements, or to one |
| A4 | moved to run time | changing an array while iterating is memory-safe; index-based iteration may skip or hit an out-of-range index, which is an error the code must handle (array `apply` is partial, `?`; UNVERIFIED that the iterator behaves exactly so) |
| A5 | removed | return any reference; `this->T` replaces lifetimes as the annotation for "the receiver's capability" |
| A6 | removed | fields hold references; each field states a capability (`var buddy: Wombat iso`) |
| A7 | removed | collected heap |
| A8 | removed | cycles within an actor are collected; cycles of blocked actors are found by a cycle-detector actor |
| A9 | removed inside an actor; across actors there is no shared mutable state at all, only messages | no RefCell, no Mutex (the guarantees page: "Pony needs no locks") |
| A10 | kept, in another form | six capabilities, `consume`, `recover`, arrow types: a new set of choices to learn |
| A11 | kept for `iso` | `consume` is a move, and use after `consume` is a compile error; a field cannot be consumed, only swapped out by "destructive read" (`b = buddy = consume a`) |
| A12 | removed with a catch | a lambda that captures references is `ref` (not sendable); it may change captured objects, but assigning to a captured local does not change the outer local (the tutorial's example prints `Count: 0`) |
| A13 | removed | `match x \| let p: Pool => ...` tests the run-time type, and the capability travels with the match |
| A14 | n/a (replaced) | no async/await; actors and behaviours are asynchronous by construction |
| A15 | removed | data-race freedom checked by capabilities; one scheduler thread per core |
| A16 | n/a | not part of the memory model |
| A17 | UNVERIFIED | capability errors are reported as hard to read; I found no source with measured or quoted examples |
| A18 | UNVERIFIED | not read in the sources |
| A19 | UNVERIFIED | Pony is said to have no global variables; not confirmed in a source read here |
| A20 | kept | FFI declared with `use @name[Ret](...)`; the compiler trusts the declared C signature ("you MUST get the parameter and return types right"); C pointers are `Pointer[T] tag` |

### 3.3 Costs

- **Per operation**: within an actor, a reference use is a plain pointer use (no check, no count;
  ORCA has no barriers). Sending a mutable object graph to another actor costs a trace of the graph
  on send and on receive. In ORCA's `heavyRing` benchmark this tracing reached **21% of total
  execution time** for the largest graph (depth 16, 4 cores), and passing the graph as `val`
  removed it ("almost for free") (ORCA paper, section 6.6).
- **Memory**: the ORCA paper's footprint (maximum resident set size) was **in general worse than the
  JVM's C4 collector**, because Pony collects only between behaviours and keeps reference-counting
  tables (section 6.7). A behaviour that allocates in a long loop keeps all of it until it ends
  (tutorial, Garbage Collection). The per-object header size was not found in the sources read
  (UNVERIFIED).
- **Measurement setup** of that paper: AMD Opteron 6276 (32 cores, 8 NUMA nodes, 128 GB), Debian 8,
  Pony 0.8.0 against Erlang 18.3 and Scala 2.11.6/Akka on OpenJDK 1.8, micro-benchmarks only (the
  authors call them "not enough for a comprehensive evaluation"). **No comparison with C exists in
  it, and I found none elsewhere.**
- **What it cannot do**: deterministic destruction (memory is freed by the collector); sharing a
  mutable structure between threads (only messages); a C-sized footprint.

### 3.4 Maturity

Pony is **shipped but pre-1.0**. ponyc 0.77.1 was released on 2026-10-22 (GitHub API). The project
says Pony is used in production and "semi-regularly introduces breaking changes" (README text seen
through search). Wallaroo Labs built its stream processor in Pony (an OSCON 2018 abstract, seen
through search). Current corporate use is UNVERIFIED.

---

## 4. One example across the three: the EnTT `get` that ODR-286 is about

The job: `get[T](e)` finds the pool in a map and returns a writable reference to the component.
Then `emplace` runs while that reference is in use (the ecs4 probe
`target/analysis/ecs4/accept_change_during_loop.em`).

| | `get` returning a reference through a map lookup | `emplace` while the reference is live |
|---|---|---|
| Ember today | E3060 with class pools; compiles with `Box` pools (`target/analysis/ecs5/box1.em`) | refused (`box_bad.em`: E3025) |
| Vale | compiles, no annotations | compiles; the next use of the reference is checked, and halts if the element was destroyed (or, for inline elements, if the array moved: UNVERIFIED) |
| Valen | compiles; the result's path is "somewhere inside `reg.pools`" | refused at compile time, like Rust, unless the pool is "type-stable" (not implemented) |
| Austral | the reference cannot leave the borrow statement unless the region is a parameter; returning it through a map lookup is not shown to be possible (UNVERIFIED) | refused: the registry is borrowed |
| Pony | compiles (`fun ref get(i: USize): this->T ?`) | compiles; the collector keeps everything alive; every component is a separate heap object |

None of the three removes the real hazard (a growing array moves its elements) without paying for it
somewhere: a check (Vale), a refusal (Valen, Austral, Ember today), or a heap object per element plus
a collector (Pony). EnTT's own answer is pages that never move. The phase-2 Hylo/C# agent measured
that on this laptop (`target/redesign/phase2-hylo-csharp/results.txt`, clang -O2, CPU 1): appends
were faster with 4096-element pages (about 25 ms against 35 ms), and in-order sums were slower (about
74-86 ms of work against 58-69 ms; with 65536-element pages, 62-70 ms).

---

## 5. What Ember could take

Each idea names the owner's kind: **(a)** a more Pythonic spelling of Rust's guarantee, or **(b)**
a smarter mechanism that makes the hazard impossible or harmless, so the rule is not needed.

### 5.1 Path-based invalidation instead of "shared XOR mutable" (from Valen / Mojo group borrowing): kind (b)

Rust's rule guards against one hazard: a write through one reference frees memory that another
reference points into. Path-based invalidation keeps that guarantee and drops everything else:
references to an object, and to its inline fields, stay valid through any change to the object.
Only references *into* a child group (container elements, variant payloads, boxed contents) are
invalidated by a call that may change that child group.

```ember
fn attack(a: ref mut Entity, d: ref mut Entity):     # may be the same entity: fine
    d.hp -= a.power

attack(world.entities[0], world.entities[1])         # today: refused as two `ref mut`
hp = ref mut player.hp
player.take_damage(3)                                # hp is inline: still valid
hp += 1
ring = ref player.rings[0]
player.rings.push(Ring())                            # may move the elements
print(ring.power)  # error: `ring` points into player.rings, which `push` on line 9 may have
                   # moved; take the reference again after the push
```

- Removes A1, A3, A9 (within one thread), most of A17 (the error names the changing call), and
  parts of A12 (callbacks that receive the object that holds them, phase 1's A32).
- Cost against C: **zero at run time** (compile-time only). Possible loss: the optimiser can no
  longer assume two `ref mut` do not alias (no `noalias`), which the group-borrowing post itself
  names as a possible slowdown. Not measured; it should be measured on Ember's own benchmarks
  before adoption.
- What a function must say: which of its parameters' child groups it may change. The Pythonic
  spelling is to infer this inside a module and print it in the signature only at module
  boundaries, the way Ember infers regions today. This is a proposal, not something Valen has
  shown.
- Risks: unimplemented in Mojo, unreleased in Valen, no soundness proof, closures not handled,
  and aliasing copies (`a[] = b[]`) need a run-time `distinct` check. Threads need a separate rule
  (5.5).

### 5.2 Generational references for references stored in fields: kind (b)

Make the hazard harmless instead of forbidding it. A reference stored in a field (a parent pointer,
an observer, a camera's target, a reference into an ECS pool) carries the object's generation,
and each use is checked. A dangling use is a deterministic panic, never a use-after-free.

```ember
class Node:
    parent: Node              # a stored reference: generation-checked on use; no Weak, no Rc
    children: Array[Node]     # owned

fn root_of(n: Node) -> Node:  # returns a stored-style reference: no lifetime to write
    while not n.is_root():
        n = n.parent          # one generation check here
    return n
```

- Removes A5 (for stored results), A6, A7, A8 and A10 (no Rc, Weak, RefCell or handle-vs-box
  choice). Combined with 5.1 it removes A9.
- Cost against C, measured here (section 6): **+0.57 ns per in-order use (+23%), +4.61 ns per random
  use (+45%)** and **8 bytes more per stored reference**, plus 8 bytes per object unless the
  allocator rounding absorbs it, as it did in this test. Vale's published whole-program figure:
  +10.84% (one program, 2021).
- Use counter generations (deterministic), not random ones (probabilistic). Counter generations
  mean a freed slot is reused only for an object of the same size, or is retired when its counter
  wraps; that is how Vale's original design works.
- Against Ember today: class handles pay count updates on copy and drop, plus a run-time access
  check per field ([EXC-18]). In my test a non-atomic count update per use cost about as much as
  the bigger layout and less than the generation check, so the gain is not "generations are
  cheaper". The gains are no count traffic when references are copied and stored, no cycles to
  leak, and no wrapper types. This trade must be measured on Ember's benchmarks before choosing.
- Local and temporary references stay statically checked (5.1) at zero cost, so the check applies
  only where C code would hold a long-lived pointer.

### 5.3 Pages that never move for container elements that are referenced: kind (b)

The ECS gap in ODR-286 is that adding to a pool moves its elements. EnTT pages its pools; Valen
plans a "type-stability exception". In Ember this would be a container whose growth never moves
an element (`PagedArray`, or a pool flag), so path-based invalidation does not invalidate element
references on `push`. Only `remove` and `clear` do.

```ember
for e, pos in reg.view_mut[(Pos,)]():
    if pos.x > 100:
        reg.emplace(spawn_child(e), Pos(0, 0))   # pools are paged: pos stays valid
    pos.x += 1
```

- Removes A2 and A4 for appends (despawn still needs collect-then-remove, or a generation-checked
  handle from 5.2).
- Cost against C: the cost of EnTT's own layout, which C++ users accept. Measured by the Hylo/C#
  agent (section 4): faster appends, slower in-order sums (by about 25% with 4096-element pages
  and close to equal with 65536-element pages; the ranges are in section 4).

### 5.4 Inferred freezing (Vale's `pure` / region borrowing, without the annotation): kinds (a) and (b)

While a function or loop writes nothing reachable from an argument, nothing can be freed, so every
run-time check on that data can be dropped. That covers 5.2's generation checks and Ember's
per-field access checks on class objects. Vale makes the programmer write `pure` or `r'`. Ember can
infer it, because it already analyses which parameters a function writes (`mut` parameters).

```ember
fn total_hp(ships: Array[Ship]) -> int:     # writes nothing: compiled with no checks
    return sum(s.hp for s in ships)
```

- Removes the run-time cost of 5.2 in read-only code. Vale's prototype measured no cost beyond
  bounds checks on its benchmark (first-regions-prototype, section 1.3).
- Cost: compile time and code size, since a function may need a checked and an unchecked copy;
  Vale monomorphises both (zero-cost-borrowing-regions-overview). Not measured.

### 5.5 Threads: "move it or freeze it" (Pony's `iso`/`val`, spelled the Python way): kind (a)

Pony's rule is that only unique (`iso`), deeply immutable (`val`) or opaque (`tag`) data may reach
another thread. Ember can keep the guarantee and drop the vocabulary. A value passed to a thread
must be moved (the sender loses it) or frozen. Freezing a uniquely owned value is a type change
with no copy, which is what Pony's `recover` proves. Shared mutable state between threads lives
only inside a `Mutex`, which owns its data (Vale's design: every mutex contains an isolate).

```ember
table = build_table()                 # uniquely owned
frozen = freeze(table)                # no copy; table is gone, frozen is read-only forever
for i in range(8):
    spawn(worker, frozen, move chunks[i])
```

- Removes A15's Send/Sync noise and, with the `Mutex`-owns-data rule, the guard-escape hole that
  phase 1 found (A33). Generational references (5.2) stay single-thread data unless frozen. A
  frozen object can never be freed while shared, so its references need no checks.
- Cost against C: zero for the checks; freezing is free. Shared counts for frozen data that
  outlives the spawning scope would be atomic. Not measured.

### 5.6 Smaller items

- **Higher RAII / must-use types (Vale)**: kind (b) for forgotten cleanup. It is not needed for the
  owner's three goals, so skip it unless asked; its known costs (linearity spreads, interaction with
  panics) are in 1.1.
- **Listing the unsafe modules (Austral's `Unsafe_Module`)**: a build report of every module that
  uses raw pointers or FFI. Cheap and useful for A20, but outside the memory model itself.

### 5.7 What not to take

- **Pony's collector**: there is no published comparison with C, its footprint was worse than C4's in
  its own paper, and memory is freed only between behaviours. It fails the "memory like C" goal.
- **Austral's exactly-once rule and written regions**: every resource needs an explicit destroy
  call, and regions are spelled `generic [R: Region]` and `&[T, R]`. This is more annotation than
  Ember has today, so it moves away from goal 3.
- **Pony's six capabilities as written syntax**: they are the A10 problem in a new form; 5.5 keeps
  the guarantee with two words.
- **Random generations**: they make detection probabilistic, which breaks the "deterministic panic"
  requirement.

---

## 6. The measurement

`target/redesign/phase2-vale-austral-pony/genref.c` models one dereference four ways over 4,194,304
separately `malloc`'d objects (16-byte payload):

- `raw`: a plain C pointer.
- `fat`: a 16-byte reference with a generation word in the object, but no check.
- `genref`: the same layout, plus the check `if (r.p->gen != r.gen) halt`.
- `rc`: add 1 to the object's count on taking the reference and subtract 1 on dropping it, with a
  free at 0.

`*_seq` walks the reference array in order 20 times (83,886,080 uses). `*_rand` makes 16,777,216
uses at random indices from the shared generator in `target/redesign/bench/harness.h`. The
generation is computed from the index, so every mode draws the same random numbers. All checksums
match across modes, which shows every mode did the same work.

- Build: `clang -O2 genref.c -o genref.exe` (clang 22.1.8).
- The program pins itself to logical CPU 1, a P-core per `docs/AUTOPILOT.md`. The laptop was on
  mains power (Win32_Battery status 2, 100%), with an Intel Core Ultra 9 275HX.
- 7 rounds were run, each mode in its own process so that peak memory belongs to that mode.
- `work_ms` times the work after the fill, and the tables above use the median of 7. Raw output:
  `target/redesign/phase2-vale-austral-pony/results.txt` (2026-10-24).
- Some single runs were much slower (for example, `genref_rand` max 508 ms against a median of
  250.9 ms; `raw_rand` max 340.9 ms against 173.6 ms). The medians are not affected.

Earlier runs, overwritten, were made while I fixed the program. The first run let the generation
consume the random generator, so its checksums differed between modes. The second put the check
behind a per-iteration choice. Their numbers are not used.

What the model leaves out: Vale's real allocator and object layout; the cost of count updates on
stores and copies across a real program; and atomics. It measures the dereference alone.

## Sources

- Vale, generational references (updated 2023-07-09): https://verdagon.dev/blog/generational-references
- Vale, regions overview (2022-11-14): https://verdagon.dev/blog/zero-cost-borrowing-regions-overview
- Vale, first regions prototype (2023-07-11): https://verdagon.dev/blog/first-regions-prototype
- Vale, linear types and own-lending, the "linear-aliasing model" (2023-06-01): https://verdagon.dev/blog/linear-types-borrowing
- Vale, Higher RAII (2024-05-14): https://verdagon.dev/blog/higher-raii-uses-linear-types
- Vale, hybrid-generational memory (2021-01-02): https://verdagon.dev/blog/hybrid-generational-memory
- Vale, single ownership without borrow checking (2023-06-15): https://verdagon.dev/blog/single-ownership-without-borrow-checking-rc-gc
- Vale, the myth of zero-overhead memory safety (2023-06-19): https://verdagon.dev/blog/myth-zero-overhead-memory-safety
- Vale, structured concurrency (2022-02-15): https://verdagon.dev/blog/seamless-fearless-structured-concurrency
- Vale, memory safety page: https://vale.dev/memory-safe
- Vale releases: https://github.com/ValeLang/Vale/releases
- Group borrowing (2025-08-28): https://verdagon.dev/blog/group-borrowing
- Valen announcement (2026-09-17): https://verdagon.dev/blog/golden-spike-reviving-vale-valen
- Valen group borrowing (2026-10-11): https://verdagon.dev/blog/valen-group-borrowing
- Valen reverse borrow checking (2026-10-18): https://verdagon.dev/blog/reverse-borrow-checking
- Austral introduction (2022-12-28): https://borretti.me/article/introducing-austral
- Austral spec: https://austral-lang.org/spec/spec.html
- Austral compiler: https://github.com/austral/austral (commits via the GitHub API)
- Pony capabilities: https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html
- Pony guarantees: https://tutorial.ponylang.io/reference-capabilities/guarantees.html
- Pony recover: https://tutorial.ponylang.io/reference-capabilities/recovering-capabilities.html
- Pony consume: https://tutorial.ponylang.io/reference-capabilities/consume-and-destructive-read.html
- Pony arrow types: https://tutorial.ponylang.io/reference-capabilities/arrow-types.html
- Pony lambdas: https://tutorial.ponylang.io/expressions/object-literals.html
- Pony match: https://tutorial.ponylang.io/pattern-matching/match.html
- Pony FFI: https://tutorial.ponylang.io/c-ffi/calling-c.html
- Pony GC: https://tutorial.ponylang.io/appendices/garbage-collection.html
- ORCA paper (OOPSLA 2017): https://www.doc.ic.ac.uk/~scd/ORCA_17.pdf (text extracted locally with a
  stdlib script, because the PDF reader was missing: `pdftoppm` is not installed)
- Pony releases: https://api.github.com/repos/ponylang/ponyc/releases
