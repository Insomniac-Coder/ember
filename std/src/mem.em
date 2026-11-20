## `std.mem` — low-level memory facilities.
##
## `[UNS-10]` gives `UnsafeCell[T]` this public module identity. Its private
## one-field representation and its two operations are compiler-known during
## Phase 2 so no source code can accidentally expose safe access to `value`.

[T]
pub struct UnsafeCell:
    value: T

## `[UNS-5]`, `[FFI-8]` — a pointer whose reads and writes are volatile: each one happens, in the
## order written, never merged, moved or removed (a device register, memory another agent
## changes). C's `volatile T*` is a `Volatile[*mut T]` (`Volatile[*T]` for `const volatile T*`); to C
## it is the pointer.
@derive(Copy)
[P]
pub struct Volatile:
    pointer: P

    pub static fn new(pointer: P) -> Volatile[P]:
        return Volatile(pointer)

    ## The pointer itself.
    pub fn as_ptr() -> P:
        return this.pointer

extend[T] Volatile[*mut T]:
    ## A volatile read of the `T` the pointer points to.
    @safety("the pointer is valid for a read of a `T`")
    pub unsafe fn read_volatile() -> T:
        return volatile_read(this.pointer)

    ## A volatile write of `value` where the pointer points.
    @safety("the pointer is valid for a write of a `T`")
    pub unsafe fn write_volatile(value: T):
        volatile_write(this.pointer, value)

extend[T] Volatile[*T]:
    ## A volatile read of the `T` the pointer points to.
    @safety("the pointer is valid for a read of a `T`")
    pub unsafe fn read_volatile() -> T:
        return volatile_read(this.pointer)

## `[UNS-5]` — compiler-known: the volatile read and write a `Volatile` makes.
[T]
fn volatile_read(pointer: *T) -> T:
    pass

[T]
fn volatile_write(pointer: *mut T, value: T):
    pass

## `[UNS-5]` — `count` values of `T` from `src` to `dst`, read and written through the raw pointers
## bit for bit, as C's `memmove` copies: the two ranges may overlap. Nothing is dropped, so a value
## that needs drop is afterwards in both places.
@safety("`src` is valid for reading `count` values of `T` and `dst` for writing them, and a value that needs drop is ended once, from one of the two places")
[T]
pub unsafe fn copy(src: *T, dst: *mut T, count: usize):
    bytes = count * size_of[T]()
    from = src as *u8
    to = dst as *mut u8
    if (to as usize) <= (from as usize):
        i: usize = 0
        while i < bytes:
            to.add(i).write(from.add(i).read())
            i += 1
    else:
        i = bytes
        while i > 0:
            i -= 1
            to.add(i).write(from.add(i).read())

## `[UNS-5]` — `copy`, for ranges that do not overlap, as C's `memcpy` copies.
@safety("`src` is valid for reading `count` values of `T` and `dst` for writing them, the two ranges do not overlap, and a value that needs drop is ended once, from one of the two places")
[T]
pub unsafe fn copy_nonoverlapping(src: *T, dst: *mut T, count: usize):
    bytes = count * size_of[T]()
    from = src as *u8
    to = dst as *mut u8
    i: usize = 0
    while i < bytes:
        to.add(i).write(from.add(i).read())
        i += 1

## `[UNS-5]` — `value`'s bits as a `B`. `value` is handed over and not dropped: the `B` given back
## holds what it held.
@safety("`A` and `B` have the same size, and `value`'s bits are a valid `B`")
[A, B]
pub unsafe fn transmute(owns value: A) -> B:
    from = MaybeUninit[A].uninit()
    from.write(owns value)
    to = MaybeUninit[B].uninit()
    copy_nonoverlapping(ref_to_ptr(ref from) as *u8, ref_to_ptr(ref mut to) as *mut u8, size_of[B]())
    return to.assume_init()

## `[UNS-5]`, `[ARN-11]` — a `T` whose bytes are all zero, which is a valid `T` for every `Zeroable`
## type (`@derive(Zeroable)`, a number, a `bool`), so the call is safe.
[T: needs Zeroable]
pub fn zeroed() -> T:
    i: usize = 0
    # SAFETY: `T` is `Zeroable`, so a `T` whose bytes are all zero is valid, and every byte is written
    unsafe:
        slot = MaybeUninit[T].uninit()
        to = ref_to_ptr(ref mut slot) as *mut u8
        while i < size_of[T]():
            to.add(i).write(0)
            i += 1
        return slot.assume_init()

## `[OWN-6]`, `[DRP-1]` — `mem.drop(owns x)` ends `x` now: the name given to it ends, for every
## type, numbers included, as for every `owns` parameter, and a later use is `E3040`. Handed to
## this frame, the value is dropped at its end, once.
[T]
pub fn drop(owns x: T):
    pass

