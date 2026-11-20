# Clause ECS-6.1: "Iteration order is insertion order with swap-remove holes: deterministic for a
# given sequence of operations."
#$ test: run-pass
#$ rules: ECS-6, ECS-2
#$ stdout: 1 2 3
#$ stdout: 3 2
# Three values in insertion order; removing the first moves the last into its slot
# (`[ECS-2]`'s swap-and-pop).

from std.ecs import Registry

@derive(Copy)
struct Tag:
    n: int

fn order(reg: Registry) -> Array[int]:
    out: Array[int] = []
    for e, t in reg.view[Tag]():
        out.push(owns t.n)
    return out

fn main():
    reg = Registry()
    a = reg.create()
    b = reg.create()
    c = reg.create()
    reg.emplace(a, owns Tag(n=1))
    reg.emplace(b, owns Tag(n=2))
    reg.emplace(c, owns Tag(n=3))
    o = order(reg)
    println(o[0], o[1], o[2])
    reg.remove[Tag](a)
    p = order(reg)
    println(p[0], p[1])
