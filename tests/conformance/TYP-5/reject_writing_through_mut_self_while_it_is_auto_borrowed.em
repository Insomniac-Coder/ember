#$ test: compile-fail
#$ rules: TYP-5, BRW-1
# The auto-borrow of `mut self` is a shared borrow of the caller's value: no
# write through `self` while the reference taken of it is used.

struct Person:
    age: int

    fn bump(mut self) -> int:
        r = age_of(self)
        self.age = 9    #$ error[E3021]: cannot be written while it is borrowed
        return r

fn age_of(p: ref Person) -> ref int:
    return p.age

fn main():
    p = Person(age=3)
    println(p.bump())
