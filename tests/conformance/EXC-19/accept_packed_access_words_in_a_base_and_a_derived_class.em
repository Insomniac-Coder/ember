#$ test: run-pass
#$ rules: EXC-19, OBJ-1, CLS-10
#$ profiles: debug, release, shipping
#$ stdout: 1 2 10 20 dd 2
# `[EXC-19]` — each class level's access words stand side by side before that
# level's own fields, with a zero word after an odd count, and a base class's
# layout stays a prefix of a derived class's. Reading a base field through a
# base handle while writing the derived class's fields never conflicts.

open class Base:
    a: Array[int]
    n: int = 0

    fn init(mut self):
        self.a = [1, 2]

    fn set_n(mut self, value: int):
        self.n = value

class Derived(Base):
    b: Array[int] = []
    c: String = ""

fn main():
    d = Derived()
    as_base: Base = d
    for x in as_base.a:
        d.b.push(x * 10)
        d.c.push('d')
    as_base.set_n(len(d.b))
    println(d.a[0], d.a[1], d.b[0], d.b[1], d.c, d.n)
