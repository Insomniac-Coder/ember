# A23 (and phase3-break-D2 N1): keep the old value of a field, overwrite the field, use the old value.
struct Person:
    name: String
    log: Array[String]

fn rename(mut p: Person, owned new_name: String):
    old = p.name
    p.name = new_name
    p.log.push(f"renamed {old}")

fn main():
    p = Person(name="ann", log=[])
    rename(p, "bob")
    println(p.log)
