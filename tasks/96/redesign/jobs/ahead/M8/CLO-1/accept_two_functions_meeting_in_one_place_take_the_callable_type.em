# Clause CLO-1.1: "Where two different functions meet in one place (`if c: f = a else: f = b`, a
# second assignment, a conditional's two branches, a collection's elements), the place has the
# callable type both convert to."
#$ test: run-pass
#$ rules: CLO-1, CLO-3, CLO-10
#$ stdout: 4 6
#$ stdout: 9
#$ stdout: 2
#$ stdout: 3
#$ stdout: 2
# Two named functions meet in an `if`, a second assignment and a list, with a lambda among them.

fn double(x: int) -> int:
    return x * 2

fn triple(x: int) -> int:
    return x * 3

fn pick(c: bool) -> fn(int) -> int:
    if c:
        f = double
    else:
        f = triple
    return f

fn main():
    println(pick(true)(2), pick(false)(2))
    g = double
    g = triple
    println(g(3))
    fs = [double, triple]
    fs.push(owns fn(x: int) -> int => x + 1)
    for h in fs:
        println(h(1))
