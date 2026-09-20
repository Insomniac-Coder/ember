Yes. I think ODR-017 should be closed, but I would **slightly change the framing** from the wording in the queue.

The cleanest resolution is:

> **`Shared[T]` gets an explicit strong-owner API, while `Weak` is generalized to be the weak form of a counted owner. `Weak[C]` remains valid and unchanged for classes; the companion of `Shared[T]` is `Weak[Shared[T]]`.**

That avoids introducing a second weak-pointer concept and avoids making `Weak[T]` mean two different things depending on whether `T` happens to be a class.

The current spec already gives us most of the machinery: `Shared[T]` is a reference-counted heap value, has the same header as classes, is `Copy` via retain, and already says its mutation follows the class exclusivity rules.  The existing `Weak[C]` has `Weak(h)`, `Weak[C].empty()`, `upgrade()`, and the existing weak-count/deinitialization semantics.  

## ODR-017 resolution

### 1. `Shared[T]` construction

Use the same constructor convention as `Box`:

```ember
s = Shared(value)
```

Normative:

```ember
Shared(value: T) -> Shared[T]
```

It allocates one counted heap object, copies/moves `value` into it according to the ordinary argument mode, and returns the first strong handle.

This is preferable to `Shared.new(value)` because the specification already uses constructor-style `Box(v)` and the language explicitly treats `Shared` as an ordinary library type rather than a compiler-special type. 

### 2. `Shared.get()`

Make:

```ember
s.get() -> ref T
```

It returns a **shared borrow** of the contained `T`.

That means:

```ember
s = Shared(Player())

r = s.get()
print(r.health)
```

does not create another `Shared`, does not retain, and does not transfer ownership.

The borrow keeps the underlying allocation live for the lifetime of the returned reference, using the same existing loan/liveness machinery already used for handles and views.

This is important because the current spec already has the rule that a loan through a counted handle keeps the handle alive until derived references are no longer used. 

### 3. Add `Shared.get_mut()`

This is the missing half:

```ember
s.get_mut() -> ref mut T
```

with a `mut` receiver:

```ember
get_mut(mut self) -> ref mut T
```

So:

```ember
mut s = Shared(Player())

r = s.get_mut()
r.health = 100.0
```

The important part is that **`mut self` does not imply unique ownership**.

`Shared[T]` is deliberately shared ownership. Therefore `get_mut()` enters the existing exclusivity machinery rather than inventing a uniqueness rule.

Semantics:

```text
get()      → shared/reader access
get_mut()  → exclusive/writer access
```

A `get_mut()` conflicts with any live shared or mutable long-term access to the same allocation.

That reuses `[EXC-1]`/`[EXC-2]` instead of creating a fourth aliasing model. The current class model already has precisely this read/write access-state machinery. 

I would explicitly state:

> `Shared[T]` mutation uses the same dynamic-exclusivity semantics as class-object mutation; no `Shared`-specific aliasing mechanism exists.

That is much better than trying to make `Shared[T]` secretly behave like Rust's `Arc<T>` where mutation simply isn't available.

---

# 4. What exactly is `Weak`?

This is the key design choice.

I would **not** define a separate `Weak[T]` meaning "weak pointer to a `Shared[T]` payload."

That would make these two things ambiguous:

```ember
Weak[Foo]
```

Does it mean:

```text
weak class Foo
```

or:

```text
weak Shared[Foo]
```

That is exactly the kind of duplicated semantic vocabulary Ember's simplicity consolidation is trying to eliminate.

Instead:

## `Weak[O]` means "weak form of counted owner O"

There are two valid families in v1:

```ember
Weak[C]             # weak class handle
Weak[Shared[T]]     # weak Shared owner
```

The existing `Weak[C]` remains exactly what it is today.

The new companion to `Shared[T]` is therefore:

```ember
Weak[Shared[T]]
```

This also meshes cleanly with the existing FFI distinction: `CppWeak[T]` remains a different foreign ownership mechanism and never interconverts with Ember's weak owners. 

---

# 5. Weak construction

Reuse the existing constructor form:

```ember
w = Weak(s)
```

