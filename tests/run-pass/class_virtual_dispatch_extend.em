#$ test: run-pass
#$ rules: DSP-1, DSP-2, CLS-4, IFC-1
#$ profiles: debug, release, shipping
#$ stdout: 6

open class Base:
    virtual fn tick(self) -> i32:
        return 1

    fn init(mut self):
        pass

class Child(Base):
    fn init(mut self):
        super.init()

extend Child:
    override fn tick(self) -> i32:
        return 6

fn main():
    child = Child()
    base: Base = child
    println(base.tick())
