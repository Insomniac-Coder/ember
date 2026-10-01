#$ test: run-fail
#$ rules: EXC-18, EXC-1, DSP-2
#$ profiles: debug, release
#$ panics: exclusivity violation: write access to Person.alias while a read access to Person.alias is active
# `[EXC-18]` — the caller of a virtual getter cannot know which override runs,
# so its view holds every field an override reachable from the static class
# borrows: `Nick.label` borrows `alias`, so writing `alias` through another
# handle while the view lives panics, though `Person.label` borrows `name`
# (D-467).

open class Person:
    name: String
    alias: String

    fn init(mut self):
        self.name = "ann"
        self.alias = "zed"

    virtual fn label(self) -> str:
        return self.name

    fn realias(self):
        self.alias = "kit"

class Nick(Person):
    fn init(mut self):
        super.init()

    override fn label(self) -> str:
        return self.alias

fn main():
    k: Person = Nick()
    r = k
    m = k.label()
    r.realias()
    println(m)