For a `Shared[T]`:

```ember
s = Shared(Player())
w = Weak(s)
```

The type is inferred as:

```ember
Weak[Shared[Player]]
```

The operation:

* does **not** consume `s`;
* does **not** retain a strong reference;
* increments the weak count;
* creates a non-owning handle.

Likewise, the existing:

```ember
w = Weak(obj)
```

continues to produce:

```ember
Weak[MyClass]
```

No second constructor is needed.

---

# 6. Empty weak handles

Reuse the already-established API:

```ember
w = Weak[Shared[Player]].empty()
```

and:

```ember
w = Weak[MyClass].empty()
```

`empty()` produces a null/non-targeting weak handle and does not reference any allocation.

This preserves the existing class API rather than inventing `new_weak()`, `null_weak()`, etc. The existing spec already defines `Weak[C].empty()`. 

---

# 7. Weak copying and dropping

`Weak[O]` should be `Copy`.

Copying:

```ember
w2 = w
```

retains the **weak count**, not the strong count.

Dropping a weak handle releases the weak count.

This is the natural extension of the existing class control-block model, where the implicit weak side keeps the allocation itself alive after the strong count reaches zero. The current object representation already has separate `strong_count` and `weak_count`, with the block freed only after both sides are gone. 

---

# 8. `upgrade()`

For:

```ember
w: Weak[Shared[T]]
```

define:

```ember
w.upgrade() -> Option[Shared[T]]
```

For the existing class form:

```ember
w: Weak[C]
w.upgrade() -> Option[C]
```

So the return type is simply the **counted owner type represented by the weak handle**.

Examples:

```ember
s = Shared(Player())
w = Weak(s)

if Some(s2) = w.upgrade():
    print(s2.get().health)
```

If the last strong owner has already disappeared:

```ember
None
```

The existing deinitialization rule remains authoritative: `upgrade()` must fail once the object is deinitialising, so there is no resurrection path. 

For `T: Sync`, the strong-count acquisition during `upgrade()` must use the same atomic lifetime protocol as other cross-thread strong-handle traffic. This matches the existing statement that `Shared[T]` counts are atomic when `T: Sync`. 

---

# 9. The complete API becomes very small

That's the part I like most.

### `Shared[T]`

```ember
Shared(value: T) -> Shared[T]

s.get() -> ref T

s.get_mut() -> ref mut T
```

plus ordinary `Copy`/retain/drop behavior.

### Weak owner

```ember
Weak[O].empty() -> Weak[O]

Weak(owner: O) -> Weak[O]

w.upgrade() -> Option[O]
```

where:

```text
O = class handle
O = Shared[T]
```

That's it.

No:

```text
Shared.new()
Shared.clone()
Shared.weak()
Shared.downgrade()
Weak.new()
Weak.from_shared()
Weak.try_get()
Weak.get_or_null()
```

None of those are necessary.

That fits Ember's current simplicity rule extremely well: reuse the existing constructor, `Option`, `Copy`, borrow, exclusivity and weak-count mechanisms instead of creating parallel APIs. The specification already explicitly favors reuse/inference before new public concepts. 

---

# 10. One thing I would correct in the existing spec

There's a small inconsistency worth repairing alongside ODR-017.

The selection table currently says:

```text
Shared[T] → borrow-checked
```

but immediately says:

```text
mutation through Shared follows the exclusivity rules of Part VIII §3
```

while Part VIII's implementation model is dynamic exclusivity over a counted object. 

I'd change the table to:

```text
Shared[T] | shared, counted | borrow-checked + dynamic exclusivity for long-term mutable access | deterministic, at count 0 | ...
```

More precisely:

> **Static borrowing determines whether a reference may be formed; dynamic exclusivity protects aliased long-term accesses through the shared allocation.**

That makes the architecture explicit without creating a new safety system.

---

# 11. Rules I'd add

I would use the existing `HEAP-*` and `WK-*` families rather than introducing a new `SHR-*` rule family.

### `HEAP-3` — Shared construction

`Shared(value)` allocates one counted heap object containing `T` and returns its first strong handle.

### `HEAP-4` — Shared access

