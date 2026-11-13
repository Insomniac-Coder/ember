#$ test: run-pass
#$ rules: STR-1, OWN-10
#$ stdout: m 2 x 5 1 true
## `[STR-1]` - a field with a default may be omitted; a struct with no fields has a (pub) memberwise
## constructor; an argument for an `owns` field is made with `owns` on its line (`[OWN-10]`).

struct Mesh:
    points: Array[f64]

struct Model:
    name: String = "m"
    mesh: owns Mesh
    scale: int = 2

struct Empty:
    pass

fn main():
    m = Model(mesh=owns Mesh(points=[0.0]))
    n = Model("x", owns Mesh([1.0]), 5)
    _e = Empty()
    println(m.name, m.scale, n.name, n.scale, n.mesh.points.len(), true)
