#$ test: compile-fail
#$ rules: LT-1, SPN-1
# `[LT-1]` rule 3's force: the result borrows ALL view-typed parameters, so
# mutating `q` while `v` is live is rejected — the same shape as SPN-1's
# `as_span` case (`E3021`), with the loan starting at the call (`pick` may
# return either argument, so the caller treats it as borrowing both).

fn pick(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a

fn main():
    p: Array[i32] = Array[i32]()
    p.push(1)
    q: Array[i32] = Array[i32]()
    q.push(2)
    v = pick(p, q)
    q.push(3)              #$ error[E3021]: `q` is borrowed here and mutably borrowed elsewhere
    println(v[0])
