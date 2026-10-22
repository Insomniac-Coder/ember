#$ test: run-pass
#$ rules: FFI-8
#$ profiles: debug, development, release
#$ stdout: 60
#$ stdout: 7-abc-2.50
#$ stdout: 3 apples, 4 pears
# `[FFI-8]` — a variadic C function is called in `unsafe`, each further argument with the type the
# program gives it (`as c_int`, a typed local). C's `va_list` is `std.ffi.VaList`: a function C
# calls receives one and passes it on, once, to one that reads it.

from std.ffi import VaList, c_char, c_int
import c "./variadic.c" with (overlay = "overlays/variadic.em") as v

fn shown(bytes: ref [u8; 32], length: c_int) -> String:
    return String.from(bytes[0..length as int].to_str().unwrap())

## What the logger calls with its arguments: they go on to `vsnprintf`.
extern "C" fn sink(context: *mut void, format: *c_char, owned arguments: VaList):
    bytes: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    length: c_int = 0
    unsafe:  # SAFETY(ffi): `bytes` holds 32, and `arguments` are what the format reads
        length = v.format_list(ref_to_ptr(ref mut bytes) as *mut c_char, 32, format, arguments)
    println(shown(ref bytes, length))

fn main():
    wide: i64 = 10
    total: i64 = 0
    bytes: [u8; 32] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    length: c_int = 0
    unsafe:  # SAFETY(ffi): each further argument is the type the count or the format names
        total = v.sum_wide(3, wide, 20i64, wide * 3)
        length = v.format_into(ref_to_ptr(ref mut bytes) as *mut c_char, 32, c"%d-%s-%.2f", 7 as c_int, c"abc", 2.5 as f64)
    println(total)
    println(shown(ref bytes, length))
    unsafe:  # SAFETY(ffi): the format reads an `int`, a string, an `int` and a string
        v.log_through(sink, null[*mut void](), c"%d %s, %d %s", 3 as c_int, c"apples", 4 as c_int, c"pears")
