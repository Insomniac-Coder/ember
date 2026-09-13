#! language "0.9.6"
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

## `[OWN-6]`'s `take`, `replace`, and `swap` remain compiler-known while the
## standard library is staged; calls through this public module still use
## ordinary generic inference, mutable-place checks, borrows, and moves.
pub fn replace[T](mut place: T, owned value: T) -> T:
    pass

pub fn take[T: Default](mut place: T) -> T:
    pass

pub fn swap[T](mut a: T, mut b: T):
    pass
