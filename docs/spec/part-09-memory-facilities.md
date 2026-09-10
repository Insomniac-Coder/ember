# Part IX — Memory Facilities

## IX.0 Choosing a storage mechanism

Ember has several storage and lifetime mechanisms on purpose — Part 0 row 2 makes
storage a property of the declared type rather than something the compiler infers,
and that only works if the programmer can choose. This table is the choice, in one
place. It is **normative guidance**, not a new rule: every cell restates a rule from
the part named in the last column.

| Use | Ownership | Aliasing | Destruction | Threads | Reach for it when |
|---|---|---|---|---|---|
| `struct` / `enum` | unique, moved | borrow-checked | end of scope, reverse order | `Send`/`Sync` by field | **the default.** Data with no identity: maths, components, messages (VII) |
| `Box[T]` | unique, heap | borrow-checked | deterministic, on drop | by `T` | one owner, but the value must be on the heap — recursion, a large payload, an unsized tail (IX.1) |
| `class` | shared, counted | dynamic exclusivity | deterministic, at count 0 | atomic count iff `Sync` | the thing has **identity** and several places refer to it: a scene node, an observer, an editor panel (VIII) |
| `Shared[T]` | shared, counted | borrow-checked | deterministic, at count 0 | atomic count iff `Sync` | shared ownership of a **value** type, where `class`'s identity and header are not wanted. Advanced; prefer `class` (IX.1) |
| `Weak[C]` | none | — | never | follows `C` | breaking a cycle, or observing something you do not keep alive (VIII.5, `[WK-1]`) |
| `Arena` | region | borrow-checked | all at once, at reset | thread-confined | many values with one lifetime: a frame, a level load, a parse (IX.2) |
| `Handle[Tag]` | none — an index | `Copy` | the pool decides | plain value | a resource the engine owns and may recycle: GPU objects, ECS entities. Generation-checked, so a stale handle is caught (IX.6) |
| `Cell[T]` | the owner's | interior, whole-value | with the owner | `!Sync` | a counter or memo inside a `struct` you only have a `ref` to (IX.7) |
| `RefCell[T]` | the owner's | interior, runtime-checked | with the owner | `!Sync` | as `Cell`, but you need a reference to the inside; the check is present in every profile (`[CELL-9]`) |
| `Mutex[T]` / `RwLock[T]` | the owner's | synchronised | with the owner | `Sync` | mutation shared **across threads** (XI) |
| `ForeignBox[T]` | foreign, adopted | raw | the foreign destructor | foreign contract | an owned pointer from C or C++ (XVI, `[FFI-36]`) |
| `CppShared[T]` | foreign, `std::shared_ptr` | raw | C++'s count | C++'s rules | a `std::shared_ptr` crossing the boundary. **Not** `Shared[T]` — two independent counts over one object is a double free (`[FFI-17a]`) |

* `[SEL-1]` **The order above is the order to try.** A design that reaches for
  `class` where a `struct` would do pays a heap allocation, a header, reference
  counting and dynamic exclusivity for nothing, and `[CLS-*]`'s ergonomics are not
  a reason to skip the question. `ember inspect --alloc` reports which mechanism a
  declaration actually used.
* `[SEL-2]` **Two mechanisms are never interchangeable across the foreign
  boundary.** `Shared[T]` and `CppShared[T]` both denote shared ownership and use
  *different reference counts*; `Weak[C]` and `CppWeak[T]` likewise. Converting one
  to the other by transmute or by an overlay declaration is `E5065`, and the
  diagnostic names the double free it prevents.

## IX.1 The library heap types

| Type | Semantics | Copy? | Thread |
|---|---|---|---|
| `Box[T]` | uniquely owned heap `T`; `Box(v)`, `b.get()`, deref via auto-deref; `Box[dyn I]` for existentials | move-only | `Send` iff `T: Send` |
| `Shared[T]` | reference-counted heap `T` (a struct behaving like a class instance) with `Weak[T]`; same header as classes; `s.get()`; mutation through `Shared` follows the exclusivity rules of Part VIII §3 | Copy (retain) | count atomic iff `T: Sync` |
| `Array[T]` | growable buffer `{ptr, len, cap}`; SSO not applied; `Array.with_capacity(n)`; `reserve`, `push`, `pop`, `insert`, `remove`, `swap_remove`, `retain`, `drain`, `clear`, `truncate`, `extend`, `as_span`, `as_mut_span`, `sort`, `sort_by`, `binary_search` | move-only; `Clone` if `T: Clone` | `Send`/`Sync` iff `T` |
| `String` | UTF-8 `{ptr, len, cap}` with **inline storage for ≤ 23 bytes** (SSO) | move-only | `Send + Sync` |
| `Map[K, V]` | open-addressing hash map (SwissTable-style), `K: Hash + Eq` | move-only | iff `K, V` |
| `Set[T]`, `Deque[T]`, `BitSet`, `SmallArray[T, N]` (inline up to N), `Pool[T]` (slot map with generational keys) | as named | | |

