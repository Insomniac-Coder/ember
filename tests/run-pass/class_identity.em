#$ test: run-pass
#$ rules: CLS-1, OBJ-2
#$ profiles: debug, release, shipping
#$ stdout: 1
#$ stdout: 0
#$ stdout: 1
#$ stdout: 1

# `is` compares class-handle identity, not the values of their fields.  The
# derived/base pair also exercises the compiler's ordinary upcast at the
# identity boundary.
open class Base:
    value: i32

    fn init(mut self, value: i32):
        self.value = value

class Child(Base):
    fn init(mut self, value: i32):
        super.init(value)

fn main():
    first = Child(7)
    same = first
    other = Child(7)
    base: Base = first
    if first is same:
        println(1)
    else:
        println(0)
    if first is other:
        println(1)
    else:
        println(0)
    if first is base:
        println(1)
    else:
        println(0)
    if first is not other:
        println(1)
    else:
        println(0)
