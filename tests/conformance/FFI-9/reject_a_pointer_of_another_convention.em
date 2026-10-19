#$ test: compile-fail
#$ rules: FFI-9
#$ profiles: debug
# `[FFI-9]` — the convention is part of a function pointer's type: a `"system"`
# function is not an `extern "C" fn` (on 32-bit Windows they are called
# differently), and a name that is no convention is refused.

pub extern "system" fn add(a: i32, b: i32) -> i32:
    return a + b

pub extern "pascal" fn sub(a: i32, b: i32) -> i32: #$ error[E0104]: `extern "pascal"` is not a calling convention
    return a - b

fn main():
    f: extern "C" fn(i32, i32) -> i32 = add #$ error[E2020]: expected `extern "C" fn(i32, i32) -> i32`, found `extern "system" fn(i32, i32) -> i32`
    g: extern "cdecl" fn(i32) -> i32 #$ error[E0104]: `extern "cdecl"` is not a calling convention
    println(f(1, 2))
