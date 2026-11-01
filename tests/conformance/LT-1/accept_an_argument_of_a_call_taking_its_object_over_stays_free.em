#$ test: run-pass
#$ rules: LT-1
#$ stdout: [1, 2]
# `[LT-1]` rule 1 covers every method, `owns this` too: the result points into `this` (into what it
# holds, `xs`), so `ys` is free while `s` lives.

@view
struct V:
    r: Span[int]

    fn m(a: Array[int]) owns this -> Span[int]:
        return this.r

fn main():
    xs = [1, 2]
    ys = [3]
    v = V(r=xs)
    s = v.m(ys)
    ys.push(owns 4)
    println(s)
