#$ test: run-pass
#$ rules: TYP-41, CT-1
#$ stdout: 9 0 16 0 [1, 4, 9]
## `[TYP-41]` — "a function may compute one": a const argument that calls a
## function is evaluated while compiling (`[CT-1]`), and two types whose
## arguments evaluate to the same value are one type.

fn square(x: int) -> int:
    return x * x

fn side() -> int:
    return 4

fn main():
    grid: [int; square(3)] = [0; 9]
    same: [int; 9] = grid
    big: [int; square(side())] = [0; 16]
    small: [int; 3] = [1, 4, 9]
    println(len(grid), same[8], len(big), big[15], small)
