#$ test: run-pass
#$ rules: R3
#$ stdout: 3
## R3: the nearest program that is allowed: the line that makes the mesh for the `owns` field says
## `owns` (verdict 74); the mesh is put in its own block and held alone there.

struct Mesh:
    verts: Array[int]

struct Entity:
    mesh: owns Mesh

fn main():
    e = Entity(mesh=owns Mesh(verts=[1, 2]))
    e.mesh.verts.push(3)
    println(e.mesh.verts.len())
