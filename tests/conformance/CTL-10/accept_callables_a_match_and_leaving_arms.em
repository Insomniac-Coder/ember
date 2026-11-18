#$ test: run-pass
#$ rules: CTL-10, CLO-1
#$ stdout: 10
#$ 15
#$ one
#$ 2
#$ 0
#$ 2
## `[CTL-10]` - a name declared in every arm of an `if`/`elif`/`else` or an exhaustive `match` that
## completes normally is declared after the statement: at the one type, or where the arms give it
## different functions, at the callable type they convert to; an arm that ends in `panic` or
## `continue` need not declare it.

fn double(x: int) -> int:
    return x * 2

fn triple(x: int) -> int:
    return x * 3

fn pick(n: int):
    if n == 0:
        f = double
    elif n == 1:
        f = triple
    else:
        f = double
    println(f(5))

fn m(n: int):
    match n:
        0:
            label = "zero"
        1:
            label = "one"
        _:
            label = "many"
    println(label)

fn boom(n: int):
    if n > 0:
        v = n
    else:
        panic("neg")
    println(v)

fn loop_it():
    for i in 0..3:
        if i == 1:
            continue
        else:
            w = i
        println(w)

fn main():
    pick(0)
    pick(1)
    m(1)
    boom(2)
    loop_it()
