# Part VII — Ownership, Borrowing and Lifetimes

This part specifies the **value world**. Class instances (the object world) are covered in Part VIII; the rules here apply to class *handles* as `Copy` values and to *accesses through* handles as described there.

## VII.1 Ownership

* `[OWN-1]` Every value has exactly one owner: a local variable, a field of an owned value, an element of an owned container, a temporary, or a `static`.
* `[OWN-2]` When the owner goes out of scope (block end, reverse declaration order), or is overwritten, or is a temporary at statement end, the value is **dropped**: its `drop` method (if any) runs, then its fields are dropped in reverse declaration order (arrays: elements in index order).
* `[OWN-3]` Moves transfer ownership; the source is dead. Use of a moved-from place is `E3040 use of moved value` with two labels (the move site, the use site) and a `help` suggesting `.clone()` if `Clone`. Conditional moves are handled by **drop flags** (a hidden `bool` per local that is conditionally moved), never by making the program illegal.
* `[OWN-4]` A loop body that moves a value declared outside the loop is `E3041` unless the value is reassigned before the next iteration on every path.
* `[OWN-5]` Overwriting a place that holds a live value drops the old value first (after evaluating the new value: `x = f(x)` moves `x` into `f`, then stores).
* `[OWN-6]` `mem.take(mut place) -> T` (replaces with `Default`), `mem.replace(mut place, new) -> T`, `mem.swap(mut a, mut b)`, `mem.forget(owned x)` (skips drop; leaking is not unsafe **except for `@must_drop` types, which it rejects — `[THR-6]`**).

## VII.2 Copy

* `[OWN-7]` Values of `Copy` types are duplicated bitwise on use; the source remains valid. A `Copy` type has no `drop`. Class handles are `Copy` at the language level but their copy performs a retain and their drop a release (Part VIII); they are excluded from `[TYP-15]`-style bitwise guarantees (the compiler emits the RC ops).
* `[OWN-8]` `Clone.clone(self) -> Self` is the explicit deep copy for non-`Copy` types. `@derive(Clone)` generates field-wise clones. Cloning a class handle is the same as copying it (shallow); deep-copying an object requires an explicit method.

## VII.3 Borrows

A **borrow** creates a reference (`ref T` or `ref mut T`) — or a view containing one — to a place, without transferring ownership. Borrows are created:

* implicitly, when passing a place to a `borrowed` (default) or `mut` parameter, calling a method with `self`/`mut self`, iterating with `for`, or using an operator whose interface takes borrows;
* explicitly, with the keyword forms `ref place` / `ref mut place` (needed only when initialising a `ref`-typed local or a view struct field).

The rules (`[BRW-*]`) are Rust's, restated:

* `[BRW-1]` **Aliasing XOR mutability.** At any program point, a place may have either any number of live shared borrows, or exactly one live mutable borrow, and while a mutable borrow is live the owner may not read, write, move, or drop the place; while shared borrows are live the owner may read (and copy) but not write, move, or drop.
* `[BRW-2]` **Liveness (NLL).** A borrow is live from its creation until the last use of any value derived from it (the reference itself, a reborrow, a view built from it, a return value tied to it). Scope end is irrelevant. This is what makes `n = v.len(); v.push(n)` legal.
* `[BRW-3]` **Two-phase borrows.** For `v.push(v.len())`, the mutable auto-borrow of `v` for the receiver is *reserved* first and *activated* only when the call happens; shared borrows in the arguments are permitted in between. This applies to method receivers and `mut` arguments whose argument expression is a simple place.
* `[BRW-4]` **Disjoint fields.** `ref mut a.x` and `ref mut a.y` may be live simultaneously if `x` and `y` are distinct fields of a struct/tuple (not through a method call — a method takes all of `self`). This holds through arbitrary nesting of field projections. It does **not** hold through class-handle access (`h.x` and `h.y` are accesses on an aliased object: Part VIII §3 governs) — for *conflicting accesses*. Keeping the object allocated is governed by `[RC-5]`, which applies to class-handle projections exactly as to any other place.
* `[BRW-5]` **Indices are not disjoint.** `ref mut a[i]` and `ref mut a[j]` conflict unless both indices are constants and different. `split_at_mut`, `chunks_mut`, `iter_mut` and `SoA` column borrows are the sanctioned ways to obtain multiple mutable borrows into one container.
* `[BRW-6]` **Reborrows.** From `r: ref mut T` one may create `ref r.f` (shared reborrow, freezing `r` for its duration) or `ref mut r.f` (mutable reborrow, suspending `r`). Passing a `ref mut` local to a `mut` parameter reborrows rather than moving it.
* `[BRW-7]` **No borrow of a moved or uninitialised place.** `E3050`.
* `[BRW-8]` A shared borrow of a `Copy` place may be replaced by a copy when the callee's parameter is `Copy` and ≤ 16 bytes; this is an ABI decision and never observable.

