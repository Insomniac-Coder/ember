#$ test: compile-fail
#$ rules: CLS-4, IFC-1
#$ error[E2110]: override of a method that is not virtual

open class Base:
    fn init(mut self):
        pass

    fn tick(self):
        pass

class Child(Base):
    fn init(mut self):
        super.init()

extend Child:
    override fn tick(self):
        pass

fn main():
    println(1)
