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
## `[THR-15]` — `channel[T](capacity=n)` makes a queue of at most `n` values (one or more) that any
## number of `Sender[T]`s fill and one `Receiver[T]` empties, in order; `channel_unbounded[T]()` one
## with no bound, which allocates as it grows. `tx.send(v)` waits while the queue is full and gives
## `v` back in a `SendError` once the receiver is gone; `rx.recv()` waits while it is empty and gives
## `None` once every sender is gone and the queue is empty. `try_send` and `try_recv` never wait.
## `for v in rx:` receives until `None`. A `Sender` is `Send`, `Sync` and `Clone`; a `Receiver` is
## `Send`. Written here in Ember: a `Mutex` around a ring of values, and two event counts a waiting
## side sleeps on, bumped only when it is asleep.
##
## The compiler knows these types by their names here (as it knows `std.collections`'s arena
## collections), lowers `lock` and `try_lock` to the runtime's lock word (`ember_mutex_lock`,
## `ember_mutex_try_lock`) and a guard's end to `ember_mutex_unlock`. The word is private, one
## pointer wide, starts at 0, and is the runtime's alone: on Windows the system's SRWLOCK, elsewhere
## a futex word.
##
## ODR-275 — what changes through a shared reference to one of these types is kept in an
## `UnsafeCell` (`[UNS-10]`): a lock's value and word, an atomic's value. A shared reference to
## anything else is only read through.

from std.mem import UnsafeCell

pub struct Mutex[T]:
    value: UnsafeCell[T]
    word: UnsafeCell[usize] = UnsafeCell(0)

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
    value: UnsafeCell[T]

pub struct RwLock[T]:
    value: UnsafeCell[T]
    word: UnsafeCell[usize] = UnsafeCell(0)

@view
pub struct RwLockReadGuard[T]:
    value: ref T

@view
pub struct RwLockWriteGuard[T]:
    value: ref mut T

pub struct Once:
    state: Atomic[u32] = Atomic(0)

    pub fn call_once[Call: once fn()](self, owned f: Call):
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

pub struct SendError[T]:
    ## The value that was not sent.
    pub value: T
    ## True once the receiver is gone; false when `try_send` found the queue full.
    pub disconnected: bool

## A first-in, first-out ring of `slots`, `count` of them full from `head` on, and who is asleep on
## it: changed only under the channel's lock, so a value that finds nobody asleep costs the lock
## and nothing more. A bounded channel's ring is its capacity; an unbounded one's doubles when full.
struct ChannelQueue[T]:
    slots: Array[Option[T]]
    head: int
    count: int
    receiver_waiting: bool
    receiver_gone: bool
    senders_waiting: int

    fn push(mut self, owned value: T):
        if self.count == self.slots.len():
            self.grow()
        at = self.head + self.count
        if at >= self.slots.len():
            at -= self.slots.len()
        self.slots[at] = Some(value)
        self.count += 1

    fn pop(mut self) -> Option[T]:
        if self.count == 0:
            return None
        value = mem.replace(self.slots[self.head], None)
        self.head += 1
        if self.head == self.slots.len():
            self.head = 0
        self.count -= 1
        return value

    fn grow(mut self):
        size = self.slots.len()
        bigger: Array[Option[T]] = []
        for i in range(self.count):
            at = self.head + i
            if at >= size:
                at -= size
            bigger.push(mem.replace(self.slots[at], None))
        wanted = 16 if size == 0 else size * 2
        while bigger.len() < wanted:
            bigger.push(None)
        self.slots = bigger
        self.head = 0

struct ChannelState[T]:
    queue: Mutex[ChannelQueue[T]]
    ## 0: no bound.
    capacity: int
    senders: Atomic[u32]
    ## Bumped, under the lock, by a send or the last sender's end that found the receiver asleep.
    arrivals: Atomic[u32]
    ## Bumped, under the lock, by a receive or the receiver's end that found senders asleep.
    departures: Atomic[u32]

pub struct Sender[T]:
    state: SyncShared[ChannelState[T]]

    pub fn send(self, owned value: T) -> Result[void, SendError[T]]:
        state = self.state.get()
        waited = false
        while true:
            seen: u32 = 0
            with q = state.queue.lock():
                if waited:
                    q.senders_waiting -= 1
                if q.receiver_gone:
                    return Err(SendError(value, true))
                if state.capacity == 0 or q.count < state.capacity:
                    q.push(value)
                    if q.receiver_waiting:
                        q.receiver_waiting = false
                        state.arrivals.wrapping_fetch_add(1)
                        state.arrivals.wake_one()
                    return Ok(())
                q.senders_waiting += 1
                seen = state.departures.load()
            waited = true
            state.departures.wait(seen)
        return Ok(())

    pub fn try_send(self, owned value: T) -> Result[void, SendError[T]]:
        state = self.state.get()
        with q = state.queue.lock():
            if q.receiver_gone:
                return Err(SendError(value, true))
            if state.capacity == 0 or q.count < state.capacity:
                q.push(value)
                if q.receiver_waiting:
                    q.receiver_waiting = false
                    state.arrivals.wrapping_fetch_add(1)
                    state.arrivals.wake_one()
                return Ok(())
        return Err(SendError(value, false))

    fn drop(mut self):
        state = self.state.get()
        if state.senders.fetch_sub(1) == 1:
            with q = state.queue.lock():
                if q.receiver_waiting:
                    q.receiver_waiting = false
                    state.arrivals.wrapping_fetch_add(1)
                    state.arrivals.wake_one()

extend[T] Sender[T] implements Clone:
    fn clone(self) -> Sender[T]:
        self.state.get().senders.fetch_add(1, MemoryOrder.Relaxed)
        return Sender(self.state)

pub struct Receiver[T]:
    state: SyncShared[ChannelState[T]]

    pub fn recv(mut self) -> Option[T]:
        state = self.state.get()
        while true:
            seen: u32 = 0
            with q = state.queue.lock():
                match q.pop():
                    Some(value):
                        if q.senders_waiting > 0:
                            state.departures.wrapping_fetch_add(1)
                            state.departures.wake_one()
                        return Some(value)
                    None:
                        if state.senders.load() == 0:
                            return None
                        q.receiver_waiting = true
                        seen = state.arrivals.load()
            state.arrivals.wait(seen)
        return None

    pub fn try_recv(mut self) -> Option[T]:
        state = self.state.get()
        with q = state.queue.lock():
            value = q.pop()
            if value.is_some() and q.senders_waiting > 0:
                state.departures.wrapping_fetch_add(1)
                state.departures.wake_one()
            return value

    pub fn next(mut self) -> Option[T]:
        return self.recv()

    fn drop(mut self):
        state = self.state.get()
        with q = state.queue.lock():
            q.receiver_gone = true
            if q.senders_waiting > 0:
                state.departures.wrapping_fetch_add(1)
                state.departures.wake_all()

extend[T] Receiver[T] implements Iterator:
    type Item = T

pub fn channel[T](capacity: int) -> (Sender[T], Receiver[T]):
    assert(capacity >= 1, "a channel holds at least one value")
    return open_channel[T](capacity)

pub fn channel_unbounded[T]() -> (Sender[T], Receiver[T]):
    return open_channel[T](0)

fn open_channel[T](capacity: int) -> (Sender[T], Receiver[T]):
    slots: Array[Option[T]] = []
    while slots.len() < capacity:
        slots.push(None)
    queue = ChannelQueue(slots, 0, 0, false, false, 0)
    state = SyncShared(ChannelState(Mutex(queue), capacity, Atomic(1), Atomic(0), Atomic(0)))
    return (Sender(state), Receiver(state))
