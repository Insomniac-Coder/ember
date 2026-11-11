#$ test: compile-fail
#$ rules: TYP-45, HEAP-9
# `[TYP-45]` — `Box` is not a type of Ember: one owner in a block of its own is `owns T`, made with
# `owns` on the line that makes the value for its place. `[HEAP-9]` — an `owns T` reads as its `T`,
# with nothing written in between, so there is no `get` of the block.

struct Mesh:
    points: Array[f64]

struct Entity:
    mesh: Box[Mesh]  #$ error[E1010]: `Box` is not a type of Ember

struct Scene:
    mesh: owns Mesh

fn main():
    e = Scene(mesh=Box(Mesh(points=[0.0])))  #$ error[E1010]: `Box` is not Ember
    println(e.mesh.get().points.len())  #$ error[E1010]: `e.mesh` is an `owns Mesh`, which reads as its `Mesh`: there is no `get`
