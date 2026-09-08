# Part IX — Memory Facilities

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
* `[ARN-6]` `ScopedArena`: `with scope = frame.scope():` creates a nested mark; the block's allocations are released at block end (LIFO), giving job-local memory (Part XI §6).

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
* `[UNS-4]` Invariants safe code may assume and unsafe code MUST uphold: every `ref` is non-null, aligned, points to initialised memory of the right type, and is not aliased by a `ref mut` while live; every `Span` length is within its allocation; every class handle points to a live object with a correct header; every `str` is valid UTF-8; no `Send`/`Sync` violation.
* `[UNS-5]` `MaybeUninit[T]`, `transmute[A, B]`, `ptr.copy_nonoverlapping`, `mem.zeroed[T]()` (requires `T: Zeroable`, an unsafe marker interface auto-derived for all-scalar/POD structs) are provided in `std.mem`.
* `[UNS-6]` Inline assembly: `unsafe asm("…", inputs, outputs, clobbers)` following LLVM's constraint syntax; the C backend rejects it (`E5090`) except on Clang/GCC where it emits `__asm__ volatile`. Prefer `std.cpu` intrinsics.

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

* `[CELL-1]` `Cell[T]` requires `T: Copy`. Its API is `Cell(v)`, `get(self) -> T`, `set(self, v: T)`, `replace(self, v: T) -> T`, `update(self, f: fn(T) -> T)`. All take `self` (a shared borrow) and mutate.
* `[CELL-2]` `Cell` never hands out a reference to its contents, so no aliasing rule can be violated and **no runtime check is needed**. `get` is a load; `set` is a store. There is no overhead relative to a plain field.
* `[CELL-3]` `Cell[T]` is `!Sync` (`[THR-1]`): it may be moved between threads if `T: Send`, but never shared. The `Sync` equivalent is `Atomic[T]`.
* `[CELL-4]` A `Cell` field does not make its containing struct mutable in any other respect, and does not affect `Copy` derivation: `Cell[T]` is itself `Copy` when `T: Copy`, and copying a `Cell` copies the value it holds at that moment.

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
* `[CELL-9]` The borrow-state counter is 1 machine word; in the `shipping` profile with `exclusivity = "unchecked"` the checks are compiled out and a violation is UB, matching `[EXC-1]`'s treatment of class exclusivity. In `debug` and `release` the checks are always present.
* `[CELL-10]` `RefCell` is not a synchronisation primitive and not a substitute for restructuring. The diagnostic for a borrow error (`[DIA-7]`, shape B4) suggests `RefCell` **only** when the conflicting accesses are provably not simultaneous in the same expression — never as a first suggestion.

### Which to reach for

| Situation | Use |
|---|---|
| Mutating a `Copy` field through a shared borrow | `Cell[T]` |
| Mutating a non-`Copy` value (container, `String`) through a shared borrow, single-threaded | `RefCell[T]` |
| Object graph with aliased mutation | `class` (Part VIII) — already gives this behaviour with no wrapper type |
| Same, across threads | `Mutex[T]` / `RwLock[T]` / `Atomic[T]` |
| Two mutable views into one container | `split_at_mut`, `chunks_mut`, `columns_mut` — no wrapper needed |

`[CELL-11]` `Cell`, `RefCell`, `Ref`, `RefMut`, `Atomic`, `Mutex`, `RwLock` live in `std.cell` and `std.sync`; `Cell` and `RefCell` are **not** in the prelude, so using them is a visible import.

---

