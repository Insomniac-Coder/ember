#$ test: run-pass
#$ rules: DSP-1, DSP-2, CLS-4, IFC-1
#$ profiles: debug, development, release
#$ stdout: 6

open class Base:
    virtual fn tick() -> i32:
        return 1

    fn init():
        pass

class Child(Base):
    fn init():
        super.init()

extend Child:
    override fn tick() -> i32:
        return 6

fn main():
    child = Child()
    base: Base = child
    println(base.tick())
