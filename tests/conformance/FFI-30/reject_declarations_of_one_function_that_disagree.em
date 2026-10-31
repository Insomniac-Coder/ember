#$ test: compile-fail
#$ rules: FFI-30
#$ error[E0104]: C function `ffi30_add` is declared here with another C signature than elsewhere
# `[FFI-30]` (D-744) — every declaration of one C function, and the program's own definition of it,
# passes and returns what the others do: `ffi30_add` is defined taking two 4-byte integers and
# declared elsewhere taking two 8-byte ones (`E0104`, as for a C symbol defined twice).

import helpers.exported
import helpers.declared

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @safety("`ffi30_add` adds two `i32`s")
    unsafe fn ffi30_add(a: i32, b: i32) -> i32

fn main():
    println(exported.ffi30_add(2, 3))
