# Part X — Effects and Performance Contracts

## X.1 Effects

The compiler infers, for every function, an **effect set** ⊆ `{Alloc, Sync, Panic, Unsafe, FFI, Block}`:

| Effect | Introduced by |
|---|---|
| `Alloc` | any call to `ember_alloc`/`realloc`, class instantiation, `Box`, `Shared`, container growth, `String` formatting, f-strings, `Arena` growth, boxed closures |
| `Sync` | atomic RMW ops with ordering stronger than relaxed, `Mutex`/`RwLock` lock, channel send/receive, thread spawn/join, `Sync`-class retain/release (atomic) |
| `Panic` | `panic`, `assert`, bounds checks, overflow checks (debug), `unwrap`, exclusivity checks, division |
| `Unsafe` | body contains an `unsafe` block or the function is `unsafe fn` |
| `FFI` | calls to `extern` functions |
| `Block` | `Mutex.lock`, `join`, `sleep`, channel blocking receive, `File.read` — any call the runtime marks blocking |

* `[EFF-1]` Effects are computed per function from its body and the (already computed) effects of its callees, over the call graph, with recursion handled by fixpoint (recursive SCCs are assumed to have the union of their members' direct effects).
* `[EFF-2]` Calls through `dyn` or function values contribute the effects declared on the interface method or function type; a method in an interface may declare `@noalloc` and implementers MUST satisfy it (`E4010`). A `fn(A) -> R` parameter type is assumed to carry all effects unless written `@noalloc fn(A) -> R`.
* `[EFF-3]` `extern` functions carry effects declared in their contract (`@ffi(effects=[FFI])` by default; `@ffi(effects=[FFI, Alloc, Block])` if the binding says so).
* `[EFF-4]` Effects are part of a function's public interface for the purpose of caching: a change to a callee's effect set invalidates callers' contract checks (Part XIX build graph).

## X.2 Hard contracts

* `@noalloc fn`: `[EFF-5]` MUST NOT have `Alloc` in its effect set. Violations report the **full call chain** to the allocation site: `error[E4001]: @noalloc function `cull` reaches an allocation: cull → collect_visible → Array.push → ember_alloc`.
* `@nosync fn`: MUST NOT have `Sync`.
* `@noblock fn`: MUST NOT have `Block`.
* `@nopanic fn` **(v2)**: MUST NOT have `Panic` — requires proving away bounds checks; reserved.
* `[EFF-6]` A contract applies to the whole reachable call graph, including drop glue for locals, default arguments, and operator impls.
* `[EFF-7]` `unsafe: @assume_noalloc(expr)` overrides the analysis for one call (e.g. a C function known not to allocate); it is `unsafe` because the compiler cannot verify it.
* `[EFF-8]` Contracts are inherited by overriding methods (an `override` of a `@noalloc virtual fn` must be `@noalloc`).

## X.3 Inspection

`ember inspect path.to.fn` prints the effect set, the allocation sites reachable, inlining decisions, whether loops vectorised, the chosen ABI for each parameter, and `size_of`/`align_of`/field offsets for types. This is the primary tool for making the compiler's allocation, layout, dispatch and vectorisation decisions visible rather than implicit.

---

