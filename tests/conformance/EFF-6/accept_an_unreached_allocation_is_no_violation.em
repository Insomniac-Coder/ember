#$ test: run-pass
#$ rules: EFF-6
#$ stdout: 4
# `[EFF-6]` — only what the function reaches counts: `grow` allocates, but `measure` never calls
# it.

fn grow(mut xs: Array[int]):
    xs.push(0)

@noalloc
fn measure(xs: Span[int]) -> int:
    return xs.len()

fn main():
    values = [1, 2, 3]
    grow(values)
    println(measure(values.as_span()))
