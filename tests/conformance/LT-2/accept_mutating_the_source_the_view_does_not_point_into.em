#$ test: run-pass
#$ rules: LT-1, LT-22, BRW-1
#$ stdout: 1
# `[LT-1]`, `[LT-22]` (verdict 137) — `v` points only into `p`, since `pick` returns its first
# argument, and the compiler reads that from `pick`'s code: changing `q` while `v` is used is no
# conflict (0.9.10 took the result to point into both, the intersection, and refused it).

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
