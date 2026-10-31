#$ test: compile-fail
#$ rules: CLS-11

open class Base:
    name: String = "base"

    fn init():
        pass

fn show(b: Base):
    println(b.name)

class Derived(Base):
    tag: int

    fn init(tag: int):
        this.tag = tag
        show(this) #$ error[E2102]: `this` is used before `super.init(...)`
        super.init()

fn main():
    _d = Derived(1)
