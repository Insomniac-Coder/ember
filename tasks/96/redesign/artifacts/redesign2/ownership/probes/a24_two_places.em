# A24: the same lines with a struct and with a class: one mesh stored in two entities' lists.
struct MeshS:
    verts: Array[f32]

class MeshC:
    verts: Array[f32] = []

fn with_struct():
    a: Array[MeshS] = []
    b: Array[MeshS] = []
    m = MeshS(verts=[1.0])
    a.push(m)
    b.push(m)

fn with_class():
    a: Array[MeshC] = []
    b: Array[MeshC] = []
    m = MeshC()
    a.push(m)
    b.push(m)
    a[0].verts.push(2.0)
    println(b[0].verts.len())

fn main():
    with_struct()
    with_class()
