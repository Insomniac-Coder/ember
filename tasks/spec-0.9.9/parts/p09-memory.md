---

# Part IX — Memory Facilities

## IX.0 Choosing a storage mechanism

The declared type decides storage and lifetime; the compiler never picks between them. This table is
the choice, in the order to try (`[SEL-1]`): a design that reaches for a `class` where a `struct` does
pays a heap allocation and a header for nothing, and one that reaches for `shares` where one owner does
pays a count for nothing.

| Mechanism | Ownership | Aliasing | Destroyed | Reach for it when |
|---|---|---|---|---|
| `struct`, `enum`, tuple | one owner, handed over by `=` | checked when compiling | when its owner ends | **the default**: data with no identity |
| `Array`, `Map`, `Set`, `String` | one owner, handed over by `=` | checked when compiling | when its owner ends | collections |
| `owns T` | one owner, in a heap block of its own | checked when compiling | when its owner ends | one owner, but it must be in a block of its own: recursion, a large value, `owns dyn I` |
| `class` | one owner unless a line says `shares`; a heap block of its own | checked when compiling | when its last owner ends | the thing has identity (`is`), subclasses, or `weakref` links to it |
| `shares T`, `shares mut T` | several owners on one thread, counted | checked when compiling | when its last owner ends | several places own one value; `shares mut` for owners that change it |
| `thread_shared T` | several owners on any threads, counted for several threads (`[RC-4]`) | changed only through `Atomic`, a lock, a channel or a `guarded by` field | when its last owner ends | the same across threads (`[HEAP-10]`) |
| `weakref T` | none | — | never | back links; observing without keeping alive |
| `sharedref mut T` | none: a view several holders share | checked when compiling | never | several holders change one object they do not own, on one thread (`[LT-46]`) |
| `Arena`, `FixedArena` | region | checked when compiling | all at once | many values with one lifetime: a frame, a parse |
| `Pool[T]` + `Handle[T]` | the pool | handles are plain `Copy` values | the pool decides | resources the program recycles: entities, GPU objects |
| `Cell[T]` | the owner's | replaced whole, never viewed | with the owner | a value set whole, by any name that may change its holder |
| `Mutex[T]`, `RwLock[T]`, `Atomic[T]` | the owner's | synchronised | with the owner | changes shared across threads |

* `[SEL-1]` *(changed in 0.9.10)* *(changed in 0.9.15)* The order above is the order to try. `ember inspect --alloc`
  reports the mechanism each declaration uses: every local, parameter and field the program writes,
  by the row its declared type picks, and every struct, enum and class it declares, by its kind. A
  type the table has no row for is reported as what it is: a number, `bool`, `char`, `void` or a
  range type as a `value`; a view (`ref T`, `ref mut T`, a span, `StrView`) or a guard as a `view` of
  another's storage; a function value as a `callable`; a raw pointer as a `raw pointer`. Tuples and
  fixed arrays are the first row's (ODR-108).
* `[SEL-2]` *(changed in 0.9.15)* `shares` and `weakref` and C++'s `std::shared_ptr`/`std::weak_ptr`
  (`CppShared`, `CppWeak`) never convert into each other: they use different counts, and conflating
  them frees twice (`E5065`, `[WK-14]`).

## IX.1 The heap types

* `[HEAP-1]` *(changed in 0.9.15)* Every heap type allocates through `ember_alloc`/`ember_realloc`/`ember_free`, or
  through the memory manager its type names with `using` (`[ALC-1]`), and carries the `Alloc` effect
  where it allocates.
* `[HEAP-2]` Growable buffers double, from a minimum of four elements; `shrink_to_fit` is explicit.
* `[HEAP-8]` *(new in 0.9.9)* Capacity arithmetic is checked: a requested capacity whose byte size
  overflows `usize`, or exceeds the allocator's limit, panics with `capacity overflow` before any
  allocation. No length or capacity computation in the runtime wraps.
* `[HEAP-9]` *(new in 0.9.9)* *(changed in 0.9.15)* Heap storage for elements of type `T` is aligned to at least
  `align_of[T]()`, whatever that alignment is.
