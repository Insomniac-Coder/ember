#$ test: compile-fail
#$ rules: FFI-8
#$ profiles: debug
# `[FFI-8]` (ODR-249) — a newtype is nominal: neither it nor its typedef's representation becomes
# the other except by `as`, which converts between those two only (another type goes through the
# representation, `E2020` naming it), and it has no arithmetic of its own. Its representation is a
# C typedef's target (`E5015` otherwise), and `@ffi` on a type alias takes `newtype` alone, on a
# plain alias (`E0104`).

from std.ffi import c_int

@ffi(newtype)
type fd_t = c_int

@ffi(newtype)
type handle_t = c_int

@ffi(newtype)
type name_t = String    #$ error[E5015]: `String` is no C typedef's target

@ffi(newtype, packed)    #$ error[E0104]: `@ffi` on a type alias takes `newtype` alone
type odd_t = c_int

@ffi(newtype)    #$ error[E0104]: `@ffi(newtype)` makes a plain alias of a C type nominal
type percent_t = c_int in 0 ..= 100

fn take(fd: fd_t) -> c_int:
    return fd as c_int

fn main():
    raw: c_int = 3
    take(raw)    #$ error[E2020]: expected `fd_t`, found `i32`
    fd = raw as fd_t
    back: c_int = fd    #$ error[E2020]: expected `i32`, found `fd_t`
    wide = fd as i64    #$ error[E2020]: `fd_t` cannot be cast to `i64` with `as`
    #$ help: convert to `i32` first: `(x as i32) as i64`
    other = fd as handle_t    #$ error[E2020]: `fd_t` cannot be cast to `handle_t` with `as`
    next = fd + 1    #$ error[E2020]
    println(take(fd))
