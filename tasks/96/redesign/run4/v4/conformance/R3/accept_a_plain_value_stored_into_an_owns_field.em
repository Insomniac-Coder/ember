#$ test: run-pass
#$ rules: R3
#$ stdout: 3
## R3: a plain value stored into an `owns T` place is put in its block at that store, and is held
## alone there (AN-40's `Entity`). `m` was made on its own line as `main`'s own mesh, and
## `Entity(mesh=m)` hands it over, so no `owns` is written (E0109 refuses one before a value its place
## hands over); a mesh made on the `Entity(...)` line itself says `owns` (verdict 74,
## `accept_owns_on_the_line_that_makes_a_value_for_an_owns_field`).

struct Mesh:
    verts: Array[int]

struct Entity:
    mesh: owns Mesh

fn main():
    m = Mesh(verts=[1, 2])
    e = Entity(mesh=m)
    e.mesh.verts.push(3)
    println(e.mesh.verts.len())
