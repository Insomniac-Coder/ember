#$ test: run-pass
#$ rules: TYP-5, FN-1
#$ stdout: 4
#$ 4
#$ 4
# `[TYP-5]` rule 7 inside a callee: a `mut` parameter and `mut self` are
# places of their type, so they auto-borrow for a `ref T` as a local does, and
# a generic `ref T` infers `T` through them.

struct Person:
    age: int

    fn bump() -> int:
        this.age = this.age + 1
        return inner(this)

    fn look() -> int:
        return inner(this)

fn inner(r: ref Person) -> int:
    return r.age

[T: needs Copy]
fn same(x: ref T) -> T:
    return x

fn via_mut(ref mut x: int) -> int:
    return same(x)

fn main():
    p = Person(age=3)
    println(p.bump())
    println(p.look())
    y = 4
    println(via_mut(ref mut y))
