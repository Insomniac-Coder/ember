#$ test: compile-fail
#$ rules: SIMD-2
# `[SIMD-2]` — `@simd` is a request, `@simd(assert)` one whose failure is an
# error; there is no other form.

fn scale(mut xs: MutSpan[int]):
    @simd(fast)  #$ error[E0104]: `@simd` takes no argument but `assert`
    for i in 0..xs.len():
        xs[i] *= 2

fn main():
    pass
