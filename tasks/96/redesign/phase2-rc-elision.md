# Phase 2: languages. Reference counting with compile-time elision: Lobster, Koka (Perceus), Roc, Nim ARC/ORC

Written 2026-10-24 by the phase-2 "rc-elision" research agent.

What was done, and what could not be done:

- None of the four languages is installed on this machine (`which nim koka roc lobster` finds nothing), and
  installs are not allowed, so **no program in any of the four languages was compiled or run here**. Every
  statement about how they behave comes from their documentation, papers or source files (named at each
  claim). Statements I inferred from source code without running it are marked UNVERIFIED.
- What their model costs against plain C was **measured here** with C programs that do the same work in
  each model's layout: `target/redesign/phase2-rc-elision/rc_models.c`, built with
  `clang -O2` (clang 22.1.8), run pinned to logical CPU 1 (a P-core of the Intel Core Ultra 9 275HX), on
  mains power, 5 rounds, one mode per process. Raw output: `target/redesign/phase2-rc-elision/results.txt`.
  Every number below marked "measured here" is the median of those 5 rounds.
- Four Ember probes were checked with `target/ci/ember.exe check` (type and borrow checking only, never
  run): `target/redesign/phase2-rc-elision/*.em`.

## 0. The short version

All four languages count references at run time and use the compiler to remove most of the counting.
None has Rust's borrow checker. They get memory safety without it in one of three ways:

1. **Nothing points into a buffer** (Lobster, Nim `ref`, Ember `class` today). A program that holds an
   element holds the counted *object*, and growing the array moves only pointers to it. The hazard behind
   Rust's A2/A3/A4 rules (a pointer into a buffer that moves) cannot arise. The cost is one heap object, one
   count header and one indirection per element. **Measured here: iterating 4M counted objects is 3.6x
   slower than iterating 4M inline structs, 9.7x when the objects are scattered in memory, and uses 2.4x
   the memory.**
2. **Nothing is mutable that is shared** (Koka, Roc). Values are immutable in the language. The run time
   checks "is this count 1?" before an update, and if so updates in place (Perceus "reuse", Roc
   "opportunistic in-place mutation"). This removes A1 to A6 and A11 entirely: there are no references to
   return or to invalidate, and nothing is ever "moved from". **Measured here: the is-unique check adds 10%
   to random writes into a 4M array (94.5 ms to 103.7 ms); when the array *is* shared, the first write
   silently copies it: 17.8 ms and 2x peak memory for 64 MiB.**
3. **Not checking** (Nim). Nim's `var` parameters, `var`/`lent` returns, `mitems` and `{.cursor.}` are
   pointers the compiler does not fully check, and `ptr`/`addr`/`cast` need no `unsafe` block. Nim is
   pleasant to write and fast, but it does not meet the owner's goal 2.

What Ember should take (section 5, each with its kind and its cost against C):

| # | idea | from | kind | removes | cost against C |
|---|---|---|---|---|---|
| 1 | use after move becomes a share, not an error (copy-on-write values) | Roc, Koka, Lobster, Nim | (a) | A11, most of A10 | zero where the compiler proves the move; one count + one is-unique test per write where it cannot (measured +10%); a hidden copy only when both copies are really written |
| 2 | per-object "thread-shared" bit instead of `@sync` classes | Koka | (b) | the A15 annotation part | free in the fast path (one test, folded into the drop test); a one-time marking walk when an object graph is sent |
| 3 | an 8-byte object header | Koka, Lobster, Nim ARC | (b) for goal 1 | none (memory) | Ember's header is 24 bytes; 16 bytes saved per object |
| 4 | cycle report at exit in the debug profile; cycle collector only for types that can form cycles | Lobster, Nim ORC | (b) | A8 (choosing `Weak`), A7 | debug only: zero in release; a collector costs 8 more header bytes and a root-buffer registration per decrement of a cyclic type |
| 5 | update through the store, never return a reference (ECS) | Roc, Koka FBIP | (a) | A5 for ODR-286 | none when inlined; a copy of the component when not |

Not to take: Nim's unchecked pointers (breaks goal 2), Roc's ban on mutation and its always-atomic counts
(breaks goals 1 and 3), Lobster's non-escaping closures (breaks goal 3), counted elements as the *default*
for arrays (breaks goal 1: the 3.6x above).

---

## 1. Lobster

### 1.1 The model in plain words

