# Attack on A23's "move at overwrite" cost claim: the overwrite happens on one path only, so at the
# print `old` either owns the moved-out name or still refers into p.name; which one is known only at
# run time, so a drop flag is needed (as [OWN-3] already uses for moves made on some paths).
struct Person:
    name: String
    log: Array[String]

fn maybe_rename(mut p: Person, owned new_name: String, really: bool):
    old = p.name
    if really:
        p.name = new_name
    println(old)

fn main():
    p = Person(name="ann", log=[])
    maybe_rename(p, "bob", true)
