#$ test: run-pass
#$ rules: CORO-4, CORO-3, ERR-2
#$ profiles: debug, development
#$ stdout: 0 1 0 1 2
#$ deferred
#$ 2 3
#$ 5
# `[CORO-4]` — a generator holding another across its `yield`s (the inner frame is a field of the
# outer one), a `defer` that runs when the body ends, and `?` returning early with the result.

gen fn upto(n: int) -> Generator[int]:
    for i in 0..n:
        yield i

gen fn flattened(limits: Span[int]) -> Generator[int]:
    for n in limits:
        for x in upto(n):
            yield x

gen fn with_defer() -> Generator[int]:
    defer:
        println("deferred")
    yield 2
    yield 3

fn parse(s: str) -> Option[int]:
    if s == "x":
        return None
    return Some(s.len())

gen fn parsed(words: Span[str]) -> Generator[int, Option[int]]:
    total = 0
    for w in words:
        n = parse(w)?
        total += n
        yield n
    return Some(total)

fn main():
    limits = [2, 3]
    out = [x for x in flattened(limits)]
    println(out[0], out[1], out[2], out[3], out[4])
    d = [x for x in with_defer()]
    println(d[0], d[1])
    words = ["ab", "cde", "x", "zz"]
    g = parsed(words)
    s = 0
    while true:
        match g.next():
            Some(n):
                s += n
            None:
                break
    println(s)
