#$ test: run-pass
#$ rules: TYP-36, TYP-37, STD-15
#$ profiles: debug, development, release
#$ stdout: true true false true
#$ Less Equal Greater Less
#$ [[1], [1, 2], [1, 3], [2]]
#$ [['a', 'b'], ['b']]
#$ [[0, 9], [1, 1]]
#$ true [2] [1]
#$ true Greater
## `[TYP-36]` — an `Array[T]` and a `[T; N]` are `Ord` lexicographically when `T` is: the first
## elements that differ decide, and an array that is the start of another is the smaller. So
## `<`, `cmp`, `sort`, `sorted`, `min`, `max` and a `T: Ord` bound take them (it was `E2040`,
## D-854). Floats order by totalOrder inside them, as `cmp` does (`[TYP-37]`).

fn bigger[T: Ord](a: T, b: T) -> bool:
    return a > b

fn main():
    a = [1, 2]
    b = [1, 3]
    println(a < b, [1] < a, b <= a, a == [1, 2])
    println(a.cmp(b), a.cmp([1, 2]), [2].cmp(b), Array[int]().cmp(a))
    lists = [[2], [1, 3], [1], [1, 2]]
    lists.sort()
    println(lists)
    words: Array[Array[String]] = [["b"], ["a", "b"]]
    println(sorted(words))
    fixed: Array[[int; 2]] = [[1, 1], [0, 9]]
    fixed.sort()
    println(fixed)
    println(bigger(b, a), [[1], [2]].iter().max().unwrap(), [[1], [2]].iter().min().unwrap())
    p: [float; 2] = [0.0, -0.0]
    q: [float; 2] = [0.0, 1.0]
    println(p < q, [float.NAN].cmp([1.0]))
