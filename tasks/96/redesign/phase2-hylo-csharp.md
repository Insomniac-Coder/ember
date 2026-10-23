# Phase 2: Hylo and C# (ref struct, Span, scoped) as memory models

Written 2026-10-24 for the memory-model rework (owner's directive of 2026-10-23). Two languages that
try to give memory safety without Rust's annoyances, studied against the shared list A1..A20.

How this was done, so every claim can be traced:

- Hylo: its language tour (docs.hylo-lang.org), its specification (github.com/hylo-lang/specification,
  `spec.md`), the C++ committee paper P2676R0 "The Val object model" (Abrahams, Parent, Racordon,
  Sankel, 2022), the paper "Implementation Strategies for Mutable Value Semantics" (Racordon,
  Shabalin, Zheng, Abrahams, Saeta, Journal of Object Technology 21(2), 2022; called "JOT 2022"
  below), and "Who Owns the Contents of a Doubly-Linked List?" (Racordon, Programming 2025 companion,
  OASIcs vol. 134, paper 25; called "Racordon 2025"). The PDFs were saved by WebFetch and read with
  `pdftotext`, which is already on this machine. The spec's raw Markdown was fetched once with `curl`
  into the session scratchpad to be read (a page read, nothing run). **No Hylo compiler is installed
  and installing one is forbidden, so no Hylo code was compiled.** Hylo examples are quoted from its
  documentation or spec unless marked "written by me, not compiled".
- C#: Microsoft Learn pages (ref struct, method parameters, ref locals, CollectionsMarshal,
  List enumerator), the C# 11 design note "low-level struct improvements" (github.com/dotnet/csharplang,
  `proposals/csharp-11.0/low-level-struct-improvements.md`), and Stephen Toub's "All About Span"
  (MSDN Magazine, January 2018). **Every C# claim marked "probe" was compiled and run here**: .NET SDK
  8.0.423 (already installed), C# 12, restored offline from an empty package folder (nothing
  downloaded). Probe: `target/redesign/phase2-hylo-csharp/csprobe/Program.cs`; its output, including
  the compiler errors of the refused cases: `target/redesign/phase2-hylo-csharp/csprobe_results.txt`.
- Cost against C of two ideas taken from these languages (paged storage, index handles) was measured
  here in C: `target/redesign/phase2-hylo-csharp/paged_vs_handles.c`, results in
  `target/redesign/phase2-hylo-csharp/results.txt` (clang 22.1.8 at -O2, in `C:/Program Files/LLVM`,
  pinned to logical CPU 1, a performance core on this laptop per `docs/AUTOPILOT.md`, on mains power
  checked before every round, 5 rounds, the median reported).
- Ember's side: `docs/spec-source/Ember_v0.9.10_Hardened_35.md`, `tasks/95/odr-pending.md` (ODR-286,
  ODR-280), `tasks/95/ecs-entt-design.md`, and `ember check` on `target/analysis/ecs4` and `ecs5`
  (today: `pos.em` is `E3060`, `box_bad.em` is `E3025`, the other four pass).

---

## Part 1. Hylo (formerly Val)

### 1.1 The model in plain words

Hylo has **no references you can hold**. Every variable owns its value; two variables never share
one. You still get in-place mutation without copies, because a function can borrow a variable for the
length of the call. How it borrows is said by one of four words on the parameter (Hylo tour,
"Functions and methods"):

```hylo
fun offset_let(_ v: let Vector2, by delta: let Vector2) -> Vector2 { ... }   // read only (the default)
fun offset_inout(_ target: inout Vector2, by delta: Vector2) { ... }         // may change it in place
fun offset_sink(_ base: sink Vector2, by delta: Vector2) -> Vector2 { ... }  // takes ownership
fun init_vector(_ target: set Vector2, x: sink Float64, y: sink Float64) { ... } // fills an uninitialised one

offset_inout(&v, by: (x: 1, y: 1))    // `&` marks the mutation at the call
```

Under the hood `inout` is a pointer, possibly into the middle of an array (JOT 2022, section 5:
"An inout argument is passed as a (possibly interior) pointer"). What makes it safe is that the
pointer is **second class**: it cannot be stored in a variable or a field, cannot be returned, and its
value is never visible to the program (JOT 2022 abstract and section 5). The compiler only has to check
one thing at each call: the same path is not passed twice. `swap(&x[0], &x[1])` is accepted;
`swap(&x[f(0)], &x[1])` is refused because `f(0)` might be 1 (JOT 2022, section 3).

To get at a part of a value without returning a reference, Hylo has **projections**: subscripts and
computed properties that `yield` instead of `return`. The subscript runs up to the `yield`, hands the
caller the part, waits until the caller's last use of it, then runs the rest (Hylo spec, "Yield
statements": control comes back "after the last use of the yielded projection at the call site").

