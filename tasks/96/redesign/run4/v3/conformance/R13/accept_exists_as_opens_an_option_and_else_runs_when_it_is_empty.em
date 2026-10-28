#$ test: run-pass
#$ rules: R13
#$ stdout: 4
#$ none
## R13: `if e exists as v:` runs its block when the `Option` holds a value and names it `v` there;
## `else:` runs when it holds nothing (verdict 26).

fn first_even(xs: Array[int]) -> Option[int]:
    for x in xs:
        if x % 2 == 0:
            return Some(x)
    return None

fn show(xs: Array[int]):
    if first_even(xs) exists as v:
        println(v)
    else:
        println("none")

fn main():
    show([1, 4, 6])
    show([1, 3])
