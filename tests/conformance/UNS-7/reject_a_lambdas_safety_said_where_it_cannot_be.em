#$ test: compile-fail
#$ rules: UNS-7, UNS-12
# `[UNS-7]` (verdict 126) — a lambda written in unsafe code is an `unsafe fn(…)` value, and says what
# its callers must make true with `@safety` written just before it; one without it is `L3015`. A
# lambda written outside unsafe code is safe to call, and `@safety` on it is `E0104`.

fn main():
    values: Array[int] = [1, 2]
    p = values.as_mut_span().as_mut_ptr()
    # SAFETY: the lambda writes through `p`, which points to `values[0]`
    unsafe:
        set = fn(v: int) => p.write(v)    #$ warning[L3015]: this lambda is written in unsafe code, so it is an `unsafe fn(…)` value
        set(3)
    twice = @safety("nothing") fn(v: int) => v * 2    #$ error[E0104]: `@safety` applies only to an `unsafe fn`
    println(twice(values[0]))
