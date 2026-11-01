#$ test: compile-fail
#$ rules: GRM-15, OWN-9
# `[GRM-15]` — `owns` before a name whose `=` already hands the value over is `E0109`, whose
# message says to remove it.

struct Mesh:
    points: Array[float]

struct Entity:
    mesh: Mesh

fn main():
    m = Mesh(points=[0.0])
    e = Entity(mesh=Mesh(points=[1.0]))
    e.mesh = owns m    #$ error[E0109]: `e.mesh = m` already hands `m` over; remove `owns`
    a = Mesh(points=[2.0])
    b = owns a    #$ error[E0109]: `b = a` already hands `a` over; remove `owns`
    println(len(e.mesh.points), len(b.points))
