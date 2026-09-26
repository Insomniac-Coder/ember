---

# Part IX — Memory Facilities

## IX.0 Choosing a storage mechanism

The declared type decides storage and lifetime; the compiler never picks between them. This table is
the choice, in the order to try (`[SEL-1]`): a design that reaches for a `class` where a `struct` does
pays a heap allocation, a header, reference counting and exclusivity checks for nothing.

| Mechanism | Ownership | Aliasing | Destroyed | Reach for it when |
|---|---|---|---|---|
| `struct`, `enum`, tuple | unique, moved | borrow-checked | end of scope | **the default**: data with no identity |
| `Array`, `Map`, `Set`, `String` | unique, moved | borrow-checked | end of scope | collections |
| `Box[T]` | unique, heap | borrow-checked | on drop | one owner, but it must be on the heap: recursion, a large value, `dyn` |
| `class` | shared, counted | dynamic exclusivity | at count 0 | the thing has identity and many places refer to it |
| `Shared[T]` | shared, counted | borrow-checked + dynamic exclusivity | at count 0 | shared ownership of a plain value without declaring a class, on one thread |
| `SyncShared[T]` | shared, atomically counted | read-only; mutation through `T`'s locks or atomics | at count 0 | the same across threads (`[HEAP-10]`) |
| `Weak[O]` | none | — | never | back-pointers; observing without keeping alive |
| `Arena`, `FixedArena` | region | borrow-checked | all at once | many values with one lifetime: a frame, a parse |
| `Pool[T]` + `Handle[T]` | the pool | handles are plain `Copy` values | the pool decides | resources the program recycles: entities, GPU objects |
| `Cell[T]`, `RefCell[T]` | the owner's | interior | with the owner | mutating a value you hold only a shared borrow of |
| `Mutex[T]`, `RwLock[T]`, `Atomic[T]` | the owner's | synchronised | with the owner | mutation shared across threads |

* `[SEL-1]` The order above is the order to try. `ember inspect --alloc` reports the mechanism each
  declaration uses.
* `[SEL-2]` `Shared`/`Weak` and C++'s `std::shared_ptr`/`std::weak_ptr` (`CppShared`, `CppWeak`)
  never convert into each other: they use different counts, and conflating them frees twice (`E5065`).

## IX.1 The heap types

* `[HEAP-1]` Every heap type allocates through `ember_alloc`/`ember_realloc`/`ember_free` and carries
  the `Alloc` effect where it allocates.
* `[HEAP-2]` Growable buffers double, from a minimum of four elements; `shrink_to_fit` is explicit.
* `[HEAP-8]` *(new in 0.9.9)* Capacity arithmetic is checked: a requested capacity whose byte size
  overflows `usize`, or exceeds the allocator's limit, panics with `capacity overflow` before any
  allocation. No length or capacity computation in the runtime wraps.
* `[HEAP-9]` *(new in 0.9.9)* Heap storage for elements of type `T` is aligned to at least
  `align_of[T]()`, whatever that alignment is.
* `Box[T]`: one owner; `Box(v)`, auto-deref, `b.get()`; `Box[dyn I]` holds any implementer. Move-only.
* `[HEAP-3]` `Shared(value) -> Shared[T]` allocates one counted block holding `value`.
* `[HEAP-4]` *(changed in 0.9.9)* `s.get() -> ref T` borrows the payload: it begins a checked read
  access (`[EXC-2]`) that ends where the loan dies (`[EXC-18]`), and the loan keeps the block alive
  (`[RC-5]`).
* `[HEAP-5]` *(changed in 0.9.9)* `s.get_mut() -> ref mut T` begins a checked write access
  (`[EXC-1]`) for the loan's life; it does not require the `Shared` to be unique, and conflicts with
  any live `get` or `get_mut` through any copy.
* `[HEAP-6]` `Shared[T]` is `Copy`: copying retains, dropping releases.
* `[HEAP-7]` *(changed in 0.9.9)* `Weak[O]` exists for `O` a class handle, a `Shared[T]` or a
  `SyncShared[T]` (`[WK-11]`).
