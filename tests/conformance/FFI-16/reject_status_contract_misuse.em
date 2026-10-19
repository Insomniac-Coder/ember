#$ test: compile-fail
#$ rules: FFI-16, FFI-11
# `[FFI-16]` — `@ffi(status=E, ok=X, error=N)` is written on the C enum `E`
# itself, a `@repr` `@non_exhaustive` unit-only enum, `X` one of its variants;
# `result(status)` needs that enum as the declared result (ODR-137).

@repr(i32)
@non_exhaustive
@ffi(status=Code, ok=Fine, error=CodeError)  #$ error[E0104]
enum Code:
    Good = 0
    Bad = 1

@repr(i32)
@non_exhaustive
@ffi(status=Other, ok=Good, error=OtherError)  #$ error[E0104]
enum Other2:
    Good = 0

@ffi(status=Plain, ok=Good, error=PlainError)  #$ error[E0104]
enum Plain:
    Good
    Bad

@repr(i32)
@non_exhaustive
@ffi(status=Real, ok=Good, error=RealError)
enum Real:
    Good = 0
    Bad = 1

unsafe extern "C":
    @ffi(result(status))  #$ error[E0104]
    safe fn plain_result(n: i32) -> i32

    @ffi(param(v, out))  #$ error[E5012]
    safe fn no_count(mut v: i32) -> Real

fn main():
    println(plain_result(1))
