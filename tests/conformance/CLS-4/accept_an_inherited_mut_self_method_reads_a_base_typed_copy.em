#$ test: run-pass
#$ rules: CLS-4, OBJ-1
#$ profiles: debug, release, shipping
#$ stdout: 3
#$ assert-c: contains("_up = (struct em_obj_Base*)(*(")
#$ assert-c: !contains("((struct em_obj_Base**)")
# D-460 — an inherited `mut self` method is passed the address of a copy of
# the handle in the base class's type, never the derived handle's own address
# reinterpreted: reading a `struct Derived*` through a `struct Base*` lvalue is
# undefined in C (and clang 20 and later tell pointer types apart).

open class Base:
    n: int

    fn bump(mut self):
        self.n += 1

class Derived(Base):
    pass

fn main():
    d = Derived(1)
    d.bump()
    d.bump()
    println(d.n)
