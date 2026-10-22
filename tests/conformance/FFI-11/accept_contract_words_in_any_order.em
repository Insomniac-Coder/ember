#$ test: run-pass
#$ rules: FFI-11, FFI-10
#$ profiles: debug, development, release
#$ stdout: 6 0 4 3 ember
# `[FFI-11]` — a slot's contract words have no order (an overlay writes them in any, `[GRM-35]`):
# `param(values, count(n), borrowed)`, `param(value, nullable, borrowed, one)`, `param(v, one, out)`
# and `result(from(static), nul_terminated, borrowed)` read as the same words written
# ownership-first.

unsafe extern "C":
    @ffi(param(values, count(n), borrowed), link_name="ordered_sum")
    safe fn sum(values: Span[i32], n: usize) -> i32
    @ffi(param(value, nullable, borrowed, one), link_name="ordered_or_zero")
    safe fn or_zero(value: Option[ref i32]) -> i32
    @ffi(param(v, one, out), link_name="ordered_three")
    safe fn three(mut v: i32)
    @ffi(result(from(static), nul_terminated, borrowed), link_name="ordered_name")
    safe fn name() -> cstr

pub extern "C" fn ordered_sum(values: *i32, n: usize) -> i32:
    total: i32 = 0
    i: usize = 0
    while i < n:
        unsafe:  # SAFETY(ffi): C's contract: `values` holds `n` elements
            total += read(values, i)
        i += 1
    return total

pub extern "C" fn ordered_or_zero(value: *i32) -> i32:
    if value.is_null():
        return 0
    unsafe:  # SAFETY(ffi): C's contract: a non-null `value` is one element
        return read(value, 0)

pub extern "C" fn ordered_three(v: *mut i32):
    unsafe:  # SAFETY(ffi): C's contract: `v` is one element to write
        write(v, 0, 3)

pub extern "C" fn ordered_name() -> cstr:
    return c"ember"

fn main():
    values: Array[i32] = [1, 2, 3]
    four: i32 = 4
    println(sum(values.as_span()), or_zero(None), or_zero(Some(ref four)), three(), name().to_str().unwrap())
