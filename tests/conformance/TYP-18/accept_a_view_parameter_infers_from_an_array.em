#$ test: run-pass
#$ rules: TYP-18, TYP-5, SPN-1
#$ stdout: 5 1.5
#$ stdout: [15, 16]
# D-408 — an `Array[T]` or a `[T; N]` argument for a `Span[T]` or `MutSpan[T]`
# parameter becomes a view of its elements (`[TYP-5]` rule 6), so the generic
# argument is read off the element type. `pick(ys)` was `E2060`, cannot tell
# what `T` is.

fn pick[T: Copy](xs: Span[T]) -> T:
    return xs[0]

fn bump[T: Copy + Add[T, Output = T]](mut xs: MutSpan[T], by: T):
    for i in range(xs.len()):
        xs[i] = xs[i] + by

fn main():
    ys = [5, 6]
    fixed: [f64; 2] = [1.5, 2.5]
    println(pick(ys), pick(fixed))
    bump(ys, 10)
    println(ys)
