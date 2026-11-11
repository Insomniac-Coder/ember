#$ test: run-pass
#$ rules: TYP-45, HEAP-9, OWN-10, GRM-15
#$ stdout: 2
#$ 3
#$ 2
#$ 2
#$ 9
# `[TYP-45]`, `[HEAP-9]`, `[OWN-10]` — an `owns T` holds its value in a block of its own and reads,
# and is changed, as its `T`. The line that makes a value for an `owns` place says `owns`: a field,
# a field given a new value (which frees the old one), an `Option[owns T]`, an `owns dyn I` element
# that `push` takes over (one `owns` says both, `[GRM-15]`), and a function's `owns T` result.

interface Pool:
    fn size() -> int

struct IntPool implements Pool:
    items: Array[int]

    fn size() -> int:
        return this.items.len()

struct Mesh:
    points: Array[f64]

struct Entity:
    mesh: owns Mesh

struct Tree:
    value: int
    left: Option[owns Tree]

struct Registry:
    pools: Array[owns dyn Pool]

fn nine() -> owns int:
    return owns 9

fn main():
    e = Entity(mesh=owns Mesh(points=[0.0, 1.0]))
    println(e.mesh.points.len())
    e.mesh = owns Mesh(points=[1.0, 2.0, 3.0])
    println(e.mesh.points.len())
    t = Tree(value=1, left=Some(owns Tree(value=2, left=None)))
    match t.left:
        Some(l): println(l.value)
        None: println(0)
    r = Registry(pools=[])
    r.pools.push(owns IntPool(items=[1, 2]))
    println(r.pools[0].size())
    n = nine()
    println(n + 0)
