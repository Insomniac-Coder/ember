#$ test: run-pass
#$ rules: EXC-18, LT-1, BRW-8, EXC-19
#$ profiles: debug, release, shipping
#$ stdout: ann [1] ann ann
#$ stdout: bob
# `[LT-1]` rule 1 — a method returning a view borrows its receiver, a class handle
# included (passed by address, `[BRW-8]`), so a getter may return a view of a field.
# `[EXC-18]` — the view carries the field's read access to its caller until its last
# use; renaming after that is no conflict (D-218).

open class Person:
    name: String
    tags: Array[int]

    fn init(mut self):
        self.name = "ann"
        self.tags = [1, 2]

    fn get_name(self) -> str:
        return self.name

    fn relay(self) -> str:
        return self.get_name()

    virtual fn label(self) -> str:
        return self.name

    fn first_tags(self) -> Span[int]:
        return self.tags[..1]

    fn rename(mut self):
        self.name = "bob"

fn main():
    p = Person()
    n = p.get_name()
    t = p.first_tags()
    println(n, t, p.relay(), p.label())
    q = p
    q.rename()
    println(p.get_name())
