#$ test: compile-fail
#$ rules: TYP-26
# `[TYP-26]` — there is no overloading: two functions of one name in one scope are `E1030`.

fn size(x: int) -> int:
    return x

fn size(x: String) -> int:         #$ error[E1030]: `size` is already declared
    return 1

fn main():
    println(size(1))