`[HEAP-1]` All of these allocate through `ember_alloc`/`ember_realloc`/`ember_free` and report allocation to the `Alloc` effect (Part X). `[HEAP-2]` Growth factor is 2× with a minimum of 4 elements; `shrink_to_fit` is explicit.

## IX.2 Arenas

```ember
frame = Arena.with_capacity(16 * MB)          # one chunk; grows by chunk (default 1 MB) if exceeded
cmd   = frame.alloc(RenderCommand(...))       # -> ref mut RenderCommand, region = borrow of `frame`
list  = frame.alloc_array[Instance](count)    # -> MutSpan[Instance], zero-initialised if T: Zeroable else Default
tmp   = frame.alloc_uninit[u8](bytes)         # -> MutSpan[MaybeUninit[u8]]
frame.reset()                                 # requires `mut frame` and no live borrows (borrow checker enforces)
```

* `[ARN-1]` `Arena` is a move-only struct. All `alloc*` methods take `self` (shared borrow) so many allocations can be outstanding; they return views with the arena's region. `reset()` and drop take `mut self`, so `[BRW-1]` guarantees no live view survives a reset.
* `[ARN-2]` Values allocated in an arena are **not dropped individually**. `[ARN-3]` Allocating a type that `needs_drop` in an arena is `E3090` unless the call is `alloc_nodrop` (explicit acknowledgement that `drop` will never run) — this keeps arenas free of destructor bookkeeping and prevents silent resource leaks of handles/GPU objects.
* `[ARN-4]` `Arena` allocation is bump allocation with alignment padding; it is `@noalloc`-clean **only** if the arena is `@noalloc`-declared (`Arena.fixed(buffer: MutSpan[u8])`, which never grows and panics on exhaustion) — a growing arena carries the `Alloc` effect on the growth path. The type `FixedArena` is provided for hot paths.
* `[ARN-5]` `ArenaArray[T]`, `ArenaMap[K,V]` are container variants whose backing storage is an arena view; they are view types (`@view`) and follow `[TYP-15]`.
* `[ARN-6]` `ScopedArena`: `Arena.scope(mut self) -> ScopedArena` takes a **mutable** borrow of the parent arena, held for the `ScopedArena`'s whole region. `with scope = frame.scope():` creates a nested mark; the block's allocations are released at block end (LIFO), giving job-local memory (Part XI §6). While a scope is live the parent MUST NOT be allocated from, reset, or dropped — `E3096 arena is scoped here`, with `help: allocate from `scope` instead, or take this allocation before opening the scope`. Nested scopes are obtained from the `ScopedArena` (`ScopedArena.scope(mut self)`), which nests marks to any depth.
* `[ARN-7]` LIFO rewind is a **safety property, not a convenience**. An implementation MUST NOT provide any operation that lowers an arena's bump pointer, invalidates a mark, or reuses arena bytes while a view whose region derives from that arena is live. Every such operation MUST take `mut self` on the arena whose bytes it reclaims, which is what makes `[BRW-1]` the enforcing rule. `[ARN-1]`'s argument extends to `scope`, `reset` and drop alike.

## IX.3 Allocators

```ember
unsafe interface Allocator:
    fn alloc(mut self, layout: Layout) -> Result[*mut u8, AllocError]
    fn dealloc(mut self, ptr: *mut u8, layout: Layout)
    fn realloc(mut self, ptr: *mut u8, old: Layout, new_size: usize) -> Result[*mut u8, AllocError]: ...default...
```

