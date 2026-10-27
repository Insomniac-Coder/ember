#$ test: run-pass
#$ rules: CLO-3, FN-6
#$ profiles: debug, development, release
#$ stdout: 31 8
#$ stdout: 34 9
#$ assert-c-count: contains("em_5applyG8a771c17434ecee2(") == 4
#$ assert-c-count: contains("int64_t em_5applyG") == 2
# `[CLO-3]` — a callable parameter is an implicit generic: each function
# passed to it gets `apply`'s own copy, in which the call through the
# parameter is a direct call to that function (one the C compiler can
# inline). `triple` is passed twice, so its copy stays a function and shows
# in the C: declared, defined, and called twice. Each lambda is a function of
# its own, passed once: its copy is inlined where it is called and not emitted.
# (`[MNG-1]`: each copy is an instance of `apply`, `em_5applyG` and its hash. Since ODR-278 a
# function's argument is its own type, `triple`'s, so its copy is the only one declared and
# defined; before, `triple` and the lambdas were one type, `fn(int) -> int`, and `apply` had a copy
# for that type as well.)

fn apply(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn triple(x: int) -> int:
    return x * 3 + 1

fn main():
    println(apply(triple, 10), apply(fn(x) => x - 2, 10))
    println(apply(triple, 11), apply(fn(x) => x - 2, 11))
