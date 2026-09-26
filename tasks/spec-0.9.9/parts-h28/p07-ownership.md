---

# Part VII — Ownership, Borrowing and Regions

This Part specifies the **value world**: structs, enums, collections and views. Class objects are
Part VIII; here a class handle is simply a `Copy` value whose copy retains.

## VII.1 Ownership

* `[OWN-1]` Every value has exactly one owner: a local, a field of an owned value, an element of an
  owned collection, a temporary, or a `static`.
* `[OWN-2]` A value is **dropped** when its owner goes out of scope (block end, reverse declaration
  order), when its owner is overwritten, or at the end of the statement that created it as a
  temporary. Dropping runs the value's `drop` method, if any, then drops its fields in reverse
  declaration order.
* `[OWN-3]` A move transfers ownership and leaves the source uninitialised. Using a moved-from place is
  `E3040`, labelled at the move and at the use, with the help `.clone()` when the type is `Clone`. A
  value moved on only some paths is tracked with a hidden drop flag; conditional moves are legal.
* `[OWN-4]` A loop body that moves a value declared outside the loop is `E3041`, unless every path
  assigns it again before the next iteration.
* `[OWN-5]` Assigning to a place that holds a live value evaluates the new value, then drops the old
  one, then stores the new one. `Cell.set` stores first and drops after (`[CELL-1]`); `MaybeUninit.write`
  stores without dropping (`[ARN-8]`). These three orders are distinct and none generalises the others.
