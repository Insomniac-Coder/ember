#$ test: run-fail
#$ rules: EXC-18, EXC-1, EXC-15
#$ profiles: debug, development, release
#$ stdout: ann lee
#$ panics: exclusivity violation: write access to Person.first while a read access to Person.first is active
# `[EXC-18]` — views returned together (a tuple of them) carry their fields'
# accesses to the caller for as long as any of them, or anything they are copied
# into, is used (F2). The first rename comes after the views' last use; the second
# does not.

class Person:
    first: String
    last: String

    fn init():
        this.first = "ann"
        this.last = "lee"

    fn names() -> (StrView, StrView):
        return (this.first.as_str(), this.last.as_str())

    fn rename():
        this.first = "bob"

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
