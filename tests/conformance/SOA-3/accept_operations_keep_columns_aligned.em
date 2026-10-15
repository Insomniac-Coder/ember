#$ test: run-pass
#$ rules: SOA-3, SOA-6, SOA-7, DSJ-4, CTL-3b
#$ profiles: debug, release
#$ stdout: bob 8 16
#$ Some(Named(name='bea', score=5))
#$ 4 ann 3
#$ apart 3 1
#$ 1
#$ 5
#$ 8
#$ Some(Named(name='cy', score=8)) 2 None
#$ 0 true
# `[SOA-3]` — `push`, `pop`, `swap_remove`, `clear`, `reserve`, `get` and `len` keep the columns
# aligned element for element, growing one block (`[SOA-7]`); the elements' `String`s are dropped
# with them. A column is a view: `assert_disjoint` proves two apart (`[DSJ-4]`), and `for` iterates
# one (`[CTL-3b]`).

struct Named:
    name: String
    score: int

fn total(s: Span[int]) -> int:
    t = 0
    for v in s:
        t += v
    return t

fn main():
    ns = SoA[Named].with_capacity(2)
    ns.push(Named("ann".to_string(), 3))
    ns.push(Named("bob".to_string(), 5))
    ns.push(Named("cy".to_string(), 8))
    println(ns[1].name, ns[2].score, total(ns.score))
    ns[1].name = "bea".to_string()
    println(ns.get(1))
    ns.push(Named("dee".to_string(), 1))
    println(ns.len(), ns.swap_remove(0).name, ns.len())
    ns.reserve(100)
    match mem.assert_disjoint(ns.name, ns.score):
        Ok((names, scores)):
            println("apart", names.len(), scores[0])
        Err(_):
            println("overlap")
    for s in ns.score:
        println(s)
    println(ns.pop(), ns.len(), ns.get(5))
    ns.clear()
    println(ns.len(), ns.is_empty())
