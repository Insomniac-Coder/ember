# Attack on A23's "move at overwrite": two names refer into the place. Python prints "renamed ann".
# If only `old` were re-pointed at the moved-out value, `alias` would still point at p.name and print
# the NEW name, silently.
struct Person:
    name: String
    log: Array[String]

fn rename(mut p: Person, owned new_name: String):
    old = p.name
    alias = old
    p.name = new_name
    p.log.push(f"renamed {alias}")
    println(old)

fn main():
    p = Person(name="ann", log=[])
    rename(p, "bob")
    println(p.log[0])
