#$ test: run-pass
#$ rules: STD-19, CLO-2, CLO-15
#$ profiles: debug, development, release
#$ stdout: true 3 false 2
#$ Some(4) 3 Some(2) 3
#$ 14 5 Some(14) 4
#$ Some(5) Some(1) 10
#$ Some(5) Some(1) 10
## `[STD-19]` (ODR-210) — the consumers that take a callable (`any`, `all`, `find`, `position`,
## `fold`, `reduce`, `max_by_key`, `min_by_key`, `max_by`, `min_by`) take it over, as the
## adapters and `for_each` do: it is called from a place of the consumer's own, so a lambda that
## writes what it captures is accepted (`[CLO-2]`) and the caller has its counter back once the
## call returns. With the callable borrowed, each was `E3023` inside std.

fn main():
    xs = [3, 1, 4, 1, 5]
    looked = 0
    tall = fn(x: ref int) -> bool:
        looked += 1
        return x > 3
    println(xs.iter().any(tall), looked, end=" ")
    seen = 0
    positive = fn(x: ref int) -> bool:
        seen += 1
        return x > 1
    println(xs.iter().all(positive), seen)
    tries = 0
    big = fn(x: ref int) -> bool:
        tries += 1
        return x > 3
    found = xs.iter().find(big)
    at_tries = 0
    one = fn(x: ref int) -> bool:
        at_tries += 1
        return x == 4
    println(found, tries, xs.iter().position(one), at_tries)
    steps = 0
    add = fn(a: int, b: ref int) -> int:
        steps += 1
        return a + b
    folded = xs.iter().fold(0, add)
    joins = 0
    plus = fn(a: int, b: int) -> int:
        joins += 1
        return a + b
    println(folded, steps, xs.iter().copied().reduce(plus), joins)
    keys = 0
    key = fn(x: ref int) -> int:
        keys += 1
        return x * 1
    least = 0
    low = fn(x: ref int) -> int:
        least += 1
        return x * 1
    println(xs.iter().copied().max_by_key(key), xs.iter().copied().min_by_key(low), keys + least)
    compared = 0
    order = fn(a: int, b: int) -> Ordering:
        compared += 1
        return a.cmp(b)
    other = 0
    order2 = fn(a: int, b: int) -> Ordering:
        other += 1
        return a.cmp(b)
    println(xs.iter().copied().max_by(order), xs.iter().copied().min_by(order2), compared + other + 2)
