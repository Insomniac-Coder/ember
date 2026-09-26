#$ test: run-fail
#$ rules: EXC-18, EXC-15, DSP-2
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Person.name while a read access to Person.name is active
# `[EXC-18]` — through a virtual call the caller cannot know which field an
# override's view borrows, so it reads every field of the receiver object until
# the view's last use; a `mut self` call on that object meanwhile panics.

open class Person:
    name: String

    fn init(mut self):
        self.name = "ann"

    virtual fn label(self) -> str:
        return self.name

    fn rename(mut self):
        self.name = "bob"

fn main():
    p = Person()
    n = p.label()
    q = p
    q.rename()
    println(n)
