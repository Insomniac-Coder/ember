#$ test: run-pass
#$ rules: FFI-8
#$ profiles: debug, development, release
#$ stdout: 42 42
#$ stdout: -7
#$ stdout: 2.5
#$ stdout: 3
# `[FFI-8]` (ODR-249) — a newtype's value is its representation's bits: copied as they are, kept in
# a struct field, an array and an `Option`, returned, and converted back with `as` to the same value
# (the compiled program and the evaluator alike).

from std.ffi import c_int

@ffi(newtype)
type fd_t = c_int

@ffi(newtype)
type gain_t = f64

@layout(c)
@derive(Copy)
struct Opened:
    fd: fd_t
    gain: gain_t

fn reopen(opened: Opened) -> fd_t:
    return opened.fd

fn main():
    fd = 42 as fd_t
    copy = fd
    opened = Opened(copy, 2.5 as gain_t)
    println(reopen(opened) as c_int, fd as c_int)
    held: Option[fd_t] = Some(-7 as fd_t)
    match held:
        Some(inner):
            println(inner as c_int)
        None:
            println("none")
    println(opened.gain as f64)
    several: [fd_t; 3] = [1 as fd_t, 2 as fd_t, 3 as fd_t]
    println(several[2] as c_int)
