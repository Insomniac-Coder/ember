#$ test: run-pass
#$ rules: EXC-19, EXC-18, TYP-22
#$ profiles: debug, development, release
#$ stdout: 7 7
#$ stdout: 8 5
#$ stdout: 2 9
# `[EXC-19]`, `[EXC-18]` (D-913) — a store into a class field holding a `Box[dyn I]` takes the
# field's own access word, as a store into any field does: the constructor's first write of the
# field (`Holder(Box(alias))`, whose box a call makes) and a replacement of a box whose concrete is a
# class or a struct. Through the box's table it read the box before the store had written it (an
# access violation) and began and ended the access on two different objects.

interface Speak:
    fn speak() -> int

class Mote implements Speak:
    n: int = 0
    keep: Option[Holder] = None

    fn speak() -> int:
        return this.n

struct Square:
    side: int

extend Square implements Speak:
    fn speak() -> int:
        return this.side

class Holder:
    s: owns dyn Speak

fn main():
    m = Mote()
    m.n = 7
    alias = m
    h = Holder(owns alias)
    println(h.s.speak(), m.speak())
    m.n = 8
    h.s = owns Square(5)
    println(m.speak(), h.s.speak())
    m.keep = Some(Holder(owns Square(2)))
    again = m
    h.s = owns again
    m.n = 9
    match m.keep:
        Some(k):
            println(k.s.speak(), h.s.speak())
        None:
            pass
