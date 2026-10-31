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

## `[DRP-1]` — the public, safe spelling for ending a value's life early.
## Taking the value through an `owned` parameter transfers it into this frame;
## ordinary scope-end destruction then runs its destructor exactly once.
[T]
pub fn drop(owns value: T):
    pass

## `[RC-3]` — an explicit use at this point. The borrowed parameter neither
## transfers ownership nor extends a handle beyond the call; it lets code
## using a raw pointer state where its owner must still be live.
[T]
pub fn keep_alive(value: T):
    pass

## `[OWN-6]`'s `take`, `replace`, `swap`, and `forget` remain compiler-known while the
## standard library is staged; calls through this public module still use
## ordinary generic inference, mutable-place checks, borrows, and moves.
[T]
pub fn replace(ref mut place: T, owns value: T) -> T:
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

## Consumes the value without running its destructor. `[THR-6]`'s rejection
## for `@must_drop` values becomes active with that later type mechanism; this
## operation does not invent a parallel marker in the meantime.
[T]
pub fn forget(owns value: T):
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

## `[ALC-2]` (ODR-175) — the size and the alignment of a block of memory an `Allocator` gives.
pub struct Layout:
    pub size: usize
    pub align: usize

## `[ALC-2]` (ODR-175) — what gives a program its heap memory. Implementing it is `unsafe`
## (`unsafe extend T implements Allocator:`): the implementer promises that `allocate` gives memory
## valid for `layout.size` bytes, aligned to `layout.align` and reached through nothing else, or
## null when it has none; that `deallocate` is given back only what `allocate` or `reallocate` gave,
## with the layout it was asked for; and that `reallocate` gives a block of `new_size` bytes holding
## the first of `p`'s (`p` then given back), or null, leaving `p` as it was. A binary package may
## make one the global allocator, which every heap type allocates through (`[ALC-3]`).
pub unsafe interface Allocator:
    fn allocate(layout: Layout) -> *mut u8
    fn deallocate(p: *mut u8, layout: Layout)
    fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8

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
    fn deallocate(p: *mut u8, layout: Layout):
        global_deallocate(p, layout.size, layout.align)

    ## The new block the runtime allocates, the old one given back; or null.
    fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
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
    fn deallocate(p: *mut u8, layout: Layout):
        system_deallocate(p, layout.size, layout.align)

    ## The block grown or shrunk in place, or a new one the platform allocates holding the first
    ## bytes of `p` (`p` then given back); or null, `p` as it was.
    fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
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
    a.deallocate(p, Layout(size, align))

## `[LAY-2]` — the byte offset of a field of struct `T`, named as written:
## `offset_of[Header](length)`. The compiler reads the name; this declaration
## makes the path real.
[T]
pub fn offset_of(field: StrView) -> usize:
    return 0
