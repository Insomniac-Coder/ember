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
## `[THR-14]` — `Atomic[T]` holds an integer, a `bool` or a raw pointer, changed only by atomic
## operations: `load`, `store`, `swap`, `compare_exchange(current, new)` (`Ok(previous)` when it
## stored, `Err(actual)` when the value was not `current`), and on an integer `fetch_add` and
## `fetch_sub` (which panic when the result overflows, like `+`, after the wrapped value is stored),
## `checked_fetch_add`/`checked_fetch_sub` (`None`, storing nothing, when the result is out of
## range), `wrapping_fetch_add`/`wrapping_fetch_sub`, and `fetch_and`/`fetch_or`/`fetch_xor` (on a
## `bool` too). Each takes `order: MemoryOrder = MemoryOrder.SeqCst` last; a load cannot be
## `Release` or `AcqRel`, nor a store `Acquire` or `AcqRel` (`E7006`). An `Atomic` is `Send` and
## `Sync`.
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

pub enum MemoryOrder:
    Relaxed
    Acquire
    Release
    AcqRel
    SeqCst

pub struct Atomic[T]:
    value: T

pub struct RwLock[T]:
    value: T
    word: u32 = 0

@view
pub struct RwLockReadGuard[T]:
    value: ref T

@view
pub struct RwLockWriteGuard[T]:
    value: ref mut T
