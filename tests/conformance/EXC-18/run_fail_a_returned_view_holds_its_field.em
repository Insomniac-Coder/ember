#$ test: run-fail
#$ rules: EXC-18, EXC-1, EXC-15
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Person.name while a read access to Person.name is active
# `[EXC-18]` (D-218) — a view of a field returned from a method, here through a
# second method, holds the field's read access in the caller until its last use.
# Renaming through another handle meanwhile would free the text it views; it
# panics instead.

class Person:
    name: String

    fn init(mut self):
        self.name = "ann"

    fn get_name(self) -> str:
        return self.name

    fn relay(self) -> str:
        return self.get_name()

    fn rename(mut self):
        self.name = "bob"

fn main():
    p = Person()
    n = p.relay()
    q = p
    q.rename()
    println(n)
