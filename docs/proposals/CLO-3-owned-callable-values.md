# Proposal: `[CLO-3]`'s owned callable values

2026-10-01, written while the owner was away; **nothing here is built**. It waits for the owner's
answer to the three questions at the end.

## What is missing

`[CLO-3]`: `fn(A) -> R` written anywhere but a parameter (a field, a local annotation, a return
type, a collection element, a `static`) is an **owned callable value**: a function pointer plus the
captured state it owns. `[CLO-10]`: up to three pointer-sized words of state are held inline with no
allocation; more goes in one heap allocation (`Alloc`); a call is one indirect call.

Today such a value can hold only a function or a lambda that captures nothing. A capturing lambda,
an `owned fn`, or a callable parameter cannot be stored (`E2020`, expected `fn(i64) -> i64`, found
the closure's environment). This blocks `[CLO-4]` (its view rule and `E3063` with the help
`owned fn`), `[CLO-6a]` (a `once fn` in a collection), `[CLO-7]`'s callback-storing APIs, and N3.
The iterator adapters did **not** need it (ADR-091: they hold the lambda by value through a
`[CLO-14]` bound, as `[STD-19]`'s "allocate nothing" requires).

## Today's representation

A `fn(A) -> R` value is one pointer to a static descriptor, `{ native, c }`: the function and its C
entry. A capture-free lambda is lowered to a function with such a descriptor. A capturing lambda's
value is its environment, a struct of the captures (borrowed, or owned for `owned fn`), and it is
called directly, never through a pointer.

## The proposal

**Value:** four words, `{ vtable, state[3] }`.

* `vtable` points to a static table per (closure, signature): `call(state*, args...) -> R` and
  `drop(state*)` (null when nothing needs dropping). A function or capture-free lambda has a table
  whose `call` ignores `state`, so every `fn(A) -> R` value has one shape and one call sequence:
  `v.vtable->call(&v.state, args)`, the one indirect call `[CLO-10]` allows.
* `state` holds the environment when it fits in three words (the common case: one to three
  captures); otherwise `state[0]` points to one heap block holding it, and the table's `call` and
  `drop` go through that pointer. Which one a closure uses is known where it is converted, so the
  choice costs nothing at a call.
* Converting a lambda to `fn(A) -> R` happens at a coercion site (`[TYP-5]` rule 10), as now: the
  environment is moved into `state` (or into its heap block), and the table is the closure's. A
  lambda that borrows is a view, so the value is a view type and `[TYP-15]` decides where it may be
  stored (`E3063`, help `owned fn`): this is `[CLO-4]`.
* Dropping one calls `drop` on its state. A `once fn` value (N3) is the same shape with `once` in
  its type: calling it moves it out of its place (`[CLO-6a]`).
* `extern "C" fn` stays a bare C function pointer (capture-free only), unchanged.

**Calls through a parameter are unchanged**: a callable parameter stays an implicit generic,
monomorphised, a direct call (`[CLO-3]`'s zero-cost form). Only a value stored in some other
position pays the indirect call.

## Questions for the owner

1. **Is `fn(A) -> R` still `Copy`?** Today it is (a pointer to a static table). An owned callable
   value that owns state cannot be copied bit for bit. Options: (a) it is move-only, and `Clone` when
   its state is (programs that copy a stored function value today would need `.clone()` or a
   parameter); (b) it is `Copy` only when its type says it holds nothing, which needs a second type.
   The spec's "owned value" reads as (a). **Recommended: (a).**
2. **Is four words the right size?** `[CLO-10]` fixes three words of inline state; with the table
   pointer a value is 32 bytes on a 64-bit target, where it is 8 today. Arrays of callbacks grow
   fourfold. The spec already chose this; confirming it is the intent.
3. **Heap state:** one allocation owned by the value, freed by `drop`. A value converted from a
   lambda with large captures therefore allocates at the conversion, reported by `ember inspect
   --alloc` once that exists (`[PHIL-2]`, not built). Fine?

## The owner's answers (2026-10-01)

1. **(a):** an owned callable value is move-only, and `Clone` when its state is.
2. **Three words of inline state**, as `[CLO-10]` says: a value is four words (32 bytes on a 64-bit target).
   For comparison, C++'s `std::function<long long(long long)>` is 64 bytes here (clang with Microsoft's
   library, measured 2026-10-01).
3. **Yes:** state larger than three words goes in one heap allocation owned by the value.

Nothing is built yet: the owner gives the go.

## Size of the work

Types (the value's layout and `Copy`), MIR (conversion at coercion sites, drop), C codegen (tables
per closure and signature, the call sequence), the checker (the view rule for a borrowing lambda,
`E3063`), N3's `once fn` type, and every test that copies a stored function value. Roughly the size
of ADR-085's relaxed units. The owner's rule "test before proposing" applies: a measurement of the
indirect call against a direct one, MSVC and clang, comes with the first commit.
