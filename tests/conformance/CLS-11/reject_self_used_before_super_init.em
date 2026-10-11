#$ test: compile-fail
#$ rules: CLS-11

open class Base:
    name: String = "base"

    fn init(self):
        pass

fn show(b: Base):
    println(b.name)

class Derived(Base):
    tag: int

    fn init(self, tag: int):
        self.tag = tag
        show(self) #$ error[E2102]: `self` is used before `super.init(...)`
        super.init()

fn main():
    _d = Derived(1)
