## `std.thread` — threads (Part XI.2).
##
## `[THR-10]` — `thread.spawn(owned fn() => …)` runs the function on a new thread and gives a
## `JoinHandle[R]`. What the function captures and what it returns must be `Send` (`E7004`, naming
## the capture), and it may capture no view: the thread may outlive every local of the code that
## started it (`E3063`; a scope, `[THR-5]`, is the way to borrow). `h.join()` waits for the thread
## and gives what its function returned; dropping the handle detaches the thread, which runs on.
## When `main` returns the process ends, other threads with it (`[THR-16]`).
##
## `[THR-5]` — `thread.scope()` gives a `Scope`, which only `with scope = thread.scope():` may hold
## (`E3014`), and which cannot be moved (`E3014`) or stored (`E3015`). On every exit from the block it
## first joins every thread spawned in it that is not joined yet. `[THR-11]` — `scope.spawn(f)` runs
## `f` on a new thread; `f` may borrow anything that lives longer than the block, and each borrow it
## makes lasts until the block ends, so two threads may not write one place, nor one write what
## another reads (`E3021`/`E3022`). What it borrows shared must be `Sync` (`E7005`); what it borrows
## mutably, what it captures and what it returns, `Send` (`E7004`). `h.join()` waits early.
##
## The compiler knows `spawn`, `scope` and these types by their names here: it gives each `JoinHandle[R]`
## a packet holding the function and its result, and the runtime starts, joins and detaches the
## thread. The declarations give them their names.

pub struct JoinHandle[R]:
    thread: *mut u8

pub fn spawn[R](owned f: fn() -> R) -> JoinHandle[R]:
    pass

@must_drop
@view
pub struct Scope:
    anchor: ref u8
    tasks: *mut u8

pub fn scope() -> Scope:
    panic("`thread.scope` is the compiler's")

@view
pub struct ScopedJoinHandle[R]:
    anchor: ref u8
    thread: *mut u8
