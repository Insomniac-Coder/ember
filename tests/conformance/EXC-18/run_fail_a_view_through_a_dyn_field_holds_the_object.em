#$ test: run-fail
#$ rules: EXC-18, EXC-15, TYP-22
#$ profiles: debug, development, release
#$ stdout: ann
#$ panics: exclusivity violation: write access to Person.label while a read access to Person.label is active
# `[EXC-18]` (D-913) — a view returned by a call on a `Box[dyn I]` a class field holds reads every
# field of the object behind the box until the view's last use, as one held in a local does: the
# first rename comes after the first view's last use, the second while the second view lives. The
# access names the object behind the field (its place a `Deref` after the field), not the field's
# own word.

interface Named:
    fn name(self) -> str

class Person implements Named:
    label: String

    fn init(mut self):
        self.label = "ann"

    fn name(self) -> str:
        return self.label

    fn rename(mut self):
        self.label = "bob"

class Holder:
    named: Box[dyn Named]

fn main():
    p = Person()
    alias = p
    h = Holder(Box(alias))
    n = h.named.name()
    println(n)
    p.rename()
    m = h.named.name()
    p.rename()
    println(m)
