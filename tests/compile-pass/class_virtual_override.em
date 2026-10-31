#$ test: compile-pass
#$ rules: CLS-4

open class Base:
    virtual fn tick():
        pass

class Child(Base):
    override fn tick():
        pass

fn main():
    println(1)
