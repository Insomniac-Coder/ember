# The same on a field of a local struct ([OWN-3]'s partial-move tracking, p23c).
struct Person:
    name: String

fn main():
    p = Person(name="ann")
    if true:
        defer:
            println(p.name)
        old = p.name
        println(old)
    p.name = "bob"
    println(p.name)
