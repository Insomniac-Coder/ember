#$ test: compile-fail
#$ rules: OWN-10, TYP-46
#$ help: write `e = Entity(mesh=owns Mesh(verts=[1, 2]))`
## design rules: R3
## R3: `owns` is written on the line that makes a value for an `owns` field, with or without a memory
## manager (verdict 74; AN-40's `Entity(mesh=owns Mesh(...))`): `mesh` holds its mesh alone in its own
## block, and the line that makes it says so (E2123).

struct Mesh:
    verts: Array[int]

struct Entity:
    mesh: owns Mesh

fn main():
    e = Entity(mesh=Mesh(verts=[1, 2]))    #$ error[E2123]: `mesh` owns its mesh; say so where it is made
    println(e.mesh.verts.len())
