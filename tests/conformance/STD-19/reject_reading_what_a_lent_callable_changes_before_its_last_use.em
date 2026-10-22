#$ test: compile-fail
#$ rules: STD-19, BRW-2, CLO-2
## `[STD-19]` (ODR-240) — a named callable that changes what it captured is lent to each std
## function it is passed to, and what it captured stays lent while it is still used: reading
## `looked` between two calls that lend `tall` is `E3021`, as is reading `mapped` while an adapter
## holding `double` is still to be used.

fn main():
    xs = [3, 1, 4]
    looked = 0
    tall = fn(x: ref int) -> bool:
        looked += 1
        return x > 3
    a = xs.iter().any(tall)
    println(looked)  #$ error[E3021]: `looked` cannot be read while it is mutably borrowed
    b = xs.iter().any(tall)
    mapped = 0
    double = fn(x: ref int) -> int:
        mapped += 1
        return x * 2
    doubled = xs.iter().map(double)
    println(mapped)  #$ error[E3021]: `mapped` cannot be read while it is mutably borrowed
    println(a, b, doubled.sum())
