#$ test: compile-fail
#$ rules: CLS-4
#$ error[E2110]: override of a method that is not virtual

open class Base:
    fn tick(self):
        pass

class Child(Base):
    override fn tick(self):
        pass

fn main():
    println(1)
