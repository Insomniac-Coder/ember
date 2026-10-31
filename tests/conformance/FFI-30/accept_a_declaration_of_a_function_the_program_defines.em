#$ test: run-pass
#$ rules: FFI-30
#$ profiles: debug, development, release
#$ stdout: 5 5
# `[FFI-30]` (D-744) — a foreign declaration may name a C function the program itself defines
# (`pub extern "C" fn`, or an `@export`): the declaration binds that definition, as one in a header
# the package implements would.

import helpers.exported

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("`ffi30_add` adds two `i32`s")
    unsafe fn ffi30_add(a: i32, b: i32) -> i32

fn main():
    through_c: i32 = 0
    unsafe:  # SAFETY(ffi): two scalars
        through_c = ffi30_add(2, 3)
    println(through_c, exported.ffi30_add(2, 3))
