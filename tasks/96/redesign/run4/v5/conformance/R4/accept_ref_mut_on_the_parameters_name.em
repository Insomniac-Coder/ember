#$ test: run-pass
#$ rules: FN-1
#$ stdout: 2
## design rules: R4
## R4: the mode goes on the parameter's name (`ref mut n: int`), and the call writes the same words;
## `n += 1` reaches the caller's `a` (AN-27).

fn bump(ref mut n: int):
    n += 1

fn main():
    a = 1
    bump(ref mut a)
    println(a)
