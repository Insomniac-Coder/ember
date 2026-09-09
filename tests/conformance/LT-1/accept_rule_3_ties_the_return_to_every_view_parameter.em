#$ test: run-pass
#$ rules: LT-1
# Rule 3: "the return's region is the **intersection** of all view-typed
# parameters' regions (the returned reference may point into any of them; the
# caller treats it as borrowing all of them)."
#
# `pick` returns its first argument and the caller must still treat `q` as
# borrowed, because nothing in the signature says which one came back. That is
# what `@borrows` exists to narrow, and `tests/conformance/LT-1a/` is the pair
# to this one.

fn pick(a: Span[i32], b: Span[i32]) -> Span[i32]:
    return a

fn main():
    p: Array[i32] = Array[i32]()
    p.push(1)
    q: Array[i32] = Array[i32]()
    q.push(2)
    v = pick(p, q)
    println(v[0])
#$ stdout: 1
