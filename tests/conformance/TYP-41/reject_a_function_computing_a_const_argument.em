#$ test: compile-fail
#$ rules: TYP-41, CT-1
## `[TYP-41]` — "a function may compute one", which needs the compile-time
## evaluator (`[CT-1]`), not built yet: refused, never ignored.

fn square(x: int) -> int:
    return x * x

fn main():
    grid: [int; square(3)] = [0; 9]    #$ error[E0900]: a function computing a const argument is not supported yet
