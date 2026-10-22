# Rulings made under the delegation, waiting for the next spec cut (Hardened_35)

Cut 34 (2026-10-22, ADR-268) wrote ODR-169..ODR-237 into the spec; their record is
`docs/OWNER-QUEUE.md`. Open, for the owner: ODR-238 (`[CLO-15]`, the user registry), and in ODR-237
`==` between two different kinds behind `dyn`.

- ODR-239 [ALC-2], [ALC-3] `std.mem.System` is the platform's allocator (what the runtime uses when no global allocator is named): a package's global allocator may wrap it (a counting or tracing allocator delegates to `System()`), while `Global` stays the package's configured allocator, so `Array[T, Global]` never bypasses it and an allocator that delegates to `Global` from inside the global allocator recurses (documented, as Rust's). AA Q1 (option b). NOT BUILT.
- ODR-240 [STD-19], [CLO-*] THE OWNER'S ANSWER (2026-10-22), not delegated; replaces ODR-210 (withdrawn: the owner rejected ownership): every standard-library function that takes a callable, the consumers (`any`, `all`, `find`, `position`, `fold`, `reduce`, `for_each`, the `_by` and `_by_key` forms) and the adapters (`map`, `filter`, …) alike, borrows a named callable the caller passes: the caller keeps it and may pass it again, and a callable that changes what it captured is lent with leave to change it (what it captured stays lent while it is still used, `[BRW-2]`). Lead, applying it: a lambda written at the call is given to the call, since nothing else can name it, so an adapter's iterator made from one can be kept or returned (`return xs.iter().map(fn(x) => x * 2)`); mechanism: a named callable given to a callable parameter is passed as `ref mut` (a reference to a callable is callable), a lambda written at the call by value. NOT BUILT (helper AA built ODR-210's ownership; to be redone).
- ODR-207 confirmed by the owner (2026-10-22): `sorted(m.keys())` gives the sorted keys, copied, as `sorted(m)` does.
- ODR-209 confirmed by the owner (2026-10-22): a one-line lambda body ends at the statement's end of line; a body on the indented lines below is a block, as before.
- ODR-211 confirmed by the owner (2026-10-22): `@borrows(self.inner)` on a field whose type is a type parameter, so the iterator adapters' items point into what the wrapped iterator's do.
- ODR-215 confirmed by the owner (2026-10-22): built-in types' C names under `std.core`, so a program's own `str` never shares a C name with the built-in one.
- ODR-213 confirmed by the owner (2026-10-22), with the lead's extension at cut 34: an ASCII `_` that would read as an escape is escaped in every name, so `café` and a plain `caf_u00E9_` never share a C name.
- ODR-239 confirmed by the owner (2026-10-22): `std.mem.System`, the platform's allocator, for a global allocator to wrap.
- ODR-175 confirmed by the owner (2026-10-22): an allocator is anything that can give out, take back and resize a block of memory (`allocate`, `deallocate`, `reallocate`); implementing it is `unsafe`.
