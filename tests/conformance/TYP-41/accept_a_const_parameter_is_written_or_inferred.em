#$ test: run-pass
#$ rules: TYP-41, TYP-18
#$ stdout: 9
#$ stdout: 12
#$ stdout: 3
## `[TYP-41]` — a `const` parameter's argument is written in brackets
## (`zeros[3]()`), or inferred from an argument's length, and inside the body
## `N` is its value.

[const N: int]
fn total(xs: [int; N]) -> int:
    sum = 0
    for x in xs:
        sum += x
    return sum + N

[const N: int]
fn zeros() -> [f32; N]:
    return [0.0; N]

fn main():
    println(total([1, 2, 3]))
    println(total[2]([5, 5]))
    println(len(zeros[3]()))
