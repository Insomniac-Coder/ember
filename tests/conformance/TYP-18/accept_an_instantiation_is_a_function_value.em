#$ test: run-pass
#$ rules: TYP-18, FN-6
#$ stdout:
#$ 5 2
#$ 3 true
# `[TYP-18]` — `id[int]` names one instantiation of a generic function, and
# that is a function value (`[FN-6]`): passed, stored, called (D-221).

fn id[T](x: T) -> T:
    return x

fn apply(f: fn(int) -> int, v: int) -> int:
    return f(v)

fn main():
    fs = [id[int], id[int]]
    println(apply(id[int], 5), fs[0](2))
    g: fn(int) -> int = id[int]
    h = id[bool]
    println(g(3), h(true))