* `owns T`: one owner, in a heap block of its own; it reads, and is changed, as its `T`, with nothing
  written in between. `owns dyn I` holds any implementer of `I` (`[IFC-5]`). The line that makes a value
  for an `owns` place says `owns` before it (`mesh=owns Mesh(points=[0.0])`,
  `left=Some(owns Tree(value=2))`; `E2123` otherwise, `[OWN-10]`), and `owns` on a class type is
  `E2114`, since a class object always has a block of its own. It is not `Copy`, and it is `Clone` when
  `T` is: `copy of` it makes a new block holding a copy of the value, so a struct or enum with an `owns`
  field derives `Clone` (`[STR-5]`). `owns dyn I` is `Clone` when `I` requires it
  (`interface I: Clone:`), copying through the entry `I`'s method table holds for it (`[TYP-22]`). A
  foreign type held `owns` is ended through its contract's destructor (Part XVI).
* `[HEAP-3]` *(changed in 0.9.15)* A value with several owners (`shares`, `shares mut`,
  `thread_shared`) lives in one heap block that holds its owner count, and, where `weakref` links may
  reach it, its weak count (`[OBJ-1]`). A class object already has its block, so another owner only
  adds one to its count; any other value is put in a block where it is made shared
  (`count = shares mut Counter()`, `other = shares items`), one allocation. Each added owner is first
  compared with the count's maximum (`[RC-1]`).
* `[HEAP-6]` *(changed in 0.9.15)* `shares T`, `shares mut T`, `thread_shared T` and `weakref T` are not
  `Copy`: another owner is made only by a line that says so (`shares x`, `shares mut x`,
  `thread_shared x`), and another link only by `weakref x`, each adding one to a count; `=` from a whole
  name hands the owner or the link over, and its end subtracts one.
* `[HEAP-7]` *(changed in 0.9.9)* *(changed in 0.9.15)* `weakref T` links to a value that lives on its
  own: a class object, or a value held `shares` or `thread_shared`. A `weakref` of a part (a field, an
  element stored inline) is `E2119` (`[WK-16]`).
* `[HEAP-10]` *(new in 0.9.9)* *(changed in 0.9.15)* A `shares` owner, and a `weakref` link to a value
  held `shares`, never reach another thread: their counts are plain memory (`E7015`). Shared ownership
  across threads is `thread_shared T`, whose counts are kept for several threads (its making thread's
  owners with plain instructions, the other threads' owners and the weak count with atomic ones,
  `[RC-4]`); no owner changes a `thread_shared` value directly, only through an `Atomic`, a `Mutex`,
  an `RwLock`, a channel or a `guarded by` field under its lock (`E7014`, Part XI).

## IX.2 Arenas

```ember
@derive(Zeroable)
struct Cmd:
    id: int
    cost: f32

fn build(ref mut frame: Arena, n: int) -> MutSpan[Cmd]:
    cmds = ref mut frame.alloc_zeroed[Cmd](n)   # new memory in `frame`, which `cmds` may write
    for i in 0..n:
        cmds[i].id = i
    return cmds

fn main():
    frame = Arena.with_capacity(64 * 1024)
    cmds = build(ref mut frame, 8)
    println(cmds.len())
    frame.reset()                               # allowed: `cmds` is no longer used
```

* `[ARN-1]` *(changed in 0.9.15)* `Arena` is a struct, and not `Copy`. Its `alloc*` methods move the
  arena's cursor, so they are called through a name that may change the arena (its owner, or a
  `ref mut` name or parameter: `fn build(ref mut frame: Arena, n: int)`). Each gives a view of memory
  that no earlier view of the arena reaches, and the views point into the arena's memory, not into its
  cursor: an `alloc*` call ends no earlier view and overlaps none, so many allocations are outstanding at
  once, each a place of its own that a `ref mut` name may write. The views live no longer than the arena
  (`[LT-1]`, `[LT-44]`). `reset()`, `scope()` (`[ARN-6]`) and the arena's end may reuse or free that
  memory: each ends every view into the arena, and a later use of one is `E3060`.
* `[ARN-2]` *(changed in 0.9.15)* Values placed with `alloc*` are never dropped one by one, so an
  `alloc*` of a type that needs drop is `E3090`, whose help gives the line with `using`. `Arena` and
  `FixedArena` implement `Allocator` (`[ALC-2]`): a value that needs drop is made in an arena as an
  `owns T using frame`, an `Array[T] using frame` or a class object `using frame` (`[ALC-1]`), whose end
  runs its `drop` and gives its block back to the arena, which frees nothing one by one. There is no
  `alloc_nodrop`.
