#$ test: compile-fail
#$ rules: TYP-41
## `[TYP-41]` rule 1 — each instance computes its arguments again; one that
## divides by zero is `E6004` at the use that makes that instance, never a
## type mismatch.

fn bad[const N: int]() -> [int; N // (N - 3)]:
    return [0; N // (N - 3)]

fn main():
    y = bad[5]()
    println(len(y))
    x = bad[3]()    #$ error[E6004]: a const argument of `bad`'s signature divides by zero with `N` = 3