* `[ALC-1]` `Array[T, A: Allocator]`, `Map[K, V, A]`, `Box[T, A]` accept an allocator type parameter (default `Global`). Allocator instances are passed at construction (`Array.new_in(alloc)`), stored by value if zero-sized, else as a `ref`/handle.
* `[ALC-2]` Implementing `Allocator` is `unsafe` (the implementer promises the returned memory is valid, aligned and not aliased).
* `[ALC-3]` The global allocator can be replaced at link time by defining `@export("ember_global_allocator") static ALLOC: dyn Allocator` in the binary package.
* `[ALC-4]` Allocation failure is a **panic** for the standard containers (`alloc` returns `Result`, containers unwrap). `try_reserve` exists for code that must handle it.

## IX.4 Raw pointers and `unsafe`

```ember
unsafe fn read_u32_le(p: *u8) -> u32:                 # unsafe fn: caller must uphold "p points to ≥4 readable bytes"
    return (p.read() as u32) | ((p.offset(1).read() as u32) << 8) | ...

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4: return None
    unsafe:                                            # SAFETY: bounds checked above
        return Some(read_u32_le(data.as_ptr()))
```

* `[UNS-1]` Operations requiring an `unsafe` context: dereferencing `*T`/`*mut T` (`read`, `write`, `deref`, `deref_mut`, index), pointer arithmetic (`offset`, `add`, `sub`), pointer casts (`[TYP-7]`), calling an `unsafe fn`, calling any `extern` function not covered by a verified contract (`[FFI-*]`), accessing `static mut`, `transmute`, `get_unchecked`, `assume_init`, implementing an `unsafe interface`, inline assembly.
* `[UNS-2]` An `unsafe:` block does not disable the borrow checker, bounds checks on safe types, or type checking; it only permits the operations above.
* `[UNS-3]` The lint `L3010 unsafe block larger than necessary` fires when statements inside an `unsafe` block need no unsafe permission.
* `[UNS-4]` Invariants safe code may assume and unsafe code MUST uphold: every `ref` is non-null, aligned, points to initialised memory of the right type, and is not aliased by a `ref mut` while live; every `Span` length is within its allocation; every class handle points to a live object with a correct header; every `str` is valid UTF-8; no `Send`/`Sync` violation. No two views (`Span`, `MutSpan`, `str`, `Ref`, `RefMut`, a `@view struct`, or a `ref`) that are simultaneously live in safe code may overlap unless both are shared. Constructing overlapping views through raw pointers, `transmute`, or a foreign call and handing them to safe code is undefined behaviour; `[SIMD-3]` and `[CG-C-4]` depend on this invariant. Unsafe code MUST NOT use a raw pointer derived from a `ref` after that reference's region has ended, nor one derived from a class-object field after the object's last live handle has been released (`[RC-5]`).
* `[UNS-5]` `MaybeUninit[T]`, `transmute[A, B]`, `ptr.copy_nonoverlapping`, `mem.zeroed[T]()` (requires `T: Zeroable`, an unsafe marker interface auto-derived for all-scalar/POD structs) are provided in `std.mem`.
* `[UNS-6]` Inline assembly: `unsafe asm("…", inputs, outputs, clobbers)` following LLVM's constraint syntax; the C backend rejects it (`E5090`) except on Clang/GCC where it emits `__asm__ volatile`. Prefer `std.cpu` intrinsics.
* `[UNS-7]` **`@safety("…")` on an `unsafe fn`.** Every `pub unsafe fn` SHOULD carry at least one `@safety("<obligation>")` attribute stating in one sentence, per obligation, what the caller must guarantee. The text is normative documentation, not a checked expression; *(0.6.2 leftover removed 2026-09-09; see `docs/spec-amendments.md`)* **`std` MUST carry a `@safety` on every `unsafe fn` it exports.** A `pub unsafe fn` outside `std` with no `@safety` is `L3015 undocumented unsafe obligation`, at **warn**, whose fix-it inserts `@safety("TODO: state the caller's obligation")`; `L3016` reports a `@safety` text still reading `TODO`.
* `[UNS-8]` **An `unsafe` block records the obligations it discharges.** The compiler MUST record, per `unsafe:` block, the `unsafe fn`s called within it together with their `[UNS-7]` obligations, and MUST emit `W3012 unsafe block with no SAFETY note` at `warn` when the block is not preceded by a `## SAFETY:` doc comment. This is a reporting rule and introduces **no fourth tier**: `[TIER-1]`'s three boundaries are unchanged and neither attribute licenses any operation.
* `[UNS-10]` **`UnsafeCell[T]`** is the lowest-level interior-mutability primitive and lives in `std.mem` beside `[UNS-5]`'s facilities. It permits mutation of its contained storage through **shared** access, and **only from `unsafe` code**. Its API is `UnsafeCell(owned v: T)`, `get(self) -> *mut T` — which needs an `unsafe` context, being a raw pointer under `[UNS-1]` — and `into_inner(owned self) -> T`, which is safe because the cell is consumed and nothing is shared. It hands out **no** safe `ref T` or `ref mut T`, performs **no** runtime borrow check, and provides **no** synchronisation. `UnsafeCell[T]` is **never `Copy`**, is always `!Sync` (`[THR-1]`), and is `Send` when `T: Send`. It exists so that expert library code can implement an abstraction whose invariant cannot be expressed with ordinary borrowing, `Cell`, `RefCell` or a synchronisation primitive. `Cell` (exposes no reference), `RefCell` (runtime borrow check), `Mutex`/`RwLock` (synchronisation) and `UnsafeCell` (the author establishes the invariant) are one hierarchy, and this is its floor — which is what `[CELL-9]`'s sentence about a package needing an unchecked primitive refers to. It does **not** change the semantics of `Cell`, `RefCell`, `Mutex` or `RwLock`, and `exclusivity = "unchecked"` (`[EXC-1]`) does not change its semantics either.
* `[UNS-10a]` **`UnsafeCell` suspends nothing globally.** It does not disable `[BRW-1]`, lifetime or region checking, type checking or bounds checking, and it introduces no further safety tier: `[UNS-2]` applies to it unchanged. Unsafe code MAY temporarily violate the static aliasing proof **inside** the abstraction, and MUST NOT allow a conflicting or otherwise invalid reference to escape into Safe Ember. A safe abstraction may be built on `UnsafeCell`, but hiding an unsafe operation behind a safe signature does not make that abstraction sound: the implementation MUST establish the invariant before it exposes a safe value. The obligations are `[UNS-4]`'s, and the author is responsible for each as it applies — validity, initialisation, type correctness, alignment and non-nullness for raw access, aliasing, lifetime and region validity, correct destruction, and thread-safety. `@safety` (`[UNS-7]`) and `[UNS-8]`'s obligation record are the machinery; no separate documentation or safety system is introduced for it.
* `[UNS-10b]` **`UnsafeCell` is an expert facility and diagnostics MUST NOT suggest it.** No `[DIA-*]` shape may name it as a fix, in the spirit of `[CELL-10]`'s restraint about `RefCell`. It is **not permitted in `@static_safe` code** (`E3105`): `@static_safe` requires safety to be established statically, and `UnsafeCell` delegates it to an unsafe implementation. It introduces no representation or ABI behaviour of its own — foreign use follows the ordinary explicit representation contract — no special hot-reload semantics, and no inherent `Nondet` effect; any effect arises from the abstraction built over it.

