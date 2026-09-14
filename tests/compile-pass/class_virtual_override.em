#$ test: compile-pass
#$ rules: CLS-4

open class Base:
    virtual fn tick(self):
        pass

class Child(Base):
    override fn tick(self):
        pass

fn main():
    println(1)
