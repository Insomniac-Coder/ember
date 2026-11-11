#$ test: compile-fail
#$ rules: TYP-45, TYP-14, HEAP-9
# `[TYP-45]` — `Box` is not a type of Ember: one owner in a block of its own is `owns T`, made with
# `owns` on the line that makes the value for its place. `[HEAP-9]` — an `owns T` reads as its `T`,
# with nothing written in between: `.get()` on an `owns` value is `E2072` (`[TYP-14]`).

struct Mesh:
    points: Array[f64]

struct Entity:
    mesh: Box[Mesh]  #$ error[E1010]: `Box` is not a type of Ember: write `owns Mesh`

struct Scene:
    mesh: owns Mesh

fn main():
    e = Scene(mesh=Box(Mesh(points=[0.0])))  #$ error[E1010]: `Box` is not Ember
    println(e.mesh.get().points.len())  #$ error[E2072]: an `owns Mesh` reads as a `Mesh`; remove `.get()`
