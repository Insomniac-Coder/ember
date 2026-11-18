#$ test: run-pass
#$ rules: GRM-5, GRM-29, LEX-13
#$ stdout: 2 1
#$ stdout: 3 1
#$ stdout: 7 5
#$ stdout: 4 5 6
## `[GRM-5]` — `a, b = b, a` swaps two names in scope; `q, r = divmod(7, 2)` declares both; `_` discards its
## part; a fixed array is destructured.

fn divmod(a: int, b: int) -> (int, int):
    return a // b, a % b

fn main():
    a = 1
    b = 2
    a, b = b, a
    println(a, b)
    q, r = divmod(7, 2)
    println(q, r)
    _, s = divmod(15, 7)
    keep, _ = (7, 99)
    println(keep, s + 4)
    fixed: [int; 3] = [4, 5, 6]
    x, y, z = fixed
    println(x, y, z)
