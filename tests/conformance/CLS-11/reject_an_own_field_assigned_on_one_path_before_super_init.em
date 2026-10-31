#$ test: compile-fail
#$ rules: CLS-11

open class Base:
    fn init():
        pass

class Derived(Base):
    tag: String

    fn init(owns tag: String):
        if tag.len() > 3:
            this.tag = tag
        super.init() #$ error[E2102]: field `tag` is not assigned on every path before `super.init(...)`

fn main():
    _d = Derived("t")
