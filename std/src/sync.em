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
## `[THR-9]` — `Once` runs an initialiser exactly once: the first `o.call_once(f)`, on any thread,
## runs `f`; every other call waits until that run has finished, then returns without running its
## own. `o.is_completed()` tells whether the run has finished. A `Once` is `Send` and `Sync`. A
## `call_once` inside the run it is waiting for never returns.
##
## `[HEAP-10]` — `SyncShared[T]` is `Shared[T]` for several threads: `SyncShared(value)` makes the
## first owner, a copy is another (its counts are atomic), and the last one's end drops the value,
## on whichever thread that is. It offers `get() -> ref T` only: mutation goes through `T`'s own
## locks or atomics. It and its `Weak` are `Send` and `Sync` when `T` is both (`[THR-8]`,
## `[THR-9]`). The compiler builds it as it builds `Shared`; the declaration here gives it its
## name.
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

pub struct Once:
    state: Atomic[u32] = Atomic(0)

    pub fn call_once(self, f: once fn()):
        if self.state.load(MemoryOrder.Acquire) == ONCE_DONE:
            return
        if self.state.compare_exchange(ONCE_NEW, ONCE_RUNNING, MemoryOrder.Acquire).is_ok():
            f()
            self.state.store(ONCE_DONE, MemoryOrder.Release)
            self.state.wake_all()
            return
        while self.state.load(MemoryOrder.Acquire) != ONCE_DONE:
            self.state.wait(ONCE_RUNNING)

    pub fn is_completed(self) -> bool:
        return self.state.load(MemoryOrder.Acquire) == ONCE_DONE

const ONCE_NEW: u32 = 0
const ONCE_RUNNING: u32 = 1
const ONCE_DONE: u32 = 2

pub struct SyncShared[T]:
    value: *mut T
