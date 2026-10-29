#$ test: run-pass
#$ rules: FFI-6, FFI-29, FFI-29a, BLD-FFI-1a
#$ warning[L3015]: `inline_swap` is a C function the program calls in unsafe code
#$ warning[L3015]: `inline_store` is a C function the program calls in unsafe code
#$ warning[L3015]: `inline_scale` is a C function the program calls in unsafe code
#$ warning[L3015]: `inline_twice` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ assert-c: contains("_cshim_c_h_inline_math_inline_scale")
#$ assert-c: not contains(" inline_scale(")
#$ stdout: 30 21 2 1 9
#$ stdout: 28
# `[FFI-6]` — `import c "…" with (…) as m` binds the header's module to `m`. `[FFI-29]` — the shim
# holds an external wrapper for each `static inline` function, and `[FFI-29a]` the binding names
# the wrapper, never the function itself, which has internal linkage. `[BLD-FFI-1a]` — the header
# is parsed, and the shim compiled, with the import's `defines`: the macro's value and the
# wrapper's arithmetic both see `INLINE_MATH_SCALE=3`.

import c "inline_math.h" with (defines = ["INLINE_MATH_SCALE=3"]) as m

extern "C" fn add_four(x: i32) -> i32:
    return x + 4

fn main():
    p = m.Pair(first=1, second=2)
    stored: i32 = 0
    unsafe:  # SAFETY(ffi): `stored` is a live local the call writes once
        q = m.inline_swap(p)
        m.inline_store(ref_to_ptr(ref mut stored), 9)
        println(m.INLINE_MATH_FACTOR, m.inline_scale(7), q.first, q.second, stored)
        println(m.inline_twice(Some(add_four), 20))
