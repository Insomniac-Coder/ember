#$ test: run-pass
#$ rules: TYP-16, LEX-15a
#$ profiles: debug, development
#$ stdout: (1, 2) ('a', 'b') 3
#$ stdout: [1, 2, 3] 6
#$ stdout: 2
# `[TYP-16]` — a generic type alias stands for its value with each argument in place of its
# parameter, as transparently as a plain alias (`[LEX-15a]`): `Pair[int]` is `(int, int)`, and an
# alias may name another, or be named by a plain one.

type Pair[T] = (T, T)
type Table[K, V] = Map[K, Array[V]]
type Numbers = Sorted[int]
type Sorted[T: Ord] = Array[T]

fn swap[T: Copy](p: Pair[T]) -> Pair[T]:
    return (p.1, p.0)

fn total(xs: Numbers) -> int:
    sum = 0
    for x in xs:
        sum += x
    return sum

fn main():
    a: Pair[int] = (2, 1)
    b: Pair[String] = ("a", "b")
    println(swap(a), b, a.0 + a.1)
    xs: Numbers = [1, 2, 3]
    println(xs, total(xs))
    t: Table[String, int] = {}
    t["k"] = [4, 5]
    println(t["k"].len())