* `[HEAP-10]` *(new in 0.9.9)* `Shared[T]` is never `Send` or `Sync`: its counts and access state are
  plain memory. Shared ownership across threads uses `std.sync.SyncShared[T]` (`T: Send + Sync`, `[THR-9]`), whose counts
  are atomic and which offers `get() -> ref T` only; mutation goes through `T`'s own synchronisation,
  as for a `@sync` class (`[THR-1]`).

## IX.2 Arenas

```ember
@derive(Zeroable)
struct Cmd:
    id: int
    cost: f32

fn build(frame: Arena, n: int) -> MutSpan[Cmd]:
    cmds = frame.alloc_zeroed[Cmd](n)       # borrows `frame`: no annotation needed ([LT-44])
    for i in 0..n:
        cmds[i].id = i
    return cmds

fn main():
    frame = Arena.with_capacity(64 * 1024)
    cmds = build(frame, 8)
    println(cmds.len())
    frame.reset()                           # legal: `cmds` is no longer used
```

* `[ARN-1]` `Arena` is a move-only struct. Its `alloc*` methods take `self` (a shared borrow), so many
  allocations are outstanding at once, and return views that borrow the arena. `reset()` and dropping
  take `mut self`, so no view survives them (`[BRW-1]`).
* `[ARN-2]` Values in an arena are never dropped individually. Allocating a type that needs drop is
  `E3090`, unless through `alloc_nodrop`, which acknowledges that its `drop` will never run.
