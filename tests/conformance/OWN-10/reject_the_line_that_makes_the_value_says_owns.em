#$ test: compile-fail
#$ rules: OWN-10

struct Mesh:
    points: Array[float]

struct Entity:
    mesh: owns Mesh

struct Tree:
    value: int
    left: Option[owns Tree] = None

struct Count:
    n: owns int

fn main():
    c = Count(n=1)                                      #$ error[E2123]
    e = Entity(mesh=Mesh(points=[0.0]))                 #$ error[E2123]
    e.mesh = Mesh(points=[1.0])                         #$ error[E2123]
    t = Tree(value=1, left=Some(Tree(value=2)))         #$ error[E2123]
    println(e.mesh.points.len(), t.value, c.n)
