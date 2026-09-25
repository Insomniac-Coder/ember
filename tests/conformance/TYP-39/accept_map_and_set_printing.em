#$ test: run-pass
#$ rules: TYP-39, STD-9
#$ stdout: {}
#$ stdout: {'a': 1, "it's": 2}
#$ stdout: set()
#$ stdout: {3, 1}
#$ stdout: m={'a': 1, "it's": 2}
#$ stdout: Holder(m={1: ['x']})
# `[TYP-39]` (ODR-034) — a `Map` prints as Python's `dict` does and an empty
# one as `{}`; a `Set` as `{3, 1}`, and an empty one as `set()`. Keys and
# values print by their `Debug`, strings in Python's quotes.

struct Holder:
    m: Map[int, Array[String]]

fn main():
    m: Map[String, int] = {}
    println(m)
    m["a"] = 1
    m["it's"] = 2
    println(m)
    s = Set[int]()
    println(s)
    s.add(3)
    s.add(1)
    println(s)
    println(f"m={m}")
    println(Holder({1: ["x"]}))
