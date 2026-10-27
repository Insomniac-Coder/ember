#$ test: compile-fail
#$ rules: R4, R8
#$ help: write `n = xs[i]` above line 15, `grow(ref mut xs, ref mut n)` in place of line 15, and `xs[i] = n` below it
## R4, R8 (f): verdict 88 allows one whole variable under two names, and only that: a part whose position
## is not known when compiling, given beside its whole list, stays refused (E3022), since `grow` may
## reshape `a` while `b` points into it (the `LOG[0]` problem). The help gives `grow` a number of its own
## and writes it back after the call.

fn grow(ref mut a: Array[int], ref mut b: int):
    a.push(0)
    b += 1

fn bump_at(i: int):
    xs = [1, 2, 3]
    grow(ref mut xs, ref mut xs[i])    #$ error[E3022]: `grow` changes `a` and `b` as two different places, and `b` is `xs[i]`, part of `xs`, which `a` names
    println(xs[i])

fn main():
    bump_at(1)
