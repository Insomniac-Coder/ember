#$ test: run-pass
#$ rules: FFI-16, ERR-6, FFI-11, FFI-8
#$ profiles: debug, release
#$ stdout: made 6
#$ stdout: failed Status.NoMemory true
#$ stdout: failed Status(7) StatusError(code=Status(7))
#$ stdout: checked
#$ stdout: run Status.Busy
#$ stdout: pair 4 9
# `[FFI-16]`, `[ERR-6]` — a C status enum, declared by hand as C maps it (`@repr`,
# `@non_exhaustive`, `[FFI-8]`), marked `@ffi(status=Status, ok=Success,
# error=StatusError)`: the compiler declares `struct StatusError(code: Status)`
# implementing `Error`, and each function contracted `result(status)` returns
# `Result[T, StatusError]`, `T` its `out` parameters: none (`void`), one (its
# type) or several (a tuple, in C's order) (ODR-137). An `out` parameter is
# written `mut p` at its C position (C's `T*`) and leaves the callable
# signature (`[FFI-11]`). A code that names no variant still arrives: the enum
# is non-exhaustive. The C side is the exported Ember functions below, which
# return C's raw `int`s.

@repr(i32)
@non_exhaustive
@ffi(status=Status, ok=Success, error=StatusError)
enum Status:
    Success = 0
    NoMemory = -1
    Busy = 2

unsafe extern "C":  # SAFETY(ffi): each declaration below matches its C definition
    @ffi(param(out, out, one), result(status), link_name="make_c")
    safe fn make(n: i32, mut out: i32) -> Status

    @ffi(result(status), link_name="check_c")
    safe fn check(n: i32) -> Status

    @ffi(param(a, out, one), param(b, out, one), result(status), link_name="pair_c")
    safe fn pair(mut a: i32, n: i32, mut b: i64) -> Status

pub extern "C" fn make_c(n: i32, out: *mut i32) -> i32:
    if n < 0:
        return -1
    if n > 100:
        return 7
    unsafe:  # SAFETY(ffi): C's contract: `out` is valid
        write(out, 0, n * 2)
    return 0

pub extern "C" fn check_c(n: i32) -> i32:
    if n == 6:
        return 0
    return 2

pub extern "C" fn pair_c(a: *mut i32, n: i32, b: *mut i64) -> i32:
    unsafe:  # SAFETY(ffi): C's contract: both are valid
        write(a, 0, n * n)
        write(b, 0, (n + 1) as i64 * (n + 1) as i64)
    return 0

fn run(n: i32) -> Result[void, StatusError]:
    made = make(n)?
    check(made)?
    return Ok(())

fn main():
    match make(3):
        Ok(v):
            println("made", v)
        Err(e):
            println("failed", e)
    match make(-5):
        Ok(v):
            println("made", v)
        Err(e):
            println("failed", e, e.code == Status.NoMemory)
    match make(500):
        Ok(v):
            println("made", v)
        Err(e):
            println("failed", e, f"{e:?}")
    match run(3):
        Ok(_):
            println("checked")
        Err(e):
            println("run", e)
    match run(4):
        Ok(_):
            println("checked")
        Err(e):
            println("run", e)
    match pair(2):
        Ok((a, b)):
            println("pair", a, b)
        Err(e):
            println("failed", e)
