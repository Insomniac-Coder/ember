#$ test: run-pass
#$ rules: EXC-18, EXC-19, DSP-2
#$ profiles: debug, release
#$ stdout: ann [1, 2]
#$ stdout: zed [1, 2, 3]
# `[EXC-19]` — reading one field while writing another never conflicts. A view
# from a virtual getter holds the fields the bodies the call can reach borrow
# (here `name` in `Person` and `alias` in `Nick`), not the whole object, so
# `tags` may be written meanwhile (D-467; it read-locked every field).

open class Person:
    name: String
    alias: String
    tags: Array[int]

    fn init(mut self):
        self.name = "ann"
        self.alias = "zed"
        self.tags = [1]

    virtual fn label(self) -> str:
        return self.name

class Nick(Person):
    fn init(mut self):
        super.init()

    override fn label(self) -> str:
        return self.alias

fn main():
    p = Person()
    q = p
    n = p.label()
    q.tags.push(2)
    println(n, q.tags)
    k: Person = Nick()
    r = k
    m = k.label()
    r.tags.push(2)
    r.tags.push(3)
    println(m, r.tags)