* `[ARN-3]` `alloc(v) -> ref mut T` moves `v` in. `alloc_array[T](n) -> MutSpan[T]` requires `T` not to
  need drop and `T: Default`, and initialises each element with `T.default()` in index order (`E2040`
  without `Default`). `alloc_zeroed[T](n) -> MutSpan[T]` requires `T` not to need drop and
  `T: Zeroable`, and fills with zero bytes. Neither stands in for the other: a `Zeroable` type's
  `alloc_array` still calls its `default`, and an implementation fills with zeros instead only where
  that is the same (a scalar's standard `Default`) (ODR-064). `alloc_uninit[T](n) ->
  MutSpan[MaybeUninit[T]]` initialises nothing.
* `[ARN-4]` *(changed in 0.9.9)* A growing `Arena` carries `Alloc` on its growth path. `FixedArena`
  (made by `Arena.fixed(buffer: MutSpan[u8])` or `FixedArena.with_capacity(n)` at start-up) never
  grows, carries no `Alloc` on allocation, and panics on exhaustion (`try_alloc` returns `Option`).
  `FixedArena` is the arena for `@noalloc` code.
* `[ARN-5]` `ArenaArray[T]` and `ArenaMap[K, V]` are fixed-capacity containers whose storage is taken
  from an arena once, at construction; they never grow or move, so views of their elements stay valid
  while the element and the arena do. Adding past capacity returns `Err(CapacityError.Full)` and changes
  nothing. Their elements, keys and values must not need drop. They live in `std.collections` and are
  not in the prelude.
* `[ARN-5c]` `ArenaArray[T]` provides `len`, `capacity`, `is_empty`, `get(i) -> Option[ref T]`,
  `get_mut`, `push(v) -> Result[void, CapacityError]`, `insert(i, v) -> Result[void, CapacityError]`,
  `remove(i) -> Option[T]`, `clear`, `iter` and `iter_mut` (in index order); indices follow the `Array`
  rules.
* `[ARN-5d]` `ArenaMap[K, V]` (with `K: Eq + Hash`) provides `len`, `capacity`, `is_empty`, `get`,
  `get_mut`, `insert(k, v) -> Result[Option[V], CapacityError]` (replacing returns the old value),
  `remove`, `contains_key`, `clear` and `iter`, which follows insertion order like `Map` (`[STD-11]`).
* `[ARN-5g]` No operation of either container touches the arena after construction: nothing grows,
  moves the arena's cursor or allocates elsewhere.
* `[ARN-6]` `arena.scope() -> ScopedArena` takes a mutable borrow of the arena for the scope's life;
  `with s = frame.scope():` allocates from `s` and releases everything at the block's end, last in first
  out. Using the parent while a scope is live is `E3096`, whose help is to allocate from the scope.
  Scopes nest: `s.scope()` opens one inside another.
* `[ARN-7]` No operation lowers an arena's allocation pointer or reuses its bytes while a view into it
  is live; every such operation takes `mut self`.
* `[ARN-8]` `MaybeUninit[T]` has the size and alignment of `T` and does not claim to hold a valid `T`;
  dropping it never drops a `T`. `MaybeUninit[T].uninit()` is safe; `write(mut self, owned v) -> ref
  mut T` stores without dropping previous bytes; `unsafe assume_init(owned self) -> T` asserts
  initialisation. `MutSpan[MaybeUninit[T]]` has `write_at(i, v)` and `unsafe assume_init(owned self)
  -> MutSpan[T]`. There is no other conversion.
* `[ARN-10]` A panic inside `T.default()` during `alloc_array` aborts (`[PAN-1]`); no partially
  initialised span becomes reachable.
* `[ARN-11]` `Zeroable` is an `unsafe` marker: all-zero bytes are a valid `T`. The scalars, raw
  pointers, a range including zero, and tuples and fixed arrays of `Zeroable` elements are; a struct
  is when it declares `@derive(Zeroable)`, which the compiler proves field by field (`E2080`), since
  only its author can vouch for its invariants (a `NonZero`'s field is an integer). A `ref`, a range
  excluding zero, or an enum whose zero discriminant is invalid is never `Zeroable`. `T: Zeroable` is
  a bound a generic function may state (`alloc_zeroed`, `mem.zeroed`). Being `Zeroable` never changes
  what `alloc_array` does (`[ARN-3]`).

## IX.3 Allocators

* `[ALC-1]` `Array[T, A: Allocator = Global]`, `Map`, `Set` and `Box` accept an allocator type
  parameter; an allocator instance is passed at construction (`Array.new_in(a)`). In `Map[K, V, H,
  A]` and `Set[T, H, A]` it follows the hasher (`[STD-11]`).
* `[ALC-2]` Implementing `Allocator` is `unsafe`: the implementer promises valid, aligned, unaliased
  memory.
* `[ALC-3]` *(changed in 0.9.9)* A binary package may replace the global allocator with
  `[build] global_allocator = "module.NAME"` naming a `static` whose type implements `Allocator`; an
  embedding host may pass allocation functions to `ember_rt_init` (`[FFI-27]`).
* `[ALC-4]` Allocation failure in a standard container panics (`out of memory`). `try_reserve` and
  `try_push` report it as a value.

## IX.4 Raw pointers and `unsafe`

```ember
# SAFETY: the caller guarantees `p` points to four readable bytes
unsafe fn read_u32_le(p: *u8) -> u32:
    return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))
```

* `[UNS-1]` An `unsafe` context is required to: dereference, read or write through `*T`/`*mut T`;
  offset a pointer; cast between pointers or pointers and integers; call an `unsafe fn`; call an
  `extern` function without a verified contract (Part XVI); access a `static mut`; `transmute`; call
  `get_unchecked` or `assume_init`; implement an `unsafe` interface.
* `[UNS-2]` `unsafe` permits exactly those operations. It does not turn off borrow checking, bounds
  checks on safe types, overflow checks or type checking.
* `[UNS-3]` `L3010` reports an `unsafe` block containing statements that need no `unsafe`.
* `[UNS-4]` Unsafe code MUST uphold what safe code assumes: every reference is non-null, aligned, points
  to a live initialised value of its type and is not aliased by a live `ref mut`; every view's length is
  within its allocation; no two live views overlap unless both are shared; every class handle points to
  a live object with a correct header; every `str` is UTF-8; every value is valid for its type (`bool`,
  `char`, range types, enums); `Send` and `Sync` are respected. A raw pointer derived from a reference
  is not used after that reference's region ends.
* `[UNS-5]` `std.mem` provides `Volatile[*T]` (volatile reads and writes), `transmute[A, B]`,
  `copy_nonoverlapping`, `zeroed[T]()` (requires
  `T: Zeroable`) and `MaybeUninit`.
* `[UNS-6]` Inline assembly is `unsafe asm("…", …)` on toolchains that support it and `E5090`
  otherwise.
* `[UNS-7]` Every `pub unsafe fn` carries `@safety("…")` stating the caller's obligation; without it,
  `L3015`. The standard library carries one on every unsafe function it exports.
* `[UNS-8]` *(changed in 0.9.9)* Every `unsafe:` block and `unsafe fn` carries a safety note
  (`[LEX-23]`): `# SAFETY: …` or `# SAFETY(category): …` on the line before it or at the end of its first
  line. Without one, `W3012`. `ember tcb` lists every block with its note, its category and the
  obligations of the unsafe functions it calls.
