# Clause TYP-22.1: "and is not generic, except over a type parameter used only as the type of a
# parameter that views its value (a plain or `ref mut` parameter), which gets one `dyn` entry (`[V:
# needs Visitor]` above `fn visit(ref mut v: V)`)"
#$ test: run-pass
#$ rules: TYP-22, IFC-5
#$ stdout: 3
#$ stdout: 4
# `Tree.visit` is generic over `V`, used only as a viewed parameter, so `Tree` is usable as `dyn
# Tree`; the one entry runs for any visitor.

interface Visitor:
    fn see(n: int)

interface Tree:
    [V: needs Visitor]
    fn visit(ref mut v: V)

struct Counter implements Visitor:
    total: int = 0

    fn see(n: int):
        this.total += n

struct Maxer implements Visitor:
    best: int = 0

    fn see(n: int):
        if n > this.best:
            this.best = n

struct Leaf implements Tree:
    n: int

    [V: needs Visitor]
    fn visit(ref mut v: V):
        v.see(this.n)

fn walk_count(t: dyn Tree) -> int:
    c = Counter()
    t.visit(ref mut c)
    return c.total

fn walk_max(t: dyn Tree) -> int:
    m = Maxer()
    t.visit(ref mut m)
    return m.best

fn main():
    println(walk_count(Leaf(3)))
    println(walk_max(Leaf(4)))
