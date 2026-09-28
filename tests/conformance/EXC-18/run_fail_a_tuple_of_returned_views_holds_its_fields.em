#$ test: run-fail
#$ rules: EXC-18, EXC-1, EXC-15
#$ profiles: debug, release, shipping
#$ stdout: ann lee
#$ panics: exclusivity violation: write access to Person.first while a read access to Person.first is active
# `[EXC-18]` — views returned together (a tuple of them) carry their fields'
# accesses to the caller for as long as any of them, or anything they are copied
# into, is used (F2). The first rename comes after the views' last use; the second
# does not.

class Person:
    first: String
    last: String

    fn init(mut self):
        self.first = "ann"
        self.last = "lee"

    fn names(self) -> (str, str):
        return (self.first.as_str(), self.last.as_str())

    fn rename(mut self):
        self.first = "bob"

fn main():
    p = Person()
    (a, b) = p.names()
    println(a, b)
    q = p
    q.rename()
    (c, d) = p.names()
    kept = c
    q.rename()
    println(kept, d)
