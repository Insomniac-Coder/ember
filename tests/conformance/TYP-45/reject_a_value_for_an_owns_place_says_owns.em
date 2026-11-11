#$ test: compile-fail
#$ rules: OWN-10, GRM-15, OWN-9
# `[OWN-10]` — the line that makes a value for a place that owns it alone says `owns` (`E2123`).
# `[GRM-15]` — one `owns` says both the block and the hand-over (a second is `E0109`), and `owns`
# before a value made on the line for a place that is not an `owns T` is `E0109`.

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

fn main():
    e = Entity(mesh=Mesh(points=[0.0, 1.0]))  #$ error[E2123]: `mesh` owns its mesh; say so where it is made: `mesh=owns Mesh(points=[0.0, 1.0])`
    e.mesh = Mesh(points=[1.0, 2.0])  #$ error[E2123]: `mesh` owns its mesh; say so where it is made: `e.mesh = owns Mesh(points=[1.0, 2.0])`
    t = Tree(value=1, left=Some(Tree(value=2, left=None)))  #$ error[E2123]: `left` holds its tree in a block of its own; say so where it is made: `left=Some(owns Tree(value=2, left=None))`
    r = Registry(pools=[])
    r.pools.push(owns owns IntPool(items=[1, 2]))  #$ error[E0109]: one `owns` already says that the pool has a block of its own and that `push` takes it over; remove one: `r.pools.push(owns IntPool(items=[1, 2]))`
    n = 4
    b = owns (n * 2)  #$ error[E0109]: `=` already hands a value made on the line over; remove `owns`: `b = n * 2`
    println(b, t.value)
