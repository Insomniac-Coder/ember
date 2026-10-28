#$ test: run-pass
#$ rules: R3
#$ stdout: 3
## R3: a plain value stored into an `owns T` place is put in its block at that store, and is held
## alone there (AN-40's `Entity`).

struct Mesh:
    verts: Array[int]

struct Entity:
    mesh: owns Mesh

fn main():
    e = Entity(mesh=Mesh(verts=[1, 2]))
    e.mesh.verts.push(3)
    println(e.mesh.verts.len())
