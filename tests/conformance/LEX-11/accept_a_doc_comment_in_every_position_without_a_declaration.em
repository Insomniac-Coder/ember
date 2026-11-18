#$ test: run-pass
#$ rules: LEX-11
#$ stdout: 12
## `[LEX-11]` - a `##` comment in every position that has no declaration after it (before a variant, a
## field, an interface method, inside brackets, before `else`, before a match arm, at the end of a block)
## is discarded in silence and changes nothing.

enum Color:
    ## before a variant
    Red
    ## another
    Green

struct P:
    ## before a field
    x: int
    y: int  ## trailing

interface Shape:
    ## before a method
    fn area() -> int

fn f(a: int,
     ## inside brackets
     b: int) -> int:
    xs = [
        ## inside a list
        1,
        2,
    ]
    if a > 0:
        ## before a statement
        return b
    ## before else
    else:
        ## in else
        return xs[0]

fn g(x: int) -> int:
    r = match x:
        ## before an arm
        1 => 10
        ## another
        _ => 20
    return r

fn main():
    ## before while
    while false:
        ## in while
        pass
    try_it = f(1, 2)
    ## last statement doc
    println(try_it + g(1))
    ## before a nested fn
    fn inner():
        pass
    inner()
