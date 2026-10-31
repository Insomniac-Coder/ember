#$ test: run-fail
#$ rules: EXC-18, EXC-1, DSP-2
#$ profiles: debug, development
#$ panics: exclusivity violation: write access to Person.alias while a read access to Person.alias is active
# `[EXC-18]` — the caller of a virtual getter cannot know which override runs,
# so its view holds every field an override reachable from the static class
# borrows: `Nick.label` borrows `alias`, so writing `alias` through another
# handle while the view lives panics, though `Person.label` borrows `name`
# (D-467). The write is the field's own (a method that changes its object, as
# `realias` does, holds every field for its call until M6b, `name` too).

open class Person:
    name: String
    alias: String

    fn init():
        this.name = "ann"
        this.alias = "zed"

    virtual fn label() -> StrView:
        return this.name

    fn realias():
        this.alias = "kit"

class Nick(Person):
    fn init():
        super.init()

    override fn label() -> StrView:
        return this.alias

fn main():
    k: Person = Nick()
    r = k
    m = k.label()
    r.alias = "kit"
    println(m)
