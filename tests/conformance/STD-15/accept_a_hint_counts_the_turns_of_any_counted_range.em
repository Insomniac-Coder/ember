#$ test: run-pass
#$ rules: STD-15, PHIL-5, CTL-3
#$ profiles: debug, release, shipping
#$ assert-c-count: contains("vec_reserve_hint(") == 8
#$ assert-c: contains(" + 1u, 1u)")
#$ assert-c: contains(", 2u)")
#$ stdout:
#$ 21 210
#$ 10 -5
#$ 20 0
#$ 10 45 285
#$ 6 0
#$ 20 5 20
# The room a counted loop asks for is `limit - counter` turns (one more for
# `a..=b`) times the fewest pushes on a turn: an inclusive range, a range
# from a negative start, two pushes a turn, two lists in one loop (a hint
# each), and a range that runs no turn (the hint asks for nothing). A loop
# that versioning (`[OPT-2]`) copies, because it reads `data[i]` with no
# bound proved, still asks: its push's reference is made on each copy, and
# each copy asks (eight hints in all; one copy runs).

fn fill(start: int, stop: int) -> Array[int]:
    xs: Array[int] = []
    for i in start..stop:
        xs.push(i)
    return xs

fn doubled(data: Array[int], n: int) -> Array[int]:
    out: Array[int] = []
    for i in 0..n:
        out.push(data[i] * 2)
    return out

fn sum(xs: Array[int]) -> int:
    total = 0
    for x in xs:
        total += x
    return total

fn main():
    inclusive: Array[int] = []
    for i in 0..=20:
        inclusive.push(i)
    println(f"{len(inclusive)} {sum(inclusive)}")

    negative: Array[int] = []
    for i in -5..5:
        negative.push(i)
    println(f"{len(negative)} {sum(negative)}")

    pairs: Array[int] = []
    for i in 0..10:
        pairs.push(i)
        pairs.push(-i)
    println(f"{len(pairs)} {sum(pairs)}")

    a: Array[int] = []
    b: Array[int] = []
    for i in 0..10:
        a.push(i)
        b.push(i * i)
    println(f"{len(a)} {sum(a)} {sum(b)}")

    some = fill(3, 9)
    none = fill(10, 3)
    println(f"{len(some)} {len(none)}")

    data = fill(0, 20)
    twice = doubled(data, 20)
    other = doubled(data, 5)
    println(f"{len(twice)} {len(other)} {len(data)}")
