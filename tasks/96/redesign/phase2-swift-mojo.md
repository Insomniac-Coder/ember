# Phase 2: languages. Swift and Mojo

Research agent report, 2026-10-24. The scope is Swift (ARC, value types with copy-on-write, static and
dynamic exclusivity, `inout`, `~Copyable` with `borrowing`/`consuming`, non-escapable types, lifetime
dependencies, `Span`) and Mojo (ownership with inferred origins, ASAP destruction, `ref` returns). The
annoyance ids are A1 to A20 from the task. No new ids are added.

How the claims were checked:

- **No Swift or Mojo toolchain is installed on this machine.** `which swift swiftc mojo` finds
  nothing. No install was made, because the rules forbid it. Every Swift or Mojo code example below
  comes from the language's own documents or proposals, or is my own example built from them. **None
  was compiled.** An example whose exact compiler behaviour is not stated in a cited document is
  marked UNVERIFIED.
- Ember behaviour was checked with `target/ci/ember.exe check` on probe files in
  `target/redesign/p2sm/` (section 5), and on the owner's probes in `target/analysis/ecs4` and
  `ecs5`. No Ember program was run.
- Every number names its source. The source list is in section 6 (S1 to S17 for Swift, M1 to M12 for
  Mojo). Where I found no published measurement, I say so instead of estimating.

---

## 0. The short version

1. **Swift removes most of Rust's annoyances in two ways. It makes references second-class, so they
   last for one access and never escape. It turns shared mutable state into class objects that are
   counted and checked at run time.** You cannot return a reference, store one in a struct or hold
   one across a statement, so A2, A4, A5, A6 and A7 never come up in ordinary code. The price is paid
   at run time: reference counting (RC), copy-on-write uniqueness checks, and dynamic exclusivity
   checks on class properties and globals. The published measurements are large. Swift programs
   spent 42% (client) and 15% (server) of their time in RC operations in a 2018 study (S4). Turning
   on dynamic exclusivity checks in release builds slowed one microbenchmark by 840% (S3).
