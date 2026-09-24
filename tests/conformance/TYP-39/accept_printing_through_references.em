#$ test: run-pass
#$ rules: TYP-39, TYP-14
#$ stdout: Some(4)
#$ None
#$ (Person(name='ann', age=3), 1)
#$ Some('a')
# D-227 — a reference inside a printed value prints what it points to, as a
# `ref` read through does: `Array.get`'s `Option[ref T]`, a tuple holding one.

struct Person:
    name: String
    age: int

fn first(a: Array[int]) -> Option[ref int]:
    return a.get(0)

fn main():
    xs = [4, 5]
    println(first(xs))
    empty: Array[int] = []
    println(first(empty))
    p = Person(name="ann", age=3)

    pair = (ref p, 1)
    println(pair)
    names: Array[String] = ["a"]
    println(names.get(0))