```hylo
subscript min(_ x: yielded Int, _ y: yielded Int): Int {
  let   { if y > x { x } else { y } }
  inout { if y > x { &x } else { &y } }   // a conditionally chosen mutable part: no annotation
}
min[&x, &y] = 3
```

P2676 explains the implementation: the code after the `yield` point in the caller becomes a closure
that the subscript calls with the part ("inversion of control"); a computed part that is not stored
anywhere can still be projected mutably and written back after the caller is done:

```hylo
type Angle {
  public var radians: Double
  public property degrees: Double {
    inout { var d = radians * 180.0 / Double.pi
            yield &d
            radians = d * Double.pi / 180.0 }        // runs after the caller has changed d
  }
}
```

A projection can also be bound to a local, and while it lives the projected variable is out of reach
(Hylo spec, "Projections", rule 8 and its example):

```hylo
fun f1() {
  var x = 42
  inout y = x // mutable projection begins here
  print(x)    // error: 'x' is projected mutably
  print(y)    // mutable projection ends afterward
}
```

Other parts of the model that matter here:

- **No implicit copies.** `let` parameters cannot escape; returning one asks for `.copy()`:
  `v // error: 'v' cannot escape; return 'v.copy()' instead.` (Hylo tour). Arrays are uniquely owned:
  in the Hylo standard library `Array` has one stored property, `storage: DynamicBuffer<Int, Element>`,
  and `copy()` builds a new buffer element by element; no reference count, no copy-on-write
  (`StandardLibrary/Sources/Array.hylo` in github.com/hylo-lang/hylo, the repository its README now
  calls the "Outdated Implementation").
- **No classes.** The spec has no class declaration; shared mutable objects are not part of the safe
  language (P2676: "Without first-class pointers and references in Val's safe subset ...").
- **Stored projections.** A type may hold a projection, written `[let T]`, `[inout T]` or
  `[yielded T]` (spec, "Stored projection type expressions"); an object that captures a projection
  keeps it until the object's own lifetime ends (spec, "Projections", rule 10). No lifetime names exist.
- **Method bundles.** One method name can have `let`, `inout` and `sink` bodies; the compiler picks
  one from how the call is used (Hylo tour, the `Vector2.offset` example).
- **Globals are immutable.** "A global binding declaration shall be introduced with `let`" and is
  initialised at its first use (spec, binding declarations, rules 4.1 and 2).
- **Unsafe escape hatch**: `unsafe` expressions and `Builtin.Pointer` (spec).
- **Downcasts give a projection.** "The result of a downcast expression `e` is a projection of the
  left operand ... The evaluation of a downcast expression terminates the program at runtime if the
  dynamic type ... is not subtype" (spec, cast expressions, rule 5). This is exactly the piece ODR-286
  says Ember is missing ("`cast` of a `dyn I` place giving a reference").

### 1.2 The annoyances A1..A20 in Hylo

| id | verdict | how |
|---|---|---|
| A1 whole-object borrow | kept, narrower | a call checks paths, so `f(&a.x, &a.y)` is fine and `f(&a, &a.y)` is refused (JOT 2022 figure 2); a method with `inout self` takes all of `self`. Ember's `[BRW-10]` (a method borrows only the fields it uses) already goes further. |
| A2 container invalidation | kept (static) | while `inout e = &a[0]` lives, `a` is "inaccessible" (spec rule 8), so `&a.append(1)` is an error. It is rarer because a projection cannot be returned or stored outside a short scope. |
| A3 two elements at once | kept (static) | constant different indices only (JOT 2022 section 3); otherwise a library split: P2676 lists "a collection can be projected into a pair of independent Slices". |
| A4 change while iterating | kept (static) | `for let e in a` projects `a`; changing `a` in the body is the A2 error. |
| A5 returning references | **removed** | nothing returns a reference; a subscript yields instead, so there is nothing to annotate, conditional choice included (the `min` example). Kind (b): the subscript's frame stays alive while the caller uses the part, so "the local dies at return" cannot happen. |
| A6 structs holding references | mostly removed | a stored projection `[inout T]` needs no lifetime parameter; the object is simply bounded by what it captured (spec rule 10). How much of this the compiler implements: UNVERIFIED. |
| A7 self-referential | removed by design | nothing can point into its owner; the pattern is replaced by offsets (Racordon 2025's list stores `pred`/`succ` as `Int` offsets into the list's own array). |
| A8 graphs, parent pointers, lists | removed by design, moved to run time | a graph is tables keyed by identifiers; a list is an array with integer links (Racordon 2025, listings 3 and 4). A stale id is a wrong lookup or a bounds-check panic, never a dangling pointer. |
| A9 shared mutable state | removed (not expressible) | no `Rc<RefCell>`: there are no shared objects. The state has one owner and is passed `inout`. |
| A10 ownership kinds | **removed** | one kind: values. No Box/Rc/Arc/Cell/Weak. |
| A11 moves and clones | kept, and stricter | use after `sink` is an error ("used after being consumed"); every copy is an explicit `.copy()` (P2676: "Val never copies implicitly"). This is less Pythonic than Rust's implicit `Copy` for integers. |
| A12 closures | partly removed | a closure that mutates a captured variable says so with `inout` on its signature; a stateful closure owns its state in a capture list: `fun[var i = 0]() inout -> Int { ... }` (Hylo tour). No `'static`: an escaping closure captures by copy or `sink`. |
| A13 trait objects, downcasting | moved to run time, simpler | existentials `any P`; a downcast is checked and terminates the program on failure, and its result is a projection, so a registry lookup plus downcast can be mutated in place (spec). Object safety rules: UNVERIFIED. |
| A14 async | not settled | spec has `async` expressions and `await` as a consuming method; the concurrency page says the design "is still in progress". No `Pin` is needed in principle because no reference into a frame can exist (my inference). |
| A15 threads | removed (claimed), design open | P2676 claims "thread safety by construction"; the tour shows `spawn` with structured join and passing disjoint `inout` slices to two tasks; there are no `Send`/`Sync` words in the docs. Data-race freedom follows from no shared mutable state, but the concurrency design is unfinished. |
| A16 generics friction | partly | `conformance A: Copyable where ...` declarations are separate from the type; method bundles remove the read/write/consume triplication. Orphan-style rules: UNVERIFIED. |
| A17 error messages | improved | messages name the fix: "overlapping `inout` access to `v`; pass `v.copy()` as the second argument instead" (Hylo tour). |
| A18 initialisation | removed | the `set` convention initialises a variable through a parameter; definite initialisation is checked. |
| A19 global mutable state | **removed** (kind b) | globals are `let` only, lazily initialised (spec). |
| A20 unsafe, FFI | kept | `unsafe` expressions and `Builtin.Pointer`; since there are no references, there is no "pointer turned into a reference" step: unsafe code works on the pointer. |

