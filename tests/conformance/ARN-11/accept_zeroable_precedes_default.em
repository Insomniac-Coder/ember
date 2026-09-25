#$ test: run-pass
#$ rules: ARN-3, ARN-11, ARN-12, TST-23
# `[ARN-3]` — `alloc_array` zeroes a `Zeroable` element type rather than
# calling its `Default`. Until D-281, a source `extend i32 implements
# Default` returning 77 made the precedence observable; `[TYP-36]` now gives
# `i32` its `Default` in std, a second one is `E2041`, and every built-in
# `Default` is zero. The non-zero case returns with `@derive(Zeroable)` on a
# struct, which is not built yet.

fn main():
    arena = Arena.with_capacity(16)
    values = arena.alloc_array[i32](1)
    println(values[0])
#$ stdout: 0
