#$ test: run-pass
#$ rules: LT-1, BRW-8
#$ stdout: ann
#$ 1
# `[LT-1]` rule 1 — a borrowed receiver of any type is the result's source: a
# plain struct, and a `Copy` one, which a view-returning method takes by
# address (`[BRW-8]`, ODR-024).

struct Person:
    name: String

    fn name_of(self) -> str:
        return self.name

@derive(Copy)
struct Pair:
    a: int
    b: int

    fn first(self) -> ref int:
        return ref self.a

fn main():
    p = Person(name="ann")
    println(p.name_of())
    q = Pair(a=1, b=2)
    println(q.first())
