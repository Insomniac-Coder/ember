# Clause LT-35.1: "A call through a function value, or through an interface, `dyn` or virtual
# method, counts every function the program can put there (`[EFF-1]`)"
#$ test: compile-fail
#$ rules: LT-35, EFF-1, EFF-2
# `f` holds `left_of` when it is called, but the program also puts `right_of` there, which reads
# `p.right`; after `right` is pushed to, the call needs the region `p.right` had. No text in the
# spec: code only. The foreign-function case (every field used and kept) is M12's: not here.

struct Pair:
    left: Span[int]
    right: Span[int]

fn left_of(p: Pair) -> int:
    return p.left[0]

fn right_of(p: Pair) -> int:
    return p.right[0]

fn main():
    left = [7]
    right = [8]
    p = Pair(left=left, right=right)
    g: fn(Pair) -> int = right_of
    println(g(p))
    f: fn(Pair) -> int = left_of
    right.push(owns 9)
    f = g
    println(f(p))    #$ error[E3060]
