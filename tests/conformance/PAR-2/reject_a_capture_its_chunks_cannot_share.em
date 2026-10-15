#$ test: compile-fail
#$ rules: PAR-2
#$ error[E7005]: the parallel loop's chunks borrow `cell`, a `Shared[i64]`, which is not Sync
# `[PAR-2]` — a `@parallel` loop's captures are checked as a task's are: every chunk reads them at
# once, on several threads, so what they borrow is `Sync`.

fn spread(mut xs: MutSpan[int]):
    cell = Shared(3)
    @parallel
    for i in 0..xs.len():
        xs[i] = cell.get()

fn main():
    pass
