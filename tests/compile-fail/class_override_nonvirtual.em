#$ test: compile-fail
#$ rules: CLS-4
#$ error[E2110]: override of a method that is not virtual

open class Base:
    fn tick():
        pass

class Child(Base):
    override fn tick():
        pass

fn main():
    println(1)
