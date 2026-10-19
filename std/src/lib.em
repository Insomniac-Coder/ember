## `std` — the standard library's root module (`[MOD-1]`).
##
## Part XV lists the module set. What exists so far:
##
##   `std.core`  Part IV §8's interfaces, `Ordering`, and Part XIII's `Error`,
##               `From` and `AnyError`
##   `std.fmt`  the `Formatter` a written `Display` or `Debug` writes through
##   `std.math`  Part XV's scalar mathematics
##   `std.collections`  `Map`, `Set` and hashing (`[STD-11]`, `[HASH-1]`), `Pool` and `Handle` (`[HND-1]`), and
##                      `[ARN-5]`'s Arena-backed collection declarations
##   `std.mem`  `[UNS-10]`'s lowest-level interior-mutability primitive
##   `std.arena`  what the prelude's arenas are (Part VII.6)
##   `std.simd`  vector and mask types (Part XII.2)
##   `std.ecs`  entities, component storage and the component registry (Part XII.3)
##   `std.sync`  locks, atomics, `Once` and `SyncShared` (Part XI.2)
##   `std.thread`  `spawn` and `JoinHandle` (`[THR-10]`)
##   `std.jobs`  the job system: `submit`, `wait`, `scope` (Part XI.5)
##   `std.coroutine`  gameplay sequencing on generators: `Wait`, `Scheduler`
##                    (`[CORO-11]`)
##   `std.hot`  hot reload's program side: `ReloadError` and `checkpoint` (Annex B)
##   `std.process`  the command line, `exit` and `abort` (`[STD-24]`)
##   `std.env`  the environment: `var` (`[STD-24]`)
##   `std.random`  `Rng`, seeded or from the system's generator (`[STD-25]`)
##   `std.time`  `Instant`, `Duration` and `sleep` (`[STD-23]`)
##
## `Option`, `Result`, `Array` and `String` are still compiler-known: Part XX.1
## makes them so "until Phase 2's generics let the standard library write
## them", and `[MOD-5]`'s prelude is satisfied by that route until each one can
## be written here.

pass
