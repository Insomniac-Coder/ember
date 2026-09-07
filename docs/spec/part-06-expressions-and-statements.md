# Part VI — Expressions and Statements

## VI.1 Evaluation order

* `[EXP-1]` Operands, arguments, array/tuple elements and struct constructor arguments are evaluated **left to right**, fully, before the operation/call. Named arguments are evaluated in source order, not parameter order.
* `[EXP-2]` Assignment `lhs = rhs` evaluates `rhs` first, then the place expression `lhs` (index/field sub-expressions), then stores. For `a[i] = f(i)`, `f(i)` runs before `i` is read for the index. Augmented assignment `a[i] += x` evaluates `x`, then the place once, then reads, ops, writes.
* `[EXP-3]` `and`/`or` short-circuit; `x if c else y` evaluates `c` first and exactly one branch.
* `[EXP-4]` Temporaries created during an expression statement are dropped at the end of that statement in reverse creation order; temporaries bound by `with`/`for`/`if`-conditions live to the end of the construct.

## VI.2 Places and values

A **place expression** denotes memory: a local, a field of a place, an index into a place, a dereference of a `ref`/`ref mut`/`Box`, a class-handle field access (`h.f` is a place inside the object), a `static`. Everything else is a value expression. `[EXP-5]` Mutation, mutable borrow and move require a place; `f(x).y = 1` is `E2140` unless `f` returns `ref mut`.

**Moves:** a place of non-`Copy` type used as a value (assignment RHS, `owned` argument, `return`, closure capture by `owned fn`) is **moved**; the source becomes uninitialised until reassigned. `[EXP-6]` Moving out of a field of a struct that has `drop` is `E3010`; moving out of an element of an array/span is `E3011` (use `mem.take`, `swap`, `pop`, `drain`); moving out of a class-object field is `E3012` (classes may be aliased); moving out of a `ref`/`ref mut` is `E3013`. Moving out of a plain struct's field is allowed and leaves the struct partially moved; it cannot be used as a whole until the field is reassigned.

## VI.3 Operators

| Expression | Desugar | Notes |
|---|---|---|
| `a + b` etc. | `Add.add(a, b)` | scalars are built-in |
| `a == b` | `Eq.eq(a, b)` | `!=` is `not (a == b)` |
| `a < b` | `Ord.cmp(a,b) == Less` or `PartialOrd` | floats use `PartialOrd`; `NaN` comparisons are `false` |
| `a is b` | handle identity (same object) | only for class handles and `ref`s; `E2150` otherwise |
| `x in coll` | `coll.contains(x)` | `Contains` interface |
| `a[i]` | `Index.index(a, i)` / `IndexMut.index_mut(a, i)` | selected by context (read vs write/mut-borrow) |
| `a[i..j]`, `a[..j]`, `a[i..]` | `a.slice(range)` → `Span`/`MutSpan` | bounds-checked; `..` and `..=` |
| `a?.f`, `a?.m()` | `match a: Some(v) => Some(v.f), None => None` | on `Option`; also `Result` (maps `Ok`) |
| `e?` | early-return on `None`/`Err(e)` with `From` conversion for errors | function return type MUST be `Option`/`Result` |
| `x as T` | numeric/pointer cast (`[TYP-6]`) | |
| `h as? D` / `h as! D` | dynamic downcast of class handle | `Option[D]` / panic |
| `**` | `pow` | integer `**` with negative exponent is `E2151` |
| `f"…{e:spec}…"` | `Formatter` calls on `Display`/`Debug` | allocates a `String`; `E4001` in `@noalloc` (use `format_to(buf, …)`) |

`[EXP-7]` Integer division truncates toward zero; `%` has the sign of the dividend (C/Rust semantics). `div_euclid`/`rem_euclid` exist.

## VI.4 Control flow

* `if`/`elif`/`else`, `while`, `for x in iterable`, labeled `break label`/`continue label`, `match`, `with`, `defer`, `return`.
* `[CTL-1]` `for pattern in expr` desugars to:
  ```
  __it = IntoIterator.into_iter(expr)   # for a place expression of non-Copy type: Iterable.iter(expr) (borrowing)
  while Some(pattern) = __it.next():
      body
  ```
  The **borrowing** rule: `for x in v` where `v` is a place borrows `v` (yields `ref T`); `for x in v.iter_mut()` yields `ref mut T`; `for x in owned v` consumes. `[CTL-2]` The iterable is borrowed for the whole loop; mutating it inside the loop is a borrow error `E3020` (the diagnostic suggests index loops or `retain`/`drain`).
