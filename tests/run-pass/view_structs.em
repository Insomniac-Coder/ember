#$ test: run-pass
#$ rules: LT-2, LT-5, TYP-14
#$ stdout: 9
#$ stdout: 7

## The legal side of `a_view_struct_carries_its_region.em`: the struct's region
## ends where the last use of it does, and the owner is free again after that.

@view
struct Cursor:
    at: ref i32

fn hand_back(a: ref i32) -> Cursor:
    return Cursor(a)

fn main():
    k: i32 = 4
    c: Cursor = hand_back(ref k)
    v: i32 = c.at
    ## `c` is finished with, so `k` is the owner's again.
    k = 5
    println(v + k)

    ## A view struct built and read inside one expression borrows for no
    ## longer than the expression.
    n: i32 = 7
    println(Cursor(ref n).at)
