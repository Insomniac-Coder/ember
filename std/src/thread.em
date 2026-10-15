## `std.thread` — threads (Part XI.2).
##
## `[THR-10]` — `thread.spawn(owned fn() => …)` runs the function on a new thread and gives a
## `JoinHandle[R]`. What the function captures and what it returns must be `Send` (`E7004`, naming
## the capture), and it may capture no view: the thread may outlive every local of the code that
## started it (`E3063`; a scope, `[THR-5]`, is the way to borrow). `h.join()` waits for the thread
## and gives what its function returned; dropping the handle detaches the thread, which runs on.
## When `main` returns the process ends, other threads with it (`[THR-16]`).
##
## The compiler knows `spawn` and `JoinHandle` by their names here: it gives each `JoinHandle[R]`
## a packet holding the function and its result, and the runtime starts, joins and detaches the
## thread. The declarations give them their names.

pub struct JoinHandle[R]:
    thread: *mut u8

pub fn spawn[R](owned f: fn() -> R) -> JoinHandle[R]:
    pass
