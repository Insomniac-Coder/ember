#$ test: run-pass
#$ rules: R3, R1
#$ stdout: 3
## R3, R1: `e.mesh = m` hands `m`, a mesh that already has a name, over to the field with no word, as
## `b = a` does; `owns` is written only on a line that makes a value (verdict 97: E0109 keeps only `owns`
## before a name that already exists).

struct Mesh:
    verts: Array[int]

struct Entity:
    mesh: owns Mesh

fn main():
    e = Entity(mesh=owns Mesh(verts=[1]))
    m = Mesh(verts=[1, 2, 3])
    e.mesh = m
    println(e.mesh.verts.len())
