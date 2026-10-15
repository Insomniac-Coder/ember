## `std.sync` — sharing values between threads (Part XI.2).
##
## `[THR-3]` — `Mutex[T]` is a value type holding a `T` and one lock word. `m.lock()` waits until
## no other thread holds the lock, takes it and gives a `MutexGuard[T]`: a view of `m` that reads
## through to `ref mut T`, and whose end releases the lock, which `with g = m.lock():` places at
## the end of the block. `m.try_lock()` never waits: `None` when another thread holds it. There is
## no poisoning: a panic aborts the process (`[PAN-1]`). A `Mutex[T]` is `Send` and `Sync` when `T`
## is `Send` (`[THR-9]`); a guard is not `Send`.
##
## `RwLock[T]` is the same with two kinds of guard: `read()` gives an `RwLockReadGuard[T]`, which
## reads through to `ref T` and which any number of threads may hold together, and `write()` an
## `RwLockWriteGuard[T]`, which reads through to `ref mut T` and which one thread holds alone;
## `try_read` and `try_write` never wait. An `RwLock[T]` is `Send` when `T` is, and `Sync` when `T`
## is `Send` and `Sync` (`[THR-9]`: `RwLock[Cell[int]]` is not `Sync`).
##
## The compiler knows these types by their names here (as it knows `std.collections`'s arena
## collections), lowers `lock` and `try_lock` to the runtime's lock word (`ember_mutex_lock`,
## `ember_mutex_try_lock`) and a guard's end to `ember_mutex_unlock`. The word is private, starts
## at 0, and is the runtime's alone.

pub struct Mutex[T]:
    value: T
    word: u32 = 0

@view
pub struct MutexGuard[T]:
    value: ref mut T

pub struct RwLock[T]:
    value: T
    word: u32 = 0

@view
pub struct RwLockReadGuard[T]:
    value: ref T

@view
pub struct RwLockWriteGuard[T]:
    value: ref mut T