## `[RC-3]` — an explicit use at this point. The plain parameter only reads: it neither hands
## the value over nor keeps it past the call; it lets code using a raw pointer state where its
## owner must still be live.
[T]
pub fn keep_alive(value: T):
    pass

## `[OWN-6]` — `take`, `replace` and `swap` take values out of places a hand-over may not
## otherwise leave empty; a call writes `ref mut` before each place (`mem.take(ref mut
## this.items)`, `[FN-2a]`). They are compiler-known while the standard library is staged; calls
## through this public module still use ordinary generic inference, place checks, borrows and
## hand-overs. There is no `mem.forget`: a value always ends with its clean-up (`E1010`); C keeps
## one handed to a parameter its declaration says it takes, or given with `into_raw()` inside
## `unsafe:` (`[OWN-12]`).
[T]
pub fn replace(ref mut place: T, owns new: T) -> T:
    pass

[T: needs Default]
pub fn take(ref mut place: T) -> T:
    pass

[T]
pub fn swap(ref mut a: T, ref mut b: T):
    pass

## `[DSJ-1]` — two views from sources the program knows do not overlap: `Ok`
## when their address ranges are apart, `Err` with both unchanged when they
## overlap. Compiler-known; the operands must be views (`E3095` otherwise).
[A, B]
pub fn assert_disjoint(owns a: A, owns b: B) -> Result[(A, B), (A, B)]:
    pass

## `[DSJ-9]` — two to eight views compared pairwise; compiler-known, so the
## declaration shows two.
[A, B]
pub fn assert_disjoint_all(owns a: A, owns b: B) -> Result[(A, B), (A, B)]:
    pass

## `[DSJ-5]` — `assert_disjoint`, panicking when the views overlap.
[A, B]
pub fn assert_disjoint_or_panic(owns a: A, owns b: B) -> (A, B):
    pass

## `[DSJ-7]` — the views do not overlap, unchecked: violating it is undefined
## behaviour.
@safety("The two views' address ranges do not overlap.")
[A, B]
pub unsafe fn assume_disjoint(owns a: A, owns b: B) -> (A, B):
    pass

## Part IX §5 — target-layout queries. These declarations make the public
## `std.mem` paths real; the compiler folds them through its canonical layout
## descriptor rather than executing these staging bodies.
[T]
pub fn size_of() -> usize:
    return 0

[T]
pub fn align_of() -> usize:
    return 0

## `[ALC-2]` — the size and the alignment of a block of memory an `Allocator` gives. Both are read
## anywhere and set only here: `Layout(size, align)` checks that `align` is a power of two and that
## `size` rounded up to a multiple of `align` fits in an `isize`, and panics (`invalid Layout`)
## otherwise, so no other `Layout` exists.
pub struct Layout:
    pub(read) size: usize
    pub(read) align: usize

    pub fn init(size: usize, align: usize):
        if align == 0 or (align & (align - 1)) != 0:
            panic("`invalid Layout`: an alignment that is not a power of two")
        if size > (isize.MAX as usize) - (align - 1):
            panic("`invalid Layout`: a size that rounded up to a multiple of its alignment does not fit in an `isize`")
        this.size = size
        this.align = align

## `[ALC-2]` (ODR-175) — what gives a program its heap memory. Implementing it is `unsafe`
## (`unsafe extend T implements Allocator:`): the implementer promises that `allocate` gives memory
## valid for `layout.size` bytes, aligned to `layout.align` and reached through nothing else, or
## null when it has none; that `deallocate` is given back only what `allocate` or `reallocate` gave,
## with the layout it was asked for; and that `reallocate` gives a block of `new_size` bytes holding
## the first of `p`'s (`p` then given back), or null, leaving `p` as it was. A binary package may
## make one the global allocator, which every heap type allocates through (`[ALC-3]`). Giving memory
## back, or moving it, is `unsafe`: Ember may still use it, and a wrong `Layout` is undefined in C, so
## a program's own call of `deallocate` or `reallocate` is made inside `unsafe:` (`E3100`); the
## calls the compiler makes for a value made `using` a manager are its own (`[ALC-1]`).
pub unsafe interface Allocator:
    fn allocate(layout: Layout) -> *mut u8
    @safety("`p` came from this manager's `allocate` or `reallocate` with `layout`, and nothing uses it after")
    unsafe fn deallocate(p: *mut u8, layout: Layout)
    @safety("`p` came from this manager's `allocate` or `reallocate` with `old`, and nothing uses it after; only the result is used")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8

## `[ALC-2]` (verdict 107) — the size and alignment of a piece of a manager that implements
## `FoundByAddress`.
pub const PIECE: usize = 65536

