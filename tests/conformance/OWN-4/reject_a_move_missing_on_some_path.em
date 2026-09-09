#$ test: compile-fail
#$ rules: OWN-4
# `[OWN-4]`'s quantifier: reassigned "before the next iteration on EVERY path".
# The `if` branch puts the value back; the `else` branch moves without doing
# so, and that move is `E3041`.

fn consume(owned xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    i: i32 = 0
    while i < 2:
        if i == 0:
            println(consume(a))
            a = Array[i32]()
            a.push(9)
        else:
            println(consume(a))   #$ error[E3041]: `a` is moved in a loop
        i = i + 1
