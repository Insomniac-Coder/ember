#! language "0.8.3"
## `std` — the standard library's root module (`[MOD-1]`).
##
## Part XV lists the module set. What exists so far:
##
##   `std.core`  Part IV §8's interfaces, and `Ordering`
##   `std.math`  Part XV's scalar mathematics
##   `std.collections`  `[ARN-5]`'s Arena-backed collection declarations
##   `std.borrow`  callback-scoped `Span`/`MutSpan` composition helpers
##   `std.mem`  `[UNS-10]`'s lowest-level interior-mutability primitive
##
## `Option`, `Result`, `Array` and `String` are still compiler-known: Part XX.1
## makes them so "until Phase 2's generics let the standard library write
## them", and `[MOD-5]`'s prelude is satisfied by that route until each one can
## be written here.

pass
