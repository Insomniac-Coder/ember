#$ test: compile-fail
#$ rules: CLO-4, CLO-2, BRW-1
# "A closure outlives what it captures" is checked by the ordinary borrow
# rules and needs no closure-specific analysis. `[CLO-2]` makes a read-only
# capture a **shared borrow**, so the environment holds a real loan of `m` —
# and `push` wanting `m` mutably while that loan is live is `[BRW-1]`, found
# by the same machinery that finds it anywhere else.

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn main():
    m: Array[i32] = Array[i32]()
    m.push(1)
    g = fn(x: i32) => x + m[0]
    m.push(2)              #$ error[E3021]: `m` is borrowed here and mutably borrowed elsewhere
    println(apply(g, 10))
