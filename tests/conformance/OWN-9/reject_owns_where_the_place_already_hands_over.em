#$ test: compile-fail
#$ rules: OWN-9, GRM-15

struct Entity:
    mesh: String

fn main():
    m = String("a")
    n = String("b")
    e = Entity(mesh=String("c"))
    e.mesh = owns m        #$ error[E0109]: `e.mesh = m` already hands `m` over; remove `owns`
    b = owns n             #$ error[E0109]: `b = n` already hands `n` over; remove `owns`
    c = owns String("d")   #$ error[E0109]
    println(e.mesh, b, c)
