#$ test: compile-fail
#$ rules: FFI-11a
#$ help: choose one, count(n), nul_terminated, fixed(N), or inout_count(p)
# `[FFI-11a]` — a pointer contract with no count word is `E5012`, which lists the five count words
# and names each sibling parameter that looks like a length: an unsigned integer (`width`,
# `height`, a `c_uint`), or a name ending `Count`, `Len`, `Size` or `N` (`itemCount`, `bufSize`,
# `rowsN`, here signed). `label`, a signed number with another name, is none.

from std.ffi import c_uint

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(data, borrowed))    #$ error[E5012]: possible length parameter: width, height, itemCount, bufSize, rowsN
    @safety("callers pass what the C function `pack` requires")
    unsafe fn pack(data: *u8, width: u32, height: c_uint, itemCount: i32, bufSize: i64, rowsN: i32, label: i32) -> i32

fn main():
    pass
