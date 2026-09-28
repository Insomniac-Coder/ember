#$ test: run-fail
#$ rules: EXC-19, EXC-15, EXC-1, OBJ-1
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: write access to Derived.a while a read access to Derived.a is active
# `[EXC-19]` — with each class level's words packed, a `mut self` call on a
# derived object still checks the base class's words: a loop reading the base
# field through a base-typed handle makes it panic.

open class Base:
    a: Array[int]

    fn init(mut self):
        self.a = [1]

class Derived(Base):
    b: Array[int] = []

    fn grow(mut self):
        self.b.push(len(self.b))

fn main():
    d = Derived()
    as_base: Base = d
    for x in as_base.a:
        d.grow()
        println(x)