Lobster (Wouter van Oortmerssen, a game-programming language with Python-like indentation syntax) has two
kinds of user types ([language reference](https://aardappel.github.io/lobster/language_reference.html)):

- a `class` is "a dynamically allocated object copied by reference": a heap object with a count;
- a `struct` is stored inline in its parent (a variable, a vector, another object) and copied by value.

```lobster
struct int2:
    x:int
    y:int

class Player:
    pos:int2
    hp:int
```

Every heap object is counted. The compiler's **ownership analysis** ("compile time reference counting",
[memory management page](https://aardappel.github.io/lobster/memory_management.html)) gives each allocation
one owner, usually the first variable, field or vector element it is stored in. Every other use is a
**borrow** and emits no count operation. Only where a second place really needs to own the value does the
compiler insert an increment. Where Rust would refuse the program, Lobster pays an increment instead:

```lobster
var x = "hello"
def f(y):
    x = "world"     // overwrites the value y borrows
    print y         // prints "hello": the caller keeps the old value alive for the call
f(x)
```

A variable that is never reassigned and is initialised from another place borrows from it. If that place
is later overwritten while the variable is still used, the variable is **retroactively made an owner**: its
initialiser gets an increment and its scope end a decrement. The page says the program "behaves the same"
either way; only the cost changes.

Errors remain only where no caller can be held responsible: the page's example is
`f(v[0], v.pop())`, which reports "cannot modify x while y borrows it". The page calls these errors very
rare and reports that moving "several dozens of Lobster programs" from run-time to compile-time counting
needed changes in "2 locations".

Return values are owned (one exception: returning a single variable is a borrow). Function arguments are
specialised per caller as owned or borrowed, except for large functions and dynamic dispatch.

Other parts that matter here (language reference):

- Function values are "always non-escaping": a lambda may use the enclosing variables, but calling it after
  the enclosing frame ended is a run-time error.
- Threads: each thread runs its own VM; "it does not allow threads to share memory or any other VM state";
  threads exchange copied messages through a tuple space.
- Cycles are not collected; the program reports leftover objects at exit, and the programmer breaks the
  cycle by setting a field to `nil`.
- Header (source [`vmdata.h`](https://raw.githubusercontent.com/aardappel/lobster/master/dev/src/lobster/vmdata.h)):
  `RefObj` is a type index (`type_elem_t tti`) plus `int refc`, 8 bytes assuming both are 4 bytes
  (the source does not state the widths: UNVERIFIED). A vector's elements are 8-byte `Value`s, a struct
  element taking `width` of them inline.

### 1.2 A1 to A20 in Lobster

| id | verdict | how |
|---|---|---|
| A1 whole-object borrow | removed (b) | objects are reached through counted pointers; a method holding one field does not lock the others; if the field it holds is overwritten, the holder becomes an owner (an increment), not an error |
| A2 container invalidation | removed (b) | an element of a `class` type is a counted pointer; growing the vector moves pointers, not objects; a `struct` element read out is a copy. The one refusal left is `f(v[0], v.pop())` |
| A3 two mutable elements | removed (b) | two elements are two object pointers (or two struct copies) |
| A4 change while iterating | UNVERIFIED | the docs do not say what `for` does when the vector grows inside the loop; class elements stay valid by (b) above |
| A5 returning references | removed (a) | there are no references to return: a function returns the object (owned, +1) or a struct copy |
| A6 structs holding references | removed (a) | fields hold objects (counted) or structs (inline); no lifetime parameter exists |
| A7 self-referential | moved to run time | allowed; it is a cycle and leaks, reported at exit |
| A8 graphs, parent pointers | moved to run time | allowed; cycles leak and are reported at exit; the docs mention no weak reference |
| A9 shared mutable state | removed (b) | a `class` object is shared and mutated freely; no `RefCell`, no run-time borrow panic, no lock (no shared memory between threads) |
| A10 ownership kinds | removed (a) | two kinds: `class` (shared, counted) and `struct` (value) |
| A11 moves, clone | removed (a) | no move is visible; a second owner costs an increment the compiler inserts |
| A12 closures | kept, differently | lambdas capture freely but cannot escape: storing a callback for later is a run-time error. Python programmers will hit this (event handlers) |
| A13 trait objects | removed (a) | dynamic dispatch on the first argument, `is` type test, `switch` on a class narrows the variable |
| A14 async | not applicable | no async in the language reference |
| A15 threads | removed (b), at a cost | no shared memory: no data race is possible, but sharing a large structure means copying it |
| A16 generics friction | removed (a) | generic functions are specialised per use without written bounds (errors appear at the instantiation) |
| A17 error messages | mostly removed | few borrow errors exist; the remaining ones name the borrower ("cannot modify x while y borrows it") |
| A18 initialisation | UNVERIFIED | not studied |
| A19 global mutable state | removed | top-level `var`; one VM per thread, so no race |
| A20 unsafe and FFI | not applicable | no user-level unsafe code; native functions are written in C++ |

### 1.3 Costs

- **Count operations removed:** the memory-management page says the analysis "was able to remove around
  95% of runtime reference count operations". The page gives no benchmark, no program list and no
  measurement method for that figure. It is the author's claim (UNVERIFIED by any published measurement I
  found). Lobster has a `--rcstats` option that counts every increment the generated code performs
  (`vmdata.h` comment), so the number is measurable, but not here (Lobster is not installed).
- **Per object:** 8 bytes of header (see above, widths assumed) plus the allocator's own overhead, plus one
  pointer in the parent.
- **What a counted element costs against an inline one, measured here** (`rc_models.c`, 4,194,304
  16-byte components, 20 summing passes):

| mode | layout | work ms (median of 5) | vs C | peak memory |
|---|---|---|---|---|
| `inline_iter` | C: one contiguous array of structs | 57.9 | 1.00x | 69,028 KB |
| `boxed_iter` | one heap object (8-byte count + 16-byte value) per element, array of pointers, count operations elided | 206.3 | 3.56x | 168,348 KB (2.44x) |
| `boxed_iter_shuffled` | the same, pointer order shuffled (a heap after churn) | 563.9 | 9.74x | 168,352 KB |
| `boxed_iter_rc` | the same as `boxed_iter` plus one non-atomic increment and decrement per element (elision failed) | 276.9 | 4.78x | 168,352 KB |

  The 168 MB is 32 MiB of pointers plus about 33 bytes per object (24 bytes asked of `malloc`, the rest is
  the Windows heap's per-block overhead). The 2.44x is therefore partly the allocator; Koka's mimalloc with
  8-byte alignment would waste less (the Perceus paper says so for its own allocator, below).
- **What it cannot express or makes slow:** a struct element cannot be mutated through a long-lived
  reference (there are no references into a vector); escaping closures; shared-memory parallelism.

### 1.4 Maturity

Shipped and maintained by one author: GitHub releases v2025.3 to v2026.8, the last dated 22 September
([releases page](https://github.com/aardappel/lobster/releases)). I found no production user besides the
author's own game prototypes, which the memory-management page cites. Call it: shipped, small, one-person.

---

## 2. Koka with Perceus

### 2.1 The model in plain words

Koka (Microsoft Research, Daan Leijen) is a functional language with effect types. Data is immutable
algebraic types; mutation exists only as local `var`s and first-class `ref` cells. Memory is managed by
**Perceus** ("PrEcise Reference Counting with rEUse and Specialization", Reinking, Xie, de Moura, Leijen,
[MSR-TR-2020-42 v4](https://www.microsoft.com/en-us/research/uploads/prod/2020/11/perceus-tr-v4.pdf); PLDI 2021):

- **Precise counting.** The compiler inserts `dup` (increment) and `drop` (decrement) so that a value is
  dropped right after its *last use*, not at the end of its scope. The paper proves this "garbage free":
  only live data is retained (for programs without cycles).
- **Borrowed parameters.** A parameter marked `^` is borrowed: the caller keeps ownership, and no count
  operation happens at the call. From Koka's standard library
  ([`lib/std/core/list.kk`](https://raw.githubusercontent.com/koka-lang/koka/master/lib/std/core/list.kk)):

  ```koka
  pub fip fun map(xs : list<a>, ^f : a -> e b) : e list<b>
  pub fip fun is-empty( ^xs : list<a> ) : bool
  ```

- **Reuse analysis.** When a `match` takes apart a value whose count is 1, the memory of the old
  constructor is reused for the new one: a functional update becomes an in-place write. From the Koka tour
  ([`doc/spec/tour.kk.md`](https://raw.githubusercontent.com/koka-lang/koka/master/doc/spec/tour.kk.md)),
  red-black tree insertion, where the tour says "Every `Node` is actually reused in the fast path without
  doing any allocations":

  ```koka
  fun ins(t: tree, k: int, v: bool): tree {
    match t
      Leaf -> Node(Red, Leaf, k, v, Leaf)
      Node(Red, l, kx, vx, r)
        -> if (k < kx) then Node(Red, ins(l, k, v), kx, vx, r)
           ...
  ```

  If the tree is shared, the same code copies the path instead: persistence is kept, and the program
  cannot tell which happened.
- **FBIP and `fip`.** "Functional but in-place": write the algorithm purely, get in-place updates. Since
  v2.4.2 (2023) the keywords `fip` and `fbip` let the compiler *check* that a function runs without
  allocation (`fip`) or without allocation beyond deallocation (`fbip`), provided its arguments are not
  shared (Lorenzen, Leijen, Swierstra, "FP²: Fully in-Place Functional Programming", ICFP 2023,
  [TR](https://www.microsoft.com/en-us/research/publication/fp2-fully-in-place-functional-programming/)).
- **Struct update** is a copy with some fields changed, in place when unique:

  ```koka
  struct person
    age : int
    name : string

  fun birthday( p : person ) : person
    p( age = p.age + 1 )
  ```

- **Mutable locals** `var x := 0` cannot escape: the tour says "you get a type error if a local variable
  escapes through a function expression". `ref` cells (`val r = ref(0)`, read `!r`, write `r := 1`) are
  first-class and counted.
- **Threads.** Every object has a "thread-shared" state. Objects start unshared; passing an object to a new
  thread marks it and everything it reaches (once). The count of a shared object is stored *negative*, so
  one test, `if (rc <= 1) slow path else rc--`, catches both "free it" and "use atomics" (paper §2.7.2).
- **Cycles** can only be built through `ref` cells (immutable inductive data cannot be cyclic, paper
  §2.7.4). There is no cycle collector; the programmer clears a `ref` to break a cycle.
- **Header** ([`kklib.h`](https://raw.githubusercontent.com/koka-lang/koka/master/kklib/include/kklib.h)):
  8 bytes: `uint8_t scan_fsize`, `uint8_t _field_idx`, `uint16_t tag`, `int32_t refcount`. The count is 0
  for a unique reference ("for a faster free test in drop"), negative for thread-shared objects, and
  "sticky" on overflow (the object is never freed).

### 2.2 A1 to A20 in Koka

| id | verdict | how |
|---|---|---|
| A1 | not applicable | fields are never mutated; `p(age = …)` builds a new value, in place when unique |
| A2 | removed (b) | no references into containers exist; an update yields a new container (in place if unique); old holders keep the old version |
| A3 | removed (b) | the same: two "elements at once" are two values |
| A4 | removed (b) | the loop iterates a value; building the changed collection does not affect it. If the iterated list is still referenced, the update copies |
| A5 | removed (a) | functions return values; there are no references to return |
| A6 | removed (a) | data holds values |
| A7 | removed by prohibition | immutable inductive data cannot be cyclic; a `ref` cycle leaks |
| A8 | kept, as a design burden | trees are natural; parent pointers and doubly linked lists need `ref` cells, and their cycles leak unless cleared by hand |
| A9 | removed (b) | a `ref` cell is shared mutable state with no borrow panic: `!r` duplicates the value read, `r := x` swaps and drops the old one |
| A10 | removed (a) | one kind: values, counted by the compiler; plus `var` and `ref` |
| A11 | removed (b) | no moves exist; a second use costs a `dup` |
| A12 | partly kept | a closure cannot capture a `var` that would escape (a type error, like Rust's `'static`); capturing a `ref` cell works everywhere |
| A13 | not applicable | no subtyping or trait objects; dispatch uses data types, effect handlers and implicit parameters (`^?order2` above) |
| A14 | removed (b) | async is an effect handler: no `Pin`, no `Send` |
| A15 | removed (b) for immutable data | sharing marks the objects; immutable data cannot race; `ref` cells shared between threads need the atomic slow path (§2.7.3) |
| A16 | not applicable | no type classes; implicit parameters instead |
| A17 | removed | there are no ownership errors to explain |
| A18 | not applicable | values are constructed whole |
| A19 | moved to effects | no global mutable variable; state is an effect handled by the caller |
| A20 | not applicable | C FFI through `extern` declarations |

### 2.3 Costs, with the published measurements

The Perceus paper, §4 and Fig. 9: Koka 2.0.3 against OCaml 4.08.1, GHC 8.6.5, Swift 5.3, Java SE 15.0.1
and C++ (gcc 9.3.0), on a 6-core AMD 3600XT at 3.8 GHz, 64 GiB, Ubuntu 20.04; times and peak working
set are the median of 10 runs, normalised to Koka; each benchmark runs 1 to 5 seconds and uses up to
300 MiB. The benchmarks "stress memory allocation with little computation". What the paper states in its
text (the bar heights in the figure are not reliably readable from the text layer, so I quote only what is
written out):

- **rbtree** (42 million insertions into a red-black tree): Koka's purely functional version is "within
  10% of the C++ performance", where C++ is the in-place `std::map`. Java is close to C++ but "uses almost
  10x the memory of Koka (1.7GiB vs. 170MiB)". Koka with the Perceus optimisations disabled ("no-opt") is
  "more than 2x slower".
- **deriv, nqueens** (lots of sharing): the optimisations help little ("no-opt" is "only just slightly
  slower" on deriv), because shared values take the slow, copying path.
- **cfold**: the "no-opt" version uses 11% *less* memory, because reuse "holds on to memory for later
  reuse".
- **The C++ baseline frees nothing** in deriv, nqueens and cfold ("we do not reclaim memory at all"), so
  those three are not a like-for-like comparison with C.
- **Atomic counts**: running all benchmarks with atomic count operations instead of the thread-shared bit
  was "a slowdown from 5% (rbtree) up to 59% (nqueens)".
- The paper itself warns that cross-language results "depend not only on memory reclamation but also on
  the different optimizations performed by each compiler".

Per operation, from the paper's C code (§2.7.2): `dup` is an increment; `drop` is
`if (rc <= 1) drop_check(b); else rc--;`, one load, one compare, one store on the fast path. An update
of a unique value adds an "is unique" test. Measured here, the same shapes in C:

| mode | what | work ms | vs C |
|---|---|---|---|
| `inline_set` | C: 16,777,216 random `a[i].a += 1` into 4M structs | 94.5 | 1.00x |
| `unique_set` | the same, testing the list's count before each write (Roc's `List.set`, Koka reuse) | 103.7 | 1.10x |
| `cow_first_set` | ONE write into a 64 MiB list whose count is 2 (the hidden copy) | 17.8 | peak 134,572 KB vs 69,028 KB |
| `boxed_iter_rc` vs `boxed_iter` | one non-atomic `dup`+`drop` pair per element visit | +70.6 ms over 83.9M visits | about 0.84 ns per pair |
| `boxed_iter_atomic` vs `boxed_iter` | one atomic increment + acquire-release decrement per visit | +487.4 ms over 83.9M visits | about 5.8 ns per pair |

(The per-pair figures are the difference in medians divided by 20 passes x 4,194,304 elements. The
non-atomic pair is measured with a compiler barrier between increment and decrement so clang cannot fold
them, which models a borrowed value used across opaque code.)

Memory per object: 8 bytes of header. Koka's own claim is that precise dropping lowers peak memory against
scope-based counting; the paper's cfold result shows reuse can raise it by 11%.

What it cannot express or makes slow: in-place mutation of shared data (by design); cyclic structures
without leaks; a program whose data is shared at the update point silently copies (the `cow_first_set`
row); the speed of an update depends on a run-time count the source does not show, which the `fip`
keyword exists to make checkable.

### 2.4 Maturity

Research language. The Koka book's front page lists v3.2.8 (2026-09-17) and says "Koka v3 is a research
language that is currently under development and not ready for production use"
([book](https://koka-lang.github.io/koka/doc/book.html)). The same counting approach (borrowed parameters,
reset/reuse) came from Lean 4 ("Counting Immutable Beans", Ullrich and de Moura, cited by the Perceus paper
as [46]), and Lean 4 is in wide use as a theorem prover; I did not verify Lean's production status here.

---

## 3. Roc

### 3.1 The model in plain words

Roc (Richard Feldman) is a pure functional language with Python-like readability goals. Values are
immutable. Numbers, records, tuples and non-recursive tag unions live on the stack; strings, lists, boxes
and recursive tag unions are heap values with a count. Builtin operations "update them in place when their
reference counts are 1", using the Perceus scheme (Roc language reference, read through a third-party
mirror: [langref/expressions.md](https://git.joshthomas.dev/language-servers/roc/src/branch/issue-8696/langref/expressions.md);
the canonical copy was not found, so the details are UNVERIFIED against roc-lang.org). Roc calls this
**opportunistic in-place mutation**:

```roc
# semantically a new list; at run time an in-place write when `list` is unique
updated = List.set(list, 3, 42)
```

Because the language has no mutation primitive on shared data, "the language's lack of direct mutation
primitives lets it rule out reference cycles at language design time", and immutability "rules out data
races" ([roc-lang.org/functional](https://roc-lang.org/functional), as summarised by the search tool;
the second quote is from the page as fetched). The same mirror says counts are updated **atomically**, for
thread safety (UNVERIFIED on the canonical site). A Roc *platform* (the host program, written in another
language) decides how memory is allocated: "A platform for noninteractive command-line scripts can skip
deallocations altogether" ([roc-lang.org/fast](https://www.roc-lang.org/fast)).

### 3.2 A1 to A20 in Roc

| id | verdict | how |
|---|---|---|
| A1 to A6 | removed (b) or not applicable | as in Koka: no references, no mutation of shared data; records are updated with `{ rec & field: v }` (record-update syntax from Roc's tutorial, UNVERIFIED here) |
| A7, A8 | removed by prohibition | cycles cannot be built; graphs are written with ids into a `Dict` (the programmer's job) |
| A9 | removed by prohibition | there is no shared mutable state in application code; state lives in the platform |
| A10 | removed (a) | one kind: values |
| A11 | removed (b) | no moves |
| A12 | removed by prohibition | closures capture immutable values |
| A13 | not applicable | no trait objects; abilities are static; heterogeneous data uses tag unions |
| A14 | moved to the platform | effects are run by the platform; no `Pin` |
| A15 | moved to the platform | application code has no threads |
| A16 | UNVERIFIED | abilities exist; I did not verify their coherence rules |
| A17 | removed | no ownership errors |
| A18 | not applicable | records are constructed whole |
| A19 | removed by prohibition | top-level values are constants, evaluated at compile time ("Roc evaluates top-level values at compile time", /functional) |
| A20 | moved to the platform | unsafe code lives in the host language |

### 3.3 Costs

Roc publishes goals, not measurements: "Automatic memory management has some unavoidable runtime overhead",
"it's a non-goal to outperform languages that support memory unsafety or manual memory management", and the
goal is to "normally outperform the fastest mainstream garbage-collected languages"
([/fast](https://www.roc-lang.org/fast)). I found no benchmark table on the site. Per operation, Roc's
in-place update is the is-unique test measured above (`unique_set`, 1.10x on random writes) and, if
counts are atomic as the mirror says, the atomic pair (`boxed_iter_atomic`, about 5.8 ns per pair here;
Perceus measured 5% to 59% for whole programs). What it cannot express: any mutable shared structure,
cycles, direct pointers.

### 3.4 Maturity

Experimental. The compiler is being rewritten from Rust to Zig; Feldman's progress post reports
"feature parity with the original compiler" and that 0.1.0 is planned (as quoted by the search tool from
[Why Roc Is Moving Away From Rust to Zig](https://weeklyrust.substack.com/p/why-roc-is-moving-away-from-rust)
and related posts; I could not confirm a 0.1.0 release date). No numbered release exists as far as I
found.

---

## 4. Nim ARC and ORC

### 4.1 The model in plain words

Nim (Andreas Rumpf) has Python-like syntax and compiles to C. Since Nim 2.0 the default memory manager is
**ORC**; the documentation page I read is for Nim 2.2.12 and says "The recommended switch for newly written
Nim code is `--mm:orc`" ([mm.html](https://nim-lang.org/docs/mm.html)).

- **Value types** (`object`, `seq`, `string`) have destructors and **move semantics**
  ([destructors.html](https://nim-lang.org/docs/destructors.html)). An assignment from a location's last
  read becomes a move (`=sink` then `=wasMoved`); any other assignment is a `=copy`. A value passed to a
  `sink` parameter that is used afterwards is **copied, not refused**: "If it cannot be proven to be the
  last usage of the location, a copy is done instead". Only `ensureMove` makes it a compile error.
- **`ref` types** are counted, with plain (non-atomic) counts: "ARC in Nim doesn't use atomic RC"
  ([ARC/ORC introduction](https://nim-lang.org/blog/2020/10/15/introduction-to-arc-orc-in-nim.html)).
  Counting is scope-based and deterministic; **cursor inference** turns a copy into a borrowed alias when
  the source is not mutated.
- **Borrowed pointers**: `var T` parameters and returns, and `lent T` returns, are hidden pointers; for
  `lent`, "It is proven by the compiler that the pointer does not outlive its origin". `{.cursor.}` marks a
  field "not involved in the reference counting, it is a raw pointer without runtime checks".
- **ORC** adds a cycle collector "based on the pretty well known 'trial deletion' algorithm"
  ([introducing ORC](https://nim-lang.org/blog/2020/12/08/introducing-orc.html)). The compiler generates
  collector calls only for types that can be cyclic; `{.acyclic.}` declares a type acyclic, which the post
  says "can be crucial" to get ORC close to ARC. Cycle roots are registered and unregistered in O(1).
- **Header** (source [`lib/system/arc.nim`](https://raw.githubusercontent.com/nim-lang/Nim/devel/lib/system/arc.nim)):
  `rc: int` (8 bytes) under ARC; ORC adds `rootIdx: int` (8 more), "thanks to this we can delete potential
  cycle roots in O(1)". The low 3 (ARC) or 4 (ORC) bits of `rc` are flags. Atomic operations only under
  `--mm:atomicArc` with threads.

```nim
type
  Node {.acyclic.} = ref object
    kids: array[2, Node]
    data: string
```

### 4.2 A1 to A20 in Nim

| id | verdict | how |
|---|---|---|
| A1 | removed, unchecked | `var` parameters alias freely; there is no exclusivity rule |
| A2 | removed for `ref` elements (b); **unchecked** for value elements | a `seq[ref T]` element is a counted pointer and survives growth. A `var T` pointer into a `seq[T]` (from `mitems` or a `var` return) dangles if the seq grows while it is held (inferred from the iterator source below: UNVERIFIED by a run) |
| A3 | removed, unchecked | two `var` pointers into one seq are allowed |
| A4 | moved to run time (debug only) | `items` and `mitems` end each step with `assert(len(a) == L, "the length of the seq changed while iterating over it")` ([`lib/system/iterators.nim`](https://raw.githubusercontent.com/nim-lang/Nim/devel/lib/system/iterators.nim)). The assert runs *after* the loop body, so a write through the yielded `var` pointer after an `add` in the same body happens before the check; and `assert` is turned off when assertions are off (UNVERIFIED which build flags do that) |
| A5 | removed (a) for simple cases | `var T`/`lent T` returns need no annotation; the compiler checks they derive from the first parameter (the rule as documented for `lent`) |
| A6 | kept | objects cannot hold `var`/`lent`; they hold `ref` or `ptr` (view types are experimental) |
| A7, A8 | removed (b) | ORC collects cycles; `{.cursor.}` back pointers are raw and unchecked |
| A9 | removed (b) | `ref` objects are shared and mutated freely; no `RefCell` |
| A10 | removed (a) | `object` (value), `ref` (counted), `ptr` (unsafe) |
| A11 | removed (b) | use after move compiles: the compiler copies instead. The copy is silent and can be O(n) for a `seq` |
| A12 | removed (b) | closures capture into a counted heap environment; their cycles are collected by ORC |
| A13 | removed (a) | `method` dispatch, `of` type test, checked object conversion |
| A14 | removed (b) | async is a library macro; cycles from callbacks are collected (the reason the ORC post recommends ORC for async) |
| A15 | kept, partly unsafe | one shared heap with non-atomic counts: sharing a `ref` between threads races on the count; `Isolated[T]` and channels move data; the `gcsafe` effect forbids touching GC'd globals from a thread procedure |
| A16 | removed (a) | generics are checked at instantiation; concepts are optional |
| A17 | not applicable | few ownership errors |
| A18 | removed (a) | objects are zero-initialised by default |
| A19 | partly kept | globals are allowed; `gcsafe` restricts them in threads |
| A20 | kept, **unsafe** | `ptr`, `addr`, `cast` are ordinary expressions with no `unsafe` boundary |

### 4.3 Costs, with the published measurements

The ORC post's benchmark: an asynchronous "Hello World" HTTP server from Nim's standard library with about
135 MB of JSON in a global table, load from `wrk`, machine not named, written "to stress the differences"
and "not supposed to model realistic workloads (yet!)". ORC against Nim's mark-and-sweep:

| run | metric | ORC | mark & sweep |
|---|---|---|---|
| full collection every 10 requests | avg latency | 320.49 µs | 65.31 ms |
| | requests/s | 30,963.96 | 282.69 |
| | max memory | 137 MiB | 153 MiB |
| no forced collection | avg latency | 274.84 µs | 1.49 ms |
| | max latency | 1.10 ms | 46.41 ms |
| | requests/s | 34,948.95 | 39,561.97 |
| | max memory | 137 MiB | 333 MiB |

No comparison with C was published there. Per operation: a non-atomic increment and decrement (measured
pair here: about 0.84 ns); ORC adds a root-buffer registration when a potentially cyclic object's count is
decremented (the post says O(1); not measured here). Per object: 8 bytes (ARC) or 16 bytes (ORC) of
header. The collector's work depends on an adaptive threshold, which the ARC/ORC introduction calls only
partly deterministic.

What it cannot express safely: a long-lived pointer into a growable buffer, threads sharing `ref` data
without isolation, any FFI without unchecked pointers.

### 4.4 Maturity

Shipped and in production use: ORC is the default of Nim 2.x (documentation for 2.2.12 read). Nim has
been used in production for years (I did not collect user names: UNVERIFIED).

---

## 5. What Ember could take, and what it costs against C

Ember today, for reference (spec `docs/spec-source/Ember_v0.9.10_Hardened_35.md`):
class objects are counted handles with a **24-byte header** (`[OBJ-1]`: strong u32, weak u32, access u32,
flags u32, type pointer) plus per-field access words (`[EXC-19]`); counts are atomic iff the class is
`Sync` (`[RC-4]`); guaranteed elisions `[RC-2a]`..`[RC-2e]` already cover borrowed parameters (Lobster's
and Koka's main win); an object dies when its last owner ends "as the source says ... and never earlier"
(`[RC-3]`), which rules out Perceus's drop-at-last-use for objects with observable `drop`s.

The four Ember probes (all in `target/redesign/phase2-rc-elision/`, `ember.exe check` only):

| probe | shape | result |
|---|---|---|
| `class_elem_push.em` | `p = players[0]` (a class handle), `players.push(…)`, `p.hp -= 1` | compiles: Ember already has Lobster's answer to A2 for class elements |
| `struct_elem_push.em` | the same with a struct element and `ref mut players[0]` | `E3022` "`players[0]` is already mutably borrowed" |
| `class_iter_push.em` | `for e in enemies: e.hp -= 1; enemies.push(Enemy(5))` with class elements | `E3020` "cannot mutate `enemies` while it is borrowed by this loop", although every element is a counted handle and the push cannot invalidate `e` |
| `use_after_move.em` | `b = a` then `a.push(4)` on an `Array` | `E3040` "`a` is borrowed after it has been moved out of" (idea 1) |

### Idea 1: a use after move becomes a share, not an error. Kind (a)

The four languages agree on this, and it removes A11 and most of A10 for Python programmers. In Rust
and in Ember today, `b = a` moves an `Array`, and a later `a.push(1)` is an error. In Roc and Koka it is
two values; in Lobster the second owner gets an increment; in Nim the compiler copies.

```ember
# today: E3040 "`a` is borrowed after it has been moved out of" (probe use_after_move.em)
a = [1, 2, 3]
b = a
a.push(4)          # proposed: compiles; a and b are separate values (Python would alias; Ember keeps value semantics)
```

How, without giving up C speed: keep Ember's move analysis exactly as it is. Where the compiler proves the
source dead (every case that compiles today) nothing changes: no count, no test, the C code is identical.
Only where today's program would be *refused*, the buffer gets a count of 2 and the first write through
either name copies (Roc's and Koka's run-time is-unique test). Lobster's "retroactively made an owner" is
the same rule.

Cost against C, measured here: zero in every program that compiles today. In a program that relies on
the new rule: an 8-byte count on the buffer (Ember's `Array` has none today), the is-unique test on each
write (+10% on 16.8M random writes, `unique_set` 103.7 ms against `inline_set` 94.5 ms; the test cannot
be hoisted out of the loop because the writes go to memory of the same type), and a full copy when both
names are written (17.8 ms and 2x peak memory for 64 MiB, `cow_first_set`). Nim's eager copy at the
assignment pays that copy even when neither name is written again; the lazy version does not.

What the owner must weigh: the copy is invisible in the source. `ember inspect` (`[RC-6]`) already lists
surviving count operations; it would list these too.

### Idea 2: a per-object thread-shared bit instead of `@sync` classes. Kind (b)

Ember today makes the programmer choose, per class, atomic or plain counts (`[RC-4]`), part of A15. Koka
removes the choice: every object starts thread-local with plain counts; sending an object to another thread
marks it and what it reaches as shared, once; a shared object's count is negative, so the single fast-path
test `rc <= 1` also routes shared objects to the atomic slow path.

Cost against C: on the fast path, nothing beyond the count test Ember already does. Measured here, the
atomic pair costs about 5.8 ns per element visit against 0.84 ns for the plain pair (`boxed_iter_atomic`
693.7 ms against `boxed_iter_rc` 276.9 ms); Perceus measured 5% to 59% whole-program slowdowns from making
every count atomic. The marking walk is linear in the graph sent, once per object.

What it does **not** remove: Koka is safe because shared data is immutable. Ember's objects are mutable,
so a shared object still needs Ember's exclusivity checks or a lock for its fields. This idea removes the
count-atomicity decision only, not the data-race rules.

### Idea 3: an 8-byte header. Kind (b), for goal 1

Koka: 8 bytes (tag, scan size, 32-bit count). Lobster: 8 bytes (type index, count; widths assumed). Nim
ARC: 8 bytes. Ember: 24 bytes plus access words. For a 16-byte object that is 40 bytes against Koka's 24
before allocator overhead. The weak count and flags could move to a side record allocated the first time a
`Weak` is made (UNVERIFIED that this fits all of Ember's `[WK-*]` rules), the type pointer could become a
32-bit type index (Lobster does this), and Koka's sticky overflow could replace `[RC-1]`'s overflow panic.
Removes no annotation; it is pure memory, which the owner measures.

### Idea 4: cycles: a report at exit in debug, and a collector only where a type can form a cycle. Kind (b)

A8 in Ember today means choosing `Weak` for every back pointer (spec Part VIII's `Node.parent`). Lobster's
cheapest answer: let the cycle leak and print it at exit in the debug profile, so the programmer learns where
a `Weak` is needed, instead of guessing in advance. Release builds pay nothing. Nim's fuller answer: a
trial-deletion collector that the compiler calls only for types that can be cyclic (decided from the type
graph, so `Array[int]` and acyclic classes never pay), O(1) root registration, 8 more header bytes on
those types. Cost against C for the collector: not measured here (no Nim); the ORC post shows sub-millisecond
average latencies on its server benchmark but no C baseline. It conflicts with `[RC-3]` determinism only for
objects inside a cycle, which today leak.

### Idea 5: update through the store, never return a reference (the ECS case). Kind (a)

ODR-286 is about returning `ref mut pool.dense[i]` from a pool found through a map. Koka and Roc have no
such reference at all: the pool is a value, and `set`/`update` happen at the store, in place because the
pool is unique (FBIP). Spelled for Ember:

```ember
reg.update[Position](e, fn(mut p): p.x += 1.0)       # no reference leaves the registry
```

or with Python's `with`, as the Swift/Mojo report's idea 2 proposes. With `Box` pools (ODR-286's verdict),
the pool is unique by construction, so no run-time test is needed; when the lambda is inlined, the C code is
`dense[i].x += 1.0`. Cost against C: zero when inlined; a call per update otherwise (not measured here).

### Two things not to take

- **Counted elements as the default for arrays** (Lobster's answer to A2/A3 for all class data). Ember
  already offers it for `class` elements (`class_elem_push.em` compiles). Making it the default for struct
  data costs 3.56x iteration time, 9.74x when scattered, and 2.44x memory (measured above). The ECS pools
  must stay inline.
- **Nim's unchecked pointers** (`var` into a growing seq, `{.cursor.}`, `ptr`/`addr` without `unsafe`):
  fast and pleasant, but it is the hazard Rust guards against, left open.
- **Roc's prohibitions** (no shared mutation, no cycles, atomic counts): safe and simple but not
  Python-like for the owner's game code, and atomic counts cost 5% to 59% (Perceus).
- **Lobster's non-escaping closures**: Python callbacks escape; the phase-1 A12 catalogue already shows
  this is where Python programmers suffer.

### Found in passing: `class_iter_push.em`

Ember refuses pushing to an array of class handles while iterating it (`E3020`), although the counted
elements cannot dangle. With class elements, a `for` loop could iterate by index up to the length at the
start (Nim's shape, without the assert) and the push would be safe. For inline struct elements this does not
work: `iter_mut` yields pointers into the buffer and a push can move it. That is where the paged storage of
the Hylo/C# report applies. Snapshot iteration (retain the buffer for the loop, copy on push) is **not** an
answer for `iter_mut`: writes through the loop variable would go to the snapshot and be lost.

---

## 6. Sources

- Lobster memory management: https://aardappel.github.io/lobster/memory_management.html
- Lobster language reference: https://aardappel.github.io/lobster/language_reference.html
- Lobster `vmdata.h`: https://raw.githubusercontent.com/aardappel/lobster/master/dev/src/lobster/vmdata.h
- Lobster releases: https://github.com/aardappel/lobster/releases
- Perceus, MSR-TR-2020-42 v4: https://www.microsoft.com/en-us/research/uploads/prod/2020/11/perceus-tr-v4.pdf
- FP², ICFP 2023: https://www.microsoft.com/en-us/research/publication/fp2-fully-in-place-functional-programming/
- Koka book (version, maturity statement): https://koka-lang.github.io/koka/doc/book.html
- Koka tour source: https://raw.githubusercontent.com/koka-lang/koka/master/doc/spec/tour.kk.md
- Koka `kklib.h`: https://raw.githubusercontent.com/koka-lang/koka/master/kklib/include/kklib.h
- Koka `list.kk`: https://raw.githubusercontent.com/koka-lang/koka/master/lib/std/core/list.kk
- Roc: https://roc-lang.org/functional, https://www.roc-lang.org/fast
- Roc language reference (third-party mirror): https://git.joshthomas.dev/language-servers/roc/src/branch/issue-8696/langref/expressions.md
- Roc rewrite: https://weeklyrust.substack.com/p/why-roc-is-moving-away-from-rust
- Nim memory management: https://nim-lang.org/docs/mm.html
- Nim destructors: https://nim-lang.org/docs/destructors.html
- Nim ARC/ORC introduction: https://nim-lang.org/blog/2020/10/15/introduction-to-arc-orc-in-nim.html
- Nim ORC: https://nim-lang.org/blog/2020/12/08/introducing-orc.html
- Nim `arc.nim`: https://raw.githubusercontent.com/nim-lang/Nim/devel/lib/system/arc.nim
- Nim `iterators.nim`: https://raw.githubusercontent.com/nim-lang/Nim/devel/lib/system/iterators.nim
- Measured here: `target/redesign/phase2-rc-elision/rc_models.c`, `results.txt`; probes `*.em` in the same folder.
