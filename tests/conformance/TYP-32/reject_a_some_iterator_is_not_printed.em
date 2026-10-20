#$ test: compile-fail
#$ rules: TYP-32
#$ error[E2040]: `some Iterator[Item = i64]` does not implement `Debug`, which printing it needs
# `[TYP-32]` — a `some Iterator` is an iterator and nothing more: what it holds could be printed,
# but its interface gives it no `Debug`.

fn evens(xs: Span[int]) -> some Iterator[Item = int]:
    return xs.iter().copied().filter(fn(x) => x % 2 == 0)

fn main():
    a = [1, 2]
    println(evens(a.as_span()))