* `[OWN-6]` *(changed in 0.9.9)* `mem.take(mut place: T) -> T` (leaves `Default`),
  `mem.replace(mut place: T, owned new: T) -> T`, `mem.swap(mut a: T, mut b: T)`, `mem.drop(owned x: T)`
  (drops now: a variable given to it is moved, a class handle's too though a handle is `Copy`, so it
  ends that variable's ownership and a later use is `E3040`, ODR-065) and `mem.forget(owned x: T)` (never drops; `E3015` for `@must_drop` types, `[THR-6]`) are
  the operations that move values out of places a move may not otherwise leave empty. Calls are
  written without modes: `mem.take(self.items)` (`[FN-2a]`).

## VII.2 Copy and Clone

* `[OWN-7]` A `Copy` value may be duplicated on use and the source stays valid. The duplicate is the
  value's bits, and each counted handle in it — a class handle, `Shared`, `Weak`, and any of these
  inside a struct, enum, tuple or fixed array — is retained, as dropping the copy releases it
  (`[RC-1]`); no other copy runs code, and a type with a `drop` of its own is never `Copy` (ODR-051).
* `[OWN-8]` `Clone.clone(self) -> Self` is the explicit deep copy. Cloning a class handle copies the
  handle; copying the object is a method the class writes.

## VII.3 Borrows

A **borrow** creates a reference, or a view containing one, to a place without taking ownership.
Borrows are created implicitly when a place is passed to a borrowed or `mut` parameter or receiver,
iterated, indexed or coerced to a view, and explicitly with `ref place` / `ref mut place` (needed only
to initialise a `ref`-typed local or field).

* `[BRW-1]` **Aliasing xor mutation.** At every program point a place has any number of live shared
  borrows, or exactly one live mutable borrow. While a mutable borrow is live the owner cannot read,
  write, move or drop the place; while shared borrows are live the owner can read but not write, move
  or drop. A reference local is never re-seated: `r = e` writes through `r` (and is legal only if `r`
  is `ref mut`).
* `[BRW-2]` **Liveness.** A borrow is live from its creation to the last use of anything derived from
  it — the reference, a reborrow, a view built from it, a value returned from a call that borrowed it.
  Scope does not matter: `n = v.len(); v.push(n)` is legal.
* `[BRW-3]` *(changed in 0.9.9)* **Two-phase borrows.** For a method call whose receiver is a place, or
  an argument passed to a `mut` parameter that is a place, the mutable borrow is **reserved** when the
  argument list begins and **activated** at the call. Between reservation and activation the place may
  be read, and values computed from it may be passed (`v.push(v.len())`); but a borrow of the place that
  is still live at activation — because it is itself an argument, or is held by one — conflicts with the
  activation and is `E3021`. For `fn f(mut v: Array[int], x: ref int)`, the call `f(v, v[0])` is
  therefore rejected; for `fn f(mut v: Array[int], x: int)` it is accepted, because `v[0]` is copied
  before activation.
* `[BRW-11]` *(new in 0.9.9)* **All borrows a call makes are live together.** The borrows formed for every argument and
  the receiver of one call are live simultaneously for the duration of the call. Two of them that
  conflict under `[BRW-1]` — two `mut` arguments naming overlapping places, or a `mut` argument and a
  shared view of the same place — are `E3022` (two mutable) or `E3021` (mutable and shared).
* `[BRW-4]` *(changed in 0.9.9)* **Disjoint fields.** `ref mut a.x` and `ref mut a.y` may be live
  together when `x` and `y` are different fields, through any depth of field projection. Accesses to a
  class object's fields are governed by Part VIII instead.
* `[BRW-10]` *(new in 0.9.9)* **Methods borrow the fields they use.** A call to a method of a struct or enum that is not
  visible outside its package and not `virtual` borrows only the fields its body reads or writes (and
  their transitive callees of the same kind), as computed by the compiler; so `w.bump()` may be called
  while a loop iterates `w.names` if `bump` touches only `w.count`. A method visible outside its package
  borrows all of `self`, so that changing its body cannot break a caller in another package.
* `[BRW-5]` *(changed in 0.9.9)* **Indices are not disjoint.** `ref mut a[i]` and `ref mut a[j]`
  conflict unless both indices are constants and differ. `get_pair_mut(i, j)`, `split_at`,
  `chunks_mut`, `iter_mut`, `swap(i, j)` and `SoA` columns are the ways to hold two mutable views of
  one container (shape B1). `get_pair_mut(i, j) -> Option[(ref mut T, ref mut T)]`, on an `Array` or
  a `MutSpan`, borrows the container once, checks that both indices are in range and differ, and
  only then makes the two references; otherwise it is `None` and the container is unchanged. The
  pair borrows the receiver (`[LT-1]` rule 1). Distinct elements of a zero-sized type need not have
  distinct addresses, and nothing about addresses follows from the indices (ODR-068).
* `[BRW-6]` **Reborrows.** From `r: ref mut T`, `ref r.f` freezes `r` while it lives and `ref mut r.f`
  suspends `r`. Passing a `ref mut` local to a `mut` parameter reborrows it; it does not move it.
* `[BRW-7]` Borrowing a moved or uninitialised place is `E3050`.
* `[BRW-8]` *(changed in 0.9.9)* A borrowed parameter is passed by address: the callee reads the
  caller's place (`[FN-1]`, ODR-024). Two kinds are passed as a copy instead:
  * a reference or view (a `ref`, `Span`, `MutSpan`, `str` or `@view` struct), which is passed as
    itself and keeps the regions it carries;
  * a `Copy` value that holds no `Cell` or `UnsafeCell`, which goes in registers when it is no larger
    than two pointers, except the receiver of a method whose result is a reference or view
    (`[LT-1]` rule 1).
  A reference into the own storage of a parameter passed as a copy ends with the call (`E3060`). The
  choice depends only on the declared signature, never on the body; it is an ABI decision and cannot
  be observed. An instance of a generic function, method or type has the generic's declared
  signature: a parameter its result may point into (the receiver, or a source parameter of
  `[LT-1]`'s second kind) is passed as the declaration passes it, whatever the type arguments, so
  `Holder[str]`'s receiver is passed by address as `Holder[T]`'s is (ODR-048).
* `[BRW-9]` A reference in Safe code is never null, dangling or unaligned.

## VII.4 Regions and their inference

Ember has **no lifetime syntax**. Every reference and view carries a **region** — the set of program
points at which it must be valid — and the compiler infers every region.

* `[LT-5]` Regions of locals are inferred by non-lexical liveness (§XVIII.4). Diagnostics describe
  them in source terms: "the borrow of `x` on line 12 is still needed on line 19".
* `[LT-1]` *(changed in 0.9.9)* **Signature elision.** A **source parameter** (ODR-024) is:
  * a parameter whose type is a reference or view, in any mode; or
  * a borrowed or `mut` parameter whose type is not `Copy` even when each of its type parameters is
    taken to be `Copy`: an `Array[T]`, `String`, `Map`, `Box`, `RefCell` or arena, a struct or enum
    without `@derive(Copy)`, or a tuple or fixed array holding one of these.
  A source parameter of the second kind is the caller's place (`[FN-1]`, `[BRW-8]`): a result that
  borrows it may point anywhere in it, into its own storage or into storage it owns, and the call's
  loan is on the argument (`[BCK-1]`), shared for a borrowed parameter and exclusive for `mut`. A
  `Copy` parameter that is not a reference or view is not a source parameter (so neither is a plain
  `x: T`, nor a callable parameter, `[CLO-3]`), and nor is an `owned` parameter that is not a
  reference or view. Which parameters are sources depends only on the declared signature. In an
  instance of a generic, a parameter whose type there is a reference or view is a source of that
  instance too, of the first kind: `Tree[str].leaf(v: T)` may return a view of what `v` views, and
  its callers, which call that instance, know it (ODR-048). Where rule 1 below ties the result to a
  borrowed receiver, such a parameter is a source alongside it when its declared type names a type
  parameter the declared result names too: the result's type says it can hold one, so
  `Map[str, V].entry(k) -> MapEntry[K, V, H]` returns an entry holding `k`, while `get(q) ->
  Option[ref V]` borrows only the map (ODR-069).
  For a function whose result is a reference or view:
  1. if it has a borrowed receiver (`self` or `mut self`), of any type, the result borrows from the
     receiver;
  2. else, if exactly one parameter is a source parameter, the result borrows from it;
  3. else, the result borrows from **all** source parameters together (the caller keeps all of them
     borrowed while the result lives).
  Rule 3 always type-checks; it may borrow more than the function needs. A returned reference or view
  that borrows any other parameter is rejected: `E3060` when that parameter is `owned` or is passed
  as a copy (`[BRW-8]`), and `E3062` otherwise.
* `[LT-1a]` *(changed in 0.9.9)* `@borrows(p, …)`, on its own line before the function, replaces the rule the compiler would
  apply: the result borrows from exactly the named parameters (`self` names the receiver). Returning
  a value that borrows from anything else is `E3062`. Each named parameter is a source parameter (`[LT-1]`)
  or a `mut` parameter; naming a borrowed `Copy` parameter, or an `owned` parameter that is not a
  reference or view, is `E2031`. It is needed only where rule 1 or 3 borrows more than the caller can
  afford, or to let the result borrow a `mut` parameter of a `Copy` type:

```ember
@borrows(haystack)
fn find_word(haystack: str, needle: str) -> Option[str]:
    i = haystack.find(needle)?
    return Some(haystack[i..i + needle.len()])
```

* `[LT-1b]` *(changed in 0.9.9)* The opt-in lint `L3014` reports rule 3 applying to more than one source parameter and
  names `@borrows` as the way to narrow it.
* `[LT-3]` String literals, `bytes` literals, `static` items and views of them have the `static`
  region, which outlives everything.
* `[LT-4]` *(changed in 0.9.9)* Arena allocations borrow the arena (`[ARN-1]`).
* `[LT-44]` *(new in 0.9.9)* A borrowed or `mut` `Arena`, `FixedArena` or `ScopedArena` parameter is
  a source parameter because it is not `Copy` (`[LT-1]`): a function whose only source parameter is
  one arena returns views that borrow it, with no annotation. `@borrows(arena)` is needed only when
  the function has other source parameters too.
* `[LT-6]` *(changed in 0.9.9)* Named lifetimes are not part of Ember and will not be added. Where a
  relationship between regions cannot be inferred, the diagnostic names the restructuring that
  expresses it: an owned result, an index instead of a reference, a view struct, or `@borrows`.
* `[LT-7]` *(changed in 0.9.9)* **Callable types.** Each call through a value or parameter of callable
  type `fn(P1, …, Pn) -> R` gets fresh regions for its source parameters (`[LT-1]`). `R` may borrow
  from them only as `[LT-1]` would allow for a function declared with that signature, and then only for
  the duration the caller keeps the arguments. So a callee may lend a callback views of its own
  locals, and a callback may return a view of an argument to its caller:

```ember
fn with_local(f: fn(Span[int]) -> int) -> int:
    tmp = [41]
    return f(tmp)

fn first_of(a: Span[int], b: Span[int], pick: fn(Span[int], Span[int]) -> Span[int]) -> int:
    return pick(a, b)[0]

fn main():
    println(with_local(fn(s) => s[0] + 1))
    xs = [1, 2]
    ys = [3]
    println(first_of(xs, ys, fn(a, b) => a))
```

### Views with several regions

A struct, enum or tuple holding views (a **view type**, `[TYP-34]`) may hold views of independent
sources. The compiler keeps one region per borrowed field — a **region vector** — and never forces
them to be equal.

```ember
struct Bodies:
    positions: MutSpan[f32]
    velocities: Span[f32]

fn integrate(mut b: Bodies, dt: f32):
    for i in 0..b.positions.len():
        b.positions[i] += b.velocities[i] * dt

fn main():
    pos: Array[f32] = [0.0, 1.0]
    vel: Array[f32] = [1.0, 1.0]
    b = Bodies(positions=pos, velocities=vel)
    integrate(b, 0.5)
    println(pos)
```

* `[LT-14]` A view type carries a compiler-internal region vector with one slot per borrowed field
  (fields proven to share a source may share a slot). It is never written in source (`[LT-6]`).
* `[LT-16]` Constructing a view value keeps each field's region; it never replaces them with their
  intersection.
* `[LT-17]` **Validity is conjunctive.** A view value is usable only while every region a used field
  needs is valid. A field whose source has ended cannot be used even if other fields are still valid.
* `[LT-18]` Every borrowed field is an ordinary loan under `[BRW-1]`–`[BRW-11]`; multiple regions add no
  new aliasing rule and weaken none.
* `[LT-20]` Moving, copying, destructuring, passing and returning a view preserves each field's region.
  Projecting one field carries only that field's region.
* `[LT-21]` Assigning a new view into a field recomputes that field's region; the old source is
  released on that path and the new one constrained. Merges of control flow keep every region that may
  reach a field.
* `[LT-22]` A function returning a view type gets, from its body, a summary of which result field
  borrows from which parameter (and which parameter fields each call accesses, `[LT-35]`). Callers use
  the summary; source never states it. A function whose result provenance cannot be established soundly
  is `E3065` (shape B14): return an owned value or separate views instead.
* `[LT-23]` `@borrows` on a function returning a multi-region view MUST NOT collapse it into fewer
  regions than its fields need (`E3065`).
* `[LT-24]` A region slot ends at the last use of its own field, not of the whole value.
* `[LT-25]` A view field may not borrow another field of the value that contains it (shape B5).
* `[LT-26]` A view extends no lifetime: constructing, copying or storing it retains nothing. (A view of
  a class object's field keeps the object alive only through `[RC-5]`.)
* `[LT-27]` Different regions never prove that two views do not overlap in memory; `noalias` comes
  only from `[DSJ-*]` and `[SIMD-3]`.
* `[LT-30]` Regions exist only at compile time: two values of one view type with different regions have
  one layout, one ABI, one symbol and one type identity, so region inference causes no
  code duplication.
* `[LT-34]` A view type whose fields all borrow from one source behaves exactly as a single-region
  view.
* `[LT-35]` For each function that can receive a view value, the compiler knows which fields the body
  may read, write, move, return or publish. A call requires only the regions of those fields. Where
  the target is not statically known — dynamic dispatch, a foreign function, an opaque callable — every
  field is assumed accessed and retained.
* `[LT-36]` Treating the view as a whole — passing it where every field may be used, comparing,
  hashing or formatting it, copying or moving it — requires every region to be valid.
* `[LT-38]` Moving one field out moves only that field's constraint; the remaining fields keep theirs.
* `[LT-39]` An operation that selects a field by a run-time value (reflection, a field descriptor)
  requires every region it could select (`E3065` if a region it needs has ended).
* `[LT-42]` A non-`owned` closure capturing a view records the regions of the fields it uses; an
  `owned fn` capturing a view requires every region to be `static` (`[TYP-15]`).
* `[LT-43]` Region tracking never lets a view survive a `yield` that `[CORO-6]` forbids.

## VII.5 Destruction

* `[DRP-1]` `fn drop(mut self)` runs exactly once per value, at the end of its life. Calling it
  explicitly is `E3070`; `mem.drop(x)` ends a value early.
* `[DRP-2]` Locals drop at the end of their block in reverse declaration order; fields after their
  owner's `drop`, in reverse declaration order; collection elements in index order; the active variant
  of an enum; tuple elements in reverse order.
* `[DRP-3]` Temporaries drop at the end of the statement that created them (`[EXP-4]`).
* `[DRP-4]` *(changed in 0.9.9)* A panic inside `drop` aborts the process (`[PAN-1]`). A `drop` SHOULD
  NOT block; `@noalloc` on a type's `drop` is honoured wherever that type is dropped.
* `[DRP-5]` A `drop` body may not move fields out of `self` (`[EXP-6]`); it uses `mem.take` on
  `Option` or `Default` fields.
* `[DRP-6]` Dropping a `Box[T]` drops the `T` and frees; dropping a class handle or `Shared` releases;
  dropping a plain view (`ref`, `Span`, `str`) does nothing; dropping a guard (`Ref`, `RefMut`,
  `MutexGuard`, an `RwLock`'s) ends its access or unlocks, and dropping a view type drops whatever in
  it needs drop. Carrying a region and needing drop are independent properties (`[TYP-1]`) (ODR-053).
* `[DRP-7]` *(new in 0.9.9)* A value whose `drop` may read through a reference it holds must be dropped while that
  reference's region is valid. A generic type parameter is assumed to be read by `drop` unless the type
  has no `drop` of its own. Violations are `E3060` (shape B7).

## VII.6 Views over containers: `Span` and `MutSpan`

```ember
fn normalize(mut xs: MutSpan[f32]):
    total = xs.iter().sum()
    for x in xs.iter_mut():
        x /= total

fn main():
    buf: Array[f32] = [1, 2, 3]
    normalize(buf)
    left, right = buf.as_mut_span().split_at(1)
    left[0] = right[0]
    println(buf)
```

* `[SPN-1]` An `Array[T]`, a `[T; N]` or a `String` converts to `Span[T]`/`str` where one is expected,
  and to `MutSpan[T]` where one is expected (a `mut` parameter, or a `MutSpan`-typed local or field). **The conversion takes a borrow** of the source place with the
  ordinary rules — the source cannot be mutated, moved or dropped while the view lives — whether it is
  written implicitly, as `as_span()`/`as_mut_span()`, or produced by a call.
* `[SPN-2]` Indexing a view is bounds-checked; `get(i) -> Option[ref T]` does not panic;
  `unsafe: s.get_unchecked(i)` does not check.
* `[SPN-3]` `Span[T]` is `Copy`; `MutSpan[T]` is move-only and reborrowable (`s.reborrow()`, or
  implicitly at a `mut` site).
* `[SPN-4]` `iter()` on a `Span` or `MutSpan` yields `ref T`; `iter_mut()` on a `MutSpan` yields
  `ref mut T` and reborrows it until the iterator's last use. `chunks(n)` and `chunks_mut(n)` yield
  consecutive non-overlapping views of `n` elements (the last may be shorter); `n == 0` panics in every
  profile.
* `[SPN-5]` *(changed in 0.9.9)* `split_at(i)` on a `Span` returns two `Span`s; on a `MutSpan` it
  consumes (reborrows) the view and returns two disjoint `MutSpan`s. `i > len` panics. (There is no
  separate `split_at_mut`: an `Array` is split mutably as `a.as_mut_span().split_at(i)`.) A
  `MutSpan` also has `get_pair_mut(i, j)` (`[BRW-5]`).
* `[SPN-8]` `as_ptr()` and `as_mut_ptr()` return raw pointers; extracting one is safe, using one needs
  `unsafe`, and extraction extends no region.

## VII.7 What a newcomer sees

```ember
class Player:
    name: String
    health: float = 100.0

fn total_health(players: Array[Player]) -> float:      # borrows the array
    total = 0.0
    for p in players:
        total += p.health
    return total

fn heal_all(players: Array[Player]):                   # handles: the objects can change
    for p in players:
        p.health = 100.0

fn take(owned players: Array[Player]) -> int:          # consumes
    return players.len()

fn main():
    ps: Array[Player] = [Player("a"), Player("b")]
    println(total_health(ps))
    heal_all(ps)
    n = take(ps)
    println(n)
```

Using `ps` after `take(ps)` is `E3040`, labelled at the move. Nothing in this program names a mode at a
call, a lifetime, or a borrow.