## VII.4 What a reference can point to

`ref T` points to a **place** that outlives the reference's region. Places include locals, fields, array elements, `Box` contents, class-object fields (subject to Part VIII), arena allocations, `static`s, and memory behind a `Span`. A reference can never be null, dangling, or unaligned in safe code (`[BRW-9]`).

## VII.5 Regions (lifetimes) and their inference

Ember does not have user-written lifetime names in v1. Instead:

* Every reference-typed or view-typed value has an implicit **region** — the set of program points where it must be valid.
* `[LT-1]` **Signature elision.** In a function signature with view-typed parameters and a view-typed return:
  1. if there is a `self`/`mut self` receiver that is a borrow, the return's region is the receiver's;
  2. else if exactly one parameter is view-typed, the return's region is that parameter's;
  3. else, the return's region is the **intersection** of all view-typed parameters' regions (the returned reference may point into any of them; the caller treats it as borrowing all of them). This is more permissive than Rust's elision failure and remains sound.
  `@borrows(param)` on a function overrides rule 3 to tie the return to one named parameter (e.g. `fn longest(a: str, b: str) -> str @borrows(a)`), which lets the caller keep using `b`.
* `[LT-2]` **View structs** have one region parameter. Constructing a view struct from several references gives it the intersection of their regions. A view-typed field's region is the struct's region. Multiple independent regions inside one struct are not expressible in v1; nest structs or copy data.
* `[LT-3]` **Static region.** String literals, `static` items, and `Span`s over them have the `static` region, which outlives everything and satisfies `[TYP-15]`'s storage restrictions (a `str` literal *may* be stored in a class field because its type is `str` with static region — the compiler records region `static` in the field's type; a non-static `str` cannot be stored there: `E3060 stored view may not outlive its source`).
* `[LT-4]` **Arena region.** `Arena` allocations return `ref mut T`/`MutSpan[T]` whose region is the arena's borrow; they cannot outlive the arena (`E3061`).
* `[LT-5]` **Inference.** Regions are inferred by the NLL algorithm in Part XVIII §5.7 for all locals; the programmer never writes them. Diagnostics report regions in terms of "the borrow of `x` on line N is still needed on line M".
* `[LT-1a]` **Explicit return region.** The attribute `@borrows(p₁, …, pₙ)`, written on its own line preceding the function declaration, overrides the region that rules 1–3 would assign to a view-typed return. It MUST name one or more parameters; the receiver is named as `self`. The return's region is the intersection of the named parameters' regions, and every unnamed view-typed parameter is NOT borrowed by the return, so the caller MAY continue to use it. `@borrows` overrides **all three** elision rules, **including rule 1**: a method whose result points into an argument rather than into its receiver MUST be written with `@borrows` naming that argument. Naming a parameter that is not view-typed, or writing `@borrows` on a function whose return is not view-typed, is `E2031`. If the returned value's region is not a subset of the intersection of the named parameters' regions, the body is rejected with `E3062` (shape B6).  `@borrows` is part of a function's public contract for compatibility purposes: widening it (naming fewer parameters) is a breaking change under `[VER-2]`, and an `override` of a `virtual` method MUST NOT name a superset of the base's parameters — the same inheritance rule as `[EFF-8]`.
* `[LT-1b]` **Intersection is reported at the definition.** When rule 3 assigns a view-typed return the intersection of two or more view-typed parameters' regions and the function carries no `@borrows`, the compiler emits the **opt-in lint** `L3014 return region is the intersection of N parameters` at the function's declaration, listing every parameter the caller will be unable to use while the result is live, and naming `@borrows` as the fix. Writing `@borrows` — including `@borrows` naming every view-typed parameter, which expresses the intersection deliberately — silences it. `[LT-1]`'s permissiveness is unchanged. `[LT-2a]` `L3014` is emitted likewise for a `@view struct` whose region is the intersection of two or more view-typed fields' regions.
* `[LT-7]` **Late-bound callback regions.** A callback-taking API MAY expose a callback boundary whose borrow region is chosen by the callee for each invocation. The callback's borrowed arguments are valid only for that invocation unless the API separately returns or transfers an owned value. The compiler models this boundary with one level of higher-ranked quantification: the callback is valid for any region selected by the callee at the call boundary. Region variables remain compiler-internal and MUST NOT appear in Ember source, public generic arguments, or ABI-visible type names. Implementations MUST reject any callback-local view that escapes the callback's selected region. `[THR-5]`'s and `[JOB-2]`'s scope regions are instances of this rule rather than bespoke exceptions, and `[GPU-9]`'s `pass.native(fn(cmd) => …)` is expressible under it.