## IX.5 Layout attributes

* `@layout(c)` (default): C struct layout for the target ABI.
* `@packed`: no padding; field access to unaligned fields is lowered to `memcpy`-style loads/stores (never UB); taking a `ref` to an unaligned field is `E2170` (take a copy).
* `@align(N)`: raise alignment; `N` power of two ≤ 4096.
* `@repr(u8|u16|u32|i32|…)`: enum discriminant type; required for FFI enums.
* `@gpu_layout(std140|std430|scalar)`: see Part XVII; produces `comptime`-visible offsets and asserts that the CPU layout matches (`E8001` with the offending field and both offsets if not — the programmer inserts explicit padding fields).
* `size_of[T]()`, `align_of[T]()`, `offset_of[T](field)` are comptime functions.

## IX.6 Handles for resources

The standard library provides a generational handle facility used by the GPU layer and recommended for any resource manager:

```ember
@derive(Copy, Eq, Hash, Debug)
struct Handle[Tag]:                     # Tag is a phantom type: Handle[Texture] ≠ Handle[Buffer]
    index: u32                          # 20 bits index, 12 bits generation — matches RageV's ECS.Entity packing
    fn is_null(self) -> bool
    const NULL: Handle[Tag]

struct Pool[T]:                         # slot map
    fn insert(mut self, owned v: T) -> Handle[T]
    fn get(self, h: Handle[T]) -> Option[ref T]        # generation-checked
    fn get_mut(mut self, h: Handle[T]) -> Option[ref mut T]
    fn remove(mut self, h: Handle[T]) -> Option[T]
```

