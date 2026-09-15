#$ test: run-fail
#$ rules: DSP-4, EFF-16
#$ panics: invalid downcast

open class Base:
    pass

class Child(Base):
    pass

fn main():
    base = Base()
    bad: Child = base as! Child
