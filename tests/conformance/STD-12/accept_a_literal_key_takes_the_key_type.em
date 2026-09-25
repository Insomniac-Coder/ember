#$ test: run-pass
#$ rules: STD-12, TYP-23
#$ stdout: true false one Some('two')
#$ false {2: 'two'}
# D-298 — a lookup takes any `Q: AsKey[K]`. An untyped literal key adopts
# the type the bound names when its default `int` does not meet it: `1 in s`
# on a `Set[i32]` looks up an `i32`. It was E2040, "`i64` does not implement
# `AsKey[i32]`", unless written `1i32`.

fn main():
    s: Set[i32] = {1, 2}
    m: Map[i16, String] = {1: "one", 2: "two"}
    println(1 in s, s.contains(7), m[1], m.get(2))
    m.remove(1)
    s.remove(1)
    println(1 in s, m)
