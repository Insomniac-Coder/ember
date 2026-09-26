## `std.mem` — low-level memory facilities.
##
## `[UNS-10]` gives `UnsafeCell[T]` this public module identity. Its private
## one-field representation and its two operations are compiler-known during
## Phase 2 so no source code can accidentally expose safe access to `value`.

pub struct UnsafeCell[T]:
    value: T

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