* `[ARN-3]` *(changed in 0.9.15)* `alloc(owns v: T) -> ref mut T` hands `v` over into the arena. `alloc_array[T](n) -> MutSpan[T]` requires `T` not to
  need drop and `T: Default`, and initialises each element with `T.default()` in index order (`E2040`
  without `Default`). `alloc_zeroed[T](n) -> MutSpan[T]` requires `T` not to need drop and
  `T: Zeroable`, and fills with zero bytes. Neither stands in for the other: a `Zeroable` type's
  `alloc_array` still calls its `default`, and an implementation fills with zeros instead only where
  that is the same (a scalar's standard `Default`) (ODR-064). `alloc_uninit[T](n) ->
  MutSpan[MaybeUninit[T]]` initialises nothing.
* `[ARN-4]` *(changed in 0.9.9)* *(changed in 0.9.15)* A growing `Arena` carries `Alloc` on its growth path. `FixedArena`
  (made by `Arena.fixed(ref mut buffer)` from a `MutSpan[u8]`, which it then holds as a kept `ref mut`
  view, `[LT-45]`, or by `FixedArena.with_capacity(n)` at start-up) never grows, carries no `Alloc` on
  allocation, and panics on exhaustion (`try_alloc` returns `Option`). `FixedArena` is the arena for
  `@noalloc` code.
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
* `[ARN-6]` *(changed in 0.9.15)* `arena.scope() -> ScopedArena` is called through a name that may change the arena, and the
  `ScopedArena` holds a `ref mut` view of it for the scope's life (`[LT-46]`); `with s = frame.scope():`
  allocates from `s` and releases everything at the block's end, last in first out. Using the parent
  while a scope is live is `E3096`, whose help is to allocate from the scope. Scopes nest: `s.scope()`
  opens one inside another.
* `[ARN-7]` *(changed in 0.9.15)* No operation lowers an arena's allocation pointer or reuses its bytes
  while a view into it is used: every such operation ends every view into the arena (`[ARN-1]`), and
  while a value made `using` the arena lives, the arena is changed only through its `Allocator` methods
  (`[ALC-5]`).
* `[ARN-8]` *(changed in 0.9.15)* `MaybeUninit[T]` has the size and alignment of `T` and does not claim
  to hold a valid `T`; ending it never ends a `T`. `MaybeUninit[T].uninit()` makes one;
  `write(owns v: T) -> ref mut T` stores without ending the bytes there before;
  `unsafe fn assume_init() owns this -> T` asserts initialisation. `MutSpan[MaybeUninit[T]]` has
  `write_at(i, owns v: T)` and `unsafe fn assume_init() owns this -> MutSpan[T]`. There is no other
  conversion. A `MaybeUninit` never runs its value's clean-up, so for a `T` that needs drop, `uninit()`,
  `write` and `write_at` are `unsafe` operations (`[UNS-1]`), and outside `unsafe:` each is `E3100`
  ("`MaybeUninit` never runs a `Player`'s clean-up, so a value written into it may never be freed;
  write it inside `unsafe:`, or keep the value in an `Option[shares Player]`"); for a `T` with nothing
  to drop they stay safe.
* `[ARN-10]` A panic inside `T.default()` during `alloc_array` aborts (`[PAN-1]`); no partially
  initialised span becomes reachable.
* `[ARN-11]` `Zeroable` is an `unsafe` marker: all-zero bytes are a valid `T`. The scalars, raw
  pointers, a range including zero, and tuples and fixed arrays of `Zeroable` elements are; a struct
  is when it declares `@derive(Zeroable)`, which the compiler proves field by field (`E2080`), since
  only its author can vouch for its invariants (a `NonZero`'s field is an integer). A `ref`, a range
  excluding zero, or an enum whose zero discriminant is invalid is never `Zeroable`. `T: Zeroable` is
  a bound a generic function may state (`alloc_zeroed`, `mem.zeroed`). Being `Zeroable` never changes
  what `alloc_array` does (`[ARN-3]`).

## IX.3 Memory managers: `using` and `Allocator`

```ember
struct Mesh:
    points: Array[float]

struct Enemy:
    hp: int
    mesh: owns Mesh using Arena

fn main():
    level = Arena.with_capacity(64 * 1024)
    boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using level)
    println(boss.mesh.points.len())
