#$ test: compile-fail
#$ rules: PAR-4, EFF-5
#$ error[E4002]: `@nosync` function `doubles` reaches a synchronising operation
#$ error[E4001]: `@noalloc` function `pushes` reaches an allocation
# `[PAR-4]` — a `@parallel` loop submits its chunks to the job system, so it synchronises and
# `@nosync` refuses it; what its body does counts as the function's own, so an allocation in it
# breaks `@noalloc`.

@nosync
fn doubles(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len():
        xs[i] = xs[i] * 2

@noalloc
fn pushes(mut xs: MutSpan[int]):
    @parallel
    for i in 0..xs.len():
        names: Array[int] = [i]
        xs[i] = names[0]

fn main():
    pass
