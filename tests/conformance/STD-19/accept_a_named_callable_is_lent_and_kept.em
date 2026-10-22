#$ test: run-pass
#$ rules: STD-19, CLO-2, CLO-3, BRW-2
#$ profiles: debug, development, release
#$ stdout: true true 4
#$ 28 22 2
#$ true 3 Some(4) 3 Some(2) 3
#$ 14 5 Some(14) 4
#$ Some(5) Some(1) 10 Some(5) Some(1) 10
#$ [4, 3] 5 [1, 1, 3, 4, 5] 5
#$ 2 4 6
## `[STD-19]` (ODR-240) — every std function that takes a callable, the consumers and the adapters
## alike, borrows a named callable the caller passes: the caller keeps it and may pass it again,
## and one that changes what it captured is lent with leave to change it, what it captured staying
## lent until the callable's last use (`[BRW-2]`), after which the caller reads it. A lambda
## written at the call is given to the call, so an adapter made of one may be returned.

fn doubled(xs: Span[int]) -> some Iterator[Item = int]:
    return xs.iter().map(fn(x) => x * 2)

fn main():
    xs = [3, 1, 4, 1, 5]
    ys = [9, 2]
    looked = 0
    tall = fn(x: ref int) -> bool:
        looked += 1
        return x > 3
    a = xs.iter().any(tall)
    b = ys.iter().any(tall)
    println(a, b, looked)
    mapped = 0
    double = fn(x: ref int) -> int:
        mapped += 1
        return x * 2
    first = xs.iter().map(double).sum()
    second = ys.iter().map(double).sum()
    println(first, second, mapped - 5)
    seen = 0
    positive = fn(x: ref int) -> bool:
        seen += 1
        return x > 0
    tries = 0
    big = fn(x: ref int) -> bool:
        tries += 1
        return x > 3
    at_tries = 0
    four = fn(x: ref int) -> bool:
        at_tries += 1
        return x == 4
    all_positive = xs.iter().all(positive)
    found = xs.iter().find(big)
    at = xs.iter().position(four)
    println(all_positive, seen - 2, found, tries, at, at_tries)
    steps = 0
    add = fn(a: int, b: ref int) -> int:
        steps += 1
        return a + b
    joins = 0
    plus = fn(a: int, b: int) -> int:
        joins += 1
        return a + b
    folded = xs.iter().fold(0, add)
    reduced = xs.iter().copied().reduce(plus)
    println(folded, steps, reduced, joins)
    keys = 0
    key = fn(x: int) -> int:
        keys += 1
        return x
    compared = 0
    order = fn(a: int, b: int) -> Ordering:
        compared += 1
        return a.cmp(b)
    most = xs.iter().copied().max_by_key(key)
    least = xs.iter().copied().min_by_key(key)
    largest = xs.iter().copied().max_by(order)
    smallest = xs.iter().copied().min_by(order)
    println(most, least, keys, largest, smallest, compared + 2)
    kept = 0
    keep = fn(x: int) -> bool:
        kept += 1
        return x > 2
    zs = [3, 1, 4]
    zs.retain(keep)
    zs.sort_by_key(fn(x) => -x)
    sorted_calls = 0
    by = fn(x: int) -> int:
        sorted_calls += 1
        return x
    ws = [5, 1, 4, 1, 3]
    ws.sort_by_key(by)
    println(zs, kept + 2, ws, sorted_calls)
    small = [1, 2, 3]
    out = String()
    for v in doubled(small.as_span()):
        out.push_str(f"{v} ")
    println(out.trim())
