#$ test: compile-fail
#$ rules: OWN-4
# `[OWN-4]`'s quantifier: reassigned "before the next iteration on EVERY path".
# The `if` branch puts the value back; the `else` branch moves without doing
# so, and that move is `E3040`.

fn consume(owns xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(owns 1)
    i: i32 = 0
    while i < 2:
        if i == 0:
            println(consume(owns a))
            a = Array[i32]()
            a.push(owns 9)
        else:
            println(consume(owns a))   #$ error[E3040]: `a` is handed over on line 20 and the loop comes round again, so the next round finds it gone; give it a new value before the next round (`a = …`), declare it inside the loop, or hand over a copy of your own: `copy of a`
        i = i + 1
