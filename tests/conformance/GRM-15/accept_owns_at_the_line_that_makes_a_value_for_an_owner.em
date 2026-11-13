#$ test: run-pass
#$ rules: GRM-15, TYP-45
#$ stdout: 1 2
#$ stdout: 5
#$ stdout: 2
## `[GRM-15]` — `owns e` is written, and required, on the line that makes a value for a place that owns it
## alone: a field's initialiser, an assignment to a field, a value inside `Some(...)`.

struct Mesh:
    points: Array[float] = []

struct Tree:
    value: int = 0

class Holder:
    mesh: owns Mesh

fn main():
    h = Holder(mesh=owns Mesh(points=[0.0]))
    h.mesh = owns Mesh(points=[1.0, 2.0])
    println(h.mesh.points.len() - 1, h.mesh.points.len())
    left: Option[owns Tree] = Some(owns Tree(value=5))
    if left exists as t:
        println(t.value)
    n = 2
    b = owns n
    println(b)
