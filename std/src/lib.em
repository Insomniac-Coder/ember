## `std` — the standard library's root module (`[MOD-1]`).
##
## Part XV lists the module set. What exists so far:
##
##   `std.core`  Part IV §8's interfaces, `Ordering`, and Part XIII's `Error`,
##               `From` and `AnyError`
##   `std.fmt`  the `Formatter` a written `Display` or `Debug` writes through
##   `std.math`  Part XV's scalar mathematics
##   `std.collections`  `Map`, `Set` and hashing (`[STD-11]`, `[HASH-1]`), and
##                      `[ARN-5]`'s Arena-backed collection declarations
##   `std.mem`  `[UNS-10]`'s lowest-level interior-mutability primitive
##   `std.sync`  locks, atomics, `Once` and `SyncShared` (Part XI.2)
##   `std.thread`  `spawn` and `JoinHandle` (`[THR-10]`)
##   `std.jobs`  the job system: `submit`, `wait`, `scope` (Part XI.5)
##   `std.coroutine`  gameplay sequencing on generators: `Wait`, `Scheduler`
##                    (`[CORO-11]`)
##   `std.hot`  hot reload's program side: `ReloadError` and `checkpoint` (Annex B)
##
## `Option`, `Result`, `Array` and `String` are still compiler-known: Part XX.1
## makes them so "until Phase 2's generics let the standard library write
## them", and `[MOD-5]`'s prelude is satisfied by that route until each one can
## be written here.

pass