## `[ALC-2]` (verdict 107) — a manager whose values' instance is found from their address, as C's
## `free(p)` finds its heap. Implementing it is `unsafe` (`unsafe extend PoolArena implements
## FoundByAddress:`): the implementer promises that every block it hands out lies in a piece of
## memory that starts at a multiple of `PIECE` bytes and holds the instance's address in its first
## word, within 64 bytes it never hands out, a value bigger than a piece taking a piece of its own.
## A value made in it then keeps no instance word: its growth and its end read the instance from the
## start of its piece, one mask and one load (`[ALC-1]`).
pub unsafe interface FoundByAddress:
    pass

## `[ALC-2]` — the runtime's allocator: the one every heap type uses, which is the package's global
## allocator when it names one (`[build] global_allocator`, `[ALC-3]`) or the functions its
## embedding host passes to `ember_rt_init` (`[FFI-27]`), and `System` otherwise. A global
## allocator that delegates to `Global` calls itself until the stack overflows: it wraps `System`
## (ODR-239).
pub struct Global:
    pass

# SAFETY: the runtime's allocator gives valid, aligned, unaliased blocks, or null.
unsafe extend Global implements Allocator:
    ## A block the runtime allocates, or null.
    fn allocate(layout: Layout) -> *mut u8:
        return global_allocate(layout.size, layout.align)

    ## Gives a block back to the runtime, which allocates and frees through one allocator.
    @safety("`p` came from this manager's `allocate` or `reallocate` with `layout`, and nothing uses it after")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        global_deallocate(p, layout.size, layout.align)

    ## The new block the runtime allocates, the old one given back; or null.
    @safety("`p` came from this manager's `allocate` or `reallocate` with `old`, and nothing uses it after; only the result is used")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return global_reallocate(p, old.size, new_size, old.align)

## The runtime's: it allocates a block, or gives null when there is no memory.
fn global_allocate(size: usize, align: usize) -> *mut u8:
    return null[*mut u8]()

## The runtime's: a block given back, through the allocator that allocates them.
fn global_deallocate(p: *mut u8, size: usize, align: usize):
    pass

## The runtime's: it allocates the new block and gives the old one back, or gives null.
fn global_reallocate(p: *mut u8, old_size: usize, new_size: usize, align: usize) -> *mut u8:
    return null[*mut u8]()

## `[ALC-2]`, `[ALC-3]` (ODR-239) — the platform's allocator (C's `malloc`, `realloc` and `free`,
## aligned as asked): the one the runtime uses when no global allocator is named, and the one a
## global allocator wraps, as a counting or tracing one does, since nothing configured reaches it.
pub struct System:
    pass

# SAFETY: the platform's allocator gives valid, aligned, unaliased blocks, or null.
unsafe extend System implements Allocator:
    ## A block the platform allocates, or null.
    fn allocate(layout: Layout) -> *mut u8:
        return system_allocate(layout.size, layout.align)

    ## Gives a block back to the platform's allocator, which allocates and frees them.
    @safety("`p` came from this manager's `allocate` or `reallocate` with `layout`, and nothing uses it after")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        system_deallocate(p, layout.size, layout.align)

    ## The block grown or shrunk in place, or a new one the platform allocates holding the first
    ## bytes of `p` (`p` then given back); or null, `p` as it was.
    @safety("`p` came from this manager's `allocate` or `reallocate` with `old`, and nothing uses it after; only the result is used")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return system_reallocate(p, old.size, new_size, old.align)

## The runtime's: the platform allocates a block, or gives null when there is no memory.
fn system_allocate(size: usize, align: usize) -> *mut u8:
    return null[*mut u8]()

## The runtime's: a block given back to the platform's allocator, which allocates them.
fn system_deallocate(p: *mut u8, size: usize, align: usize):
    pass

## The runtime's: the platform allocates the grown block, or gives null.
fn system_reallocate(p: *mut u8, old_size: usize, new_size: usize, align: usize) -> *mut u8:
    return null[*mut u8]()

## `[ALC-3]` — what a binary package's global allocator is reached through: the C functions the
## build writes for it call these with the `static` the manifest names, so it must implement
## `Allocator`.
[A: needs Allocator]
fn global_allocate_through(a: A, size: usize, align: usize) -> *mut u8:
    return a.allocate(Layout(size, align))

[A: needs Allocator]
fn global_deallocate_through(a: A, p: *mut u8, size: usize, align: usize):
    # SAFETY: the runtime gives back only a block this allocator gave, with the size and alignment
    # it was asked for.
    unsafe:
        a.deallocate(p, Layout(size, align))

