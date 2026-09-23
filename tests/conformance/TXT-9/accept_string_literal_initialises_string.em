#$ test: run-pass
#$ rules: TXT-9, TYP-5
#$ profiles: debug, release, shipping
#$ stdout: Ada
#$ Rex
#$ hello
#$ Bob
#$ Dr
#$ again
# Wherever a `String` is expected, a string literal produces a new `String`
# holding its text: a class constructor argument, a named struct field, an
# annotated initialiser, an `owned` argument, a return value and a
# reassignment, which drops the old buffer first.

class Person:
    name: String

struct Pet:
    name: String

fn greet(owned who: String) -> String:
    return who

fn title() -> String:
    return "Dr"

fn main():
    p = Person("Ada")
    println(p.name.as_str())
    pet = Pet(name="Rex")
    println(pet.name.as_str())
    s: String = "hello"
    println(s.as_str())
    g = greet("Bob")
    println(g.as_str())
    t = title()
    println(t.as_str())
    s = "again"
    println(s.as_str())
