## `std.mem` — low-level memory facilities.
##
## `[UNS-10]` gives `UnsafeCell[T]` this public module identity. Its private
## one-field representation and its two operations are compiler-known during
## Phase 2 so no source code can accidentally expose safe access to `value`.

pub struct UnsafeCell[T]:
    value: T

## `[UNS-5]`, `[FFI-8]` — a pointer whose reads and writes are volatile: each one happens, in the
## order written, never merged, moved or removed (a device register, memory another agent
## changes). C's `volatile T*` is a `Volatile[*mut T]` (`Volatile[*T]` for `const volatile T*`); to C
## it is the pointer.
@derive(Copy)
pub struct Volatile[P]:
    pointer: P

    pub fn new(pointer: P) -> Volatile[P]:
        return Volatile(pointer)

    ## The pointer itself.
    pub fn as_ptr(self) -> P:
        return self.pointer

extend[T] Volatile[*mut T]:
    ## A volatile read of the `T` the pointer points to.
    @safety("the pointer is valid for a read of a `T`")
    pub unsafe fn read_volatile(self) -> T:
        # SAFETY: the caller promised the pointer is valid for a read of a `T` (`@safety` above)
        unsafe:
            return volatile_read(self.pointer)

    ## A volatile write of `value` where the pointer points.
    @safety("the pointer is valid for a write of a `T`")
    pub unsafe fn write_volatile(self, value: T):
        # SAFETY: the caller promised the pointer is valid for a write of a `T` (`@safety` above)
        unsafe:
            volatile_write(self.pointer, value)

extend[T] Volatile[*T]:
    ## A volatile read of the `T` the pointer points to.
    @safety("the pointer is valid for a read of a `T`")
    pub unsafe fn read_volatile(self) -> T:
        # SAFETY: the caller promised the pointer is valid for a read of a `T` (`@safety` above)
        unsafe:
            return volatile_read(self.pointer)

## `[UNS-5]` — compiler-known: the volatile read and write a `Volatile` makes.
fn volatile_read[T](pointer: *T) -> T:
    pass

fn volatile_write[T](pointer: *mut T, value: T):
    pass

## `[DRP-1]` — the public, safe spelling for ending a value's life early.
## Taking the value through an `owned` parameter transfers it into this frame;
## ordinary scope-end destruction then runs its destructor exactly once.
pub fn drop[T](owned value: T):
    pass

## `[RC-3]` — an explicit use at this point. The borrowed parameter neither
## transfers ownership nor extends a handle beyond the call; it lets code
## using a raw pointer state where its owner must still be live.
pub fn keep_alive[T](value: T):
    pass

## `[OWN-6]`'s `take`, `replace`, `swap`, and `forget` remain compiler-known while the
## standard library is staged; calls through this public module still use
## ordinary generic inference, mutable-place checks, borrows, and moves.
pub fn replace[T](mut place: T, owned value: T) -> T:
    pass

pub fn take[T: Default](mut place: T) -> T:
    pass

pub fn swap[T](mut a: T, mut b: T):
    pass

## `[DSJ-1]` — two views from sources the program knows do not overlap: `Ok`
## when their address ranges are apart, `Err` with both unchanged when they
## overlap. Compiler-known; the operands must be views (`E3095` otherwise).
pub fn assert_disjoint[A, B](owned a: A, owned b: B) -> Result[(A, B), (A, B)]:
    pass

## `[DSJ-9]` — two to eight views compared pairwise; compiler-known, so the
## declaration shows two.
pub fn assert_disjoint_all[A, B](owned a: A, owned b: B) -> Result[(A, B), (A, B)]:
    pass

## `[DSJ-5]` — `assert_disjoint`, panicking when the views overlap.
pub fn assert_disjoint_or_panic[A, B](owned a: A, owned b: B) -> (A, B):
    pass

## `[DSJ-7]` — the views do not overlap, unchecked: violating it is undefined
## behaviour.
@safety("The two views' address ranges do not overlap.")
pub unsafe fn assume_disjoint[A, B](owned a: A, owned b: B) -> (A, B):
    pass

## Consumes the value without running its destructor. `[THR-6]`'s rejection
## for `@must_drop` values becomes active with that later type mechanism; this
## operation does not invent a parallel marker in the meantime.
pub fn forget[T](owned value: T):
    pass

## Part IX §5 — target-layout queries. These declarations make the public
## `std.mem` paths real; the compiler folds them through its canonical layout
## descriptor rather than executing these staging bodies.
pub fn size_of[T]() -> usize:
    return 0

pub fn align_of[T]() -> usize:
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
    fn allocate(self, layout: Layout) -> *mut u8
    fn deallocate(self, p: *mut u8, layout: Layout)
    fn reallocate(self, p: *mut u8, old: Layout, new_size: usize) -> *mut u8

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
    fn allocate(self, layout: Layout) -> *mut u8:
        return global_allocate(layout.size, layout.align)

    ## Gives a block back to the runtime, which allocates and frees through one allocator.
    fn deallocate(self, p: *mut u8, layout: Layout):
        global_deallocate(p, layout.size, layout.align)

    ## The new block the runtime allocates, the old one given back; or null.
    fn reallocate(self, p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
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
    fn allocate(self, layout: Layout) -> *mut u8:
        return system_allocate(layout.size, layout.align)

    ## Gives a block back to the platform's allocator, which allocates and frees them.
    fn deallocate(self, p: *mut u8, layout: Layout):
        system_deallocate(p, layout.size, layout.align)

    ## The block grown or shrunk in place, or a new one the platform allocates holding the first
    ## bytes of `p` (`p` then given back); or null, `p` as it was.
    fn reallocate(self, p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
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
fn global_allocate_through[A: Allocator](a: A, size: usize, align: usize) -> *mut u8:
    return a.allocate(Layout(size, align))

fn global_deallocate_through[A: Allocator](a: A, p: *mut u8, size: usize, align: usize):
    a.deallocate(p, Layout(size, align))

## `[LAY-2]` — the byte offset of a field of struct `T`, named as written:
## `offset_of[Header](length)`. The compiler reads the name; this declaration
## makes the path real.
pub fn offset_of[T](field: str) -> usize:
    return 0
