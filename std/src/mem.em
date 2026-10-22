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
    pub unsafe fn read_volatile(self) -> T:  # SAFETY: the caller keeps the obligation
        return volatile_read(self.pointer)

    ## A volatile write of `value` where the pointer points.
    @safety("the pointer is valid for a write of a `T`")
    pub unsafe fn write_volatile(self, value: T):  # SAFETY: the caller keeps the obligation
        volatile_write(self.pointer, value)

extend[T] Volatile[*T]:
    ## A volatile read of the `T` the pointer points to.
    @safety("the pointer is valid for a read of a `T`")
    pub unsafe fn read_volatile(self) -> T:  # SAFETY: the caller keeps the obligation
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
pub unsafe fn assume_disjoint[A, B](owned a: A, owned b: B) -> (A, B):  # SAFETY: this test sets up what the operation requires
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

## `[LAY-2]` — the byte offset of a field of struct `T`, named as written:
## `offset_of[Header](length)`. The compiler reads the name; this declaration
## makes the path real.
pub fn offset_of[T](field: str) -> usize:
    return 0
