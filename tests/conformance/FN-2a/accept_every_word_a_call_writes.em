#$ test: run-pass
#$ rules: FN-2a
#$ stdout: 2 3
#$ stdout: [1, 2] [1, 2]
#$ stdout: [1, 2]
#$ stdout: [1, 2]
#$ stdout: 21 3
#$ stdout: ['ann']
#$ stdout: 1
#$ stdout: 2
#$ stdout: [3, 4]
# `[FN-2a]` — a call says `ref mut` before an argument the callee may change and `owns` before one
# it takes over, with the parameter's own words: `bump(ref mut a)`, `take(owns xs)`, `take(owns copy
# of xs)` (keeps `xs`), `take(owns [1, 2])`, a value computed in the call in brackets, a part
# (`names.push(owns p.name)`), a function written in the call, and `mem.take(ref mut this.items)`.

import std.mem as mem

struct Counter:
    n: int

struct Person:
    name: String

struct Bag:
    items: Array[int]

    fn drain_all() -> Array[int]:
        return mem.take(ref mut this.items)

fn bump(ref mut c: Counter):
    c.n += 1

fn take(owns xs: Array[int]):
    println(xs)

fn keep(owns xs: Array[int]) -> Array[int]:
    return xs

fn main():
    a = Counter(n=1)
    bump(ref mut a)
    println(a.n, a.n + 1)
    xs = [1, 2]
    kept = keep(owns copy of xs)
    println(kept, xs)
    take(owns xs)
    take(owns [1, 2])
    m: Map[int, int] = {}
    i = 3
    m.insert(owns (i * 7), owns i)
    println(m.len() * 21, m[21])
    names: Array[String] = []
    p = Person(name="ann")
    names.push(owns p.name)
    println(names)
    ys = [1, 2]
    ys.iter().for_each(owns fn(x: int): println(x))
    b = Bag(items=[3, 4])
    println(b.drain_all())