The ECS case in Hylo (written by me, not compiled; names follow the spec's grammar):

```hylo
type Registry {
  var pools: Dictionary<TypeID, any AnyPool>
  subscript component<T>(_ e: Entity): T {
    inout {
      inout pool = &(pools[T.id]! as! Pool<T>)   // downcast of a place: a projection (spec, rule 5)
      yield &pool.dense[pool.index(of: e)]        // the caller works here; this frame stays alive
    }
  }
}
&reg.component<Position>(e).x += 1.0
```

Ember refuses the equivalent today (`target/analysis/ecs5/pos.em`, `E3060`: "`pool` is a local, so
its storage ends with the frame"). In Hylo the frame does not end until the caller is done, so the
error has no reason to exist. Adding to a pool while such a projection is live is still refused
(A2 kept).

### 1.3 Costs

Per operation (from the papers and the library source; nothing here was measured by me):

- `inout` and `let`: a pointer, as in C (JOT 2022 section 5; `let` is pass-by-value semantics
  implemented as an immutable borrow).
- `sink` (a move): a bytewise copy, no destructor on the source (JOT 2022, rule E-BINDING-MOVE).
- Arrays: a bounds check per index (P2676: "use cheap dynamic checks to ensure accesses are
  in-bounds"); no reference counting and no copy-on-write check in Hylo's own `Array` (library source
  above). The JOT 2022 prototype language (Swiftlet) did use copy-on-write and paid for it: "copy-on-write
  comes at the cost of runtime checks preceding every mutation", and "unnecessary reference counter
  checks account for most of the performance loss" against Swift (JOT 2022 section 7.5).
- Projections: inlined, a projection is the code before the `yield`, the caller's code, then the code
  after; not inlined, it is a call taking the caller's continuation as a closure (P2676 shows that
  rewrite). How the Hylo compiler emits them and what a non-inlined one costs: UNVERIFIED.
- Graphs and lists: an index plus a bounds check per hop instead of a pointer (Racordon 2025).

Memory per object: none beyond the value itself. There are no object headers, counts or access words
because there are no shared objects (spec: no class declarations; library: `Array` holds one buffer).
The array buffer has a header holding the element count; its size: UNVERIFIED.

What it cannot express, or makes slow:

- Any shared mutable object: an observer list, a scene graph node with a parent pointer, a cache two
  systems both write. Each must be redesigned as one owner plus ids, or written with `unsafe`.
- A reference kept beyond a call or a block, for example a component reference kept in a struct
  across frames. The stored-projection type exists, but such an object cannot outlive what it
  projects.
- Reseating: "one cannot 'reseat' a let or inout binding" (P2676, which suggests key paths instead).
- Python-style implicit copies: every non-`sink` escape needs `.copy()`.

Published measurements (none of them measure the Hylo compiler itself; I found no published Hylo
benchmark):

1. JOT 2022, section 7. Machine: Intel Xeon Gold 6154 at 3.00 GHz, 192 GB, Linux 5.10.40. Compilers:
   Swift 5.5.2 (`-Ounchecked`), Scala Native 0.4.2, Clang 11 (`-O3`), and their own Swiftlet compiler,
   all through LLVM. Workload: 1,344 programs generated by a fuzzer, 20 runs each, building and
   mutating nested structs and arrays; times normalised to the fastest per program. Findings, in their
   words: Swift "is the fastest language in the overwhelming majority of the benchmarks"; Swift was
   slower than C++ "only for programs with extremely large number of mutating operations (> 90%)".
   **Caveat that matters for us:** their C++ is not hand-written C. Their translator maps arrays to
   `std::vector` passed by value, and they say C++ "performs relatively poorly in read-heavy benchmarks,
   due to the cost of copying large data structures". So this shows value semantics beating a copying
   C++, not beating C. On five small programs from Marr et al. 2016 (Bounce, Mandelbrot, NBody,
   Permute, Queens), "both Swift and C++ perform the best, presenting extremely similar performance
   profiles"; the per-program ratios are in their figure 16, which the text extraction scrambles, so I
   do not quote them.
2. Racordon 2025, section 4. Machine: MacBook Pro, Apple M1, 16 GB, macOS 15.3.1; average of 10
   iterations per size; all code in Swift. Graph algorithms (Tarjan's strongly connected components,
   Dijkstra) on objects-with-references versus value tables keyed by string URLs: "no noticeable
   difference". Doubly linked list (append, traverse, remove at random, traverse after removals) on
   Swift class nodes versus array offsets: the offset version "significantly outperforms its
   reference-based counterpart, by one order of magnitude", which the author puts down to cache
   locality, Swift's atomic exclusivity check before mutating an object, and one allocation per node.
   The exact times are only in log-scale figures; I do not quote numbers. Note the baseline is Swift
   classes (counted, checked objects), not C pointers.
3. A shipped value-semantics language against C, as context only (Swift, not Hylo: Swift also has
   counted classes and copy-on-write): the Computer Language Benchmarks Game, page
   `fastest/swift-clang.html` (Swift 6.0.3, clang 19.1.1; machine per its "how programs are measured"
   page: quad-core 3.0 GHz Intel i5-3330, 15.8 GiB, Ubuntu 24.04), fastest program each: n-body Swift
   5.45 s / 24,928 KB against C 2.20 s / 2,494 KB; spectral-norm 1.45 s against 0.39 s; mandelbrot
   1.35 s against 1.23 s; binary-trees 17.62 s / 733,094 KB against 1.68 s / 176,222 KB. These compare
   whole submitted programs (different authors, different use of threads), so they say nothing
   precise about the memory model; they only warn that "value semantics" does not by itself give C
   speed.

### 1.4 Maturity

Experimental. The language site says the compiler and standard library "are still in their early
stages" (hylo-lang.org, read 2026-10-23). The old compiler repository's README calls itself
"Outdated Implementation of Hylo" and points to github.com/hylo-lang/hylo-new; neither repository's
Releases section lists a release (pages read 2026-10-23), although the site mentions pre-built
releases for Linux, macOS and Windows. The concurrency design "is still in progress" (tour). The
ECOOP 2025 talk "Designing Hylo" says the design "has solidified into a coherent system" while the
language is still under development. No production users found.

### 1.5 What Ember could take from Hylo

| idea | kind | removes | cost against C |
|---|---|---|---|
| H1. **Yield-once accessors** (a method that `yield`s a place once; the caller works on it; the method finishes after the caller's last use) | (b) the accessor's frame stays alive, so a reference made from a local there (the pool found through the map) is valid for exactly as long as the caller uses it | A5 (the `E3060` of `pos.em`), part of A13, and for class handles it gives `[EXC-18]`'s run-time access a natural start and end: the access begins before the `yield` and ends after it, inside one frame, which is what ODR-286 found impossible today ("`[EXC-18]`'s run-time access cannot follow the reference to the caller") | inlined (small accessors always are): nothing, it is the same instructions as writing the code at the call. Not inlined: one extra call for the code after the `yield` (UNMEASURED). No allocation: the frame lives on the caller's stack and never moves, so Ember's generator rule `[CORO-6]` does not apply. |
| H2. **Downcast of a place is a place** | (a) the same static guarantee, spelled as the Python programmer writes it: `pool = cast self.pools[k] as Pool[T]` then mutate `pool` | the remaining part of ODR-286 for `Box[dyn AnyPool]` pools | one type-identity compare, as `cast` does today (ODR-280: "The same check and code as today") |
| H3. **One name for read, write and consume** (method bundles) | (a) | `get[T]`/`get_mut[T]`/`try_get`/`try_get_mut` pairs in `ecs-entt-design.md` collapse to `get[T]` and `try_get[T]`, chosen by use, as Python has one `reg.get(e)` | none: each use instantiates the matching body at compile time |
| H4. **Whole/part plus ids** for graphs, lists, parent links (Racordon 2025) | (b) no references, so nothing can dangle; a stale id is a checked miss | A7, A8, and A9 where the sharing was only for navigation | measured here (Part 3): a generation-checked handle costs 1.37x to 1.75x a raw pointer on random access over 64 MiB, and 4 or 8 bytes per slot. Ember already has `Pool`/`Handle` (`[HND-1]`). |
| H5. **Error messages that name the fix** (`pass v.copy() ... instead`) | (a) | A17 | none |
| H6. **Immutable globals, lazily initialised** | (b) a global nobody can write cannot race | A19 | a once-check per access unless the compiler initialises before `main` (UNMEASURED) |

What not to take: explicit `.copy()` everywhere (against goal 3), and dropping classes outright
(Python programmers expect shared objects; that is a decision for the owner, not a free win).

---

## Part 2. C#: ref, ref struct, Span&lt;T&gt;, scoped

### 2.1 The model in plain words

C# objects live on a garbage-collected heap; class variables are references to them and can be
copied, stored and shared freely. Next to that, C# has **second-class references** for speed: a
`ref` parameter, `ref` local, `ref` return, and `ref` fields inside a `ref struct`. `Span<T>` is a
`ref struct` holding a `ref T` and a length (Toub 2018; ref-struct page). These references may point
into the stack, into the middle of a heap array, or into native memory, and the rule that keeps them
safe is simple: **a ref, or anything containing one, can never be stored on the heap**:

```csharp
// refused (probe, csprobe_results.txt):
sealed class Holder { public Span<int> items; }   // CS8345: ... unless it is an instance member of a ref struct
var arr = new View[1];                             // CS0611: Array elements cannot be of type 'View'
Func<int> g = () => sp[0];                         // CS8175: Cannot use ref local 'sp' inside ... lambda
List<Span<int>> l = null;                          // CS9244: ... may not be a ref struct ... 'List<T>'
```

The compiler gives every expression a *safe-context* and a *ref-safe-context*: the caller's context,
"return only", or the current method (C# 11 design note). It works out a method's escape purely from
its **signature**: "The ref safe context rules only look at the method signature, not the
implementation" (design note). Defaults are chosen so most code needs no annotation; `this` in a struct
method, `out` parameters, and `ref` parameters of `ref struct` type are implicitly `scoped` (the
declarations page). Where a default is wrong you write `scoped` (narrower) or `[UnscopedRef]` (wider):

```csharp
Span<int> CreateSpan(scoped ref int parameter)   // promise: parameter does not escape
static ref int LocalRef() { int x = 1; return ref x; }                // CS8168 (probe)
static Span<int> StackSpan() { Span<int> b = stackalloc int[4]; return b; }  // CS8352 (probe)
struct S { int f; public ref int F() => ref f; }                      // CS8170 (probe): 'this' is scoped
```

**There is no aliasing rule at all.** Two mutable references to one element are fine, a reference to
one field while a method writes another is fine. Memory safety comes from three other places:
the GC keeps any object alive while a reference into it exists (interior references are tracked by
the GC, which is why they are kept off the heap: "tracking them is a relatively expensive operation
for the .NET runtime's garbage collector", Toub 2018), bounds checks on every index, and the
stack-only rule (which also stops a two-word span from being torn by a racing thread, Toub 2018).

### 2.2 The annoyances A1..A20 in C#

Every row with "probe" was compiled and run here (`csprobe_results.txt`).

| id | verdict | how |
|---|---|---|
| A1 whole-object borrow | **removed** | no exclusivity: `ref int hp = ref w.hp; w.Tick(); hp += 1;` compiles and runs (probe: "hp = 11, ticks = 1"). |
| A2 container invalidation | removed for memory safety, **becomes silent wrong results** | a reference into an array survives the array being replaced, because the GC keeps the old array alive. Probe: hold `ref reg.Get<Position>(0)`, add (the pool's array is resized), write 100: "wrote 100, pool now reads 1". Same with `CollectionsMarshal.AsSpan(list)` then `Add`: "wrote 42, list[0] = 1". Microsoft only documents "Items should not be added or removed ... while the Span is in use". |
| A3 two elements at once | removed | `ref int r1 = ref a[0]; ref int r2 = ref a[0];` both written (probe: "a[0] = 7"). |
| A4 change while iterating | moved to run time | `foreach` over a `List<T>` that changes throws `InvalidOperationException` at the next step (List enumerator docs; probe). An index loop may append and sees the new items (probe: "6 items"), as a Python list does. |
| A5 returning references | removed for heap data, kept (annotated) for stack data | `public ref T Get<T>(int i) => ref ((Pool<T>)pools[typeof(T)]).dense[i];` compiles (probe): the pool came from a dictionary lookup and a downcast, exactly the case Ember refuses with `E3060`, and C# accepts it because heap arrays are valid in the caller's context. Returning a reference to a struct's own field needs `[UnscopedRef]` (CS8170 otherwise). |
| A6 structs holding references | kept, different form | a struct holding a `Span` must itself be a `ref struct`, and that property spreads to every type holding it; it cannot go in arrays, classes, lambdas or (before C# 13) generics. The heap-storable form is `Memory<T>`: an object, an index and a length, turned into a `Span` when used (Toub 2018). |
| A7 self-referential | removed by GC | objects point at each other freely. |
| A8 graphs, cycles | removed by GC | cycles are collected. |
| A9 shared mutable state | removed by GC, no checks | any number of class references, all writable. |
| A10 ownership kinds | removed | two kinds: class (shared) and struct (copied), plus Span/Memory for views. |
| A11 moves and clones | removed | no moves; struct copies are implicit (large structs are passed `in` / `ref readonly` to avoid copies). |
| A12 closures | removed | captures are by reference: the local is hoisted to a heap object (probe: "counter = 2"). Ref locals and spans cannot be captured (CS8175). |
| A13 trait objects, downcasting | moved to run time | `(Pool<T>)pools[typeof(T)]` is a checked cast (`InvalidCastException`); no object-safety rules. |
| A14 async | removed by GC, with a residue | state lives in heap objects; a `ref struct` local "can't be used in the same block as the `await`" (C# 13; before C# 13 not at all in async methods). |
| A15 threads | annotations removed, **hazard not prevented** | no `Send`/`Sync`; data races are allowed and give wrong values; memory safety under races rests on the GC and on spans never being on the heap. Fails the owner's "no data race". |
| A16 generics friction | kept, different | no retroactive interface implementation; `where T : allows ref struct` (C# 13) to accept spans in generics. |
| A17 error messages | kept | "may expose referenced variables outside of their declaration scope" (CS8352) says what is wrong, not what to write. |
| A18 initialisation | removed | definite assignment, zero-initialised structs, `out` parameters. |
| A19 global mutable state | removed (statics), racy | static fields are allowed and unprotected. |
| A20 unsafe, FFI | kept | `unsafe` pointers, `fixed` pinning, `new Span<byte>((byte*)ptr, n)` with no lifetime check (Toub 2018). |

### 2.3 Costs

Per operation:

- `ref` and `Span` access: a pointer (and a length) as in C. Indexing is bounds checked, and the JIT
  removes the check in a loop that runs from 0 to `Length` (Toub 2018 shows the x64 code for
  `Sum(Span<int>)`: no check inside the loop).
- Garbage collection: what makes A1, A2, A3, A5, A7..A14 disappear. Per object a 16-byte header on
  64-bit and a 24-byte minimum (Red Gate, "Object Overhead: The Hidden .NET Memory Allocation Cost";
  a secondary source describing the .NET Framework layout; the current CoreCLR figure: UNVERIFIED).
  Interior references are costly for the GC to track (Toub 2018).
- Enumeration check: one version compare per `MoveNext` (List enumerator docs describe the check;
  its cost: UNMEASURED).
- Silent stale writes (A2) cost nothing at run time, and that is the problem: the program continues
  with lost updates.

Published measurements, C# against C++ (the Benchmarks Game has no C# against C page): page
`fastest/csharpcore-gpp.html`, .NET SDK 9.0.100 with server and concurrent GC against g++ 14.2.0,
same i5-3330 machine as above, fastest program each, rows copied from the page:

| program | C# secs | C# KB | C++ secs | C++ KB |
|---|---|---|---|---|
| n-body | 3.20 | 32,059 | 2.15 | 2,417 |
| spectral-norm | 0.96 | 37,929 | 0.72 | 3,981 |
| fannkuch-redux | 2.35 | 33,559 | 3.33 | 5,800 |
| binary-trees | 6.40 | 1,053,831 | 1.01 | 175,108 |

The C# memory never falls below about 32 MB on these pages, which looks like the runtime's own
footprint (my reading, UNVERIFIED); binary-trees is the allocation-heavy one, and there C# takes 6.3x
the time and 6.0x the memory (how the fastest C++ entry allocates: UNVERIFIED, not read). These are whole
programs by different authors, so they bound the cost of the GC model only roughly.

What it cannot express or makes slow: a reference stored in a heap object (use `Memory<T>`, an extra
indirection); spans in lambdas, iterators, or across `await`; any guarantee against data races; any
guarantee that a write through a reference reaches the live container (A2 above).

### 2.4 Maturity

Shipped and in heavy production use. `ref` returns arrived in C# 7.0, and `ref struct`, `Span<T>`,
`ref readonly` returns and `stackalloc` into a span in C# 7.2, aimed at .NET Core 2.1 (Toub 2018; "ASP.NET Core now
has a heavy dependency on spans ... the Kestrel server's HTTP parser"); `ref` fields and `scoped` in
C# 11 (the design note sits in `proposals/csharp-11.0`); `ref struct` interfaces and
`allows ref struct` in C# 13 (ref-struct page). C# 11's change was a breaking one, handled with a
`RefSafetyRules` attribute so old libraries keep the C# 7.2 rules; the design note expected the
impact "to be very small".

### 2.5 What Ember could take from C#

| idea | kind | removes | cost against C |
|---|---|---|---|
| C1. **Keep views second-class, as now.** C# proves a mainstream audience accepts "a view cannot go in a heap field, an array, or a closure". Ember's `[TYP-15]` already says this, and better: Ember infers view types (`[TYP-34]`) where C# needs the `ref struct` keyword, and infers regions where C# needs `scoped`/`[UnscopedRef]`. Nothing to copy except the confidence. | - | - | none |
| C2. **A storable slice handle** like `Memory<T>`: a container handle plus a range, turned into a view only when used | (b) the stored thing holds no pointer, so it cannot dangle; the conversion re-checks the bounds | A6 (a struct that must keep "its part of the array" across frames) | one indirection and one bounds check per conversion, none per element |
| C3. **Run-time check when a collection changes under a loop** (version counter) | moved to run time; a deterministic panic, which the owner allows | A4 for collections reached through class handles, where the static rule cannot see | one increment per structural change, one compare per loop step (UNMEASURED); Python does the same for dicts ("dictionary changed size during iteration") |
| C4. **Index loops that see appended items** (C#'s `for (i < list.Count)`, Python's list) combined with storage that never moves on append (Part 3) | (a) for the spelling, (b) for the storage | A4 for spawning during a game loop, A2 for growth | paged storage, measured in Part 3 |
| C5. **"The GC keeps the old buffer alive"** | rejected | it removes A2's crash only by turning it into a lost write (probe: "wrote 100, pool now reads 1"); without a GC it needs a count on every buffer; the owner's bar is a deterministic panic, not a silent wrong answer | - |

---

## Part 3. Measured here: what the two "smarter solutions" cost against C

Both languages point at the same two mechanisms for the ECS: storage whose elements never move on
append (so a reference survives growth, EnTT's paged pools, `ecs-entt-design.md` item 4), and ids
instead of references (Hylo, Racordon 2025, Ember's own `Handle`). Program:
`target/redesign/phase2-hylo-csharp/paged_vs_handles.c`; raw lines in `results.txt`, which holds three blocks: the main run, a page-size check, a handle-layout check. 4,194,304
components of 16 bytes (64 MiB, larger than the caches, so these are worst cases; the ratios for data
that fits in cache are UNMEASURED). Medians of 5 rounds, logical CPU 1, mains power.

| what | plain C | the safer form | ratio | memory (peak working set) |
|---|---|---|---|---|
| fill by appending 4,194,304 (whole process, ms) | contiguous, realloc doubling: 34.974 | 4096-element pages: 24.975 | 0.71x | 70,064 KB / 69,124 KB |
| sum all, 20 passes (work ms) | contiguous: 61.670 | 64 KiB pages: 77.240 | 1.25x | as above |
| same, second block of `results.txt` | contiguous: 61.300 (1 MiB-page build) | 1 MiB pages (65,536 elements): 63.067 | 1.03x | 69,284 KB |
| 16,777,216 random reads by index (work ms) | contiguous: 76.793 | 64 KiB pages: 99.720 | 1.30x | as above |
| same (pages from the second block, contiguous from the first) | - | 1 MiB pages: 100.927 (64 KiB pages in that run: 101.845) | 1.31x | 69,284 KB |
| 16,777,216 derefs of stored references (work ms) | raw pointer: 109.163 (first block), 110.440 (third block) | handle, generation in a separate array: 191.769 / 193.140 | 1.76x / 1.75x | 78,264 KB vs 94,652 KB: +16,388 KB = 4 bytes per slot |
| same | raw pointer: 110.440 | handle, generation beside the value (24-byte slot): 150.986 | 1.37x | +8 bytes per slot (16 to 24); the measured peak (168,332 KB) is inflated by my copying the array, so I give the steady size by arithmetic: 96 MiB against 64 MiB |

What the numbers say, with the reason checked where I could:

- **Appending into pages is faster** than doubling a contiguous array (no copying on growth) and the
  peak memory is the same here (on this Windows heap the large `realloc` did not double the peak).
- **Sequential iteration over pages costs nothing once pages are big enough.** Both loops compile to
  the same vectorised inner loop (checked in the assembly: identical `movdqu`/`paddq` bodies), and the
  25% at 64 KiB pages falls to 3% at 1 MiB pages, so the cost is the page size, not the loop.
- **Random access through a page table costs about 30%** at either page size: one more dependent load
  (the page pointer) before the element (inferred from the code; not checked with hardware counters).
- **A generation-checked handle costs 1.37x to 1.75x a raw pointer** on random access; the larger
  figure comes from the generation living in a separate array (a second cache miss), the smaller from
  storing it beside the value (one miss, a bigger slot). The compare itself is cheap; the memory
  traffic is the cost.

For the ECS this means: paged pools let `emplace` run during a view loop with no new run-time check
and no loss in iteration speed (at large pages), at a 30% cost on random `get(e)` by index; handles in
place of references cost 37% to 75% on random dereference.

---

## Part 4. Summary for Ember

1. **Hylo removes A5 and A6 by not having returnable references, and gets away with it because of
   yield-once projections.** The ECS's `E3060` (`pos.em`) is exactly the case a projection fixes: the
   pool local lives in the accessor's frame, which stays alive until the caller is done. Taking only
   this (H1), plus the downcast-of-a-place rule (H2), would let Ember write the EnTT `get` the way the
   owner first asked for, with the same static rules as today and no run-time cost when inlined. For
   pools held as class objects, the projection also gives `[EXC-18]` the bracket it lacks today: the
   dynamic access begins before the `yield` and ends after it.
2. **C# removes A1, A3, A7..A14 with a garbage collector and no aliasing rules,** and its own docs
   and my probe show the price: references that silently write to dead buffers (A2), no data-race
   freedom (A15), about 32 MB of runtime and 16 bytes per object. That fails goals 1 and 2. What
   carries over is the confirmation that second-class views (which Ember already has, with less
   ceremony) are acceptable to mainstream programmers, the storable slice handle (C2), and the
   run-time version check for loops (C3).
3. **A2 and A4 in the ECS are best solved by storage, not by a type rule:** pages that never move on
   append make spawning during a loop memory-safe with the borrow rules unchanged for removal; measured
   cost against C: none on iteration at 1 MiB pages, 30% on random access by index, append faster.
4. **What stays annoying in both:** A2 and A4 for removal (swap-and-pop moves an element), A3 for
   computed indices (both use a library split), A11 (Hylo makes it worse with explicit `.copy()`),
   A15 (Hylo's answer is unfinished; C# has none), A20.

Open questions that need the owner (not ruled here):

- Should Ember add yield-once accessors (H1)? It is a new kind of function; the spelling would be
  Python's generator `yield`, used once.
- Should `Pool` pages be the default storage for ECS pools (item 4 of `ecs-entt-design.md` currently
  says contiguous), at 30% on random access by index?

## Sources

- Hylo language tour: https://docs.hylo-lang.org/language-tour/functions-and-methods ,
  .../subscripts , .../bindings , .../concurrency (read 2026-10-23)
- Hylo specification: https://github.com/hylo-lang/specification/blob/main/spec.md
- Hylo site: https://www.hylo-lang.org/ ; repositories https://github.com/hylo-lang/hylo and
  https://github.com/hylo-lang/hylo-new ; Array source
  https://github.com/hylo-lang/hylo/blob/main/StandardLibrary/Sources/Array.hylo
- P2676R0, The Val Object Model: https://www.open-std.org/Jtc1/sc22/wg21/docs/papers/2022/p2676r0.pdf
- Racordon et al., Implementation Strategies for Mutable Value Semantics, JOT 21(2) 2022:
  https://www.jot.fm/issues/issue_2022_02/article2.pdf ; earlier version https://arxiv.org/abs/2106.12678
- Racordon, Who Owns the Contents of a Doubly-Linked List?, OASIcs Programming 2025:
  https://drops.dagstuhl.de/entities/document/10.4230/OASIcs.Programming.2025.25
- ECOOP 2025 PLSS, Designing Hylo:
  https://2025.ecoop.org/details/plss-2025-papers/12/Designing-Hylo-a-programming-language-for-safe-systems-programming
- C# ref struct: https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/ref-struct
- C# method parameters and safe contexts:
  https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/method-parameters
- C# ref locals and scoped:
  https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/statements/declarations
- C# 11 low-level struct improvements:
  https://github.com/dotnet/csharplang/blob/main/proposals/csharp-11.0/low-level-struct-improvements.md
- Toub, All About Span (2018):
  https://learn.microsoft.com/en-us/archive/msdn-magazine/2018/january/csharp-all-about-span-exploring-a-new-net-mainstay
- CollectionsMarshal.AsSpan and GetValueRefOrAddDefault, List enumerator MoveNext: learn.microsoft.com
  API pages under System.Runtime.InteropServices.CollectionsMarshal and
  System.Collections.Generic.List-1.Enumerator
- Red Gate, Object Overhead: https://www.red-gate.com/simple-talk/development/dotnet-development/object-overhead-the-hidden-net-memory-allocation-cost/
- Benchmarks Game: https://benchmarksgame-team.pages.debian.net/benchmarksgame/fastest/csharpcore-gpp.html ,
  .../fastest/swift-clang.html , .../how-programs-are-measured.html
- Swift SE-0474 Yielding Accessors (accepted 2025; the same mechanism as H1 in a shipped language,
  not yet landed per the acceptance note): https://forums.swift.org/t/accepted-se-0474-yielding-accessors/80273