* `[UNS-10]` `UnsafeCell[T]` (in `std.mem`) is the primitive beneath every interior-mutability type:
  `UnsafeCell(v)`, `unsafe get(self) -> *mut T`, `into_inner(owned self) -> T`. It hands out no safe
  reference, checks nothing, is never `Copy` and never `Sync` (it is `Send` when `T` is), and is refused
  in `@static_safe` code (`E3105`). Diagnostics never suggest it.
* `[UNS-10a]` `UnsafeCell` suspends no rule globally: borrow, region, type and bounds checking all still
  apply around it. Unsafe code may break the aliasing rules inside an abstraction built on it, and must
  never let a conflicting reference escape into Safe Ember.

## IX.5 Layout

* `[LAY-2]` *(new in 0.9.9)* Layout attributes change layout exactly as stated, in every profile and on
  every backend, and are never ignored (`[ATT-6]`):
  * `@layout(c)` — the target C ABI layout (the default for every struct, `[TYP-11]`);
  * `@packed` — no padding; unaligned fields are read and written by byte copies; taking a reference
    to one is `E2170`;
  * `@align(N)` — raises the type's alignment to `N` (a power of two up to 4096); `size_of` becomes a
    multiple of `N`, and every place and heap buffer holding the type honours it (`[HEAP-9]`);
  * `@repr(u8|u16|u32|u64|i8|i16|i32|i64)` — an enum's discriminant type.
* `size_of[T]()`, `align_of[T]()` and `offset_of[T](field)` are compile-time functions.

## IX.6 Handles and pools

```ember
from std.collections import Pool, Handle

struct Texture:
    width: int
    height: int

fn main():
    pool = Pool[Texture]()
    h = pool.insert(Texture(width=64, height=64))
    println(pool.get(h).is_some())
    pool.remove(h)
    println(pool.get(h).is_some())        # stale handle: None
```

* `[HND-1]` A `Handle[T]` is a plain `Copy` value (index and generation). Every lookup compares the
  generation, in every profile, and a stale handle yields `None` (or panics in `pool[h]`).
  `unsafe: pool.get_unchecked(h)` skips the comparison.
* `[HND-2]` *(changed in 0.9.9)* `Handle[T]` is a `u64`: 32 bits of index and 32 of generation.
  `Pool[T, Bits]` may narrow it (`Pool[T, Split20]` gives a 20-bit index and a 12-bit generation in a
  `u32`).
* `[HND-3]` *(new in 0.9.9)* A slot whose generation is exhausted is retired and never reused, so an old handle can never
  become valid again. When every slot is retired or live, `insert` panics and `try_insert` returns
  `Err(CapacityError.Full)`.

## IX.7 Interior mutability: `Cell` and `RefCell`

```ember
struct Sprite:
    frame: Cell[int]

fn advance(s: Sprite):                 # `s` is borrowed, not `mut`
    s.frame.set(s.frame.get() + 1)

struct Scene:
    entities: RefCell[Array[int]]

fn add(s: Scene, e: int):
    with list = s.entities.borrow_mut():
        list.push(e)
```

