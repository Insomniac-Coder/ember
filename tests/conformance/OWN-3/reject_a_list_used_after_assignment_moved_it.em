#$ test: compile-fail
#$ rules: OWN-3, BRW-7
# `[OWN-3]` (ODR-114) — XVII.5's example: `b = a` moves the list, so `a.push(4)`, a borrow of `a`, is a
# use after the move, `E3040`, with the note that Python would have made both names one list.

fn main():
    a = [1, 2, 3]
    b = a
    a.push(owns 4)          #$ error[E3040]: `a` was handed over to `b` on line 8 and is used on line 9; to keep both, write `b = copy of a` (a copy of your own) or `b = shares a` (two owners of one value)
    println(b)
