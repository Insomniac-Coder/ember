#$ test: run-pass
#$ rules: FN-1, GRM-41
#$ stdout: 4
# `[FN-1]` — only the top of a parameter's type is refused; a `ref` inside one is a view type.

fn first(ref xs: Array[int]) -> Option[ref int]:
    return xs.get(0)

fn show(o: Option[ref int]):
    match o:
        Some(v):
            println(v)
        None:
            println("none")

fn main():
    xs = [4, 5]
    show(first(xs))
