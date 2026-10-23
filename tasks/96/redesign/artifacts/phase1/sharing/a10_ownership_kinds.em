# A10: two entities share one mesh; the declared kind (struct + Box, class, Shared) decides what compiles.

struct Mesh:
    verts: Array[f32]

struct Entity:
    mesh: Box[Mesh]

fn build() -> (Entity, Entity):
    m = Box(Mesh([0.0, 0.0, 0.0]))
    a = Entity(m)
    b = Entity(m)
    return (a, b)

class MeshC:
    verts: Array[f32] = []

struct EntityC:
    mesh: MeshC

fn build_c() -> (EntityC, EntityC):
    m = MeshC()
    a = EntityC(m)
    b = EntityC(m)
    a.mesh.verts.push(1.0)
    return (a, b)

struct EntityS:
    mesh: Shared[Mesh]

fn build_s() -> (EntityS, EntityS):
    m = Shared(Mesh([0.0]))
    a = EntityS(m)
    b = EntityS(m)
    a.mesh.get_mut().verts.push(1.0)
    return (a, b)

fn main():
    _x = build()
    _y = build_c()
    _z = build_s()
