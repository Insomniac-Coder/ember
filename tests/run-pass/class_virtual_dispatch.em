#$ test: run-pass
#$ rules: DSP-1, DSP-2, CLS-4, OBJ-2
#$ profiles: debug, release, shipping
#$ stdout: 2
#$ stdout: 3
#$ stdout: 4

open class Base:
    virtual fn tick(self) -> i32:
        return 1

    virtual fn label(self) -> i32:
        return 3

    fn init(mut self):
        pass

class Child(Base):
    override fn tick(self) -> i32:
        return 2

    virtual fn extra(self) -> i32:
        return 4

    fn init(mut self):
        super.init()

fn main():
    child = Child()
    base: Base = child
    println(base.tick())
    println(base.label())
    println(child.extra())