* `[CELL-1]` `Cell[T]` holds a `T` that can be replaced through a shared borrow: `Cell(v)`, `set(v)`,
  `replace(v) -> T`, `take() -> T` (for `T: Default`), `into_inner()`, `get() -> T` for `T: Copy`, and
  `update(f)` for `T: Copy` or `T: Default`. `set` and `replace` store the new value before dropping the
  old one, since the old value's `drop` may read the cell.
* `[CELL-2]` `Cell` never hands out a reference to its contents, so it needs no run-time check; `get` is
  a load and `set` a store, with the retain and release of any counted handle in the value
  (`[OWN-7]`).
* `[CELL-3]` `Cell[T]` is not `Sync`; it is `Send` if `T` is.
* `[CELL-4]` `Cell[T]` is `Copy` when `T` is.
* `[CELL-5]` `RefCell[T]` keeps a one-word borrow counter beside `T`. `borrow() -> Ref[T]` succeeds
  unless a mutable borrow is active; `borrow_mut() -> RefMut[T]` succeeds unless any borrow is active;
  failure panics, naming the location of the conflicting borrow.
* `[CELL-6]` `try_borrow()` and `try_borrow_mut()` return `None` on conflict, in every profile.
* `[CELL-7]` `Ref[T]` and `RefMut[T]` are views of the cell; dropping one ends its borrow. `L3011` warns
  when a guard is live across a call that can reach the same cell — not across calls that provably
  cannot (a method on the guard itself, `print`).
* `[CELL-8]` `RefCell[T]` is not `Sync`; `Mutex[T]` and `RwLock[T]` are its thread-safe counterparts,
  with the same `with g = m.lock():` shape.
* `[CELL-9]` The `RefCell` check exists in every profile.
* `[CELL-10]` A borrow diagnostic suggests `RefCell` only after the structural fixes, and only when the
  conflicting accesses are provably not simultaneous (`[DIA-9]`).
* `[CELL-12]` `RefCell[T]` is never `Copy`.

## IX.8 Establishing disjointness

When two views come from unrelated sources that the programmer knows do not overlap, and the
structural fixes (`split_at`, `chunks_mut`, SoA columns) do not apply:

```ember
from std.mem import assert_disjoint

fn blend(mut dst: MutSpan[f32], src: Span[f32]):
    match assert_disjoint(dst, src):
        Ok((d, s)):
            for i in 0..d.len():
                d[i] = d[i] * 0.5 + s[i] * 0.5
        Err((d, s)):
            for i in 0..d.len():
                d[i] = (d[i] + s[i]) * 0.5
```

* `[DSJ-1]` *(changed in 0.9.9)* `assert_disjoint(a, b) -> Result[(A, B), (A, B)]` compares the two
  views' address ranges. It consumes (reborrows) both and returns them, carrying a proven disjointness
  fact in `Ok`, and unchanged in `Err`, so the overlapping path can still use them. The cost is two
  comparisons.
* `[DSJ-2]` The fact belongs to the returned values, not to a program point; reassigning them drops it.
* `[DSJ-3]` The borrow checker treats the returned views as non-overlapping, and the backend marks them
  `restrict`/`noalias` (`[SIMD-3]`).
* `[DSJ-4]` It applies to `Span`, `MutSpan`, `SoA` columns and arena views; two single-object
  references are `E3095`.
* `[DSJ-5]` `assert_disjoint_or_panic(a, b) -> (A, B)` panics on overlap. Both forms record a
  `RuntimeCheck(Aliasing)` site whose reason is `establishes_static_fact` (`[EFF-11]`), unless the
  compiler already knows the ranges are disjoint, in which case no comparison is emitted.
* `[DSJ-6]` `assert_disjoint` is usable in `@noalloc`, `@nosync` and `@static_safe` code: it
  establishes a fact rather than deferring a check.
* `[DSJ-7]` `unsafe assume_disjoint(a, b) -> (A, B)` asserts without checking; violating it is
  undefined behaviour.
* `[DSJ-8]` There is no form that checks in one profile and assumes in another (`[PHIL-13]`).
* `[DSJ-9]` `assert_disjoint_all(v1, …, vn)` handles 2 to 8 views pairwise.
