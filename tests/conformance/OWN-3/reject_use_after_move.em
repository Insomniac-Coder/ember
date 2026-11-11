#$ test: compile-fail
#$ rules: OWN-3
# `[DIA-7a]` keys `E3040` to shape O1, "use after move".

fn take(owns xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(owns 1)
    b = a
    println(take(owns b))
    println(take(owns a))       #$ error[E3040]: `a` was handed over to `b` on line 11 and is used on line 13; to keep both, write `b = copy of a` (a copy of your own) or `b = shares a` (two owners of one value)