```

* `[ALC-1]` *(changed in 0.9.15)* **The place names the kind, the making line names the instance.**
  `using A` after a type whose value has heap memory of its own (an `owns`, `shares`, `shares mut` or
  `thread_shared` value, a class object, an `Array`, `Map` or `Set`) names the kind of memory manager
  that memory comes from, a type implementing `Allocator`: `mesh: owns Mesh using LevelArena`,
  `names: Array[String] using LevelArena`, `Map[K, V, H] using A`, `Set[T, H] using A`. Without `using`,
  it comes from the built-in manager, `Global`. The line that makes such a value names the instance
  after the value: `boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using current)` takes the
  mesh's block from the arena `current` by calling `current.allocate(…)`; a class object is made
  `Enemy(hp=5) using current`, a list `[0.0, 1.0] using current`, and `Array.new_in(current)` makes an
  empty one. A manager type with no fields has one instance (`Global`, `System`, and any such type a
  program declares): it needs none on the making line, and its values keep no instance word; every
  other manager type may have several instances. Checked
  when compiling: the instance is of the kind the place names, and a value given to a `using A` place
  was made in an instance of `A` (`E2020`: "`mesh` takes its memory from a `LevelArena`
  (`mesh: owns Mesh using LevelArena`), and this `Mesh` comes from the built-in manager; say which
  arena: `mesh=owns Mesh(points=[0.0, 1.0]) using current`"); a value made in an instance, and every
  value that holds one, lives no longer than that instance (`E3063`, `[LT-4]`); `using` after any other
  type is `E2114` ("an `int` has no memory of its own to take from `LevelArena`; remove `using`").
  `using` names where that value's own memory comes from; what the value holds comes from where its own
  fields' types say (the `points` list of that `Mesh` from the built-in manager). A value whose manager
  may have several instances keeps its instance's address, one word, so that its growth and its end go
  back to the instance it came from (`[LT-4]`). An allocator type parameter is not Ember: `Box[T, A]` and
  `Array[T, A]` are `E1010`, whose help gives `owns T using A` and `Array[T] using A`.
* `[ALC-2]` *(changed in 0.9.10)* *(changed in 0.9.15)* `std.mem` declares `pub unsafe interface Allocator` with
  `fn allocate(layout: Layout) -> *mut u8` (null on failure),
  `unsafe fn deallocate(p: *mut u8, layout: Layout)` and
  `unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8`.
  Giving memory back, or moving it, is unsafe, since Ember may still use it and a wrong `Layout` is
  undefined in C: a program's own call is made inside `unsafe:` (`E3100`: "`deallocate` gives memory
  back that Ember may still use, and a wrong `Layout` is undefined in C; write it inside `unsafe:` with a
  `# SAFETY:` note"), and the calls the compiler makes for `using` values are its own. `Layout` holds
  `size: usize` and `align: usize`, both `pub(read)`; its constructor `Layout(size, align)` checks that
  `align` is a power of two and that `size` rounded up to a multiple of `align` fits in an `isize`, and
  panics (`invalid Layout`) otherwise, so no other `Layout` exists. `Global` is the package's configured
  allocator (the one `[ALC-3]` names, else the platform's), so an `Array[T]` never bypasses it; `System`
  is the platform's, which a global allocator may wrap (a counting or tracing allocator delegates to
  `System()`). An allocator that delegates to `Global` from inside the global allocator recurses.
  Implementing `Allocator` is `unsafe` (`unsafe extend A implements Allocator:`, `[TYP-43]`): the
  implementer promises valid, aligned, unaliased memory, which stays valid until it is given back or the
  instance ends.
* `[ALC-3]` *(changed in 0.9.9)* A binary package may replace the global allocator with
  `[build] global_allocator = "module.NAME"` naming a `static` whose type implements `Allocator`; an
  embedding host may pass allocation functions to `ember_rt_init` (`[FFI-27]`).
