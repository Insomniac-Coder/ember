#$ test: run-pass
#$ rules: LT-1
#$ stdout: [3]
# `[LT-1]` rule 1 is for a borrowed receiver. An `owned self` view is one
# source parameter among the rest, so rule 3 lets the result borrow `a`.

@view
struct V:
    r: Span[int]

    fn m(owned self, a: Array[int]) -> Span[int]:
        return a

fn main():
    xs = [1, 2]
    ys = [3]
    v = V(r=xs)
    println(v.m(ys))
