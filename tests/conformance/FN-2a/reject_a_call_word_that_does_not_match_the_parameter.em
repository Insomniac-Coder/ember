#$ test: compile-fail
#$ rules: FN-2a
# `[FN-2a]` — a call says `ref mut` before an argument the callee may change, with the same words as
# the parameter: a missing `ref mut`, or one written where the parameter does not say it, is `E3028`,
# in the program's own names.
#$ help: say so at the call: `bump(ref mut a)`
#$ help: remove `ref mut`: `show(a)`

struct Counter:
    n: int

fn bump(ref mut c: Counter):
    c.n += 1

fn show(c: Counter):
    println(c.n)

fn main():
    a = Counter(n=1)
    bump(a)    #$ error[E3028]: `bump` may change `a` (its first line says `ref mut c: Counter`)
    show(ref mut a)    #$ error[E3028]: `show` only reads its argument