2. **Swift could not reach C speed for views without bringing back a restricted kind of reference.**
   Swift 6.2 (2025) added non-escapable types and `Span`/`MutableSpan`, which are views tied to the
   container's lifetime (S10, S11). Making those returnable needs lifetime-dependency annotations.
   These are still experimental (`@_lifetime`, pitch #3 of February 2026, S12), with defaults that
   cover the simple cases. This is the main lesson for Ember. Second-class references alone do not
   reach C speed. A non-escaping view type with an inferred dependency is needed, and Ember already
   has inferred regions.
3. **Mojo is Rust's model with friendlier spelling and less checking.** Its argument conventions are
   `read` (the default), `mut`, `var` and `out`. Moves use `^`. Origins (lifetimes) are inferred
   inside bodies, but a function returning a reference names its origin (`-> ref[self] T`). It
   destroys values as soon as possible (ASAP destruction), so a value dies at its last use. **Mojo
   does not yet have memory safety for collections or shared state.** Before Mojo 1.0 (August 2026),
   a reference into a `List` silently dangled after `append` reallocated the list (M4). The fix,
   "interior origins", is experimental. `ArcPointer` leaves mutation unchecked (M6). There are no
   existentials (trait objects), and the roadmap says "supporting mutable aliasing" is not started
   (M5). Mojo is not a model to copy for safety. Two of its ideas are worth taking: ASAP destruction
   and the interior-origin diagnostic.
4. **Ember already contains half of Swift.** Classes are counted handles with per-field dynamic
   exclusivity (`[EXC-1]`, `[EXC-19]`, spec Part VIII). The half Ember does not have is Swift's
   **value-side** answer. Swift never keeps a reference into a collection across a statement, and
   iterating a collection does not lock it.
5. **Ideas for Ember, each labelled with the owner's kind:**
   - (b) Iterate collections by index cursor, borrowing one element for one iteration instead of
     borrowing the container for the whole loop. This removes A4 and most of the game-loop part of
     A2. Against C, the cost is one length compare per iteration, and only in loops that change the
     container.
   - (a) Scoped access instead of returned references for the ODR-286 case: `with reg.get_mut[Pos](e)
     as p:`. This follows Swift's yielding and `mutate` accessors (S13, S14). It removes the E3060
     wall for pools found through a map. A second handle that grows the pool is caught by the
     existing access word.
   - (b) ASAP freeing for values whose drop only frees memory. Scope-end drop stays for types with a
     user `drop`, which avoids Mojo's guard trap. This lowers peak memory and costs nothing at run
     time.
   - (a) Inferred-dependency views (Swift's `@lifetime` defaults, already Ember's inferred regions)
     as the only form of stored reference. This keeps A5 and A6 annotation-free.
   - (a) Swift's region-based isolation (SE-0414) for A15. A value can go to another thread without
     any `Send` annotation when the compiler sees nothing else can reach it.

   Section 4 gives the details and the cost of each against C.

---

## 1. Swift

### 1.1 The model in plain words

Swift has two kinds of types.

**Value types** (`struct`, `enum`, tuples, and the standard collections `Array`, `Dictionary`, `Set`,
`String`) are copied on assignment. The collections are a small struct holding a pointer to a
counted heap buffer. "Copying" one copies the pointer and increments the buffer's count. The buffer
is duplicated only when someone mutates it while its count is above 1. This is copy-on-write (COW),
documented in `OptimizationTips.rst` (S9).

```swift
var a = [1, 2, 3]
var b = a          // no element copy: both share one buffer, its count is 2
b.append(4)        // b's buffer is shared, so it is copied first; a is still [1, 2, 3]
```

**Reference types** (`class`, `actor`) are heap objects reached through counted pointers. This is
automatic reference counting (ARC). The object header is two words, 16 bytes on 64-bit: a metadata
pointer and one pointer-sized refcount word that holds the strong count, the unowned count and flags
(`RefCount.h`, S7). A side table is allocated only when the first `weak` reference is formed (S7).

**References are second-class.** Ordinary Swift has no `&T` type. You cannot store a reference in a
variable or field, or return one. A function can mutate the caller's variable only through an `inout`
parameter. Semantically this is copy-in/copy-out. In practice the compiler passes the address, which
is legal because of the law of exclusivity:

```swift
func bump(_ x: inout Int) { x += 1 }
var n = 1
bump(&n)           // the access to n lasts exactly for the call
```

**The law of exclusivity** (SE-0176, Swift 4.0, S1) says two accesses to the same variable may not
overlap unless both are reads. Two kinds of access exist:

- An *instantaneous* access finishes before other code can run, for example a load or a plain
  assignment.
- A *long-term* access spans other code: an `inout` argument, a `mutating` method call, or a
  `_modify`/yielding accessor.

**Enforcement is static where it can be, dynamic where it must be** (S1, S2):

- **Static:** local variables, `inout` parameters and struct properties. A conflict is a compile
  error.
- **Dynamic:** class properties, globals and static properties, and local variables captured by
  escaping closures. A conflict traps at run time with "Simultaneous accesses to …, but modification
  requires exclusive access" (S2; the runtime's message text is in `Exclusivity.cpp`, S8). The
  per-thread access set lives in thread-local storage (S8). Since Swift 5 this check is on in release
  builds (S2).
- Class properties are checked **one stored property at a time**, never as the whole object (S1).
  This is exactly what Ember's `[EXC-19]` does.

**Ownership control, added from 2023 on** (statuses from the Swift Evolution index, S16):

- `~Copyable` structs and enums (SE-0390, Swift 5.9) are one-owner values with `deinit`. A use after
  `consume` is a compile error.
- `borrowing` and `consuming` parameter modifiers (SE-0377, 5.9). The `consume` operator (SE-0366,
  5.9).
- Non-copyable generics (SE-0427, 6.0). Partial consumption (SE-0429, 6.0).

```swift
struct FileHandle: ~Copyable {
    var fd: Int32
    consuming func close() { /* ... */ }
    deinit { /* closes fd if close() was not called */ }
}
let f = FileHandle(fd: 3)
f.close()
f.close()          // error: 'f' consumed more than once  (wording UNVERIFIED; rule from SE-0390)
```

**Non-escapable types and views** (Swift 6.2, 2025):

- `~Escapable` (SE-0446, S10) marks a value that may be copied locally and passed down, but never
  stored in a global, captured by an escaping closure, stored in an escapable type, or returned
  without a lifetime dependency.
- `Span<T>` (SE-0447) is a pointer and a count. It is copyable and non-escapable, and its subscript
  is always bounds-checked. `subscript(unchecked:)` is the unsafe opt-out (S10).
- `Array.span` and similar properties come from SE-0456. `MutableSpan` (SE-0467) is non-copyable and
  non-escapable, and is an exclusive access to the container while it lives (S11).

```swift
var array = [1, 2, 3]
var ms = array.mutableSpan     // exclusive access to array begins
ms.swapAt(0, 1)
array.append(4)                // error while ms is still used afterwards (S11)
_ = consume ms
```

**Lifetime dependencies** let a function return a non-escapable value tied to an argument. They are
still experimental (`@_lifetime(borrow x)`, `@_lifetime(copy x)`, `@_lifetime(&x)`). Pitch #3 was
posted on 2026-02-25 by Andrew Trick (S12). By that pitch, `borrow self` is already the default for an
accessor or a method with no parameters, so the common case needs no annotation (S12).

```swift
extension Array {
    var span: Span<Element> {
        @_lifetime(borrow self)          // redundant: the default for a getter (S12)
        borrowing get { /* ... */ }
    }
}
```

**In-place accessors.** These are the Swift counterpart of `ref` returns:

- Yielding accessors `yielding borrow` / `yielding mutate` (SE-0474, status "accepted", S16) are
  coroutines. They lend the stored value to the caller for one access and then resume.
- Borrow and mutate accessors (SE-0507, implemented in Swift 6.4, S14) are not coroutines. They must
  return storage that already exists in `self`; returning a local or temporary is illegal. `self` may
  not be mutated while the borrow is used. They are not allowed on classes or actors, because nothing
  can run after the access (S14).

```swift
struct Wrapper<Element: ~Copyable>: ~Copyable {
    var _element: Element
    var element: Element {
        borrow { return _element }       // a borrow of stored memory, no copy
        mutate { return &_element }      // in-place mutation
    }
}
```

**Concurrency safety:**

- Actors (SE-0306, 5.5) and `Sendable` (SE-0302, 5.7).
- Region-based isolation (SE-0414, 6.0) and `sending` (SE-0430, 6.0). The compiler tracks which
  values are reachable from which; a value whose whole region is disconnected may be sent to another
  isolation domain even if its type is not `Sendable`.
- `Mutex` (SE-0433, 6.0). Strict checking of global variables (SE-0412, 5.10).

**Unsafe code:** `UnsafePointer` and friends. Opt-in strict memory safety checking (SE-0458, 6.2)
makes each unsafe use visible.

### 1.2 A1 to A20 in Swift

| id | verdict | how |
|---|---|---|
| A1 | removed for classes and for fields of a local struct; kept inside `mutating` methods | Class properties are checked one by one (S1), so using one field while another is accessed is fine. A local struct's disjoint stored properties may overlap when the compiler proves it (S1, S2). A `mutating` method is an `inout` access to the whole `self`, so a closure touching `self.log` while `self.items` is being read conflicts (example below). |
| A2 | removed for ordinary code; kept (static) for `Span`/`MutableSpan` | No reference to an element survives a statement. A subscript yields for one access, so `append` can never invalidate anything. With a `MutableSpan`, mutating the array while the span is used is a compile error (S11). |
| A3 | kept | Simultaneous mutation of two elements of one array violates exclusivity (S1). The fix is a library method: `a.swapAt(i, j)`, or `MutableSpan.swapAt`. Splitting a `MutableSpan` into two is future work (S11). |
| A4 | removed (by value semantics) | `for x in a` iterates over the value `a` had when the loop began; mutating `a` in the body triggers a COW copy of the buffer (example below; UNVERIFIED, not compiled, follows from S9's COW rule). |
| A5 | removed (no references to return); for views, kept with defaults | Returned values are copies, and cheap for COW types. In-place returns are accessors (SE-0474, SE-0507) whose result lives for one access. A function returning a `Span` needs `@_lifetime`, defaulted for getters and parameterless methods (S12). I found no statement on conditionally returned borrows (UNVERIFIED). |
| A6 | removed for ordinary structs; kept for `~Escapable` structs | A struct can hold a class reference (owning, counted) but never a borrow. A struct that holds a `Span` must itself be `~Escapable`, and its initialiser needs `@_lifetime(copy x)` or `borrow x` (S12). |
| A7 | removed (not expressible) | Safe Swift has no interior pointers. A value can move, and nothing points into it. Self-reference is written with indices or class objects. |
| A8 | moved to run time | Classes with `weak` (zeroing, side table, S7) or `unowned` (traps if the object is gone). A cycle that is not broken leaks; Swift has no cycle collector (S5, §5). |
| A9 | moved to run time | A class *is* the shared mutable cell. No `RefCell` wrapper; the per-property dynamic check traps on conflict (S1, S2). |
| A10 | mostly removed | Two kinds instead of six: values (with COW for collections) and classes. `weak`/`unowned` remain as annotations. `~Copyable` adds unique ownership, opt-in. |
| A11 | removed for copyable types; kept (statically) for `~Copyable` | Copies are implicit. For COW types a copy is one count increment. `consume` is an optional optimisation. For `~Copyable`, use after consume is a compile error (SE-0390). Partial consumption is allowed (SE-0429). |
| A12 | mostly removed; the cycle risk is kept | Closures capture variables by reference, with no `FnMut`/`FnOnce` split. Closures are non-escaping by default (SE-0103). A variable captured by an escaping closure is boxed and checked dynamically (S1). Capturing `self` in an escaping closure can form a retain cycle, hence `[weak self]`. |
| A13 | removed | `any P` works for every protocol since 5.7 (SE-0309, S16). `as?`/`as!` downcast safely. The container is a 3-word inline buffer plus metadata plus one witness table pointer per protocol (`TypeLayout.rst`, S8). |
| A14 | removed (no `Pin`); kept: isolation checking | Async frames are runtime-managed, so there is no pinning. The `Sendable`/isolation checks apply across `await`. |
| A15 | kept at compile time, softened by inference | `Sendable` plus actors. Region-based isolation (SE-0414) infers when a non-`Sendable` value may be transferred. `Mutex` (SE-0433). Swift added SE-0466 (6.2, control default actor isolation) to cut annotation load. Its motivation text was not read; that this was the reason is UNVERIFIED. |
| A16 | mostly removed | Retroactive conformances are allowed (no Rust-style orphan error). `some P` hides bounds. Specialisation is done by the optimiser. |
| A17 | partly removed | Compile-time: "overlapping accesses" errors. My memory says these carry fix-its (copy to a local; `swapAt`), but the wording is UNVERIFIED. Run-time: the trap names the address and both accesses (S8). |
| A18 | kept, in another form | Definite initialisation of every stored property. Class initialisation is two-phase (designated/convenience initialisers), a known Swift friction (UNVERIFIED, no measurement). |
| A19 | moved to run time and compile time | Globals are lazily initialised and dynamically exclusivity-checked (S1). Swift 6 requires a global `var` to be isolated or `Sendable`, or marked `nonisolated(unsafe)` (SE-0412). |
| A20 | kept | `Unsafe*Pointer`, `withUnsafeBufferPointer { }` closures. SE-0458 marks unsafe uses explicitly. |

**A1, the struct case** (my own example, UNVERIFIED, not compiled; the rule is from S1):

```swift
struct World {
    var items: [Int] = []
    var log: [String] = []
    mutating func run() {
        items.forEach { log.append("\($0)") }   // error: overlapping accesses to 'self'
        for x in items { log.append("\(x)") }   // fine: the loop copied items (one count increment)
    }
}
```

**A4, mutation during iteration** (my own example; UNVERIFIED, not compiled; it follows from value
semantics and S9):

```swift
var pool = [1, 2, 3]
for x in pool {
    if x == 1 { pool.append(4) }   // first append copies the 3-element buffer (COW)
}
// the loop ran 3 times; pool is [1, 2, 3, 4]
```

The hazard Rust guards against (a dangling iterator) cannot happen, because the loop holds its own
counted reference to the old buffer. The cost is one O(n) copy, and only when the loop actually
mutates.

**The owner's ECS case in Swift** (ODR-286; my own sketch, UNVERIFIED, not compiled):

```swift
protocol AnyPool: AnyObject { func remove(_ e: UInt32) }
final class Pool<T>: AnyPool {
    var dense: [T] = []
    var sparse: [UInt32: Int] = [:]
    func remove(_ e: UInt32) { /* swap and pop */ }
}
final class Registry {
    var pools: [ObjectIdentifier: AnyPool] = [:]
    func pool<T>(_: T.Type) -> Pool<T> { pools[ObjectIdentifier(T.self)] as! Pool<T> }
    func patch<T>(_ e: UInt32, _ f: (inout T) -> Void) {
        let p = pool(T.self)                  // +1 count on the pool
        f(&p.dense[p.sparse[e]!])             // dynamic write access on p.dense for the call
    }
}
reg.patch(e) { (pos: inout Position) in pos.x += 1 }
```

Swift answers ODR-286 with "never return the reference". The caller passes code in, or uses an
accessor, and the access is held while that code runs. If `f` reaches the same pool through a second
handle and appends to `dense`, the dynamic check on the `dense` property traps. The result is a
deterministic panic, not a dangling pointer. This is the hazard ODR-286 describes ("a second handle
to the same pool can grow the pool's array while the reference lives"). Swift turns it into a run-time
trap and keeps `patch` legal.

### 1.3 Swift's costs, with measurements

**Per operation:**

- **RC update.** Swift's refcount word is updated atomically. The BRC paper (S4) found the compiler
  cannot avoid atomics even for single-threaded programs, because Swift compiles modules separately.
  Removing the atomic CAS (unsafely) cut execution time by 25% on average across the programs that
  still ran (S4, §3). Making the owner thread's increments non-atomic made each RC operation "more
  than twice" faster in the common case (S4, abstract).
  - Setup: Swift 4-era runtime; Intel Core i7, 4 cores at 3.5 GHz, 16 GB, Ubuntu 16.04; 10 runs
    averaged (S4, §6).
  - Swift's optimiser already removed up to 97% of RC operations in those programs (S4, §7).
- **RC share of run time.** RC operations took 42% of execution time in client programs and 15% in
  server programs, 32% across all of them (S4, §3, Figure 3; same machine).
  - An earlier IBM study (Ungar, Grove, Franke, DLS 2017) is quoted as seeing ARC take up to 80% of
    run time. A limit study making all RC non-atomic gave up to 220% speedup on 171 microbenchmarks,
    120% on 9 Benchmarks Game programs and 70% on Richards (S6). **UNVERIFIED:** I could not open the
    paper (the ACM page is 403 and the mirror certificate has expired). These numbers come from a
    search-engine abstract only.
- **Atomic vs non-atomic RC, outside Swift.** Koka with atomic-only RC ran 5% (rbtree) to 59%
  (nqueens) slower than with Koka's thread-shared flag. Setup: AMD 3600XT at 3.8 GHz, 64 GiB, Ubuntu
  20.04, median of 10 runs (Perceus TR, S5, §4). The same paper says Swift's RC "seems less
  effective" than Koka's on its allocation-heavy benchmarks.
  - I could not reliably match the figure's per-language bars to numbers from the PDF text, so no
    per-benchmark Swift ratio is given here.
- **Dynamic exclusivity check.** Each dynamically checked class-property access is a runtime call
  (S3). When run-time checks were turned on in release builds (Andrew Trick, Swift forums,
  2018-11-03, S3), 11 of the 656 public microbenchmarks in the Swift repository regressed by more
  than 10%. The worst five (numbers as listed in S3):

  | benchmark | before | after | slowdown |
  |---|---|---|---|
  | ClassArrayGetter2 | 139 | 1307 | +840% |
  | HashTest | 631 | 1233 | +95% |
  | NopDeinit | 21269 | 32389 | +52% |
  | Hanoi | 1478 | 2166 | +47% |
  | Calculator | 127 | 158 | +24% |

  - Units are the suite's own; the post does not state them.
  - These benchmarks share one pattern: an array stored in a class property and accessed repeatedly
    in a loop that does little else (S3).
  - The post gives no median. The Swift 5 blog (S2) says the impact "should be small in most cases"
    and advises avoiding class-property access in hot loops. It gives no number.
- **Swift's dynamic check stores nothing in the object.** The access set is per thread, in TLS (S8).
  The data structure behind it is UNVERIFIED: the implementation of `swift_beginAccess` was not in
  the file I fetched.
- **Ember's check works differently.** Ember puts a 4-byte access word per non-`Copy` field into the
  object (`[EXC-19]`), and its check is "a load and a compare, with a store only for a held access".
  That is more memory than Swift's per-object cost of zero, and less time than a runtime call. No
  measurement of either exists in this report.
- **COW.** Each mutation of a collection first checks that the buffer is uniquely referenced, by
  loading the count and comparing it (S9, `isKnownUniquelyReferenced`). A shared buffer is copied
  whole. I found no published per-check measurement.
- **Bounds checks.** Array and `Span` subscripts are always checked; `Span.subscript(unchecked:)` is
  the opt-out (S10).
- **Existentials.** An `any P` value is a 3-word inline buffer plus metadata plus witness-table
  pointers (S8). Calls through it are indirect. That a value too big for the buffer goes to the heap
  is standard Swift ABI knowledge, but S8 did not say so in what I read (UNVERIFIED).

**Memory per object:**

- Class instance: 16-byte header (S7), plus a side table once a `weak` reference exists (S7).
- Collections: a heap buffer with its own counted header. Its size is not measured here.
- C has none of these.

**Whole-program comparison against C**, from the Computer Language Benchmarks Game (S15), fastest
program per language. Swift 6.0.3 against Ubuntu clang 19.1.1, on the site's machine; the machine is
not stated on the fetched page. Seconds, and memory in the site's units (not stated on the page):

| benchmark | Swift s | C s | Swift mem | C mem |
|---|---|---|---|---|
| fannkuch-redux | 8.20 | 2.25 | 13,722 | 2,933 |
| n-body | 5.45 | 2.20 | 24,928 | 2,494 |
| spectral-norm | 1.45 | 0.39 | 13,689 | 4,706 |
| mandelbrot | 1.35 | 1.23 | 57,836 | 36,614 |
| fasta | 2.20 | 0.78 | 39,002 | 2,257 |
| k-nucleotide | 14.45 | 6.34 | 273,949 | 134,189 |
| binary-trees | 17.62 | 1.68 | 733,094 | 176,222 |

- These are different programs written by different people. The C programs use tuned techniques
  (SIMD, custom allocators). The gap is **not** the memory model alone.
- The site marks binary-trees as contentious. That the C entry uses a pool allocator is from my
  memory, UNVERIFIED.
- binary-trees is the closest thing to an ARC stress test here: Swift is 10.5x slower and uses 4.2x
  the memory.

**What Swift cannot express, or makes slow:**

- Returning or storing a reference into a value, until `~Escapable` plus `@_lifetime`, which are
  still experimental (S12).
- Two mutable elements at once (A3).
- A hot loop over class properties without a dynamic check per access, unless the optimiser proves
  the class property is private (S2).
- Cycles without `weak`/`unowned`.
- Non-atomic RC for single-threaded code: Swift has no language-level way to say "this object never
  crosses a thread" (S4).

### 1.4 Maturity

**Shipped, in production.** Since Swift 5 (2019), ARC, COW and both static and dynamic exclusivity
have been on by default in release builds (S2). These are the core of iOS/macOS development.

Current status of the newer parts (S16):

- `~Copyable`/`borrowing`/`consuming`: shipped (5.9, 6.0).
- `~Escapable`, `Span`, `MutableSpan`, `InlineArray`: shipped (6.2).
- Borrow/mutate accessors: shipped (6.4).
- Yielding accessors: accepted.
- Lifetime-dependency annotations: **experimental** (`@_lifetime`, feature flag `Lifetimes`), in
  pitch #3 as of 2026-02 (S12).

---

## 2. Mojo

### 2.1 The model in plain words

Mojo values are structs with one owner each. Argument conventions say what a function gets (M1):

- **default (read):** an immutable reference.
- **`mut`:** a mutable reference. Changes are visible to the caller.
- **`var`:** the function owns its argument. The caller passes a copy, or moves with the `^` sigil.
- **`out`:** an uninitialised result that the function must initialise.
- **`ref`:** a reference whose mutability follows the origin it comes from.
- **`deinit`:** the argument is destroyed by the callee.

```mojo
def take_text(var text: String):
    text += "!"

def main():
    var message = "Hello"
    take_text(message^)        # moved; using message afterwards is a compile error (M1)
```

**Argument exclusivity** (M1): a `mut` argument may not alias any other argument passed to the same
call. Mojo does not enforce this for register-passable trivial types such as `Int`, which are passed
by copy (M1).

**Copies** (M9):

- A type that is `ImplicitlyCopyable` copies on assignment.
- A type that is only `Copyable` needs `.copy()`.
- Since 1.0, `Array` is no longer `ImplicitlyCopyable` (M4), so copying an array is spelled out.

**Origins** (M2) are Mojo's lifetimes. An origin records which variable a reference points into and
whether it is mutable.

- Origins are inferred inside function bodies (`ref first = list[0]`).
- A function returning a reference writes the origin in its signature. The origin can be a union for
  a conditional return:

```mojo
def pick_one(cond: Bool, ref a: String, ref b: String) -> ref[a, b] String:
    return a if cond else b          # both a and b stay alive while the result is used (M2)
```

- A reference keeps the value it refers to alive: "a live `ref` keeps the value it refers to alive"
  (M9).
- Structs that hold references take an origin *parameter* (`struct ImmutRef[origin: ImmOrigin]`, M2).
- Escape hatches: wildcard origins (`ImmUnsafeAnyOrigin`, `MutUnsafeAnyOrigin`) and untracked origins
  (`MutUntrackedOrigin`). The docs say a wildcard origin disables ASAP destruction in its scope and
  disables argument exclusivity (M2).

**Interior origins** (Mojo 1.0, August 2026, experimental, M4). `List` element references are bound
to an *interior* origin of the list. Mutating the list (`append`, `pop`) invalidates them, and a later
use is a compile error. Before 1.0 such a reference "silently" dangled after a reallocation: the
release notes say the checker now rejects it "instead of letting it silently dangle" (M4).

```mojo
var xs = [1, 2, 3]
ref first = xs[0]
xs.append(4)          # may reallocate
print(first)          # error: use of invalidated interior reference  (message from M4)
```

**ASAP destruction** (M3): a value is destroyed right after its last use, not at the end of the
scope. This applies even to temporaries in the middle of an expression. Fields are destroyed one by
one inside `__deinit__`. The docs name two consequences: an RAII guard (a lock) can be released too
early, and a pointer whose origin was erased can dangle. "Neither produces a compiler error" (M3). The
fix is `_ = x`, which marks a later last use.

```mojo
var guard = lock.acquire()
critical_section()
_ = guard             # without this, guard may be destroyed before critical_section (M3)
```

**Pointers** (M6):

- `Pointer` is non-owning and non-nullable, with `unsafe_`-prefixed unsafe operations.
- `OwnedPointer` is the single owner, like `Box`.
- `ArcPointer` is shared ownership with a thread-safe count. Reading and writing the payload is not
  synchronised, and callers are responsible for that (M6).
- Self-referential structures, lists and trees are built with `Pointer[..., MutUntrackedOrigin]` and
  manual `alloc`/`dealloc` (M7).

**Closures** (M8) have explicit capture lists: `{imm x}`, `{mut total}`, `{var copy}`, `{var x^}`
(move) and `{ref x}`.

### 2.2 A1 to A20 in Mojo

| id | verdict | how |
|---|---|---|
| A1 | kept, probably | A `mut self` method takes the whole value. Whether `f(mut a.x, a.y)` is accepted as disjoint is UNVERIFIED; no document found says so. |
| A2 | kept (static), new in 1.0, experimental | Interior origins reject using an element reference after `append`/`pop` (M4). Before 1.0 this was a silent dangling reference, a safety hole (M4). This is Ember's current rule, spelled the same way. |
| A3 | kept | Argument exclusivity forbids a `mut` argument aliasing another argument (M1). The roadmap lists "supporting mutable aliasing" as not started (M5). |
| A4 | kept, presumably | A loop's element references belong to the list's interior origin, so mutating the list inside the loop and then using the element should be refused (UNVERIFIED: not stated for iterators). |
| A5 | partly removed | No lifetime parameters. The origin is written in the signature (`-> ref[self] T`, `-> ref[a, b] T`, `-> ref[xs[0]] T`, M2, M9). Shorter than Rust, but still written; Ember infers it today. |
| A6 | kept | A struct holding a reference has an origin parameter (M2), which spreads through types as Rust's lifetime parameters do. |
| A7 | kept, unsafe | Self-reference is built with untracked pointers and manual allocation (M7). |
| A8 | kept, unsafe | Graphs, trees and lists use untracked `Pointer` or `ArcPointer`. No weak pointer is documented (M6, M7). |
| A9 | "removed" by giving up checking | `ArcPointer` mutation is unchecked (M6). Within one thread, two copies of an `ArcPointer` can each mutate the payload. Whether a reference into a `List` inside the payload, taken through one copy, is invalidated when the list grows through the other copy is UNVERIFIED. I expect it is not caught, because the two copies have different origins. That is the ODR-286 hazard. |
| A10 | partly removed | Three kinds (`Pointer`, `OwnedPointer`, `ArcPointer`). No `Cell`/`RefCell`/`Weak`, because the checks they provide are simply absent. |
| A11 | kept, and more explicit since 1.0 | `^` moves. `.copy()` is required for non-`ImplicitlyCopyable` types, now including `Array` (M4, M9). |
| A12 | kept (explicit capture lists) | `{mut total}`, `{var x^}`, etc. (M8). Whether a `mut`-capturing closure may escape is not documented (UNVERIFIED). |
| A13 | kept, and worse than Rust | No existentials or dynamic traits; the roadmap lists "Existentials / dynamic traits" as not started (M5). An EnTT-style registry needs unsafe type erasure. |
| A14 | not applicable yet | Async is not in 1.0; it is in progress (M5, M10). |
| A15 | not addressed | No `Send`/`Sync`-like trait was found in the docs, release notes or roadmap (M4, M5). The `ArcPointer` payload is unsynchronised (M6). That `parallelize` closures are unchecked is UNVERIFIED. |
| A16 | partly removed | Compile-time parameters and `where` clauses. Conformance must be declared, and implicit duck-typed conformance was removed in 1.0 (M4). Orphan rules: UNVERIFIED. |
| A17 | unknown | One message seen: "use of invalidated interior reference" (M4). It says what, not what to do. |
| A18 | kept | The `out self` convention plus tracked initialisation state. Moving out of fields is only allowed in a deinitialiser (M3). |
| A19 | UNVERIFIED | No document read covers global mutable variables. |
| A20 | kept | `unsafe_`-prefixed operations on `Pointer` (M4, M6). |

**The owner's ECS case in Mojo** (my own sketch, UNVERIFIED, not compiled):

```mojo
struct Pool[T: Copyable & Movable]:
    var dense: List[T]
    def get(ref self, i: Int) -> ref[self] T:
        return self.dense[i]
```

A pool can return a reference into itself with a written origin. But a registry holding pools of
*different* `T` in one `Dict` needs a trait object, which Mojo does not have (M5). The only route is
an untracked or wildcard-origin pointer, and the docs say that switches off exclusivity checking and
ASAP destruction for it (M2).

### 2.3 Mojo's costs, with measurements

**Per operation:**

- The model itself adds nothing at run time. There is no RC except `ArcPointer`, no dynamic
  exclusivity check and no copy-on-write. Checking is static, or absent.
- `ArcPointer`'s count is thread-safe (M6). Whether it is atomic is not stated, so treat it as
  presumably atomic (UNVERIFIED).
- Bounds checks:
  - The current `List` docs say accessing an invalid index aborts.
  - `unsafe_get` is undefined behaviour out of range (M11).
  - A 2025 forum thread said list bounds checks were off by default unless built with
    `-D ASSERT=all` (M12).
  - The current default assertion mode is `safe`, which runs only assertions marked safe (M12).
  - Whether `List.__getitem__` is checked in a default release build of Mojo 1.2 is **UNVERIFIED**.

**Memory per object:**

- A struct has no header.
- `ArcPointer` adds a heap block with a count. Its layout is UNVERIFIED (the API page returned 404).
- ASAP destruction frees memory at the last use, so peak memory is at most what scope-end destruction
  gives, and often lower. No measurement was found.

**Published measurements against C:**

- I found **no** independent, reproducible measurement of Mojo against C on CPU code that isolates
  the memory model.
- The Mojo 1.0 announcement makes no quantitative performance claim (M10).
- An arXiv paper (2606.16059) calibrates its comparison from Modular's own material rather than its
  own measurements. It is not used here.
- Blog posts found by search report single-machine, single-workload numbers either way. They are not
  used here.

**What Mojo cannot express, or makes unsafe:**

- Trait objects.
- Safe graphs, trees with parent pointers and doubly linked lists.
- Safe shared mutable state.
- Data-race freedom.
- Collection references were safe only from 1.0, as an experimental feature.

The safety Mojo offers today covers single-owner values passed through function arguments.

### 2.4 Maturity

- Mojo 1.0 was released on 2026-08-11 (M10). The latest stable release is 1.2.0, of 2026-10-22
  (M11, the release index).
- The compiler's sources were opened in August 2026. As of the search on 2026-10-24, it is reported
  as not yet accepting outside patches. That comes from news reports found by search, not checked
  against Modular (UNVERIFIED).
- **The ownership core is shipped**: conventions, `^`, origins, ASAP destruction.
- **Interior origins are experimental** (M4).
- Existentials, mutable aliasing and "finalizing internal origins" are not started; async is in
  progress (M5).
- The roadmap says some situations "still aren't safe by default", with no full fix expected before
  phase 2 (M5).

---

## 3. Side by side

| | Swift | Mojo | Ember today |
|---|---|---|---|
| references | second-class (`inout`, accessors); `Span` is non-escapable (6.2) | first-class `ref` with origins written in signatures | first-class `ref`, regions inferred, no written lifetimes |
| element reference across `append` | impossible (no such reference); `Span` blocks `append` statically | compile error since 1.0 (experimental); silent dangling before | compile error `E3022` (probe below) |
| mutate a collection while iterating it | allowed: COW snapshot | refused, presumably (UNVERIFIED) | `E3020` (probe below) |
| shared mutable objects | classes, ARC, per-property dynamic check | `ArcPointer`, unchecked | classes, RC, per-field access word (`[EXC-19]`) |
| object header | 16 B (S7) | none for structs; `ArcPointer` UNVERIFIED | 24 B (`[OBJ-1]`) + 4 B per non-`Copy` field |
| RC atomicity | always atomic (S4) | thread-safe count (M6) | non-atomic unless `Sync` (`[RC-4]`) |
| trait objects | `any P`, safe downcast | none | `dyn I`, `Box[dyn I]`, class interfaces |
| data races | `Sendable`, actors, region isolation | not addressed | `Sync`/`@sync` (not studied here) |

---

## 4. What Ember could take, and what it costs against C

Each idea names the annoyance it targets, its kind under the owner's guidance, and its cost. Kind (a)
means a more Pythonic spelling of Rust's guarantee. Kind (b) means a smarter mechanism that removes
the hazard. None of these is a ruling. They are inputs for phase 3.

### Idea 1: iterate by cursor, borrow one element per iteration (A4, part of A2). Kind (b)

Swift shows that a loop does not have to lock its container: Swift's loop holds its own snapshot.
Ember need not copy the buffer. Its loop can hold an **index**, not a pointer, and re-fetch the element
each iteration. A reference to the current element lives until its last use inside the iteration,
under the borrow rules Ember already has.

```ember
# today: E3020 "cannot mutate `a` while it is borrowed by this loop" (probe iter_push.em)
# proposed: accepted
for x in a:
    if x == 1:
        a.push(4)
```

**Hazard check.** The element reference is per iteration, and the existing rule (`E3022`) still
refuses using it after the `push` in the same iteration. No pointer survives into the next iteration.
The cursor is an index checked against the current length, so a shrinking container ends the loop
instead of reading freed memory.

**Open semantic choice for the owner, not ruled here.**

- **Snapshot length (Swift).** The loop runs over the elements present at the start. The example runs
  3 times.
- **Live length (Python).** The example never ends, because Python's list iterator sees the appended
  items.
- **Backward (EnTT).** EnTT iterates pools from the end, which makes destroying the current entity
  safe (UNVERIFIED for EnTT 3.x specifics; ecs-entt-design.md item 3 notes EnTT allows removing the
  current entity's components).

**Cost against C:**

- In a loop whose body cannot change the container (proved, as today), there is no extra cost and the
  bounds check hoists.
- In a loop whose body can change it, there is one compare of the index against the current length
  per iteration, and the element address is re-derived each iteration (a load of the data pointer).
  C does the same loop with no check and undefined behaviour on reallocation.
- No measurement exists. One predictable compare per iteration is my estimate of the shape, not a
  number.

### Idea 2: scoped access instead of a returned reference (A5, A13, ODR-286). Kind (a)

This takes Swift's yielding and `mutate` accessors (S13, S14) and `patch` (EnTT has `patch` too).
ODR-286 found that a `ref mut` returned from a pool reached through a map and a cast cannot be proved
safe while a second handle can grow the pool, and that `[EXC-18]`'s access "cannot follow the
reference to the caller". Swift does not return the reference. It keeps the access open **while the
caller's code runs**, and the caller's code is a block:

```ember
# proposed spelling (Python's `with`): the dynamic write access on pool.dense is held for the block
with reg.get_mut[Pos](e) as p:
    p.x += 1
    reg.emplace(other, Vel(...))   # another pool: fine
    # growing the *same* pool here through any handle hits the access word: deterministic panic
```

A one-statement form, `reg.get_mut[Pos](e).x += 1`, is the same thing with the block being the
statement.

- **Hazard check.** A second handle growing `dense` is a write access to `dense` while a write access
  is held, so `[EXC-1]` panics. The mechanism already exists and is per field (`[EXC-19]`).
- **Cost against C:**
  - Per block: one retain and release of the pool handle (non-atomic for non-`Sync` classes,
    `[RC-4]`).
  - One check and two stores on the access word.
  - The 24-byte header plus 4 bytes per non-`Copy` field of the pool object.
  - Inside the block, `p` is a plain pointer.
  - C pays none of this.
  - Swift's equivalent costs a runtime call per access, which is what made ClassArrayGetter2 840%
    slower (S3). Ember's in-object word should be cheaper, but this is UNVERIFIED: not measured.

### Idea 3: ASAP freeing for memory-only values (A11 adjacent; peak memory). Kind (b)

This takes Mojo's ASAP destruction (M3) without its trap.

- A value whose drop only frees memory (`Array`, `String`, `Box` of such) is freed at its last use.
- A value with a user `drop` (a lock guard, a file) keeps scope-end drop, as `[RC-3]` already
  requires for classes.

```ember
fn load() -> int:
    raw = read_file("big.bin")     # 1 GB
    n = count_lines(raw)           # last use of raw: freed here, not at the end of the function
    build_index(n)                 # peak memory no longer includes raw
    return n
```

- **Hazard check.** A memory-only value has no observable drop, so freeing it early cannot change
  behaviour. Borrows already end at their last use (`[BRW-2]`). Mojo's guard problem (M3) cannot
  arise, because guards keep scope-end drop.
- **Cost against C:** none at run time. Peak memory is equal to or lower than scope-end freeing. C
  frees wherever the programmer writes `free`. No measurement exists.

### Idea 4: views are the only stored references, dependency inferred (A5, A6). Kind (a)

This takes Swift's `~Escapable` with defaulted `@_lifetime` (S10, S12).

- Swift went from its first release (2014) to Swift 6.2 (2025) before admitting, with `Span`, that
  C-speed views require references that are not copies. Its answer limits where those references may go: never into a global, an escaping closure
  or an escapable type. The dependency is inferred in the common cases.
- Ember already infers regions with no written lifetimes. The part to take is the restriction. A
  struct that holds a view is itself a view type, so it is never stored in a global or a class or
  captured by an `owned` closure, and its dependency is inferred from its constructor's arguments.
  This keeps A6 from spreading lifetime parameters, because there are none to spread.
- **Cost against C:** none at run time, since a view is a pointer and a count, as in C. The
  restriction is the cost: a long-lived index structure must hold indices or handles, not views.

### Idea 5: infer `Send` from regions (A15). Kind (a)

This takes Swift's region-based isolation (SE-0414) and `sending` (SE-0430).

- A value may move to another thread with no `Send` annotation when the compiler sees that nothing
  else reaches it.
- Annotations are needed only for values that really are shared.
- **Cost against C:** none at run time.
- Not studied in depth here: the precision of SE-0414's analysis, and how it would fit Ember's
  `Sync` rules, are UNVERIFIED.

### Two things not to take

- **COW for arrays.** COW would let Ember drop A4 and A11 friction the Swift way. But it puts a count
  in every buffer, a uniqueness check before every mutation and hidden O(n) copies on the mutation
  path. ARC is the cost Swift measured at 42% of client-program run time (S4). Idea 1 gets A4 without
  any of that.
- **Mojo's unchecked `ArcPointer`.** It removes A9 by giving up safety (M6), which the owner's goal 2
  forbids.

### Already in Ember (Swift's idea, keep as is)

- Per-field dynamic exclusivity on class objects (`[EXC-19]`), with non-atomic counts for non-`Sync`
  classes (`[RC-4]`). This is the BRC paper's main win (S4), obtained by the type system instead of
  biasing.
- Tuple swap `a[i], a[j] = a[j], a[i]` type-checks today (probe `swap.em`). This is the Pythonic
  spelling of Swift's `swapAt` for A3, and `get_pair_mut` covers the rest (E3022's help text).

---

## 5. Ember probes (type and borrow checking only, `ember.exe check`)

Files are in `C:/Users/ism19/Code/ember/target/redesign/p2sm/`.

| file | what it does | result |
|---|---|---|
| `iter_push.em` | `for x in a:` then `a.push(4)` in the body | `E3020`, "cannot mutate `a` while it is borrowed by this loop" (A4 today) |
| `ref_then_push.em` | `r = ref mut a[0]`; `a.push(4)`; `r += 1` | `E3022`, already mutably borrowed (A2 today; the same rule as Mojo 1.0's interior origins) |
| `two_mut.em` | `bump(a[0], a[2])` with two `mut` parameters | accepted (constant indices) |
| `two_mut_var.em` | `bump(a[i], a[j])` | `E3022`; help names `get_pair_mut(i, j)` and `split_at` (A3 today) |
| `swap.em` | `a[i], a[j] = a[j], a[i]` | accepted |
| `class_two_handles.em` | two handles `p`, `q` to one class; `for x in p.dense: q.dense.push(x)` | accepted by the checker. By `[EXC-1]`/`[EXC-19]` this should panic at run time (write to `dense` while the loop's read access is held). Not run (rules). This is Swift's "moved to run time" in Ember today. |
| owner's `ecs5/pos.em` | `ref mut` into a class pool found by `cast` of a map value | `E3060` (re-checked; ODR-286's wall) |
| owner's `ecs5/box_bad.em` | `clear()` while a returned `ref mut` lives | `E3025` |
| owner's `ecs4/accept_change_during_loop.em` | `emplace` into the pool while looping over the span returned by `view.values()` | accepted by the checker. The file's header expects `run-pass` with output 30. By `[EXC-18]` and `[EXC-1]`, the returned span carries a read access on `dense` and the `push` is a write, so a panic would be expected instead. Not run; the contradiction is noted for phase 3, not resolved here. |

---

## 6. Sources

**Swift**

- S1 SE-0176, Enforce Exclusive Access to Memory:
  https://github.com/swiftlang/swift-evolution/blob/main/proposals/0176-enforce-exclusive-access-to-memory.md
- S2 Swift 5 Exclusivity Enforcement (swift.org blog): https://www.swift.org/blog/swift-5-exclusivity/
- S3 Andrew Trick, "Enabling run-time exclusivity checking in release mode", Swift forums,
  2018-11-03: https://forums.swift.org/t/enabling-run-time-exclusivity-checking-in-release-mode/17626
- S4 Choi, Shull, Torrellas, "Biased Reference Counting", PACT 2018:
  https://iacoma.cs.uiuc.edu/iacoma-papers/pact18.pdf (text extracted locally with `pdftotext`)
- S5 Reinking, Xie, de Moura, Leijen, "Perceus", MSR-TR-2020-42:
  https://www.microsoft.com/en-us/research/uploads/prod/2020/11/perceus-tr-v4.pdf
- S6 Ungar, Grove, Franke, "Dynamic Atomicity: Optimizing Swift Memory Management", DLS 2017,
  doi 10.1145/3133841.3133843. Numbers from a search abstract only (UNVERIFIED).
- S7 Swift runtime `RefCount.h`:
  https://github.com/swiftlang/swift/blob/main/stdlib/public/SwiftShims/swift/shims/RefCount.h
- S8 Swift `docs/ABI/TypeLayout.rst`, and `stdlib/public/runtime/Exclusivity.cpp` (swiftlang/swift,
  main)
- S9 Swift `docs/OptimizationTips.rst` (swiftlang/swift, main)
- S10 SE-0446 Nonescapable Types; SE-0447 Span (swift-evolution, main)
- S11 SE-0467 MutableSpan (swift-evolution, main)
- S12 "[Pitch #3] Compile-time lifetime dependency annotations", 2026-02-25:
  https://forums.swift.org/t/pitch-3-compile-time-lifetime-dependency-annotations/84968
- S13 SE-0474 Yielding accessors (swift-evolution, main)
- S14 SE-0507 Borrow and Mutate Accessors:
  https://github.com/swiftlang/swift-evolution/blob/main/proposals/0507-borrow-accessors.md
- S15 Benchmarks Game, Swift vs C clang:
  https://benchmarksgame-team.pages.debian.net/benchmarksgame/fastest/swift.html (fetched
  2026-10-24)
- S16 Swift Evolution status index: https://download.swift.org/swift-evolution/v1/evolution.json
  (fetched 2026-10-24). The statuses quoted for SE-0103, 0302, 0306, 0309, 0366, 0377, 0390, 0412,
  0414, 0427, 0429, 0430, 0433, 0446, 0447, 0453, 0456, 0458, 0466, 0467, 0474 and 0507 come from it.
- S17 SE-0390, SE-0377, SE-0427, SE-0414, SE-0430, SE-0433, SE-0458: the proposals of those numbers
  in swift-evolution/proposals. Only SE-0390 was read in full; the others are cited for title and
  status (S16).

**Mojo**

- M1 Ownership: https://mojolang.org/docs/manual/values/ownership/
- M2 Lifetimes, origins, and references: https://mojolang.org/docs/manual/values/lifetimes/
- M3 Automatic destruction: https://mojolang.org/docs/manual/lifecycle/death/
- M4 Mojo 1.0.0 release notes: https://mojolang.org/releases/v1.0.0/
- M5 Roadmap: https://mojolang.org/docs/roadmap
- M6 Intro to pointers: https://mojolang.org/docs/manual/pointers/
- M7 Self-referential structs: https://mojolang.org/docs/manual/structs/reference/
- M8 Closures: https://mojolang.org/docs/manual/functions/closures/
- M9 Ownership cheat sheet: https://mojolang.org/docs/reference/cheat-sheets/ownership/
- M10 "Modular 26.5: Mojo 1.0 is here!", 2026-08-11:
  https://modular.com/blog/modular-26-5-mojo-1-0-is-here
- M11 Release index https://mojolang.org/docs/changelog, and the List API
  https://mojolang.org/docs/std/collections/list/List
- M12 "Boundary checks on lists do not work", Modular forum:
  https://forum.modular.com/t/boundary-checks-on-lists-do-not-work/1797 (via search summary), and
  `debug_assert` docs

**Not available:** no Swift or Mojo toolchain (no install allowed). The ACM pages for the DLS 2017
paper returned 403. The `ArcPointer` API page returned 404.
