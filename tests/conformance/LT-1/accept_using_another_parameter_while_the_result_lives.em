#$ test: run-pass
#$ rules: LT-1, LT-22, SPN-1
#$ stdout: 1
# `[LT-1]`, `[LT-22]` (verdict 137) — the caller keeps viewed exactly the arguments the called
# function's code shows the result points into: `pick` returns `a`, so `v` views `p` only, and
# changing `q` while `v` lives is no conflict. Changing `p` there still is (`E3021`, `SPN-1`).

fn pick(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a

fn main():
    p: Array[i32] = Array[i32]()
    p.push(1)
    q: Array[i32] = Array[i32]()
    q.push(2)
    v = pick(p, q)
    q.push(3)
    println(v[0])
