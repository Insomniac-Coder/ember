#$ test: compile-fail
#$ rules: LEX-13
## `[LEX-13]` — a lone `_` discards and never names a variable.

fn pair() -> (int, int):
    return (1, 2)

fn main():
    (a, _) = pair()
    println(a)
    println(_)    #$ error[E1010]: cannot find `_` in this scope
