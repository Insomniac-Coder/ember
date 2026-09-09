#$ test: compile-fail
#$ rules: OWN-3
# `[DIA-7a]` keys `E3040` to shape O1, "use after move".

fn take(xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    b = a
    println(take(b))
    println(take(a))       #$ error[E3040]: `a` has been moved out of