`Shared[T].get()` returns `ref T`, preserving the allocation's liveness for the reference's region and performing no strong retain.

### `HEAP-5` — Shared mutable access

`Shared[T].get_mut(mut self)` returns `ref mut T`. The access participates in the same static borrowing and dynamic exclusivity rules as a long-term mutable class access. It does not require or imply unique strong ownership.

### `HEAP-6` — Shared retention

`Shared[T]` is `Copy`; copying retains the strong count; dropping releases it; final release deinitializes the allocation and executes `T`'s ordinary destruction.

### `HEAP-7` — Weak owner forms

`Weak[O]` is legal exactly when `O` is a counted owner supported by the language: a class handle `C` or `Shared[T]`. `Weak[C]` preserves the existing class semantics; `Weak[Shared[T]]` is the companion weak form for `Shared[T]`.

### `WK-11` — Weak creation/copy/drop

`Weak(owner)` creates a non-owning weak handle without consuming or retaining the strong owner. `Weak[O].empty()` creates an empty weak handle. `Weak[O]` is `Copy`; copying increments the weak count and dropping decrements it.

### `WK-12` — Weak upgrade

`Weak[O].upgrade() -> Option[O]` returns a retained strong owner when the target is still strongly alive and not deinitialising; otherwise it returns `None`.

### `WK-13` — Shared weak identity

A `Weak[Shared[T]]` refers to the same allocation/control block as its source `Shared[T]`. It never creates an independent control block.

### `WK-14` — No ownership conversion

`Weak[C]` and `Weak[Shared[T]]` are distinct owner types. Neither converts to the other. `Shared[T]`/`Weak[Shared[T]]` also never convert to `CppShared[T]`/`CppWeak[T]`.

That is enough. I wouldn't add more unless an implementation test actually exposes a missing semantic case.

---

# 12. Conformance

I'd add `[TST-26]` covering:

```text
Shared construction
Shared Copy/retain
Shared final release/drop
get() → ref T
get_mut() → ref mut T
get/get_mut conflict
NLL after get/get_mut
Weak creation
Weak empty
Weak Copy/drop
upgrade while strong owner exists
upgrade after last strong owner → None
upgrade during deinitialization → None
Shared[Class] ownership cycle
Weak[Shared[Class]] breaking that cycle
class Weak[C] regression
Shared vs CppShared rejection
Weak[Shared[T]] vs CppWeak rejection
Sync/atomic upgrade path
```

The existing Phase 3 plan already explicitly lists `Shared[T]` and `Weak[T]` under the object/exclusivity implementation slice, so this closes exactly that missing contract rather than expanding the phase. 

---

# 13. Versioning

I would **not** call this `0.9.7_Hardened_4`.

This crosses the same boundary that the spec itself used for ODR-015: it changes the set of source programs that are well-defined by specifying previously undefined public API semantics.

So I would make it:

**`0.9.8` — language revision**

with:

> **ODR-017: Shared/Weak API completion**

and then `0.9.8_Hardened_1` for implementation/detail hardening afterward.

That is consistent with the current versioning rule: a semantic change to the accepted program set is a language revision, whereas hardening may not change rule meaning. 

## My final ruling

I'd lock in this model:

```text
                    counted owner
                         │
              ┌──────────┴──────────┐
              │                     │
          class C               Shared[T]
              │                     │
          Weak[C]             Weak[Shared[T]]
              │                     │
       upgrade() → C       upgrade() → Shared[T]
```

with:

```text
Shared(value)
Shared.get()       -> ref T
Shared.get_mut()   -> ref mut T

Weak(owner)
Weak[O].empty()
Weak[O].upgrade()  -> Option[O]
```

and **no new ownership or aliasing mechanism**.

I think this is the right fix because it closes ODR-017 with the **smallest public surface possible**, preserves the already-defined class `Weak[C]`, makes `Shared[T]` actually usable, and reuses Ember's existing borrow + exclusivity + counted-header architecture rather than introducing an `Arc`-like second system. The current specification already establishes that `Shared[T]` shares the class header/control-block model and that `Shared` participates in exclusivity and cycle analysis.  
