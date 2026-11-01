#$ test: run-pass
#$ rules: LT-1a, LT-1
# "`@borrows(p₁, …, pₙ)` … overrides the region that rules 1–3 would
# assign." Without it `[LT-1]` rule 3 ties the result to *every* view
# parameter, so `q` would stay borrowed; `@borrows(a)` says the result comes
# from `a` alone, and `q` is free the moment the call returns.

fn first(a: Span[i32], b: Span[i32]) -> Span[i32] from a:
    return a

fn main():
    p: Array[i32] = Array[i32]()
    p.push(owns 1)
    q: Array[i32] = Array[i32]()
    q.push(owns 2)
    v = first(p, q)
    q.push(owns 3)
    println(v[0])
#$ stdout: 1
