#$ test: run-pass
#$ rules: GRM-40
#$ stdout: 2 3
## `[GRM-40]` — a line that begins with `[` gives type parameters; a list begins a value after a
## name (`n = [10, 20].len()` is the way), and a comment at the end of the bracket line is that line's.

[T]    # the element type
fn first(xs: Array[T]) -> T:
    return xs[0]

fn main():
    n = [10, 20].len()
    println(n, first([3, 4]))