`[LT-6]` Explicit named lifetimes (`fn f['a, 'b](…)`) are **reserved for v2**; the grammar reserves the `'ident` token form (`E0007` in v1 with the message "named lifetimes are not supported in this version; restructure using @borrows or a view struct"). The message MUST name `@borrows` only when a parameter exists whose region is the intended one; where the intended region is the callee's own body, the message MUST say that the construction is not expressible in v1 and name the view-struct restructuring instead.

## VII.6 Drops, `Drop` and destruction order

* `[DRP-1]` `fn drop(mut self)` is invoked exactly once per value at the end of its life. It may not be called explicitly (`E3070`); use `mem.drop(owned x)` to drop early.
* `[DRP-2]` Locals drop at the end of their block in reverse declaration order; struct fields after the struct's `drop`, in reverse declaration order; array elements in index order; enum payload of the active variant; tuple elements in reverse.
* `[DRP-3]` Temporaries drop at the end of the enclosing statement (`[EXP-4]`).
* `[DRP-4]` `drop` bodies MUST NOT panic in `abort` mode without accepting process termination; they SHOULD NOT block. `@noalloc` on a type's `drop` is honoured transitively.
* `[DRP-5]` A `drop` method's `mut self` may not move fields out (`[EXP-6]`); use `Option`/`mem.take` for fields that must be moved during destruction.
* `[DRP-6]` Drop of a `Box[T]` drops `T` then frees; drop of a class handle releases; drop of `Shared[T]` releases; drop of a view does nothing.

## VII.7 Views over containers: `Span` and `MutSpan`

```ember
fn normalize(mut xs: MutSpan[f32]):
    total = xs.iter().sum()
    for x in xs.iter_mut():
        x /= total                    # x: ref mut f32 — writes through

buf = Array[f32]([1, 2, 3])
normalize(buf.as_mut_span())          # or simply normalize(buf) — Array coerces to MutSpan at a `mut` site
left, right = buf.as_mut_span().split_at(1)   # two disjoint MutSpans
```

* `[SPN-1]` `Array[T]` coerces to `Span[T]` at borrow sites and to `MutSpan[T]` at `mut` sites; `[T; N]` likewise; `String` to `str`.
* `[SPN-2]` Indexing a `Span` is bounds-checked; `get(i) -> Option[ref T]` is the checked-without-panic form; `unsafe: s.get_unchecked(i)`.
* `[SPN-3]` `Span[T]` is `Copy`; `MutSpan[T]` is move-only and reborrowable (`s.reborrow()` or implicit at `mut` sites).

## VII.8 Interaction summary (what beginners see)

The rules above are designed so that ordinary code needs no annotations:

```ember
fn total_health(players: Array[Player]) -> f32:      # borrows the array (no `&`)
    sum = 0.0
    for p in players:                                 # p: ref Player… but Player is a class ⇒ p is a handle copy
        sum += p.health
    return sum

fn heal_all(mut players: Array[Player]):             # `mut` ⇒ inout
    for p in players:
        p.health = 100.0                              # handle access; fine (Part VIII)

fn take(owned players: Array[Player]) -> usize:      # consumes
    return players.len()                              # array dropped here (handles released)

fn main():
    ps = Array[Player]()
    ps.push(Player("a"))
    print(total_health(ps))                           # borrowed, still usable
    heal_all(ps)                                      # mutably borrowed, still usable
    n = take(ps)                                      # moved
    print(ps.len())                                   # E3040: use of moved value `ps` (moved on the line above)
```

---