* `[CTL-3]` Ranges `a..b` (`Range[T]`), `a..=b` (`RangeInclusive[T]`), `a..` (`RangeFrom`) implement `Iterator` for integer `T` and compile to a counted loop with no iterator object in memory (guaranteed by MIR lowering of `for` over range literals — `[CTL-3a]` conformance test checks the C output has no struct temporaries).
* `[CTL-4]` `while cond: … else: …` — `else` runs if the loop exits without `break`.
* `[CTL-5]` `match` semantics: arms tested top to bottom; first match wins; guards `if` evaluated after binding; exhaustiveness required (`E2090`); unreachable arm is `W2091`. Binding modes per `[GRM-13]`.
* `[CTL-6]` `with a = e1, b = e2:` binds `a`, `b` for the block and drops them (reverse order) at block exit. `with e:` without a binding evaluates `e` and keeps the temporary alive for the block (used for guards: `with lock.acquire():`).
* `[CTL-7]` `defer:` registers a block to run at scope exit (LIFO). A `defer` block cannot `return`, `break`, or `continue` out of the enclosing function (`E2160`). It may reference locals declared before it (borrowing them until scope end).
* `[CTL-8]` `return` inside `with`/`defer`-carrying scopes runs the deferred blocks and drops locals in the correct order (drops happen after `defer` blocks of the same scope).
* `[CTL-9]` `pass` is a no-op statement required for empty blocks.

## VI.5 Closures

```ember
scale = 2.0
double = fn(x: f32) => x * scale          # borrows `scale`
adder  = fn(mut acc: Array[f32], x: f32): acc.push(x)   # block form
task   = owned fn() => process(data)      # captures `data` by move/copy/retain; may escape
```

* `[CLO-1]` A closure's type is a unique anonymous struct type implementing `Callable`. It is a **view type** if it captures anything by reference (the default). It is a plain value type if declared `owned fn` (captures by move/copy/retain) or captures nothing.
* `[CLO-2]` Capture mode is inferred per variable: read-only use ⇒ shared borrow; mutation ⇒ mutable borrow (the closure then requires a mutable place to call: `mut f`); `owned fn` ⇒ move (or copy/retain). A closure that would move a captured non-`Copy` value out (e.g. returns it) is `E3030` in v1 ("once-callable closures are not supported; clone or restructure").
* `[CLO-3]` Calling: `f(args)`. A parameter declared `f: fn(A) -> R` is a generic over `Callable` (static dispatch, monomorphised). A boxed dynamic closure is `Box[dyn fn(A) -> R]` (`E4001` in `@noalloc` because boxing allocates). An `extern "C" fn` parameter accepts only capture-free closures and named functions.
* `[CLO-4]` A non-`owned` closure cannot escape the scope of what it borrows: storing it, returning it, or passing it to a function whose parameter is `owned`/stored triggers the normal view-type rules (`[TYP-15]`).
* `[CLO-5]` Closures capturing class handles retain them (a strong reference) — the usual cycle caution applies (Part VIII §5).

## VI.6 Assertions and panics

* `assert(cond)`, `assert(cond, "msg")`, `assert_eq(a, b)`, `assert_ne(a, b)`: always compiled in every profile. `debug_assert*` compiled only in `debug`.
* `panic("msg")`, `panic(f"…")`, `unreachable()`, `todo()`: type `!`.
* `[PAN-1]` Panic policy is a **package-level** setting: `abort` (default; prints message + backtrace in debug, message only in shipping, then `abort()`), or `unwind` (v2, LLVM backend only): runs `defer` blocks and drops in reverse order up to a `catch_unwind` boundary or the thread root. Foreign frames are never unwound through (`[FFI-*]`).
* `[PAN-2]` The panic message is formatted **without allocating** when the argument is a literal or `str`; with an f-string it allocates (so `@noalloc` functions may only `panic` with literals — `E4001` otherwise).
* `[PAN-3]` A panic inside `drop` while another panic is in flight aborts immediately.

---

