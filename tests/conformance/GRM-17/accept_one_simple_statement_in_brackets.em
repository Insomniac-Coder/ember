#$ test: run-pass
#$ rules: GRM-17, LEX-6a, STD-19, CLO-2
#$ profiles: debug, development, release
#$ stdout: 6
#$ 3 3
#$ 12
#$ 1
#$ 2
#$ 3
#$ 8
## `[GRM-17]`, `[LEX-6a]` — inside brackets a lambda's `:` body is one simple statement: an
## augmented assignment, an assignment to an index or a name, `pass` or an expression, ended by
## the closing bracket or a `,` (`f(fn(i): total += i, 4)` passes two arguments). It was `E0106`
## for every assignment (D-821). `for_each` is given a lambda written at the call, as an adapter is
## (ODR-240; `feed` here takes it over), so the lambda may write what it captures.

fn show(x: int):
    println(x)

fn apply(f: fn(i32) -> i32, v: i32) -> i32:
    return f(v)

fn feed[F: fn(int)](owned f: F, n: int):
    for i in range(n):
        f(i)

fn main():
    xs = [1, 2, 3]
    total = 0
    xs.iter().for_each(fn(x): total += x)
    println(total)
    seen: Map[int, bool] = {}
    xs.iter().for_each(fn(x): seen[x] = true)
    last = 0
    xs.iter().for_each(fn(x): last = x)
    println(seen.len(), last)
    xs.iter().for_each(fn(x): pass)
    feed(fn(i): total += i, 4)
    println(total)
    xs.iter().for_each(show)
    println(apply(fn(x): x * 2, 4))