* `[ALC-4]` Allocation failure in a standard container panics (`out of memory`). `try_reserve` and
  `try_push` report it as a value.
* `[ALC-5]` *(new in 0.9.15)* **An instance stays as its values need it.** While a value made in a memory
  manager's instance lives, the instance is changed only by what allocates from it and gives blocks back
  to it: the `Allocator` methods that value's growth and end call, the making of other values in it, and
  an `Arena`'s `alloc*` (`[ARN-1]`), none of which touches a block already given out. Reading it stays
  allowed (`current.made`); any other change, through any name, is
  `E3021` ("`frame.reset()` (line 26) changes the arena `names` was made in (line 25), and `names` still
  lives; end it first: write `mem.drop(names)` above line 26"). So no safe method of a memory manager
  rewinds it, frees it or gives its bytes out again under a value made in it.

## IX.4 Raw pointers and `unsafe`

```ember
@safety("`p` points to four readable bytes")
unsafe fn read_u32_le(p: *u8) -> u32:
    # SAFETY: the caller makes `p` point to four readable bytes, as this function's @safety says
    unsafe:
        return (p.read() as u32) | ((p.add(1).read() as u32) << 8) | ((p.add(2).read() as u32) << 16) | ((p.add(3).read() as u32) << 24)

fn parse(data: Span[u8]) -> Option[u32]:
    if data.len() < 4:
        return None
    # SAFETY: the length check above guarantees four readable bytes
    unsafe:
        return Some(read_u32_le(data.as_ptr()))
```

* `[UNS-1]` *(changed in 0.9.15)* An `unsafe` context is required to: dereference, read or write
  through `*T`/`*mut T`; offset a pointer; cast between pointers or pointers and integers; call an
  `unsafe fn` (`Allocator.deallocate` and `reallocate` among them, `[ALC-2]`), or through a value of an
  `unsafe` callable type (`[FN-6]`); call an `extern` function without a verified contract (Part XVI);
  access a foreign `static mut` (`[STA-1]`); `transmute`; call `get_unchecked`, `assume_init` or
  `assume_disjoint`; give an object to C with `into_raw()` (`[OWN-12]`); make or write a `MaybeUninit`
  of a type that needs drop (`[ARN-8]`); implement an `unsafe` interface, or declare `Send` by hand
  (`unsafe extend T implements Send:`, `[TYP-43]`). Each of these outside an `unsafe` block is `E3100`,
  whose message says what the code must vouch for and gives the block to write ("`read_u32_le(buf.as_ptr())`
  (line 9) calls an `unsafe fn`, whose callers must make true what its `@safety` says; put line 9 in an
  `unsafe:` block with a `# SAFETY:` note saying why that holds here: above line 9 write `# SAFETY:` and
  the reason, then `unsafe:`, and indent line 9 under it").
* `[UNS-2]` *(changed in 0.9.15)* `unsafe` permits exactly those operations. It does not turn off the checks of Parts VII
  and VIII, bounds checks on safe types, overflow checks or type checking.
* `[UNS-3]` `L3010` reports an `unsafe` block containing statements that need no `unsafe`.
* `[UNS-4]` *(changed in 0.9.15)* Unsafe code MUST uphold what safe code assumes: every view is
  non-null, aligned, points to a live initialised value of its type, and is used as its word allows (a
  `ref mut` name is the only name of its place while it is used, a `ref mut` parameter's place is
  reached by no other name, and what a read-only view reads does not change while it is used,
  `[BRW-1]`, `[BRW-22]`); every view's length is within its allocation; every class object has a
  correct header, and every owner and every `weakref` link of it is counted there, so unsafe code never
  makes or ends an owner or a link without its count (`[OBJ-1]`); every `StrView` is UTF-8; every value
  is valid for its type (`bool`, `char`, range types, enums); `Send` and `ThreadShared` are respected. A
  raw pointer made from a view is not used after the view's last use, nor after what it points into is
  reshaped or ended.
* `[UNS-5]` `std.mem` provides `Volatile[*T]` (volatile reads and writes), `transmute[A, B]`,
  `copy_nonoverlapping`, `zeroed[T]()` (requires
  `T: Zeroable`) and `MaybeUninit`.
* `[UNS-6]` Inline assembly is `unsafe asm("…", …)` on toolchains that support it and `E5090`
  otherwise.
* `[UNS-7]` *(changed in 0.9.15)* Every `unsafe fn` carries `@safety("…")` stating what its callers must
  make true; without it, `L3015` ("`read_u32_le` is an `unsafe fn` that reads through the raw pointer
  `p`, and nothing says what its callers must make true for that; say it above the function:
  `@safety("...")`, with what `p` must point to"). It is the declaration's only note: a `# SAFETY:` note
  on an `unsafe fn` line is `W3013` ("`read_u32_le` says what its callers must make true with
  `@safety`; a `# SAFETY:` note says why one block is sound and goes above each `unsafe:` block inside
  it: remove this line"). The standard library carries `@safety` on every unsafe function it exports.
* `[UNS-8]` *(changed in 0.9.9)* *(changed in 0.9.15)* Every `unsafe:` block, and every `unsafe`
  implementation (`unsafe extend`, `unsafe extern`, `unsafe overlay`), carries a safety note
  (`[LEX-23]`) saying why the code is sound: `# SAFETY: …` or `# SAFETY(category): …` on the line
  before it or at the end of its first line. Without one, `W3012`. `ember tcb` lists every block with
  its note, its category and the obligations of the unsafe functions it calls.
* `[UNS-10]` *(changed in 0.9.15)* `UnsafeCell[T]` (in `std.mem`) is the primitive beneath `Cell`,
  `Mutex`, `RwLock`, `Atomic` and `Once`: `UnsafeCell(v)`, `unsafe fn get() -> *mut T`,
  `fn into_inner() owns this -> T`. It hands out no safe view, checks nothing, is never `Copy` and never
  `ThreadShared` (it is `Send` when `T` is), and is refused in `@no_alias_checks` code (`E3105`).
  Diagnostics never suggest it.
* `[UNS-10a]` *(changed in 0.9.15)* `UnsafeCell` suspends no rule globally: the checks of Parts VII and VIII, region, type
  and bounds checking all still apply around it. Unsafe code may break the aliasing rules inside an
  abstraction built on it, and must never let a conflicting view escape into Safe Ember.
* `[UNS-11]` *(new in 0.9.15)* **Only inside `unsafe`.** Anything that can break a safety promise
  (misuse memory, race, leak, skip a promised check, or reach memory the rules cannot see) is possible
  only inside an `unsafe` block, and no switch turns a safety check off, in any build or profile:
  `@no_runtime_checks` is not Ember (`E0104`: "there is no switch that turns safety checks off, in any
  build; to forbid run-time alias checks here write `@no_alias_checks`; to skip one bounds check you
  have proved, write `xs.get_unchecked(i)` inside `unsafe:` with a `# SAFETY:` note"). `unsafe` is one
  word with one meaning by position: before a declaration (`unsafe fn`, `unsafe interface`, an
  `unsafe fn(…)` type) it says that whoever uses it must vouch, with `@safety` saying what they must make
  true (`[UNS-7]`); before a block or an implementation (`unsafe:`, `unsafe extend`, `unsafe extern`,
  `unsafe overlay`) it says that this code vouches, with `# SAFETY:` saying why (`[UNS-8]`).
* `[UNS-12]` *(new in 0.9.15)* An `unsafe fn` body is not itself an `unsafe` context: each unsafe
  operation in it sits in an `unsafe:` block of its own, with its note, and one outside such a block is
  `E3100` ("`read_u32_le` is an `unsafe fn`, which says its callers must vouch; the reads through `p` on
  line 4 are this function's own unsafe steps, and nothing says why they are sound; put line 4 in an
  `unsafe:` block with a `# SAFETY:` note saying why the read is safe: above line 4 write `# SAFETY:` and
  the reason, then `unsafe:`, and indent line 4 under it").

## IX.5 Layout

* `[LAY-2]` *(new in 0.9.9)* Layout attributes change layout exactly as stated, in every profile and on
  every backend, and are never ignored (`[ATT-6]`):
  * `@layout(c)` — the target C ABI layout (the default for every struct, `[TYP-11]`);
  * `@packed` — no padding; unaligned fields are read and written by byte copies; taking a reference
    to one is `E2170`;
  * `@align(N)` — raises the type's alignment to `N` (a power of two up to 4096); `size_of` becomes a
    multiple of `N`, and every place and heap buffer holding the type honours it (`[HEAP-9]`);
  * `@repr(u8|u16|u32|u64|i8|i16|i32|i64)` — an enum's discriminant type; it is not written on a
    struct (`E0104`, ODR-139).

  `@packed` and `@align(N)` are not written on one type, and a `@packed` type holds no field whose
  type states an alignment with `@align`: no padding and a size that is a multiple of `N` cannot
  both hold, so either is `E0104` (ODR-138). A SIMD vector states its alignment too (it is aligned
  to its size, `[SIMD-1]`), so a `@packed` type holding one is `E0104` as well (ODR-157). `@align(N)`
  on an enum without payloads raises its alignment and rounds its size up like any other type's.
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
* `[HND-3]` *(new in 0.9.9)* *(changed in 0.9.15)* A slot whose generation is exhausted is retired and never reused, so an old handle can never
  become valid again. A retired slot's memory stays with the pool until the pool ends: a pool keeps one
  retired slot for each slot reused as many times as its generation can count (with `Split20`, one for
  every 4,096 reuses of a slot), a bounded cost that keeps that promise. When every slot is retired or
  live, `insert` panics and `try_insert` returns `Err(CapacityError.Full)`.

## IX.7 `Cell`, and changes checked when compiling

```ember
struct Sprite:
    frame: Cell[int]

fn advance(ref mut s: Sprite):          # setting a `Cell` needs a name that may change its holder
    s.frame.set(s.frame.get() + 1)

struct Scene:
    entities: Array[int] = []

    fn add(e: int):
        this.entities.push(e)

fn main():
    s = Sprite(frame=Cell(0))
    advance(ref mut s)
    scene = Scene()
    scene.add(7)
    println(s.frame.get(), scene.entities.len())
```

* `[CELL-1]` *(changed in 0.9.15)* `Cell[T]` holds a `T` that is replaced whole: `Cell(v)`, `set(v)`,
  `replace(v) -> T`, `take() -> T` (for `T: Default`), `into_inner()`, `get() -> T` for `T: Copy`, and
  `update(f)` for `T: Copy` or `T: Default`. Setting it is a change of the place that holds it, made
  through a name that may change that place: a `ref mut` name, `this` in a method called through such a
  name, or an owner of a `shares mut` value (through a plain name, `E3023`, `[BRW-15]`). `set` and
  `replace` store the new value before dropping the old one, since the old value's `drop` may read the
  cell.
* `[CELL-2]` *(changed in 0.9.15)* `Cell` never hands out a view of its contents, so it needs no run-time check; `get` is
  a load and `set` a store, with the count changes of any owner or link in the value (`[HEAP-6]`).
* `[CELL-3]` *(changed in 0.9.15)* `Cell[T]` is not `ThreadShared`; it is `Send` if `T` is.
* `[CELL-4]` `Cell[T]` is `Copy` when `T` is.
* `[CELL-13]` *(new in 0.9.15)* **No change is checked while the program runs.** Every change is made
  through a name that may make it, and the checks of Parts VII and VIII are made when compiling: no value
  carries a borrow flag or an access word. A value several owners change on one thread is held
  `shares mut` (`[RC-7]`); a value threads share and change is `thread_shared`, with an `Atomic`, a
  `Mutex`, an `RwLock` or `guarded by` fields (Part XI); a field a method changes is changed through
  `this` (`[EXC-15]`). `RefCell`, `Ref`, `RefMut`, `borrow` and `borrow_mut` are not Ember (`E1010`:
  "`RefCell` is not part of Ember; write the field plainly, `entities: Array[int]`, and change it in a
  method: `fn add(e: int):` with the body `this.entities.push(e)`; for a scene several objects own and
  change, make it once, `s = Scene(entities=[])`, write the field that keeps it in each owner as
  `scene: shares mut Scene`, and give each owner its share where it is made, for example
  `Panel(scene=shares mut s)`").

## IX.8 Establishing disjointness

When two views come from sources the compiler cannot tell apart (two values of one map, two lists
reached through links), and the structural ways (`split_at`, `chunks_mut`, `get_pair_mut`,
`if i != j:`, SoA columns) do not apply:

```ember
from std.mem import assert_disjoint, check_disjoint

fn blend(ref mut dst: MutSpan[float], src: Span[float]):
    for i in 0..dst.len():
        dst[i] = dst[i] * 0.5 + src[i] * 0.5

fn mix(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    assert_disjoint(d, s)          # stops the program if the two names are one track
    blend(ref mut d, s)

fn mix_any(ref mut tracks: Map[String, Array[float]], dst_name: StrView, src_name: StrView):
    d = ref mut tracks[dst_name].as_mut_span()
    s = tracks[src_name].as_span()
    match check_disjoint(d, s):
        Ok(_):
            blend(ref mut d, s)
        Err(_):
            pass                   # one track: blending it with itself leaves it as it is
```

* `[DSJ-1]` *(changed in 0.9.9)* *(changed in 0.9.15)* `check_disjoint(a, b) -> Result[void, void]`
  compares the two views' address ranges, and gives `Ok(())` when they do not overlap and `Err(())`
  when they do: two comparisons. It reads both views and takes neither (`[FN-1]`), so they stay usable
  after it. Its result is `@must_use`: an unused one is `W2190` ("to stop when they overlap, write
  `assert_disjoint(d, s)`, with `from std.mem import assert_disjoint` at the top; to handle both cases,
  write `match check_disjoint(d, s):`"). In its `Ok` branch the two names are known not to overlap, as
  inside `if i != j:`.
* `[DSJ-2]` *(changed in 0.9.15)* The fact belongs to the two names, not to a program point: from
  `assert_disjoint`, after the call; from `check_disjoint`, inside its `Ok` branch. Giving either name
  another value (`d = ref mut other.as_mut_span()`) drops it.
* `[DSJ-3]` *(changed in 0.9.15)* Where the fact holds, the two names are treated as not overlapping,
  so they may be given to one call where one of them is changeable (`[BRW-11]`), and the backend marks
  them `restrict`/`noalias` (`[SIMD-3]`).
* `[DSJ-4]` It applies to `Span`, `MutSpan`, `SoA` columns and arena views; two single-object
  references are `E3095`.
* `[DSJ-5]` *(changed in 0.9.15)* `assert_disjoint(a, b)` makes the same comparison and stops the
  program if the views overlap, saying where and which names: "`assert_disjoint(d, s)` on line 11: the
  two spans overlap (`dst_name` and `src_name` name one track)". Both forms record a
  `RuntimeCheck(Aliasing)` site whose reason is `establishes_static_fact` (`[EFF-11]`), unless the
  compiler already knows the ranges are disjoint, in which case no comparison is emitted.
* `[DSJ-6]` *(changed in 0.9.15)* `check_disjoint` and `assert_disjoint` are usable in `@noalloc` and
  `@nosync` code. They are the one run-time alias check there is, so inside `@no_alias_checks` each is
  `E4030` ("`mix` is `@no_alias_checks`, and `assert_disjoint(d, s)` (line 18) compares the two spans
  while running; blend from a copy, which needs no check: instead of lines 16 to 19 write
  `old = copy of tracks[src_name]` and `blend(ref mut tracks[dst_name].as_mut_span(), old.as_span())`,
  or remove `@no_alias_checks`", `[EFF-12]`).
* `[DSJ-7]` *(changed in 0.9.15)* `assume_disjoint(a, b)`, an `unsafe fn`, gives the two names the fact
  without checking; violating it is undefined behaviour.
* `[DSJ-8]` There is no form that checks in one profile and assumes in another (`[PHIL-13]`).
* `[DSJ-9]` *(changed in 0.9.15)* `check_disjoint_all(v1, …, vn)` and `assert_disjoint_all(v1, …, vn)`
  handle 2 to 8 views pairwise, as `check_disjoint` and `assert_disjoint` do.
* `[DSJ-10]` *(new in 0.9.15)* Making a view and comparing two address ranges read no element: in `mix`
  above, `s = tracks[src_name].as_span()` and `assert_disjoint(d, s)` are allowed while `d`, a `ref mut`
  name of a list's elements, is in use, and the first element read through `s`, in `blend`, comes after
  the two are proved apart. A read of an element through `s` before the proof is `E3021` (`[BRW-1]`).
