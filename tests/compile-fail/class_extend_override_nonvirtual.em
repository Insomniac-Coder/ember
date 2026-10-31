#$ test: compile-fail
#$ rules: CLS-4, IFC-1
#$ error[E2110]: override of a method that is not virtual

open class Base:
    fn init():
        pass

    fn tick():
        pass

class Child(Base):
    fn init():
        super.init()

extend Child:
    override fn tick():
        pass

fn main():
    println(1)
