#$ test: parse-pass
#$ rules: LT-2, LT-1
# "Constructing a view struct from several references gives it the
# **intersection** of their regions." The same rule governs a view returned
# from a call: `pick` returns a `Span` and `[LT-1]`'s elision ties it to every
# view parameter, so the result borrows `p` *and* `q`.
#
# This file is the accept half — it parses and type-checks; the reject half
# beside it is what proves the intersection is real.

fn pick(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a

fn main():
    p: Array[i32] = Array[i32]()
    p.push(1)
    q: Array[i32] = Array[i32]()
    q.push(2)
    v = pick(p, q)
    println(v[0])
