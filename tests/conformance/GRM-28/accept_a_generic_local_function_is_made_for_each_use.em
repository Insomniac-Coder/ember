#$ test: run-pass
#$ rules: GRM-28, CLO-12, CLO-2, TYP-16, TYP-18
#$ profiles: debug, development, release
#$ stdout: 5 hi 2.5 9
#$ stdout: 1@10 a@10
#$ stdout: 7 x
#$ stdout: 10
#$ stdout: 42@10 3
#$ stdout: 2 1 11
#$ stdout: 7:1 7:x k:1 k:x
## `[GRM-28]` — a local function may be generic: each set of type arguments its calls infer, or
## write (`ident[int](9)`, `[TYP-18]`), is a function of its own (`[TYP-16]`), each capturing the
## enclosing locals as the one closure the declaration is (`[CLO-12]`): `tagged` reads `base` and
## writes `calls` from both of its instances, `add_all` writes `total`. It may call itself, a
## closure in the block may call it, and one in a generic function is made for each of that
## function's instances (`pair`, in `labelled[int]` and `labelled[str]`).

[U: needs Display]
fn labelled(u: U) -> String:
    [T: needs Display]
    fn pair(t: T) -> String:
        return f"{u}:{t}"
    return pair(1) + " " + pair("x")

fn main():
    base = 10
    calls = 0
    [T]
    fn ident(x: T) -> T:
        return x
    [T: needs Display]
    fn tagged(x: T) -> String:
        calls += 1
        return f"{x}@{base}"
    [T: needs Copy]
    fn first(xs: Array[T]) -> T:
        return xs[0]
    [T: needs Display]
    fn count_down(n: int, label: T) -> int:
        if n == 0:
            return base
        return count_down(n - 1, label)
    println(ident(5), ident("hi"), ident(2.5), ident[int](9))
    println(tagged(1), tagged("a"))
    println(first([7, 8]), first(['x', 'y']))
    println(count_down(3, "z"))
    later = fn(v: int) -> String => tagged(v + 1)
    println(later(41), calls)
    total = 0
    [T: needs Copy]
    fn add_all(xs: Array[T], weight: int) -> int:
        for _ in xs:
            total += weight
        return xs.len()
    println(add_all([1, 2], 3), add_all(['a'], 5), total)
    println(labelled(7), labelled("k"))
