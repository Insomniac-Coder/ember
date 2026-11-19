#$ test: run-pass
#$ rules: OWN-10
#$ stdout: 2
#$ stdout: 2
#$ stdout: 2
#$ stdout: 3
#$ stdout: 5
# `[OWN-10]` — the line that makes a value for a place that owns it alone says `owns`, at first and
# later; `owns` before a `for` loop's or a `match`'s source hands the value over.

struct Mesh:
    points: Array[float]

struct Entity:
    mesh: owns Mesh

struct Tree:
    value: int
    left: Option[owns Tree] = None

interface Pool:
    fn size() -> int

struct IntPool implements Pool:
    items: Array[int]

    fn size() -> int:
        return this.items.len()

struct Registry:
    pools: Array[owns dyn Pool]

fn main():
    e = Entity(mesh=owns Mesh(points=[0.0]))
    e.mesh = owns Mesh(points=[1.0, 2.0])
    println(e.mesh.points.len())
    t = Tree(value=1, left=Some(owns Tree(value=2)))
    if t.left exists as l:
        println(l.value)
    r = Registry(pools=[])
    r.pools.push(owns IntPool(items=[1, 2]))
    println(r.pools[0].size())
    total = 0
    for x in owns [1, 2]:
        total += x
    println(total)
    match owns Some(String("hello")):
        Some(s):
            println(s.len())
        None:
            pass