`[HND-1]` `Handle` is a plain `Copy` value; `[HND-2]` the index/generation split is configurable per `Pool` (`Pool[T, INDEX_BITS=20]`).

## IX.7 Interior mutability: `Cell` and `RefCell`

`[BRW-1]`'s aliasing-XOR-mutability rule is checked statically for value types. Some correct programs cannot be written that way: a shared cache, a memoised field, an observer that mutates a counter it does not own, a value reachable from two places that both need to write it. Classes solve this for object graphs (Part VIII: refcounting plus dynamically checked exclusivity). `Cell` and `RefCell` provide the equivalent escape hatch for **value types**, so that a programmer who hits the wall on a `struct` has a ladder that stays in safe code.

Both are safe types built on `unsafe` internals. Neither is a way to opt out of the rules; each moves one specific check from compile time to runtime, and says so in the type.

### `Cell[T]` — replace the whole value, no references

```ember
struct Sprite:
    frame: Cell[u32]                          # mutable through a shared borrow
    texture: TextureHandle

fn advance(s: Sprite):                        # note: `s` is borrowed, not `mut`
    s.frame.set(s.frame.get() + 1)
```

* `[CELL-1]` `Cell[T]` places any `T`. Its unconditional API is `Cell(owned v)`, `set(self, owned v: T)`, `replace(self, owned v: T) -> T`, `into_inner(owned self) -> T`, and `take(self) -> T where T: Default`; all take `self` (a shared borrow) and mutate. `get(self) -> T` is provided only where `T: Copy`, by `extend[T: Copy] Cell[T]:` — ordinary Part V §6 machinery, no specialisation implied, `[TYP-19]` unaffected. `update(self, f: fn(T) -> T)` requires `T: Default` or `T: Copy`. **`set` and `replace` MUST store the new value before dropping the old one.** A drop can run arbitrary user code that re-enters the same `Cell` (`Cell[Box[Node]]` where `Node`'s drop reaches back and reads it); a drop-then-store implementation would leave the `Cell` observably uninitialised across that window, which is a read of uninitialised memory.
* `[CELL-4]` A `Cell` field does not make its containing struct mutable in any other respect. `Cell[T]` is `Copy` when `T: Copy`, and copying such a `Cell` copies the value it holds at that moment; `Cell[T]` for a non-`Copy` `T` is move-only, and is `Drop` iff `T` is.
* `[CELL-2]` `Cell` never hands out a reference to its contents, so no aliasing rule can be violated and **no runtime check is needed**. `get` is a load; `set` is a store. There is no overhead relative to a plain field. **`Cell`, `RefCell` and `Arena` all permit controlled mutation without exposing the unrestricted ownership model of an ordinary mutable field, and each enforces a different invariant to do it.** They are not all interior mutability: `Cell` and `RefCell` are, and `Arena` is a region allocator whose mutation happens to reach through a shared borrow. What they share is the implementation concern, not the concept. The mechanisms are distinct: `Cell[T]` replaces the whole value and hands out no reference, so nothing has to be proved and there is no runtime state; `RefCell[T]` mutates *through* a reference, so `[BRW-1]`'s question is asked at run time against a borrow counter (`[CELL-5]`..`[CELL-8]`); `Arena`'s mutation is allocation, and what it must prove is a region rather than an alias, which `[ARN-1]` does statically by giving the views the arena's region and taking `mut self` to reset. What follows for every one of them is that interior mutability never means the borrow checker stops caring: the obligation moves — to a replacement that cannot alias, to a counter, or to a region — and an implementation that satisfies any of the three by exempting a type from `[BRW-1]` has not implemented it. An implementation MAY share internal machinery between them; nothing here requires it to. *(clarified 2026-09-09; see `docs/spec-amendments.md`)*
* `[CELL-3]` `Cell[T]` is `!Sync` (`[THR-1]`): it may be moved between threads if `T: Send`, but never shared. The `Sync` equivalent is `Atomic[T]`.

### `RefCell[T]` — dynamically checked borrows of any `T`

```ember
struct Scene:
    entities: RefCell[Array[Entity]]

fn add(s: Scene, e: Entity):
    with list = s.entities.borrow_mut():      # runtime check begins here
        list.push(e)                          # list: ref mut Array[Entity]
                                              # check ends at block exit
```

* `[CELL-5]` `RefCell[T]` holds a borrow-state counter alongside `T`. `borrow(self) -> Ref[T]` succeeds unless a mutable borrow is active; `borrow_mut(self) -> RefMut[T]` succeeds unless any borrow is active. Both **panic** on failure with `E-panic: RefCell already mutably borrowed (borrowed at <file>:<line>)` — the message names the source location of the conflicting borrow, which the runtime records in debug and release profiles.
* `[CELL-6]` `try_borrow`/`try_borrow_mut` return `Option[Ref[T]]`/`Option[RefMut[T]]` for code that must handle contention rather than panic.
* `[CELL-7]` `Ref[T]`/`RefMut[T]` are **view types** (`[TYP-15]` applies) whose region borrows the `RefCell`; their `drop` releases the borrow state. They MUST be bound by `with` or a local — the lint `L3011 RefCell guard held across a call` fires when a guard is live across a function call that could re-enter the same cell.
* `[CELL-8]` `RefCell[T]` is `!Sync`. The `Sync` equivalents are `Mutex[T]` and `RwLock[T]`, whose API is deliberately the same shape (`with g = m.lock():`) so that promoting single-threaded code to shared code is a type change and nothing else.
* `[CELL-12]` **`RefCell[T]` is never `Copy`, whatever `T` is.** A `RefCell` carries mutable runtime borrow state, and copying the value would duplicate that state: two cells would then hold independent and inconsistent knowledge of the same storage, and the runtime borrow invariant `[CELL-5]`..`[CELL-8]` rests on would be unsound. Moving a `RefCell[T]` transfers the whole cell, borrow state included. Copying the contained `T` is a separate question and is unaffected. `[CELL-4]`'s field-derived `Copy` rule is stated for `Cell` and does **not** extend here: `RefCell` is an explicit exception, because its borrow state is semantically coupled to its storage in a way `Cell`'s payload is not.
* `[CELL-9]` The borrow-state counter is one machine word and the check is present in **every** profile. `exclusivity = "unchecked"` (`[EXC-1]`, ADR-004) governs dynamic **class** exclusivity only; it MUST NOT affect `RefCell`, `Ref`, `RefMut`, `Cell`, `Mutex` or `RwLock`. A package that requires an interior-mutability primitive with no check uses `unsafe` (`UnsafeCell`, `[UNS-*]`), which is visible in review and in `grep`.
* `[CELL-10]` `RefCell` is not a synchronisation primitive and not a substitute for restructuring. The diagnostic for a borrow error (`[DIA-7]`, shape B4) suggests `RefCell` **only** when the conflicting accesses are provably not simultaneous in the same expression — never as a first suggestion.
* `[CELL-6a]` `try_borrow` and `try_borrow_mut` MUST return `None` on contention in every profile. No profile setting may make them infallible; doing so would change which branch of a `match` executes, which `[PRF-1]` forbids.

### Which to reach for

| Situation | Use |
|---|---|
| Mutating a `Copy` field through a shared borrow | `Cell[T]` |
| Mutating a non-`Copy` value (container, `String`) through a shared borrow, single-threaded | `RefCell[T]` |
| Object graph with aliased mutation | `class` (Part VIII) — already gives this behaviour with no wrapper type |
| Same, across threads | `Mutex[T]` / `RwLock[T]` / `Atomic[T]` |
| Two mutable views into one container | `split_at_mut`, `chunks_mut`, `columns_mut` — no wrapper needed |

`[CELL-11]` `Cell`, `RefCell`, `Ref`, `RefMut`, `Atomic`, `Mutex`, `RwLock` live in `std.cell` and `std.sync`; `Cell` and `RefCell` **are** in the prelude (owner decision `OQ-10`): `Shared[T]` is already there and is heavier on every axis this document measures, so taxing the zero-cost facility and not the expensive one inverted the gradient the earlier rule meant to create. `[DIA-9]` still forbids offering them as a *first* suggestion, which is where the visibility actually matters.

## IX.8 Establishing disjointness: `assert_disjoint` and `assume_disjoint`

`[BRW-5]` makes two mutable borrows through computed indices conflict, because the compiler cannot in general prove `i != j`. The sanctioned structural fixes (`split_at_mut`, `chunks_mut`, `columns_mut`) cover the common cases. For the rest — two spans obtained from unrelated sources that the programmer knows do not overlap — Ember provides two clearly distinct tools, matching the two arms of `[PHIL-8]`.

### `mem.assert_disjoint` — verify now, establish for the region (safe)

```ember
from std.mem import assert_disjoint

fn blend(mut dst: MutSpan[f32], src: Span[f32]):
    match assert_disjoint(dst, src):
        Some(d, s):                       # d, s are *known* disjoint from here on
            for i in 0..d.len():
                d[i] = d[i] * 0.5 + s[i] * 0.5
        None:
            fallback_overlapping_blend(dst, src)
```

* `[DSJ-1]` `assert_disjoint(a, b)` compares the two views' address ranges (`base`, `base + len * size_of[T]()`) and returns `Some((a', b'))` when they do not overlap, `None` when they do. It **consumes** `a` and `b` and returns fresh views carrying a compile-time disjointness fact. The cost is two comparisons, once, at the call.
* `[DSJ-2]` The proof is attached to the **returned values**, not to a program point. This is deliberate: a flow-sensitive fact recorded against a line silently rots when either operand is reassigned, whereas a fact carried by a value cannot be separated from the value it describes. Reassigning `d` or `s` produces ordinary views again.
* `[DSJ-3]` The returned views are treated as **non-overlapping places** by the borrow checker (`[BRW-5]` does not apply between them) and by the aliasing facts passed to the backend (`restrict` / `noalias`, `[SIMD-3]`), so a loop over both vectorises.
* `[DSJ-4]` Applicable operands: `Span[T]`, `MutSpan[T]`, `SoA` columns, and arena views — anything whose base address and byte length are recoverable. It is **not** available for arbitrary `ref mut`s to unrelated locals (`E3095`), because two single-object references have no range to compare and the borrow checker already handles the cases that arise in practice.
* `[DSJ-5]` `assert_disjoint_or_panic(a, b) -> (MutSpan[T], Span[T])` is the panicking form. Both forms record a `RuntimeCheck(Aliasing)` site with reason `establishes_static_fact` **in the contract effect set (`[EFF-15]`)** whenever the comparison is not elided by profile-independent analysis; the comparison is elided when the compiler already knows the ranges are disjoint (the common `split_at_mut` case), in which case no site is recorded.
* `[DSJ-6]` `assert_disjoint` is `@noalloc`, `@nosync`, and usable in a `@static_safe` function: it *establishes* a static fact rather than deferring a check, and once the comparison is elided or hoisted out of a loop the body contains no aliasing check at all. When the comparison itself is emitted inside a `@static_safe` function, it is permitted — `[EFF-12]` forbids checks that stand in for an unproven property, and this one proves it.

### `unsafe assume_disjoint` — assert without verification (unsafe)

```ember
unsafe:
    d, s = assume_disjoint(dst, src)     # SAFETY: caller contract guarantees distinct allocations
```

* `[DSJ-7]` `assume_disjoint` has the same signature shape as `assert_disjoint` but performs **no check in any profile** and returns the views unconditionally. It requires `unsafe` because the programmer, not the machine, is asserting the property; violating it is UB exactly like any other broken `unsafe` precondition (`[UNS-4]`).
* `[DSJ-8]` There is deliberately **no third form** that is checked in `debug` and assumed in `shipping`. Such a construct would give a program two different meanings under two profiles, would move a memory-safety property into a build setting, and would be invisible in review because it does not contain the word `unsafe`. `[PRF-1]` already forbids profiles from changing semantics; this rule names the specific temptation.
* `[DSJ-9]` `assert_disjoint_all(v1, …, vn)` and `assert_disjoint_all_or_panic` accept 2..8 view operands, perform the n(n−1)/2 pairwise range comparisons (at most 28), consume all operands and return proof-carrying views for all of them, pairwise disjoint. `[DSJ-1]`..`[DSJ-6]` apply unchanged to each pair.

The distinction in one line: **`assert_disjoint` verifies a property and establishes it; `assume_disjoint` claims a property Ember cannot verify.**

---

