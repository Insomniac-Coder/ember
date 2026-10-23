#$ test: run-pass
#$ rules: FFI-8, TYP-36
#$ profiles: debug, development, release
#$ stdout: true true
#$ stdout: 2 seven
#$ stdout: 2
#$ stdout: 7 [7, 9]
#$ stdout: [7, 9]
# ODR-270 — an `@ffi(newtype)` type implements `Eq`, `Hash` and `Debug` as its representation does:
# a handle is compared, keyed in a `Map` or a `Set`, and shown (as its representation's value), in
# the compiled program and the evaluator alike. It has no order and no arithmetic.

from std.ffi import c_int

@ffi(newtype)
type fd_t = c_int

fn main():
    a = 7 as fd_t
    b = 7 as fd_t
    c = 9 as fd_t
    println(a == b, a != c)
    names: Map[fd_t, str] = {a: "seven", c: "nine"}
    println(names.len(), names[b])
    seen = {a, b, c}
    println(seen.len())
    println(f"{a!r} {[a, c]!r}")
    println([a, c])