## `[ALC-1]`, `[TYP-46]` — what an `owns T using A` is. The compiler makes one wherever a program
## writes `owns T using A` (`owns T using Global` is `owns T`), shows it as that, and reads it as its
## `T`, as every `owns T`. The line that makes the value takes its block from the instance it names,
## `owns Mesh(points=[0.0]) using current`, by calling `current.allocate(...)` directly, and writes
## the value into it; the end runs `T`'s clean-up and gives the block back through the manager's
## `deallocate`. What it keeps besides the value's address follows the manager (verdict 107):
## - `OwnsIn` (a manager that may have several instances): the instance's address, one word more;
## - `OwnsInOne` (a manager with no fields, so one instance: `System`, any such type): nothing;
## - `OwnsFound` (a manager implementing `FoundByAddress`): nothing, the instance read from the
##   start of the value's piece, one mask and one load;
## - `OwnsFreeNothing` (a manager whose `deallocate` does nothing, an `Arena`): nothing, and its end
##   gives nothing back.
## These calls are the compiler's own, made for the program, so they need no `unsafe:` of its.
[T, A: needs Allocator]
struct OwnsIn:
    value: *mut T
    instance: *mut A

    fn drop():
        # SAFETY: `value` came from this instance's `allocate` with `T`'s layout, the instance lives
        # longer than the value (`[LT-4]`), and the value is not used after its end.
        unsafe:
            drop(this.value.read())
            instance_at(this.instance).deallocate(this.value as *mut u8, Layout(size_of[T](), align_of[T]()))

[T, A: needs Allocator]
struct OwnsInOne:
    value: *mut T

    fn drop():
        # SAFETY: as `OwnsIn`'s; a manager with no fields has one instance, of no bytes, so any
        # aligned address is it.
        unsafe:
            drop(this.value.read())
            instance_at(align_of[A]() as *mut A).deallocate(this.value as *mut u8, Layout(size_of[T](), align_of[T]()))

[T, A: needs (Allocator, FoundByAddress)]
struct OwnsFound:
    value: *mut T

    fn drop():
        # SAFETY: as `OwnsIn`'s; the manager promises that the piece holding the block starts at a
        # multiple of `PIECE` and holds the instance's address in its first word.
        unsafe:
            drop(this.value.read())
            piece = ((this.value as usize) & ~(PIECE - 1)) as *mut *mut A
            instance_at(piece.read()).deallocate(this.value as *mut u8, Layout(size_of[T](), align_of[T]()))

[T, A: needs Allocator]
struct OwnsFreeNothing:
    value: *mut T

    fn drop():
        # SAFETY: `value` came from the instance's `allocate`, and is not used after its end; the
        # manager frees nothing one by one, so nothing is given back.
        unsafe:
            drop(this.value.read())

## `[ALC-1]` — compiler-known: the instance of a manager at the address `p` a value made in it
## keeps (`OwnsIn`), reached as a place, so that the compiler's call of its methods changes it.
@safety("`p` is the address of a live instance of `A`")
[A]
unsafe fn instance_at(p: *mut A) -> ref mut A:
    pass

## `[LAY-2]` — the byte offset of a field of struct `T`, named as written:
## `offset_of[Header](length)`. The compiler reads the name; this declaration
## makes the path real.
[T]
pub fn offset_of(field: StrView) -> usize:
    return 0

## `[ARN-2]`, `[ALC-2]` — an `Arena` is a memory manager: a value made `using frame` (`owns T using
## frame`) takes its block from the arena's memory, a pointer bump aligned up to what it asks, and
## gives nothing back one by one (`deallocate` does nothing, so the value keeps no instance word,
## `[ALC-1]`); the arena frees it all when it is reset or ends.
# SAFETY: an arena's bytes are valid and reached through nothing else until it is reset or ends,
# and each block is aligned up to the alignment asked for here.
unsafe extend Arena implements Allocator:
    ## A block of the arena's memory; it allocates when the arena grows (`[ARN-4]`).
    fn allocate(layout: Layout) -> *mut u8:
        bytes = this.alloc_uninit[u8](layout.size + layout.align - 1)
        # SAFETY: the address aligned up stays inside the bytes just taken, `align - 1` past their start
        # at most.
        unsafe:
            at = bytes.as_mut_ptr() as usize
            return ((at + layout.align - 1) & ~(layout.align - 1)) as *mut u8

    ## Nothing: the arena frees its memory together when it is reset or ends.
    @safety("`p` came from this arena's `allocate` with `layout`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        pass

    ## A new block holding the first bytes of `p`; it allocates when the arena grows.
    @safety("`p` came from this arena's `allocate` with `old`, and only the result is used after")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        q = this.allocate(Layout(new_size, old.align))
        copy_nonoverlapping(p as *u8, q, old.size if old.size < new_size else new_size)
        return q
