#$ test: compile-fail
#$ rules: CTL-0
# `[CTL-0]` — the condition of `if`, `elif`, `while` and a match guard MUST be a `bool`; there is no
# truthiness. `E2035` carries a type-directed fix-it: `not xs.is_empty()` for a container,
# `x is not None` for an `Option`, `x != 0` for a number.

fn main():
    n: i32 = 3
    xs: Array[i32] = Array[i32]()
    if n:                           #$ error[E2035]: condition must be `bool`
        println(n)
    elif xs:                        #$ error[E2035]: condition must be `bool`
        println(1)
    while n:                        #$ error[E2035]: condition must be `bool`
        n = n - 1
