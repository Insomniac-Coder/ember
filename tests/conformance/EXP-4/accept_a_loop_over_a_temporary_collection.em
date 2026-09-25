#$ test: run-pass
#$ rules: EXP-4, CTL-1
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ 1
#$ b
#$ b 2
#$ a 1
#$ found a
#$ 7
# D-299 — a temporary made by a `for` iterable lives to the end of the loop,
# however the loop ends: a set literal, a map a call returns, and the map
# behind `make().items()` are each borrowed by the loop's iterator. They
# were dropped at the end of the iterator's hidden declaration (E3060).

fn make() -> Map[String, int]:
    return {"b": 2, "a": 1}

fn find(want: str) -> bool:
    for k in make():
        if k == want:
            return true
    return false

fn main():
    for x in {3, 1, 2}:
        println(x)
        if x == 1:
            break
    for k in make():
        println(k)
        break
    for k, v in make().items():
        println(k, v)
    if find("a"):
        println("found a")
    total = 0
    for v in make().values():
        total += v
    for x in {4}:
        total += x
        continue
    println(total)
