#$ test: run-pass
#$ rules: CLS-6, DRP-1
#$ stdout: built
#$ stdout: derived
#$ stdout: base
#$ stdout: field b
#$ stdout: field a
# `[CLS-6]` — destruction runs the derived `drop`, then the base's, then ends the fields in reverse
# declaration order.

struct Note:
    name: String

    fn drop():
        println("field", this.name)

open class Base:
    a: Note
    b: Note

    fn drop():
        println("base")

class Derived(Base):
    fn drop():
        println("derived")

fn main():
    d = Derived(Note(name=String("a")), Note(name=String("b")))
    println("built")
