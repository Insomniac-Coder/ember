#$ test: run-fail
#$ rules: EXC-18, EXC-15, TYP-22
#$ profiles: debug, release, shipping
#$ stdout: ann
#$ panics: exclusivity violation: write access to Person.label while a read access to Person.label is active
# `[EXC-18]` — through a `dyn` box the caller cannot know the concrete type; the
# table's `access` entry, which only a class concrete has, reads every field of
# the object until the view's last use (F4). The first rename comes after the
# first view's last use; the second while the second view lives.

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

fn main():
    p = Person()
    alias = p
    boxed: Box[dyn Named] = Box(alias)
    n = boxed.name()
    println(n)
    p.rename()
    m = boxed.name()
    p.rename()
    println(m)
