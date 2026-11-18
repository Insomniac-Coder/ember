#$ test: run-pass
#$ rules: LT-2, LT-5, TYP-14
#$ stdout: 9
#$ stdout: 7

## The legal side of `a_view_struct_carries_its_region.em`: the struct's region
## ends where the last use of it does, and the owner is free again after that.

@view
struct Cursor:
    at: ref String

fn hand_back(ref a: String) -> Cursor:
    return Cursor(a)

fn main():
    k: String = "4444".to_string()
    c: Cursor = hand_back(ref k)
    v: int = c.at.len()
    ## `c` is finished with, so `k` is the owner's again.
    k = "55555".to_string()
    println(v + k.len())

    ## A view struct built and read inside one expression borrows for no
    ## longer than the expression.
    n: String = "7".to_string()
    println(Cursor(ref n).at)
