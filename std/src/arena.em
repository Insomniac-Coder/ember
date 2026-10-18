## `std.arena` — region allocation (Part VII.6, `[ARN-1]`..`[ARN-12]`).
##
## `Arena`, `FixedArena` and `ScopedArena` are in the prelude and compiler-known, as `Cell` is: an
## arena hands out views of memory it owns and frees them all at once (`reset()`, `scope()`, its
## drop), and the compiler proves no view outlives that (`[ARN-1]`). A growing `Arena` carries
## `Alloc` on its growth path; a `FixedArena` never grows and carries none (`[ARN-4]`). The
## containers that take their storage from an arena once (`ArenaArray`, `ArenaMap`, `ArenaSoA`)
## live in `std.collections` (`[ARN-5]`, `[SOA-4]`).
##
## `[STD-1]` — every function of this module is `@noalloc` unless its documentation says it
## allocates; the compiler checks it as a written `@noalloc` (`E4001`). The arena operations
## themselves are the compiler's, with the effects `[ARN-4]` gives them.
