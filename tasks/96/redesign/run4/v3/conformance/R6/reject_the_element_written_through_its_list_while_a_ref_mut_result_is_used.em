#$ test: compile-fail
#$ rules: R6, R8
#$ help: f = 5
## R6, R2: a name bound to a call whose result is `ref mut T` is a `ref mut` name of that place, and it
## is alone while it is used (R8 (f); AN-23, E3022).

fn first(ref mut xs: Array[int]) -> ref mut int:
    return ref mut xs[0]

fn main():
    xs = [1, 2, 3]
    f = ref mut first(ref mut xs)
    xs[0] = 5    #$ error[E3022]: `f` is the only name for `xs[0]` until its last use on line 14, and line 13 writes `xs[0]`; write it through `f`: `f = 5`
    f += 1
    println(xs[0])
